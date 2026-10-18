import ElicitContracts
open Aeneas Aeneas.Std Result ElicitGenerated
namespace AdlerProof

theorem checksum_correct : ElicitContracts.c_0a059b8c0885cf07 := by
  intro self _
  apply ElicitAdapter.spec_to_contract
  unfold Adler32.checksum
  simp only [lift]
  step*
  simp only [UScalar.val_or, core.convert.num.FromU32U16.from_val_eq] at *
  have ha := self.a.hmax
  have hb := self.b.hmax
  simp only [Nat.shiftLeft_eq, U32.size, U32.numBits] at *
  have hm : self.b.val * 65536 < 4294967296 := by scalar_tac
  norm_num at i1_post
  rw [Nat.mod_eq_of_lt hm] at i1_post
  rw [i1_post]
  have hlo : self.a.val < 2^16 := by simpa using ha
  have hor := Nat.shiftLeft_add_eq_or_of_lt hlo self.b.val
  simp only [Nat.shiftLeft_eq] at hor
  norm_num at hor
  rw [← hor]
  omega

elicit_spec checksum_spec from checksum_correct

theorem new_correct : ElicitContracts.c_8055bec449b87b3d := by
  intro _
  exact ⟨{a := 1#u16, b := 0#u16}, rfl, rfl, rfl⟩

elicit_spec new_spec from new_correct

theorem from_checksum_correct : ElicitContracts.c_9a31b7b9cb8492b0 := by
  intro sum _
  apply ElicitAdapter.spec_to_contract
  unfold Adler32.from_checksum
  simp only [lift]
  step*

elicit_spec from_checksum_spec from from_checksum_correct

theorem checksum_after_from_checksum (sum : U32) : ∃ state, Adler32.from_checksum sum = ok state ∧ state.checksum = ok sum := by
  obtain ⟨state, hs, ha, hb⟩ := from_checksum_correct sum trivial
  obtain ⟨out, ho, hv⟩ := checksum_correct state trivial
  have he : out = sum := by
    apply UScalar.eq_of_val_eq
    rw [hv, ha, hb]
    omega
  subst out
  exact ⟨state, hs, ho⟩

theorem from_checksum_after_checksum (state : Adler32) : ∃ sum, state.checksum = ok sum ∧ Adler32.from_checksum sum = ok state := by
  obtain ⟨sum, hs, hv⟩ := checksum_correct state trivial
  obtain ⟨out, ho, ha, hb⟩ := from_checksum_correct sum trivial
  have he : out = state := by
    have hA : out.a = state.a := by
      apply UScalar.eq_of_val_eq
      rw [ha, hv]
      have h := state.a.hmax
      norm_num at h
      omega
    have hB : out.b = state.b := by
      apply UScalar.eq_of_val_eq
      rw [hb, hv]
      have h := state.a.hmax
      norm_num at h
      omega
    cases out
    cases state
    exact congrArg₂ Adler32.mk hA hB
  subst out
  exact ⟨sum, hs, ho⟩

theorem lane_from_correct : ElicitContracts.c_0dfc1ce11e75ed00 := by
  intro bytes h
  apply ElicitAdapter.spec_to_contract
  unfold algo.U32X4.from
  simp only [lift]
  step*
  have hex : ∃ a b c d tail, bytes.val = a :: b :: c :: d :: tail := by
    match hx : bytes.val with
    | a :: b :: c :: d :: tail => exact ⟨a,b,c,d,tail,rfl⟩
    | [] => simp [hx] at h
    | [_] => simp [hx] at h
    | [_, _] => simp [hx] at h
    | [_, _, _] => simp [hx] at h
  obtain ⟨a,b,c,d,tail,hx⟩ := hex
  simp_all [Array.make]

elicit_spec lane_from_spec from lane_from_correct

theorem lane_add_correct : ElicitContracts.c_20b5198a3591154d := by
  intro self other h
  have h0 := h 0 (by decide)
  have h1 := h 1 (by decide)
  have h2 := h 2 (by decide)
  have h3 := h 3 (by decide)
  apply ElicitAdapter.spec_to_contract
  unfold algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step*
  all_goals simp_all [Array.set_val_eq, U32.max, U32.numBits]
  all_goals try (have := h 0 (by decide); have := h 1 (by decide); have := h 2 (by decide); have := h 3 (by decide); omega)
  intro j hj
  have hc : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases hc with rfl | rfl | rfl | rfl <;> simp_all

elicit_spec lane_add_spec from lane_add_correct

theorem lane_rem_correct : ElicitContracts.c_f321f3834d6c15c0 := by
  intro self other h
  apply ElicitAdapter.spec_to_contract
  unfold algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
  step*
  all_goals simp_all [Array.set_val_eq]
  intro j hj
  have hc : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases hc with rfl | rfl | rfl | rfl <;> simp_all

elicit_spec lane_rem_spec from lane_rem_correct

theorem lane_mul_correct : ElicitContracts.c_6c7496278e307fe7 := by
  intro self other h
  have h0 := h 0 (by decide)
  have h1 := h 1 (by decide)
  have h2 := h 2 (by decide)
  have h3 := h 3 (by decide)
  apply ElicitAdapter.spec_to_contract
  unfold algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
  step*
  all_goals simp_all [Array.set_val_eq, U32.max, U32.numBits]
  all_goals try (have := h 0 (by decide); have := h 1 (by decide); have := h 2 (by decide); have := h 3 (by decide); omega)
  intro j hj
  have hc : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases hc with rfl | rfl | rfl | rfl <;> simp_all

elicit_spec lane_mul_spec from lane_mul_correct

/-- Restoring a saved checksum preserves the exact translated continuation,
including its execution result, for every representable suffix slice. -/
theorem save_restore_resume (state : Adler32) (bytes : Slice U8) :
    ∃ sum, state.checksum = ok sum ∧
      (do let restored ← Adler32.from_checksum sum; restored.write_slice bytes) =
        state.write_slice bytes := by
  obtain ⟨sum, hs, hr⟩ := from_checksum_after_checksum state
  refine ⟨sum, hs, ?_⟩
  rw [hr]
  simp

end AdlerProof
