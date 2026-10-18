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
