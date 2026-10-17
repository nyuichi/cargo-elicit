import ElicitContracts
open Aeneas Aeneas.Std Result ElicitGenerated

namespace AdlerProof
set_option maxRecDepth 4096
set_option maxHeartbeats 800000

theorem rem_assign_correct : ElicitContracts.c_f321f3834d6c15c0 := by
  intro before divisor hnz
  have h : (algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign before divisor)
      ⦃ after => after.val.map (fun x => x.val) =
          before.val.map (fun x => x.val % divisor.val) ⦄ := by
    unfold algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
    step*
    subst_vars
    simp_all [Array.set, Array.from_val]
    apply List.ext_getElem
    · simp
    · intro j hj hk
      have hj4 : j < 4 := by simpa using hk
      have hc : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
      rcases hc with hc | hc | hc | hc <;> subst j <;> simp_all
  generalize heq : algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign before divisor = m at h ⊢
  cases m using Result.cases with
  | ret r => exact ⟨r, rfl, (WP.spec_ok r).mp h⟩
  | div => exact False.elim ((WP.spec_div).mp h)
  | vis e k => exact False.elim ((WP.spec_vis e k).mp h)

theorem mul_assign_correct : ElicitContracts.c_6c7496278e307fe7 := by
  intro before rhs hsafe
  have h0 := hsafe before.val[0] (List.getElem_mem (by simp))
  have h1 := hsafe before.val[1] (List.getElem_mem (by simp))
  have h2 := hsafe before.val[2] (List.getElem_mem (by simp))
  have h3 := hsafe before.val[3] (List.getElem_mem (by simp))
  have h : (algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign before rhs)
      ⦃ after => after.val.map (fun x => x.val) =
          before.val.map (fun x => x.val * rhs.val) ⦄ := by
    unfold algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
    step*
    all_goals try (subst_vars; simp_all [Array.set, Array.from_val])
    all_goals try (have hbound := hsafe before.val[1] (List.getElem_mem (by simp)); scalar_tac)
    all_goals try (have hbound := hsafe before.val[2] (List.getElem_mem (by simp)); scalar_tac)
    all_goals try (have hbound := hsafe before.val[3] (List.getElem_mem (by simp)); scalar_tac)
    subst_vars
    try simp_all [Array.set, Array.from_val]
    apply List.ext_getElem
    · simp
    · intro j hj hk
      have hj4 : j < 4 := by simpa using hk
      have hc : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
      rcases hc with hc | hc | hc | hc <;> subst j <;> simp_all
  generalize heq : algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign before rhs = m at h ⊢
  cases m using Result.cases with
  | ret r => exact ⟨r, rfl, (WP.spec_ok r).mp h⟩
  | div => exact False.elim ((WP.spec_div).mp h)
  | vis e k => exact False.elim ((WP.spec_vis e k).mp h)

theorem add_assign_correct : ElicitContracts.c_20b5198a3591154d := by
  intro before other hsafe
  have h0 := hsafe 0 (by simp) (by simp)
  have h1 := hsafe 1 (by simp) (by simp)
  have h2 := hsafe 2 (by simp) (by simp)
  have h3 := hsafe 3 (by simp) (by simp)
  have h : (algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign before other)
      ⦃ after => after.val.map (fun x => x.val) =
          List.zipWith (fun x y => x.val + y.val) before.val other.val ⦄ := by
    unfold algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
    step*
    all_goals try (subst_vars; simp_all [Array.set, Array.from_val])
    all_goals try (have hbound := hsafe 1 (by omega); scalar_tac)
    all_goals try (have hbound := hsafe 2 (by omega); scalar_tac)
    all_goals try (have hbound := hsafe 3 (by omega); scalar_tac)
    subst_vars
    try simp_all [Array.set, Array.from_val]
    apply List.ext_getElem
    · simp
    · intro j hj hk
      have hj4 : j < 4 := by simpa using hk
      have hc : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
      rcases hc with hc | hc | hc | hc <;> subst j <;> simp_all
  generalize heq : algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign before other = m at h ⊢
  cases m using Result.cases with
  | ret r => exact ⟨r, rfl, (WP.spec_ok r).mp h⟩
  | div => exact False.elim ((WP.spec_div).mp h)
  | vis e k => exact False.elim ((WP.spec_vis e k).mp h)

theorem default_correct : ElicitContracts.c_2fa15a7d1ba5a684 := by
  intro _
  exact ⟨{ a := 1#u16, b := 0#u16 }, rfl, rfl, rfl⟩

theorem checksum_correct : ElicitContracts.c_0a059b8c0885cf07 := by
  intro before _
  have h : Adler32.checksum before ⦃ result =>
      result.val = 65536 * before.b.val + before.a.val ⦄ := by
    unfold Adler32.checksum
    simp only [lift, bind_tc_ok]
    step*
    have ha : before.a.val < 65536 := before.a.bv.isLt
    have hb : before.b.val < 65536 := before.b.bv.isLt
    simp only [UScalar.val_or, i1_post, core.convert.num.FromU32U16.from_val_eq,
      Nat.shiftLeft_eq]
    have hmod : before.b.val * 2 ^ 16 < U32.size := by scalar_tac
    rw [Nat.mod_eq_of_lt hmod]
    simpa [Nat.mul_comm] using
      (Nat.two_pow_add_eq_or_of_lt (i := 16) ha before.b.val).symm
  generalize heq : Adler32.checksum before = m at h ⊢
  cases m using Result.cases with
  | ret r => exact ⟨r, rfl, (WP.spec_ok r).mp h⟩
  | div => exact False.elim ((WP.spec_div).mp h)
  | vis e k => exact False.elim ((WP.spec_vis e k).mp h)

theorem from_checksum_correct : ElicitContracts.c_9a31b7b9cb8492b0 := by
  intro sum _
  have h : Adler32.from_checksum sum ⦃ after =>
      after.a.val = sum.val % 65536 ∧ after.b.val = sum.val / 65536 ⦄ := by
    unfold Adler32.from_checksum
    simp only [lift]
    step*
  generalize heq : Adler32.from_checksum sum = m at h ⊢
  cases m using Result.cases with
  | ret r => exact ⟨r, rfl, (WP.spec_ok r).mp h⟩
  | div => exact False.elim ((WP.spec_div).mp h)
  | vis e k => exact False.elim ((WP.spec_vis e k).mp h)

/-- Decoding and re-encoding preserves every input checksum. -/
theorem checksum_codec_roundtrip (sum : U32) :
    ∃ state, Adler32.from_checksum sum = ok state ∧ Adler32.checksum state = ok sum := by
  obtain ⟨state, hdecode, ha, hb⟩ := from_checksum_correct sum trivial
  obtain ⟨result, hencode, hval⟩ := checksum_correct state trivial
  have heq : result = sum := UScalar.val_eq_imp _ _ (by rw [hval, ha, hb]; omega)
  exact ⟨state, hdecode, heq ▸ hencode⟩

/-- Saving and restoring preserves both representable state halves. -/
theorem state_codec_roundtrip (state : Adler32) :
    ∃ sum restored, Adler32.checksum state = ok sum ∧
      Adler32.from_checksum sum = ok restored ∧ restored = state := by
  obtain ⟨sum, hencode, hval⟩ := checksum_correct state trivial
  obtain ⟨restored, hdecode, ha, hb⟩ := from_checksum_correct sum trivial
  have habound : state.a.val < 65536 := state.a.bv.isLt
  have hbbound : state.b.val < 65536 := state.b.bv.isLt
  have hea : restored.a = state.a := UScalar.val_eq_imp _ _ (by omega)
  have heb : restored.b = state.b := UScalar.val_eq_imp _ _ (by omega)
  have heq : restored = state := by
    cases restored
    cases state
    simp_all
  exact ⟨sum, restored, hencode, hdecode, heq⟩

/-- Save/restore leaves the original update computation unchanged, including its outcome.
    Successful recurrence correctness of write_slice is a separate obligation. -/
theorem save_restore_resume (state : Adler32) (bytes : Slice U8) :
    ∃ sum restored, Adler32.checksum state = ok sum ∧
      Adler32.from_checksum sum = ok restored ∧
      Adler32.write_slice restored bytes = Adler32.write_slice state bytes := by
  obtain ⟨sum, restored, hencode, hdecode, heq⟩ := state_codec_roundtrip state
  exact ⟨sum, restored, hencode, hdecode, heq ▸ rfl⟩

theorem vector_from_correct : ElicitContracts.c_0dfc1ce11e75ed00 := by
  intro bytes hlen
  have h : algo.U32X4.from bytes ⦃ after =>
      ∀ i (hi : i < after.val.length) (hj : i < bytes.val.length),
        after.val[i].val = bytes.val[i].val ⦄ := by
    unfold algo.U32X4.from
    simp only [lift]
    step*
    subst_vars
    simp_all [Array.make, Array.from_val, core.convert.num.FromU32U8.from_val_eq]
    intro j hj hk
    have hc : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
    rcases hc with hc | hc | hc | hc <;> subst j <;> simp_all
  generalize heq : algo.U32X4.from bytes = m at h ⊢
  cases m using Result.cases with
  | ret r => exact ⟨r, rfl, (WP.spec_ok r).mp h⟩
  | div => exact False.elim ((WP.spec_div).mp h)
  | vis e k => exact False.elim ((WP.spec_vis e k).mp h)

theorem finish_correct : ElicitContracts.c_375e1cdaf1f944d2 := by
  intro before _
  obtain ⟨sum, hsum, hval⟩ := checksum_correct before trivial
  refine ⟨core.convert.num.FromU64U32.from sum, ?_, ?_⟩
  · simp [Adler32.Insts.CoreHashHasher.finish, hsum]
  · simpa only [core.convert.num.FromU64U32.from_val_eq] using hval

theorem new_correct : ElicitContracts.c_8055bec449b87b3d := by
  intro h
  exact default_correct h

end AdlerProof
