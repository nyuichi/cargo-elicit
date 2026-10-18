import ElicitContracts
open Aeneas Aeneas.Std

namespace AdlerSpec

/-- Exact chunks preserve all bytes, with no short chunk in the main iterator. -/
theorem chunks_partition {α : Type} (n : Nat) (hn : 0 < n) (xs : List α) :
    (List.toChunksExact n hn xs).1.flatten ++ (List.toChunksExact n hn xs).2 = xs ∧
    (∀ c ∈ (List.toChunksExact n hn xs).1, c.length = n) ∧
    (List.toChunksExact n hn xs).2.length < n := by
  rw [List.toChunksExact]
  split
  · simp_all
  · have ih := chunks_partition n hn (xs.drop n)
    dsimp only at *
    constructor
    · simpa only [List.flatten_cons, List.append_assoc, ih.1] using
        (List.take_append_drop n xs)
    · constructor
      · intro c hc
        simp only [List.mem_cons] at hc
        rcases hc with rfl | hc
        · simp only [List.length_take]
          omega
        · exact ih.2.1 c hc
      · exact ih.2.2
termination_by xs.length
decreasing_by simp only [List.length_drop]; omega

end AdlerSpec

namespace AdlerSpec

theorem step_eq_normalize_raw (s : Nat × Nat) (x : U8) :
    step s x = normalize (rawStep s x) := by
  simp only [step, normalize, rawStep]
  congr 1
  omega

theorem normalize_rawStep (s : Nat × Nat) (x : U8) :
    normalize (rawStep (normalize s) x) = normalize (rawStep s x) := by
  simp [normalize, rawStep, Nat.add_mod]

/-- Deferring all modular reductions preserves the scalar result. -/
theorem update_eq_raw (s : Nat × Nat) (xs : List U8) :
    update s xs = normalize (xs.foldl rawStep s) := by
  suffices h : xs.foldl step (normalize s) = normalize (xs.foldl rawStep s) by
    exact h
  induction xs generalizing s with
  | nil => rfl
  | cons x xs ih =>
    simp only [List.foldl_cons, step_eq_normalize_raw, normalize_rawStep]
    exact ih (rawStep s x)

/-- A single lane accumulates at most 255 per input byte.  The doubled
    second bound avoids division and accounts for every prefix sum. -/
theorem raw_fold_bounds (s : Nat × Nat) (xs : List U8) :
    (xs.foldl rawStep s).1 ≤ s.1 + 255 * xs.length ∧
    2 * (xs.foldl rawStep s).2 ≤
      2 * s.2 + 2 * xs.length * s.1 + 255 * xs.length * (xs.length + 1) := by
  induction xs generalizing s with
  | nil => simp
  | cons x xs ih =>
    have hx : x.val ≤ 255 := by scalar_tac
    have hxlen := Nat.mul_le_mul_left xs.length hx
    have hi := ih (rawStep s x)
    simp only [List.foldl_cons, List.length_cons, rawStep] at *
    constructor <;> nlinarith [hi.1, hi.2]

/-- The optimized inner loop performs at most 5552 lane updates between
    reductions.  Reduced starting lanes therefore never overflow u32. -/
theorem lane_chunk_fits_u32 (s : Nat × Nat) (xs : List U8)
    (ha : s.1 ≤ 65520) (hb : s.2 ≤ 65520) (hn : xs.length ≤ 5552) :
    (xs.foldl rawStep s).1 < 4294967296 ∧
    (xs.foldl rawStep s).2 < 4294967296 := by
  have h := raw_fold_bounds s xs
  have hna := Nat.mul_le_mul hn ha
  have hnn := Nat.mul_le_mul hn (Nat.add_le_add_right hn 1)
  constructor <;> nlinarith [h.1, h.2]

/-- The weighted four-byte identity used when combining the lanes. -/
theorem raw_four (s : Nat × Nat) (x0 x1 x2 x3 : U8) :
    [x0,x1,x2,x3].foldl rawStep s =
      (s.1 + x0.val + x1.val + x2.val + x3.val,
       s.2 + 4 * s.1 + 4 * x0.val + 3 * x1.val + 2 * x2.val + x3.val) := by
  simp only [List.foldl_cons, List.foldl_nil, rawStep]
  congr 1
  ring

/-- Caller-facing rule for the pinned chunk iterator model, with exact
    chunk sizes and an explicit no-loss decomposition. -/
@[step] theorem chunks_exact_spec (s : Slice U8) (n : Usize) (hn : 0 < n.val) :
    core.slice.Slice.chunks_exact s n ⦃ fun it =>
      (it.chunks.map Slice.val).flatten ++ it.remainder.val = s.val ∧
      (∀ c ∈ it.chunks, c.val.length = n.val) ∧
      it.remainder.val.length < n.val ⦄ := by
  simp only [core.slice.Slice.chunks_exact, dite_eq_left hn]
  apply (WP.spec_ok _).mpr
  have h := chunks_partition n.val hn s.val
  refine ⟨?_, ?_, ?_⟩
  · simpa using h.1
  · intro c hc
    simp only [List.mem_map] at hc
    obtain ⟨x, _, rfl⟩ := hc
    simpa using h.2.1 x.val x.property
  · simpa using h.2.2

end AdlerSpec

open Aeneas Aeneas.Std
open ElicitGenerated

namespace AdlerSpec

@[step] theorem vector_add_spec (a b : algo.U32X4)
    (h : ∀ i : Nat, i < 4 → a.val[i]!.val + b.val[i]!.val ≤ 4294967295) :
    algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign a b ⦃ fun out =>
      ∀ i : Nat, i < 4 → out.val[i]!.val = a.val[i]!.val + b.val[i]!.val ⦄ := by
  have h0 := h 0 (by decide)
  have h1 := h 1 (by decide)
  have h2 := h 2 (by decide)
  have h3 := h 3 (by decide)
  unfold algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step*
  all_goals simp_all
  all_goals try (first
    | simpa [U32.max, U32.numBits] using h 0 (by decide)
    | simpa [U32.max, U32.numBits] using h 1 (by decide)
    | simpa [U32.max, U32.numBits] using h 2 (by decide)
    | simpa [U32.max, U32.numBits] using h 3 (by decide))
  intro i hi
  have hi' : i = 0 ∨ i = 1 ∨ i = 2 ∨ i = 3 := by omega
  rcases hi' with rfl | rfl | rfl | rfl <;> simp_all

@[step] theorem vector_from_spec (bytes : Slice U8) (h : 4 ≤ bytes.val.length) :
    algo.U32X4.from bytes ⦃ fun out =>
      ∀ i : Nat, i < 4 → out.val[i]!.val = bytes.val[i]!.val ⦄ := by
  unfold algo.U32X4.from
  simp only [lift]
  step*
  intro j hj
  have hj' : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases hj' with rfl | rfl | rfl | rfl <;> simp_all

  all_goals rw [List.getElem?_eq_getElem (by omega)]; rfl

end AdlerSpec

open Aeneas Aeneas.Std
open ElicitGenerated

namespace AdlerSpec

def laneResult (a b : algo.U32X4) (cs : List (Slice U8)) (i : Nat) : Nat × Nat :=
  (cs.map fun c => c.val[i]!).foldl rawStep (a.val[i]!.val, b.val[i]!.val)

theorem raw_fold_mono (s : Nat × Nat) (xs : List U8) :
    s.1 ≤ (xs.foldl rawStep s).1 ∧ s.2 ≤ (xs.foldl rawStep s).2 := by
  induction xs generalizing s with
  | nil => simp
  | cons x xs ih =>
    have h := ih (rawStep s x)
    simp only [List.foldl_cons, rawStep] at *
    constructor <;> omega

theorem inner_spec (it : core.slice.iter.ChunksExact U8) (a b : algo.U32X4)
    (hc : ∀ c ∈ it.chunks, c.val.length = 4)
    (hs : ∀ i : Nat, i < 4 →
      (laneResult a b it.chunks i).1 < 4294967296 ∧
      (laneResult a b it.chunks i).2 < 4294967296) :
    algo.Adler32.compute_loop0_loop0 it a b ⦃ fun out =>
      ∀ i : Nat, i < 4 →
        (out.1.val[i]!.val, out.2.val[i]!.val) = laneResult a b it.chunks i ⦄ := by
  rcases it with ⟨cs, rem⟩
  induction cs generalizing a b with
  | nil =>
    unfold algo.Adler32.compute_loop0_loop0
    rw [loop]
    simp only [algo.Adler32.compute_loop0_loop0.body,
      core.slice.iter.IteratorChunksExact.next]
    simp [laneResult]
  | cons c cs ih =>
    have hfirst : ∀ i : Nat, i < 4 →
        a.val[i]!.val + c.val[i]!.val ≤ 4294967295 ∧
        b.val[i]!.val + (a.val[i]!.val + c.val[i]!.val) ≤ 4294967295 := by
      intro i hi
      have hmono := raw_fold_mono
        (rawStep (a.val[i]!.val, b.val[i]!.val) c.val[i]!)
        (cs.map fun d => d.val[i]!)
      have hsafe := hs i hi
      simp only [laneResult, List.map_cons, List.foldl_cons, rawStep] at hsafe hmono
      constructor <;> omega
    unfold algo.Adler32.compute_loop0_loop0
    rw [loop]
    simp only [algo.Adler32.compute_loop0_loop0.body,
      core.slice.iter.IteratorChunksExact.next]
    step as ⟨v, hv⟩
    · exact (hc c (by simp)).ge
    step as ⟨a', ha'⟩
    step as ⟨b', hb'⟩
    have heq : ∀ i : Nat, i < 4 → laneResult a' b' cs i = laneResult a b (c :: cs) i := by
      intro i hi
      simp only [laneResult, List.map_cons, List.foldl_cons, rawStep]
      rw [hb' i hi, ha' i hi, hv i hi]
      simp only [Nat.add_assoc]
    change algo.Adler32.compute_loop0_loop0 { chunks := cs, remainder := rem } a' b' ⦃ _ ⦄
    apply WP.spec_mono (ih a' b' (by intro d hd; exact hc d (by simp [hd])) (by
      intro i hi
      rw [heq i hi]
      exact hs i hi))
    intro out hout i hi
    rw [hout i hi, heq i hi]

/-- Specialized caller rule using the proved chunk-size overflow bound. -/
@[step] theorem inner_bounded_spec (it : core.slice.iter.ChunksExact U8) (a b : algo.U32X4)
    (hc : ∀ c ∈ it.chunks, c.val.length = 4)
    (hn : it.chunks.length ≤ 5552)
    (ha : ∀ i : Nat, i < 4 → a.val[i]!.val ≤ 65520)
    (hb : ∀ i : Nat, i < 4 → b.val[i]!.val ≤ 65520) :
    algo.Adler32.compute_loop0_loop0 it a b ⦃ fun out =>
      ∀ i : Nat, i < 4 →
        (out.1.val[i]!.val, out.2.val[i]!.val) = laneResult a b it.chunks i ⦄ := by
  apply inner_spec it a b hc
  intro i hi
  exact lane_chunk_fits_u32 _ _ (ha i hi) (hb i hi) (by simpa using hn)

/-- The short-chunk loop has the identical original translated body. -/
@[step] theorem remainder_inner_spec (it : core.slice.iter.ChunksExact U8) (a b : algo.U32X4)
    (hc : ∀ c ∈ it.chunks, c.val.length = 4)
    (hn : it.chunks.length ≤ 5552)
    (ha : ∀ i : Nat, i < 4 → a.val[i]!.val ≤ 65520)
    (hb : ∀ i : Nat, i < 4 → b.val[i]!.val ≤ 65520) :
    algo.Adler32.compute_loop1 it a b ⦃ fun out =>
      ∀ i : Nat, i < 4 →
        (out.1.val[i]!.val, out.2.val[i]!.val) = laneResult a b it.chunks i ⦄ := by
  change algo.Adler32.compute_loop0_loop0 it a b ⦃ _ ⦄
  exact inner_bounded_spec it a b hc hn ha hb

end AdlerSpec

open Aeneas Aeneas.Std
namespace AdlerSpec

theorem slices_flatten_length (cs : List (Slice U8)) (n : Nat)
    (h : ∀ c ∈ cs, c.val.length = n) :
    (cs.map Slice.val).flatten.length = cs.length * n := by
  induction cs with
  | nil => simp
  | cons c cs ih =>
    have hc := h c (by simp)
    have ht : ∀ d ∈ cs, d.val.length = n := by intro d hd; exact h d (by simp [hd])
    simp only [List.map_cons, List.flatten_cons, List.length_append, List.length_cons]
    rw [hc, ih ht]
    simp [Nat.add_mul, Nat.add_comm]

/-- Counts the exact chunks obtained from a byte-preserving iterator partition. -/
theorem chunks_count_eq (it : core.slice.iter.ChunksExact U8) (xs : List U8) (n : Nat)
    (hp : (it.chunks.map Slice.val).flatten ++ it.remainder.val = xs)
    (hc : ∀ c ∈ it.chunks, c.val.length = n) :
    it.chunks.length * n + it.remainder.val.length = xs.length := by
  have h := congrArg List.length hp
  simpa only [List.length_append, slices_flatten_length it.chunks n hc] using h

end AdlerSpec

open Aeneas Aeneas.Std ElicitGenerated
namespace AdlerSpec

@[step] theorem vector_rem_spec (a : algo.U32X4) (q : U32) (hq : 0 < q.val) :
    algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign a q ⦃ fun out =>
      ∀ i : Nat, i < 4 → out.val[i]!.val = a.val[i]!.val % q.val ⦄ := by
  unfold algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
  step*
  all_goals simp_all
  intro i hi
  have hi' : i = 0 ∨ i = 1 ∨ i = 2 ∨ i = 3 := by omega
  rcases hi' with rfl | rfl | rfl | rfl <;> simp_all

end AdlerSpec

open Aeneas Aeneas.Std ElicitGenerated
namespace AdlerSpec

/-- An execution and seed-contribution obligation for the original outer loop.
    Functional lane recombination remains a separate obligation. -/
@[step] theorem outer_execution_spec
    (size : Usize) (it : core.slice.iter.ChunksExact U8) (a b : U32)
    (av bv : algo.U32X4)
    (hsize : size.val = 22208) (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
    (hc : ∀ c ∈ it.chunks, c.val.length = 22208)
    (hav : ∀ i : Nat, i < 4 → av.val[i]!.val ≤ 65520)
    (hbv : ∀ i : Nat, i < 4 → bv.val[i]!.val ≤ 65520) :
    algo.Adler32.compute_loop0 size it a b av bv ⦃ fun out =>
      out.1.val = it.chunks.foldl (fun z _ => (z + 22208 * a.val) % 65521) b.val ∧
      out.1.val ≤ 65535 ∧
      (∀ i : Nat, i < 4 → out.2.1.val[i]!.val ≤ 65520) ∧
      (∀ i : Nat, i < 4 → out.2.2.val[i]!.val ≤ 65520) ⦄ := by
  rcases it with ⟨cs, rem⟩
  induction cs generalizing b av bv with
  | nil =>
    unfold algo.Adler32.compute_loop0
    rw [loop]
    simp [algo.Adler32.compute_loop0.body, core.slice.iter.IteratorChunksExact.next,
      hb]
    constructor
    · intro i hi; simpa using hav i hi
    · intro i hi; simpa using hbv i hi
  | cons c cs ih =>
    have hcl := hc c (by simp)
    unfold algo.Adler32.compute_loop0
    rw [loop]
    simp only [algo.Adler32.compute_loop0.body, core.slice.iter.IteratorChunksExact.next]
    step as ⟨it2, hp, hlen, hrem⟩
    have hcount := chunks_count_eq it2 c.val 4 hp hlen
    have hn : it2.chunks.length ≤ 5552 := by omega
    step as ⟨ab1, hlanes⟩
    rcases ab1 with ⟨av1, bv1⟩
    step as ⟨size32, hs32⟩
    step as ⟨inc, hinc⟩
    step as ⟨b1, hb1⟩
    simp only [algo.Adler32.compute.MOD]
    step as ⟨av2, hav2⟩
    step as ⟨bv2, hbv2⟩
    step as ⟨b2, hb2⟩
    have hb2bound : b2.val ≤ 65535 := by
      have hm := Nat.mod_lt b1.val (by decide : 0 < 65521)
      change b2.val = b1.val % 65521 at hb2
      omega
    have ha2 : ∀ i : Nat, i < 4 → av2.val[i]!.val ≤ 65520 := by
      intro i hi
      have he := hav2 i hi
      have hm := Nat.mod_lt av1.val[i]!.val (by decide : 0 < 65521)
      change av2.val[i]!.val = av1.val[i]!.val % 65521 at he
      omega
    have hbv2bound : ∀ i : Nat, i < 4 → bv2.val[i]!.val ≤ 65520 := by
      intro i hi
      have he := hbv2 i hi
      have hm := Nat.mod_lt bv1.val[i]!.val (by decide : 0 < 65521)
      change bv2.val[i]!.val = bv1.val[i]!.val % 65521 at he
      omega
    have hctail : ∀ d ∈ cs, d.val.length = 22208 := by
      intro d hd
      exact hc d (by simp [hd])
    have hrec := ih b2 av2 bv2 hb2bound ha2 hbv2bound hctail
    simp only [algo.Adler32.compute_loop0, algo.Adler32.compute_loop0.body,
      core.slice.iter.IteratorChunksExact.next, algo.Adler32.compute.MOD] at hrec
    apply WP.spec_mono hrec
    intro out hout
    refine ⟨?_, hout.2⟩
    rw [hout.1]
    simp only [List.foldl_cons]
    rw [hb2, hb1, hinc]
    simp [hs32, hsize]

end AdlerSpec

open Aeneas Aeneas.Std ElicitGenerated
namespace AdlerSpec

@[step] theorem vector_mul_spec (a : algo.U32X4) (q : U32)
    (h : ∀ i : Nat, i < 4 → a.val[i]!.val * q.val ≤ 4294967295) :
    algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign a q ⦃ fun out =>
      ∀ i : Nat, i < 4 → out.val[i]!.val = a.val[i]!.val * q.val ⦄ := by
  unfold algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
  step*
  all_goals simp_all
  all_goals try (first
    | simpa [U32.max, U32.numBits] using h 0 (by decide)
    | simpa [U32.max, U32.numBits] using h 1 (by decide)
    | simpa [U32.max, U32.numBits] using h 2 (by decide)
    | simpa [U32.max, U32.numBits] using h 3 (by decide))
  intro i hi
  have hi' : i = 0 ∨ i = 1 ∨ i = 2 ∨ i = 3 := by omega
  rcases hi' with rfl | rfl | rfl | rfl <;> simp_all

end AdlerSpec

open Aeneas Aeneas.Std ElicitGenerated
namespace AdlerSpec

@[step] theorem sum_loop_spec (it : core.slice.iter.Iter U32) (a : U32)
    (hb : a.val + ((it.slice.val.drop it.i).map (fun x => x.val)).sum ≤ 4294967295) :
    algo.Adler32.compute_loop2 it a ⦃ fun out =>
      out.val = a.val + ((it.slice.val.drop it.i).map (fun x => x.val)).sum ⦄ := by
  unfold algo.Adler32.compute_loop2
  rw [loop]
  simp only [algo.Adler32.compute_loop2.body, core.slice.iter.IteratorSliceIter.next]
  split
  next h =>
    have hn : it.i < it.slice.val.length := by simpa using h
    have hd := List.drop_eq_getElem_cons hn
    have hbound := hb
    rw [hd] at hbound
    simp only [List.map_cons, List.sum_cons] at hbound
    step as ⟨a', ha'⟩
    change a'.val = a.val + it.slice.val[it.i]'hn at ha'
    have ht : a'.val + ((it.slice.val.drop (it.i + 1)).map (fun x => x.val)).sum ≤ 4294967295 := by
      rw [ha']
      simpa [Nat.add_assoc] using hbound
    have hr := sum_loop_spec { slice := it.slice, i := it.i + 1 } a' ht
    simp only [algo.Adler32.compute_loop2, algo.Adler32.compute_loop2.body,
      core.slice.iter.IteratorSliceIter.next] at hr
    apply WP.spec_mono hr
    intro out ho
    rw [ho, hd]
    simp only [List.map_cons, List.sum_cons]
    rw [ha']
    simp [Nat.add_assoc]
  next h =>
    have hn : it.slice.val.length ≤ it.i := by simpa using Nat.le_of_not_gt h
    have hd : it.slice.val.drop it.i = [] := List.drop_eq_nil_iff.mpr hn
    simp [hd]
termination_by it.slice.val.length - it.i
decreasing_by omega

@[step] theorem sum_loop3_spec (it : core.slice.iter.Iter U32) (a : U32)
    (hb : a.val + ((it.slice.val.drop it.i).map (fun x => x.val)).sum ≤ 4294967295) :
    algo.Adler32.compute_loop3 it a ⦃ fun out =>
      out.val = a.val + ((it.slice.val.drop it.i).map (fun x => x.val)).sum ⦄ := by
  change algo.Adler32.compute_loop2 it a ⦃ _ ⦄
  exact sum_loop_spec it a hb
end AdlerSpec

open Aeneas Aeneas.Std
namespace AdlerSpec

theorem normalize_raw_fold (s : Nat × Nat) (xs : List U8) :
    normalize (xs.foldl rawStep (normalize s)) = normalize (xs.foldl rawStep s) := by
  rw [← update_eq_raw, ← update_eq_raw]
  simp [update, normalize]

/-- List-model composition; API slice representability is a separate obligation. -/
theorem update_append (s : Nat × Nat) (xs ys : List U8) :
    update s (xs ++ ys) = update (update s xs) ys := by
  rw [update_eq_raw, update_eq_raw, update_eq_raw, List.foldl_append,
    normalize_raw_fold]

end AdlerSpec
