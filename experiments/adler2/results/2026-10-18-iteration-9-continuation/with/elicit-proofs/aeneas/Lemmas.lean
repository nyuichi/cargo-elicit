import ElicitContracts
open Aeneas Aeneas.Std Result ElicitGenerated
namespace AdlerProof
theorem lane_budget (k : Nat) (hk : k ≤ 5552) :
    65520 + k * 65520 + 255 * k * (k + 1) / 2 < 4294967296 := by
  have hprod : k * (k + 1) ≤ 5552 * 5553 :=
    Nat.mul_le_mul hk (Nat.add_le_add_right hk 1)
  have hlin : k * 65520 ≤ 5552 * 65520 := Nat.mul_le_mul_right 65520 hk
  have htri : 255 * k * (k + 1) ≤ 255 * (5552 * 5553) := by
    nlinarith
  omega
end AdlerProof

namespace AdlerProof

theorem chunks_partition {α : Type} (n : Nat) (hn : 0 < n) (xs : List α) :
    (List.toChunksExact n hn xs).1.flatten ++ (List.toChunksExact n hn xs).2 = xs ∧
    (∀ c ∈ (List.toChunksExact n hn xs).1, c.length = n) ∧
    (List.toChunksExact n hn xs).2.length < n := by
  unfold List.toChunksExact
  split
  · simp_all
  · rename_i h
    have ih := chunks_partition n hn (xs.drop n)
    simp only [] at *
    constructor
    · simpa [List.flatten_cons, List.append_assoc, ih.1] using List.take_append_drop n xs
    constructor
    · intro c hc
      simp only [List.mem_cons] at hc
      rcases hc with rfl | hc
      · simp only [List.length_take]
        omega
      · exact ih.2.1 c hc
    · exact ih.2.2
termination_by xs.length
decreasing_by simp [List.length_drop]; omega

@[step]
theorem chunks_exact_spec {α : Type} (s : Slice α) (n : Usize) (hn : 0 < n.val) :
    core.slice.Slice.chunks_exact s n ⦃ fun it =>
      (it.chunks.map Slice.val).flatten ++ it.remainder.val = s.val ∧
      (∀ c ∈ it.chunks, c.val.length = n.val) ∧
      it.remainder.val.length < n.val ⦄ := by
  unfold core.slice.Slice.chunks_exact
  simp only [hn, _root_.dite_true, WP.spec_ok, Slice.from_val]
  have h := chunks_partition n.val hn s.val
  constructor
  · simpa [List.map_map, Function.comp_def, Slice.from_val] using h.1
  constructor
  · intro c hc
    obtain ⟨x, hx, rfl⟩ := List.mem_map.mp hc
    simpa only [Slice.from_val] using h.2.1 x.val x.property
  · exact h.2.2
end AdlerProof

namespace AdlerProof
@[step]
theorem from_wp (bytes : Slice U8) (h : 4 ≤ bytes.val.length) :
    algo.U32X4.from bytes ⦃ fun out =>
      ∀ i : Nat, i < 4 → out.val[i]!.val = bytes.val[i]!.val ⦄ := by
  unfold algo.U32X4.from
  simp only [lift]
  step* +scalarTac
  intro k hk
  have cases : k = 0 ∨ k = 1 ∨ k = 2 ∨ k = 3 := by omega
  rcases cases with rfl | rfl | rfl | rfl <;> simp_all [Array.make_val] <;> grind

@[step]
theorem add_wp (self other : algo.U32X4)
    (h : ∀ i : Nat, i < 4 → self.val[i]!.val + other.val[i]!.val ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign self other ⦃ fun out =>
      ∀ i : Nat, i < 4 → out.val[i]!.val = self.val[i]!.val + other.val[i]!.val ⦄ := by
  have h0 := h 0 (by omega)
  have h1 := h 1 (by omega)
  have h2 := h 2 (by omega)
  have h3 := h 3 (by omega)
  unfold algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step* +scalarTac
  all_goals simp_all [Array.set_val_eq]
  all_goals grind


end AdlerProof

namespace AdlerProof
theorem inner_step (bytes : Slice U8) (cs : List (Slice U8)) (rem : Slice U8)
    (a b : algo.U32X4) (hlen : 4 ≤ bytes.val.length)
    (ha : ∀ i : Nat, i < 4 → a.val[i]!.val + bytes.val[i]!.val ≤ U32.max)
    (hb : ∀ i : Nat, i < 4 → b.val[i]!.val + (a.val[i]!.val + bytes.val[i]!.val) ≤ U32.max) :
    algo.Adler32.compute_loop0_loop0.body { chunks := bytes :: cs, remainder := rem } a b ⦃ fun r =>
      ∃ av bv, r = .cont ({ chunks := cs, remainder := rem }, av, bv) ∧
        (∀ i : Nat, i < 4 → av.val[i]!.val = a.val[i]!.val + bytes.val[i]!.val) ∧
        (∀ i : Nat, i < 4 → bv.val[i]!.val = b.val[i]!.val + (a.val[i]!.val + bytes.val[i]!.val)) ⦄ := by
  unfold algo.Adler32.compute_loop0_loop0.body core.slice.iter.IteratorChunksExact.next
  step* +scalarTac
  refine ⟨a_vec1, b_vec1, rfl, ?_, ?_⟩
  · intro i hi
    rw [a_vec1_post i hi, val_post i hi]
  · intro i hi
    rw [b_vec1_post i hi, a_vec1_post i hi, val_post i hi]
end AdlerProof

namespace AdlerProof

def laneStep (i : Nat) (s : Nat × Nat) (bytes : Slice U8) : Nat × Nat :=
  let a := s.1 + bytes.val[i]!.val
  (a, s.2 + a)

def laneRun (cs : List (Slice U8)) (a b i : Nat) : Nat × Nat :=
  cs.foldl (laneStep i) (a, b)

def laneSafe (cs : List (Slice U8)) (a b i : Nat) : Prop :=
  ∀ k : Nat, k ≤ cs.length →
    (laneRun (cs.take k) a b i).1 ≤ U32.max ∧
    (laneRun (cs.take k) a b i).2 ≤ U32.max

theorem inner_loop (cs : List (Slice U8)) (rem : Slice U8)
    (a b : algo.U32X4)
    (hchunks : ∀ c ∈ cs, 4 ≤ c.val.length)
    (hsafe : ∀ i : Nat, i < 4 → laneSafe cs a.val[i]!.val b.val[i]!.val i) :
    algo.Adler32.compute_loop0_loop0 { chunks := cs, remainder := rem } a b ⦃ fun out =>
      ∀ i : Nat, i < 4 → (out.1.val[i]!.val, out.2.val[i]!.val) =
        laneRun cs a.val[i]!.val b.val[i]!.val i ⦄ := by
  induction cs generalizing a b with
  | nil =>
    unfold algo.Adler32.compute_loop0_loop0
    unfold loop
    simp [algo.Adler32.compute_loop0_loop0.body, core.slice.iter.IteratorChunksExact.next,
      laneRun]
  | cons c cs ih =>
    have hc : 4 ≤ c.val.length := hchunks c (by simp)
    have ha : ∀ i : Nat, i < 4 → a.val[i]!.val + c.val[i]!.val ≤ U32.max := by
      intro i hi
      have h := (hsafe i hi) 1 (by simp)
      simpa [laneRun, laneStep] using h.1
    have hb : ∀ i : Nat, i < 4 → b.val[i]!.val + (a.val[i]!.val + c.val[i]!.val) ≤ U32.max := by
      intro i hi
      have h := (hsafe i hi) 1 (by simp)
      simpa [laneRun, laneStep] using h.2
    unfold algo.Adler32.compute_loop0_loop0
    unfold loop
    apply WP.spec_bind (inner_step c cs rem a b hc ha hb)
    intro r hr
    obtain ⟨av, bv, rfl, hav, hbval⟩ := hr
    simp only []
    have hcs : ∀ c ∈ cs, 4 ≤ c.val.length := by
      intro d hd
      exact hchunks d (by simp [hd])
    have htail : ∀ i : Nat, i < 4 → laneSafe cs av.val[i]!.val bv.val[i]!.val i := by
      intro i hi k hk
      have h := (hsafe i hi) (k + 1) (by simp; omega)
      rw [hav i hi, hbval i hi]
      simpa only [List.take_succ_cons, laneRun, List.foldl_cons, laneStep] using h
    have hrec := ih av bv hcs htail
    apply WP.spec_mono hrec
    intro out hout i hi
    have h := hout i hi
    rw [hav i hi, hbval i hi] at h
    simpa only [laneRun, List.foldl_cons, laneStep] using h
theorem lane_run_bounds (cs : List (Slice U8)) (a b i : Nat) :
    (laneRun cs a b i).1 ≤ a + 255 * cs.length ∧
    2 * (laneRun cs a b i).2 ≤ 2 * b + 2 * cs.length * a + 255 * cs.length * (cs.length + 1) := by
  induction cs generalizing a b with
  | nil => simp [laneRun]
  | cons c cs ih =>
    have hd : c.val[i]!.val ≤ 255 := by scalar_tac
    have hm := Nat.mul_le_mul_left cs.length hd
    have h := ih (a + c.val[i]!.val) (b + (a + c.val[i]!.val))
    simp only [laneRun, List.foldl_cons, laneStep, List.length_cons] at *
    constructor <;> nlinarith [h.1, h.2]

theorem lane_safe_budget (cs : List (Slice U8)) (a b i : Nat)
    (hlen : cs.length ≤ 5552) (ha : a ≤ 65520) (hb : b ≤ 65520) :
    laneSafe cs a b i := by
  intro k hk
  have hn : (cs.take k).length ≤ 5552 := by simp only [List.length_take]; omega
  have h := lane_run_bounds (cs.take k) a b i
  have hm := Nat.mul_le_mul_left ((cs.take k).length) ha
  have budget := lane_budget ((cs.take k).length) hn
  constructor
  · have hlin : 255 * (cs.take k).length ≤ 255 * 5552 := Nat.mul_le_mul_left 255 hn
    scalar_tac
  · have hbound : 2 * (laneRun (cs.take k) a b i).2 ≤
        2 * 65520 + 2 * (cs.take k).length * 65520 + 255 * (cs.take k).length * ((cs.take k).length + 1) := by
      nlinarith [h.2]
    have hhalf : (laneRun (cs.take k) a b i).2 ≤
        65520 + (cs.take k).length * 65520 + 255 * (cs.take k).length * ((cs.take k).length + 1) / 2 := by omega
    scalar_tac

@[step]
theorem inner_loop_bounded (cs : List (Slice U8)) (rem : Slice U8) (a b : algo.U32X4)
    (hchunks : ∀ c ∈ cs, 4 ≤ c.val.length) (hlen : cs.length ≤ 5552)
    (ha : ∀ i : Nat, i < 4 → a.val[i]!.val ≤ 65520)
    (hb : ∀ i : Nat, i < 4 → b.val[i]!.val ≤ 65520) :
    algo.Adler32.compute_loop0_loop0 { chunks := cs, remainder := rem } a b ⦃ fun out =>
      ∀ i : Nat, i < 4 → (out.1.val[i]!.val, out.2.val[i]!.val) = laneRun cs a.val[i]!.val b.val[i]!.val i ⦄ := by
  apply inner_loop cs rem a b hchunks
  intro i hi
  exact lane_safe_budget cs _ _ i hlen (ha i hi) (hb i hi)
@[step]
theorem remainder_inner_loop_bounded (cs : List (Slice U8)) (rem : Slice U8) (a b : algo.U32X4)
    (hchunks : ∀ c ∈ cs, 4 ≤ c.val.length) (hlen : cs.length ≤ 5552)
    (ha : ∀ i : Nat, i < 4 → a.val[i]!.val ≤ 65520)
    (hb : ∀ i : Nat, i < 4 → b.val[i]!.val ≤ 65520) :
    algo.Adler32.compute_loop1 { chunks := cs, remainder := rem } a b ⦃ fun out =>
      ∀ i : Nat, i < 4 → (out.1.val[i]!.val, out.2.val[i]!.val) = laneRun cs a.val[i]!.val b.val[i]!.val i ⦄ := by
  exact inner_loop_bounded cs rem a b hchunks hlen ha hb
theorem fixed_chunks_length (cs : List (Slice U8)) (n : Nat)
    (h : ∀ c ∈ cs, c.val.length = n) :
    (cs.map Slice.val).flatten.length = cs.length * n := by
  induction cs with
  | nil => simp
  | cons c cs ih =>
    have hc := h c (by simp)
    have ht : ∀ c ∈ cs, c.val.length = n := by
      intro d hd
      exact h d (by simp [hd])
    simp only [List.map_cons, List.flatten_cons, List.length_append, List.length_cons]
    rw [hc, ih ht]
    simp [Nat.add_mul, Nat.add_comm]

theorem chunk_count_bound (cs : List (Slice U8)) (rem source : Slice U8)
    (hpartition : (cs.map Slice.val).flatten ++ rem.val = source.val)
    (hfull : ∀ c ∈ cs, c.val.length = 4)
    (hlen : source.val.length ≤ 22208) : cs.length ≤ 5552 := by
  have h := congrArg List.length hpartition
  rw [List.length_append, fixed_chunks_length cs 4 hfull] at h
  omega
end AdlerProof

namespace AdlerProof
@[step]
theorem rem_wp (self : algo.U32X4) (q : U32) (hq : 0 < q.val) :
    algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign self q ⦃ fun out =>
      ∀ i : Nat, i < 4 → out.val[i]!.val = self.val[i]!.val % q.val ⦄ := by
  unfold algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
  step* +scalarTac
  all_goals simp_all [Array.set_val_eq]
  all_goals grind
theorem rem_mod_wp (self : algo.U32X4) :
    algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign self algo.Adler32.compute.MOD ⦃ fun out =>
      ∀ i : Nat, i < 4 → out.val[i]!.val = self.val[i]!.val % 65521 ∧ out.val[i]!.val ≤ 65520 ⦄ := by
  unfold algo.Adler32.compute.MOD
  apply WP.spec_mono (rem_wp self 65521#u32 (by decide))
  intro out ho i hi
  have h := ho i hi
  change out.val[i]!.val = self.val[i]!.val % 65521 at h
  constructor
  · exact h
  · rw [h]
    have hm : self.val[i]!.val % 65521 < 65521 := Nat.mod_lt _ (by decide)
    omega
end AdlerProof

namespace AdlerProof
attribute [local step] rem_mod_wp

@[local step]
theorem inner_iter_spec (it : core.slice.iter.ChunksExact U8) (a b : algo.U32X4)
    (hc : ∀ c ∈ it.chunks, 4 ≤ c.val.length) (hn : it.chunks.length ≤ 5552)
    (ha : ∀ i : Nat, i < 4 → a.val[i]!.val ≤ 65520)
    (hb : ∀ i : Nat, i < 4 → b.val[i]!.val ≤ 65520) :
    algo.Adler32.compute_loop0_loop0 it a b ⦃ fun out =>
      ∀ i : Nat, i < 4 → (out.1.val[i]!.val, out.2.val[i]!.val) = laneRun it.chunks a.val[i]!.val b.val[i]!.val i ⦄ := by
  exact inner_loop_bounded it.chunks it.remainder a b hc hn ha hb

theorem outer_step (chunk : Slice U8) (cs : List (Slice U8)) (rem : Slice U8)
    (a b : U32) (av bv : algo.U32X4) (hlen : chunk.val.length = 22208)
    (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
    (hav : ∀ i : Nat, i < 4 → av.val[i]!.val ≤ 65520)
    (hbv : ∀ i : Nat, i < 4 → bv.val[i]!.val ≤ 65520) :
    algo.Adler32.compute_loop0.body 22208#usize a { chunks := chunk :: cs, remainder := rem } b av bv ⦃ fun r =>
      ∃ (qs : List (Slice U8)) (b' : U32) (av' bv' : algo.U32X4),
        r = .cont ({ chunks := cs, remainder := rem }, b', av', bv') ∧
        (qs.map Slice.val).flatten = chunk.val ∧
        (∀ q ∈ qs, q.val.length = 4) ∧
        b'.val = (b.val + 22208 * a.val) % 65521 ∧ b'.val ≤ 65520 ∧
        (∀ i : Nat, i < 4 →
          av'.val[i]!.val = (laneRun qs av.val[i]!.val bv.val[i]!.val i).1 % 65521 ∧
          bv'.val[i]!.val = (laneRun qs av.val[i]!.val bv.val[i]!.val i).2 % 65521 ∧
          av'.val[i]!.val ≤ 65520 ∧ bv'.val[i]!.val ≤ 65520) ⦄ := by
  unfold algo.Adler32.compute_loop0.body core.slice.iter.IteratorChunksExact.next
  simp only [lift]
  step* +scalarTac
  case hc =>
    intro c hc
    rw [iter2_post1 c hc]
  case hn =>
    exact chunk_count_bound iter2.chunks iter2.remainder chunk iter2_post iter2_post1 (by omega)
  case hnz =>
    unfold algo.Adler32.compute.MOD
    change (65521 : Nat) ≠ 0
    decide
  have hl := congrArg List.length iter2_post
  rw [List.length_append, fixed_chunks_length iter2.chunks 4 iter2_post1] at hl
  have hr : iter2.remainder.val.length = 0 := by omega
  have hp : (iter2.chunks.map Slice.val).flatten = chunk.val := by
    simpa only [List.length_eq_zero_iff.mp hr, List.append_nil] using iter2_post
  refine ⟨iter2.chunks, b2, a_vec2, b_vec2, rfl, hp, iter2_post1, ?_, ?_, ?_⟩
  · unfold algo.Adler32.compute.MOD at b2_post
    change b2.val = b1.val % 65521 at b2_post
    simp [UScalar.cast_val_eq] at i2_post
    rw [b2_post, b1_post, i2_post]
  · unfold algo.Adler32.compute.MOD at b2_post
    change b2.val = b1.val % 65521 at b2_post
    rw [b2_post]
    have hm : b1.val % 65521 < 65521 := Nat.mod_lt _ (by decide)
    omega
  · intro i hi
    have ha1 := congrArg Prod.fst (a_vec1_post i hi)
    have hb1 := congrArg Prod.snd (a_vec1_post i hi)
    simp only [Prod.fst, Prod.snd] at ha1 hb1
    refine ⟨?_, ?_, (a_vec2_post i hi).2, (b_vec2_post i hi).2⟩
    · rw [(a_vec2_post i hi).1, ha1]
    · rw [(b_vec2_post i hi).1, hb1]
end AdlerProof

namespace AdlerProof

theorem laneRun_congr (cs : List (Slice U8)) (a b c d i : Nat)
    (ha : a % 65521 = c % 65521) (hb : b % 65521 = d % 65521) :
    (laneRun cs a b i).1 % 65521 = (laneRun cs c d i).1 % 65521 ∧
    (laneRun cs a b i).2 % 65521 = (laneRun cs c d i).2 % 65521 := by
  induction cs generalizing a b c d with
  | nil => exact ⟨ha, hb⟩
  | cons x xs ih =>
    simp only [laneRun, List.foldl_cons, laneStep]
    apply ih
    · simp [Nat.add_mod, ha]
    · simp [Nat.add_mod, ha, hb]

theorem laneRun_append (xs ys : List (Slice U8)) (a b i : Nat) :
    laneRun (xs ++ ys) a b i =
      laneRun ys (laneRun xs a b i).1 (laneRun xs a b i).2 i := by
  simp [laneRun, List.foldl_append]

theorem outer_loop (cs : List (Slice U8)) (rem : Slice U8)
    (a b : U32) (av bv : algo.U32X4)
    (hchunks : ∀ c ∈ cs, c.val.length = 22208)
    (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
    (hav : ∀ i : Nat, i < 4 → av.val[i]!.val ≤ 65520)
    (hbv : ∀ i : Nat, i < 4 → bv.val[i]!.val ≤ 65520) :
    algo.Adler32.compute_loop0 22208#usize { chunks := cs, remainder := rem } a b av bv ⦃ fun out =>
      ∃ qs : List (Slice U8),
        (qs.map Slice.val).flatten = (cs.map Slice.val).flatten ∧
        (∀ q ∈ qs, q.val.length = 4) ∧
        out.1.val % 65521 = (b.val + 22208 * cs.length * a.val) % 65521 ∧
        out.1.val ≤ 65535 ∧
        (∀ i : Nat, i < 4 →
          out.2.1.val[i]!.val = (laneRun qs av.val[i]!.val bv.val[i]!.val i).1 % 65521 ∧
          out.2.2.val[i]!.val = (laneRun qs av.val[i]!.val bv.val[i]!.val i).2 % 65521 ∧
          out.2.1.val[i]!.val ≤ 65520 ∧ out.2.2.val[i]!.val ≤ 65520) ⦄ := by
  induction cs generalizing b av bv with
  | nil =>
    unfold algo.Adler32.compute_loop0
    unfold loop
    simp [algo.Adler32.compute_loop0.body, core.slice.iter.IteratorChunksExact.next]
    refine ⟨[], by simp, by simp, hb, ?_⟩
    intro i hi
    simp only [laneRun, List.foldl_nil]
    have ham : av.val[i]!.val % 65521 = av.val[i]!.val := Nat.mod_eq_of_lt (by have := hav i hi; omega)
    have hbm : bv.val[i]!.val % 65521 = bv.val[i]!.val := Nat.mod_eq_of_lt (by have := hbv i hi; omega)
    simpa only [List.getElem!_eq_getElem?_getD] using
      (And.intro ham.symm (And.intro hbm.symm (And.intro (hav i hi) (hbv i hi))))
  | cons c cs ih =>
    have hc := hchunks c (by simp)
    have ht : ∀ c ∈ cs, c.val.length = 22208 := by
      intro d hd
      exact hchunks d (by simp [hd])
    unfold algo.Adler32.compute_loop0
    unfold loop
    apply WP.spec_bind (outer_step c cs rem a b av bv hc ha hb hav hbv)
    intro r hr
    obtain ⟨qs, b', av', bv', rfl, hpart, hfull, hscalar, hbbound, hlanes⟩ := hr
    simp only []
    have ha' : ∀ i : Nat, i < 4 → av'.val[i]!.val ≤ 65520 := by
      intro i hi
      exact (hlanes i hi).2.2.1
    have hb' : ∀ i : Nat, i < 4 → bv'.val[i]!.val ≤ 65520 := by
      intro i hi
      exact (hlanes i hi).2.2.2
    have hrec := ih b' av' bv' ht (by omega) ha' hb'
    apply WP.spec_mono hrec
    intro out hout
    obtain ⟨tail, htailpart, htailfull, htailscalar, htailbound, htaillanes⟩ := hout
    refine ⟨qs ++ tail, ?_, ?_, ?_, htailbound, ?_⟩
    · simp only [List.map_append, List.flatten_append, List.map_cons, List.flatten_cons]
      rw [hpart, htailpart]
    · intro q hq
      rcases List.mem_append.mp hq with hq | hq
      · exact hfull q hq
      · exact htailfull q hq
    · rw [htailscalar, hscalar]
      simp only [List.length_cons]
      rw [Nat.mod_add_mod]
      congr 1
      ring
    · intro i hi
      have h := htaillanes i hi
      have hc := laneRun_congr tail av'.val[i]!.val bv'.val[i]!.val
        (laneRun qs av.val[i]!.val bv.val[i]!.val i).1
        (laneRun qs av.val[i]!.val bv.val[i]!.val i).2 i
        (by rw [(hlanes i hi).1]; simp)
        (by rw [(hlanes i hi).2.1]; simp)
      rw [laneRun_append]
      exact ⟨h.1.trans hc.1, h.2.1.trans hc.2, h.2.2.1, h.2.2.2⟩

@[step]
theorem outer_iter_wp (size : Usize) (it : core.slice.iter.ChunksExact U8)
    (a b : U32) (av bv : algo.U32X4)
    (hsize : size.val = 22208) (hchunks : ∀ c ∈ it.chunks, c.val.length = 22208)
    (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
    (hav : ∀ i : Nat, i < 4 → av.val[i]!.val ≤ 65520)
    (hbv : ∀ i : Nat, i < 4 → bv.val[i]!.val ≤ 65520) :
    algo.Adler32.compute_loop0 size it a b av bv ⦃ fun out =>
      ∃ qs : List (Slice U8),
        (qs.map Slice.val).flatten = (it.chunks.map Slice.val).flatten ∧
        (∀ q ∈ qs, q.val.length = 4) ∧
        out.1.val % 65521 = (b.val + 22208 * it.chunks.length * a.val) % 65521 ∧
        out.1.val ≤ 65535 ∧
        (∀ i : Nat, i < 4 →
          out.2.1.val[i]!.val = (laneRun qs av.val[i]!.val bv.val[i]!.val i).1 % 65521 ∧
          out.2.2.val[i]!.val = (laneRun qs av.val[i]!.val bv.val[i]!.val i).2 % 65521 ∧
          out.2.1.val[i]!.val ≤ 65520 ∧ out.2.2.val[i]!.val ≤ 65520) ⦄ := by
  have heq : size = 22208#usize := by
    apply UScalar.eq_of_val_eq
    simpa using hsize
  subst size
  exact outer_loop it.chunks it.remainder a b av bv hchunks ha hb hav hbv
end AdlerProof

namespace AdlerProof
@[step]
theorem mul_wp (self : algo.U32X4) (rhs : U32)
    (h : ∀ i : Nat, i < 4 → self.val[i]!.val * rhs.val ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign self rhs ⦃ fun out =>
      ∀ i : Nat, i < 4 → out.val[i]!.val = self.val[i]!.val * rhs.val ⦄ := by
  have h0 := h 0 (by omega)
  have h1 := h 1 (by omega)
  have h2 := h 2 (by omega)
  have h3 := h 3 (by omega)
  unfold algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
  step* +scalarTac
  all_goals simp_all [Array.set_val_eq]
  all_goals grind

@[step]
theorem final_inner_iter_wp (it : core.slice.iter.ChunksExact U8) (a b : algo.U32X4)
    (hc : ∀ c ∈ it.chunks, 4 ≤ c.val.length) (hn : it.chunks.length ≤ 5552)
    (ha : ∀ i : Nat, i < 4 → a.val[i]!.val ≤ 65520)
    (hb : ∀ i : Nat, i < 4 → b.val[i]!.val ≤ 65520) :
    algo.Adler32.compute_loop1 it a b ⦃ fun out =>
      ∀ i : Nat, i < 4 → (out.1.val[i]!.val, out.2.val[i]!.val) = laneRun it.chunks a.val[i]!.val b.val[i]!.val i ⦄ := by
  exact remainder_inner_loop_bounded it.chunks it.remainder a b hc hn ha hb
end AdlerProof

namespace AdlerProof

def iterSum (it : core.slice.iter.Iter U32) : Nat :=
  ((it.slice.val.drop it.i).map (fun x => x.val)).sum

theorem sum_body_step (it : core.slice.iter.Iter U32) (a : U32)
    (hi : it.i < it.slice.val.length)
    (hm : a.val + it.slice.val[it.i]!.val ≤ U32.max) :
    algo.Adler32.compute_loop2.body it a ⦃ fun r => ∃ out,
      r = .cont ({ slice := it.slice, i := it.i + 1 }, out) ∧
      out.val = a.val + it.slice.val[it.i]!.val ⦄ := by
  have hiLen : it.i < it.slice.len := by simpa using hi
  unfold algo.Adler32.compute_loop2.body core.slice.iter.IteratorSliceIter.next
  simp only [hiLen, _root_.dite_true]
  step* +scalarTac
  all_goals simp_all
  all_goals grind

@[step]
theorem sum_loop_wp (it : core.slice.iter.Iter U32) (a : U32)
    (hmax : a.val + iterSum it ≤ U32.max) :
    algo.Adler32.compute_loop2 it a ⦃ fun out => out.val = a.val + iterSum it ⦄ := by
  by_cases hi : it.i < it.slice.val.length
  · have hsum : iterSum it = it.slice.val[it.i]!.val +
        iterSum { slice := it.slice, i := it.i + 1 } := by
      have h := congrArg (fun xs : List U32 => (xs.map (fun x => x.val)).sum) (List.drop_eq_getElem_cons hi)
      simpa only [iterSum, List.map_cons, List.sum_cons, List.getElem!_eq_getElem?_getD, List.getElem?_eq_getElem hi, Option.getD_some] using h
    unfold algo.Adler32.compute_loop2
    unfold loop
    apply WP.spec_bind (sum_body_step it a hi (by omega))
    intro r hr
    obtain ⟨a', rfl, ha'⟩ := hr
    simp only []
    apply WP.spec_mono (sum_loop_wp { slice := it.slice, i := it.i + 1 } a' (by omega))
    intro out hout
    omega
  · have hsum : iterSum it = 0 := by
      unfold iterSum
      rw [List.drop_eq_nil_of_le (Nat.le_of_not_gt hi)]
      simp
    have hiLen : ¬it.i < it.slice.len := by simpa using hi
    unfold algo.Adler32.compute_loop2
    unfold loop
    simp [algo.Adler32.compute_loop2.body, core.slice.iter.IteratorSliceIter.next, hiLen, hi, hsum]
termination_by it.slice.val.length - it.i
decreasing_by omega
@[step]
theorem sum_loop3_wp (it : core.slice.iter.Iter U32) (a : U32)
    (hmax : a.val + iterSum it ≤ U32.max) :
    algo.Adler32.compute_loop3 it a ⦃ fun out => out.val = a.val + iterSum it ⦄ := by
  exact sum_loop_wp it a hmax
theorem values_sum_le (xs : List U32) (n : Nat) (h : ∀ x ∈ xs, x.val ≤ n) :
    (xs.map (fun x => x.val)).sum ≤ xs.length * n := by
  induction xs with
  | nil => simp
  | cons x xs ih =>
    have hx := h x (by simp)
    have ht : ∀ x ∈ xs, x.val ≤ n := by
      intro y hy
      exact h y (by simp [hy])
    have hs := ih ht
    simp only [List.map_cons, List.sum_cons, List.length_cons]
    nlinarith

theorem vector_sum_le (v : algo.U32X4) (n : Nat)
    (h : ∀ i : Nat, i < 4 → v.val[i]!.val ≤ n) :
    (v.val.map (fun x => x.val)).sum ≤ 4 * n := by
  have hs := values_sum_le v.val n (by
    intro x hx
    obtain ⟨i, hi, rfl⟩ := List.mem_iff_getElem.mp hx
    have hi4 : i < 4 := by have := Array.length_eq v; scalar_tac
    have hb := h i hi4
    simpa only [List.getElem!_eq_getElem?_getD, List.getElem?_eq_getElem hi, Option.getD_some] using hb)
  rw [Array.length_eq] at hs
  simpa using hs
end AdlerProof

namespace AdlerProof

def rawStep (s : Nat × Nat) (byte : U8) : Nat × Nat :=
  let a := s.1 + byte.val
  (a, s.2 + a)

def rawRun (bytes : List U8) (a b : Nat) : Nat × Nat := bytes.foldl rawStep (a, b)

theorem rawRun_seed_le (bytes : List U8) (a b : Nat) :
    a ≤ (rawRun bytes a b).1 ∧ b ≤ (rawRun bytes a b).2 := by
  induction bytes generalizing a b with
  | nil => simp [rawRun]
  | cons byte bytes ih =>
    have h := ih (a + byte.val) (b + (a + byte.val))
    simp only [rawRun, List.foldl_cons, rawStep] at *
    omega

theorem tail_body_step (it : core.slice.iter.Iter U8) (a b : U32)
    (hi : it.i < it.slice.val.length)
    (ha : a.val + it.slice.val[it.i]!.val ≤ U32.max)
    (hb : b.val + (a.val + it.slice.val[it.i]!.val) ≤ U32.max) :
    algo.Adler32.compute_loop4.body it a b ⦃ fun r => ∃ a' b',
      r = .cont ({ slice := it.slice, i := it.i + 1 }, a', b') ∧
      a'.val = a.val + it.slice.val[it.i]!.val ∧
      b'.val = b.val + (a.val + it.slice.val[it.i]!.val) ⦄ := by
  have hiLen : it.i < it.slice.len := by simpa using hi
  unfold algo.Adler32.compute_loop4.body core.slice.iter.IteratorSliceIter.next
  simp only [hiLen, _root_.dite_true, lift]
  step* +scalarTac
  all_goals simp_all
  all_goals grind

@[step]
theorem tail_loop_wp (it : core.slice.iter.Iter U8) (a b : U32)
    (ha : (rawRun (it.slice.val.drop it.i) a.val b.val).1 ≤ U32.max)
    (hb : (rawRun (it.slice.val.drop it.i) a.val b.val).2 ≤ U32.max) :
    algo.Adler32.compute_loop4 it a b ⦃ fun out =>
      (out.1.val, out.2.val) = rawRun (it.slice.val.drop it.i) a.val b.val ⦄ := by
  by_cases hi : it.i < it.slice.val.length
  · have hdrop : it.slice.val.drop it.i = it.slice.val[it.i]! :: it.slice.val.drop (it.i + 1) := by
      simpa only [List.getElem!_eq_getElem?_getD, List.getElem?_eq_getElem hi, Option.getD_some] using List.drop_eq_getElem_cons hi
    have hrun : rawRun (it.slice.val.drop it.i) a.val b.val =
        rawRun (it.slice.val.drop (it.i + 1)) (a.val + it.slice.val[it.i]!.val)
          (b.val + (a.val + it.slice.val[it.i]!.val)) := by rw [hdrop]; rfl
    have hle := rawRun_seed_le (it.slice.val.drop (it.i + 1))
      (a.val + it.slice.val[it.i]!.val) (b.val + (a.val + it.slice.val[it.i]!.val))
    rw [hrun] at ha hb
    unfold algo.Adler32.compute_loop4
    unfold loop
    apply WP.spec_bind (tail_body_step it a b hi (by omega) (by omega))
    intro r hr
    obtain ⟨a', b', rfl, ha', hb'⟩ := hr
    simp only []
    have hrec := tail_loop_wp { slice := it.slice, i := it.i + 1 } a' b'
      (by simpa only [ha', hb'] using ha) (by simpa only [ha', hb'] using hb)
    apply WP.spec_mono hrec
    intro out hout
    rw [ha', hb'] at hout
    exact hout.trans hrun.symm
  · have hdrop : it.slice.val.drop it.i = [] := List.drop_eq_nil_of_le (by omega)
    unfold algo.Adler32.compute_loop4
    unfold loop
    simp [algo.Adler32.compute_loop4.body, core.slice.iter.IteratorSliceIter.next, hi,
      rawRun, hdrop]
termination_by it.slice.val.length - it.i
decreasing_by omega
theorem rawRun_mod (bytes : List U8) (a b : Nat) :
    ((rawRun bytes a b).1 % 65521, (rawRun bytes a b).2 % 65521) = AdlerSpec.run a b bytes := by
  induction bytes generalizing a b with
  | nil => simp [rawRun, AdlerSpec.run]
  | cons byte bytes ih =>
    have h := ih (a + byte.val) (b + (a + byte.val))
    simpa [rawRun, rawStep, AdlerSpec.run, Nat.add_mod] using h

theorem rawRun_bounds (bytes : List U8) (a b : Nat) :
    (rawRun bytes a b).1 ≤ a + 255 * bytes.length ∧
    2 * (rawRun bytes a b).2 ≤ 2 * b + 2 * bytes.length * a + 255 * bytes.length * (bytes.length + 1) := by
  induction bytes generalizing a b with
  | nil => simp [rawRun]
  | cons byte bytes ih =>
    have hd : byte.val ≤ 255 := by scalar_tac
    have hm := Nat.mul_le_mul_left bytes.length hd
    have h := ih (a + byte.val) (b + (a + byte.val))
    simp only [rawRun, List.foldl_cons, rawStep, List.length_cons] at *
    constructor <;> nlinarith [h.1, h.2]

theorem tail_budget (bytes : List U8) (a b : Nat)
    (hn : bytes.length ≤ 3) (ha : a ≤ 400000) (hb : b ≤ 2000000) :
    (rawRun bytes a b).1 ≤ U32.max ∧ (rawRun bytes a b).2 ≤ U32.max := by
  have h := rawRun_bounds bytes a b
  have hm := Nat.mul_le_mul hn ha
  have ht := Nat.mul_le_mul hn (Nat.add_le_add_right hn 1)
  constructor
  · scalar_tac
  · have hmax : U32.max = 4294967295 := by scalar_tac
    rw [hmax]
    nlinarith [h.2]

@[step]
theorem tail_bounded_wp (it : core.slice.iter.Iter U8) (a b : U32)
    (hn : (it.slice.val.drop it.i).length ≤ 3)
    (ha : a.val ≤ 400000) (hb : b.val ≤ 2000000) :
    algo.Adler32.compute_loop4 it a b ⦃ fun out =>
      (out.1.val, out.2.val) = rawRun (it.slice.val.drop it.i) a.val b.val ⦄ := by
  have h := tail_budget (it.slice.val.drop it.i) a.val b.val hn ha hb
  exact tail_loop_wp it a b h.1 h.2
end AdlerProof

namespace AdlerProof
set_option maxHeartbeats 2000000

theorem rawRun_append (xs ys : List U8) (a b : Nat) :
    rawRun (xs ++ ys) a b = rawRun ys (rawRun xs a b).1 (rawRun xs a b).2 := by
  simp [rawRun, List.foldl_append]

theorem quartet_values (q : Slice U8) (h : q.val.length = 4) :
    q.val = [q.val[0]!, q.val[1]!, q.val[2]!, q.val[3]!] := by
  have hv := q.val
  generalize heq : q.val = xs at *
  match xs with
  | [a,b,c,d] => rfl
  | [] | [_] | [_,_] | [_,_,_] | _::_::_::_::_::_ => simp_all

theorem quartet_recombine (cs : List (Slice U8))
    (hfull : ∀ c ∈ cs, c.val.length = 4) (a b : Nat) :
    let s := rawRun (cs.map Slice.val).flatten a b
    let l := fun i => laneRun cs 0 0 i
    s.1 = a + (l 0).1 + (l 1).1 + (l 2).1 + (l 3).1 ∧
    s.2 + (l 1).1 + 2 * (l 2).1 + 3 * (l 3).1 =
      b + 4 * cs.length * a + 4 * ((l 0).2 + (l 1).2 + (l 2).2 + (l 3).2) := by
  induction cs using List.reverseRecOn with
  | nil => simp [rawRun, laneRun]
  | @append_singleton cs q ih =>
    have hcs : ∀ c ∈ cs, c.val.length = 4 := by
      intro c hc; apply hfull; simp [hc]
    have hq : q.val.length = 4 := hfull q (by simp)
    have h := ih hcs
    simp only [List.map_append, List.map_singleton, List.flatten_append, List.flatten_singleton,
      rawRun_append, laneRun_append, List.length_append, List.length_singleton]
    rw [quartet_values q hq]
    simp only [rawRun, laneRun, List.foldl_cons, List.foldl_nil, rawStep, laneStep]
    try simp only [List.getElem!_cons_zero, List.getElem!_cons_succ]
    dsimp only [rawRun, laneRun] at h
    constructor <;> nlinarith [h.1, h.2]


theorem quartet_mod_recombine (cs : List (Slice U8))
    (hfull : ∀ c ∈ cs, c.val.length = 4) (a b : Nat) :
    let l := fun i => laneRun cs 0 0 i
    let av := fun i => (l i).1 % 65521
    let bv := fun i => (l i).2 % 65521
    ((a + av 0 + av 1 + av 2 + av 3) % 65521,
      ((b + 4 * cs.length * a) % 65521 + 4 * (bv 0 + bv 1 + bv 2 + bv 3) +
        (65521 - av 1) + 2 * (65521 - av 2) + 3 * (65521 - av 3)) % 65521) =
      AdlerSpec.run a b (cs.map Slice.val).flatten := by
  dsimp only
  have h := quartet_recombine cs hfull a b
  have hm := rawRun_mod (cs.map Slice.val).flatten a b
  rw [← hm]
  apply Prod.ext
  · dsimp only
    rw [h.1]
    simp [Nat.add_mod]
  · dsimp only
    let l := fun i => laneRun cs 0 0 i
    let av := fun i => (l i).1 % 65521
    let bv := fun i => (l i).2 % 65521
    let base := (b + 4 * cs.length * a) % 65521
    let corrected := base + 4 * (bv 0 + bv 1 + bv 2 + bv 3) +
      (65521 - av 1) + 2 * (65521 - av 2) + 3 * (65521 - av 3)
    change corrected % 65521 = (rawRun (cs.map Slice.val).flatten a b).2 % 65521
    have hbound (i : Nat) : av i ≤ 65521 := by
      dsimp [av]; exact Nat.le_of_lt (Nat.mod_lt _ (by decide))
    have hc : corrected + (av 1 + 2 * av 2 + 3 * av 3) =
        base + 4 * (bv 0 + bv 1 + bv 2 + bv 3) + 6 * 65521 := by
      dsimp [corrected]
      have h1 := hbound 1
      have h2 := hbound 2
      have h3 := hbound 3
      omega
    have he : (corrected + (av 1 + 2 * av 2 + 3 * av 3)) % 65521 =
        ((rawRun (cs.map Slice.val).flatten a b).2 +
          (laneRun cs 0 0 1).1 + 2 * (laneRun cs 0 0 2).1 +
          3 * (laneRun cs 0 0 3).1) % 65521 := by
      rw [hc, h.2]
      simp [base, bv, l, Nat.add_mod, Nat.mul_mod]
    have hw : Nat.ModEq 65521 (av 1 + 2 * av 2 + 3 * av 3)
        ((laneRun cs 0 0 1).1 + 2 * (laneRun cs 0 0 2).1 + 3 * (laneRun cs 0 0 3).1) := by
      simp [Nat.ModEq, av, l, Nat.add_mod, Nat.mul_mod]
    have he' : Nat.ModEq 65521
        (corrected + (av 1 + 2 * av 2 + 3 * av 3))
        ((rawRun (cs.map Slice.val).flatten a b).2 +
          ((laneRun cs 0 0 1).1 + 2 * (laneRun cs 0 0 2).1 + 3 * (laneRun cs 0 0 3).1)) := by
      simpa [Nat.ModEq, Nat.add_assoc] using he
    exact hw.add_right_cancel he'


theorem vector_values (v : algo.U32X4) :
    v.val = [v.val[0]!, v.val[1]!, v.val[2]!, v.val[3]!] := by
  have h : v.val.length = 4 := by simpa using v.property
  generalize heq : v.val = xs at *
  match xs with
  | [a,b,c,d] => rfl
  | [] | [_] | [_,_] | [_,_,_] | _::_::_::_::_::_ => simp_all

theorem vector_iter_sum (v : algo.U32X4) :
    iterSum { slice := Array.to_slice v, i := 0 } =
      v.val[0]!.val + v.val[1]!.val + v.val[2]!.val + v.val[3]!.val := by
  have hv := vector_values v
  simp only [iterSum, List.drop_zero]
  simp only [Array.to_slice]
  simp
  conv_lhs => rw [hv]
  simp [Nat.add_assoc]
end AdlerProof

namespace AdlerProof
set_option maxHeartbeats 2000000

theorem run_congr (bytes : List U8) (a b c d : Nat)
    (ha : a % 65521 = c % 65521) (hb : b % 65521 = d % 65521) :
    AdlerSpec.run a b bytes = AdlerSpec.run c d bytes := by
  simp only [AdlerSpec.run, ha, hb]

theorem run_append (xs ys : List U8) (a b : Nat) :
    AdlerSpec.run a b (xs ++ ys) =
      AdlerSpec.run (AdlerSpec.run a b xs).1 (AdlerSpec.run a b xs).2 ys := by
  rw [← rawRun_mod, rawRun_append, rawRun_mod]
  apply run_congr
  · have h := congrArg Prod.fst (rawRun_mod xs a b)
    simpa using congrArg (fun x : Nat => x % 65521) h
  · have h := congrArg Prod.snd (rawRun_mod xs a b)
    simpa using congrArg (fun x : Nat => x % 65521) h

theorem combine_lane_residues (xs ys : List (Slice U8))
    (av bv av' bv' : algo.U32X4)
    (hseed : ∀ i : Nat, i < 4 →
      av.val[i]!.val = (laneRun xs 0 0 i).1 % 65521 ∧
      bv.val[i]!.val = (laneRun xs 0 0 i).2 % 65521)
    (hfinal : ∀ i : Nat, i < 4 →
      (av'.val[i]!.val, bv'.val[i]!.val) = laneRun ys av.val[i]!.val bv.val[i]!.val i) :
    ∀ i : Nat, i < 4 →
      av'.val[i]!.val % 65521 = (laneRun (xs ++ ys) 0 0 i).1 % 65521 ∧
      bv'.val[i]!.val % 65521 = (laneRun (xs ++ ys) 0 0 i).2 % 65521 := by
  intro i hi
  have hs := hseed i hi
  have hf := hfinal i hi
  have hc := laneRun_congr ys av.val[i]!.val bv.val[i]!.val
    (laneRun xs 0 0 i).1 (laneRun xs 0 0 i).2 i
    (by rw [hs.1]; exact Nat.mod_mod _ _) (by rw [hs.2]; exact Nat.mod_mod _ _)
  rw [laneRun_append]
  simpa only [← congrArg Prod.fst hf, ← congrArg Prod.snd hf] using hc
end AdlerProof

namespace AdlerProof

theorem from_checksum_wp (sum : U32) :
    Adler32.from_checksum sum ⦃ fun out =>
      out.a.val = sum.val % 65536 ∧ out.b.val = sum.val / 65536 ⦄ := by
  unfold Adler32.from_checksum
  simp only [lift]
  step* +scalarTac
end AdlerProof
