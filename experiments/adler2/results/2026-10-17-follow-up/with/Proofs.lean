import ElicitContracts
open Aeneas Aeneas.Std Result ElicitGenerated

namespace AdlerProof

private theorem spec_result {α : Type} (m : Result α) (p : α → Prop)
    (h : m ⦃ p ⦄) : ∃ r, m = ok r ∧ p r := by
  cases m using Result.cases <;> simp_all

/-- Original rem_assign body: successful execution and exact lane remainders. -/
theorem rem_assign_contract : ElicitContracts.c_f321f3834d6c15c0 := by
  unfold ElicitContracts.c_f321f3834d6c15c0
  intro self q hq
  apply spec_result
  change algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign self q
    ⦃ out => ∀ i : Nat, i < 4 → (out.val[i]!).val = (self.val[i]!).val % q.val ⦄
  unfold algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
  step*
  intro j hj
  have hcases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases hcases with h | h | h | h <;> subst j <;> simp_all

/-- Original mul_assign body, with each product required to fit in u32. -/
theorem mul_assign_contract : ElicitContracts.c_6c7496278e307fe7 := by
  unfold ElicitContracts.c_6c7496278e307fe7
  intro self rhs hsafe
  have h0 := hsafe 0 (by decide)
  have h1 := hsafe 1 (by decide)
  have h2 := hsafe 2 (by decide)
  have h3 := hsafe 3 (by decide)
  simp at h0 h1 h2 h3
  apply spec_result
  change algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign self rhs
    ⦃ out => ∀ i : Nat, i < 4 → (out.val[i]!).val = (self.val[i]!).val * rhs.val ⦄
  unfold algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
  step* by
    simp_all [U32.max_eq]
    have hb0 := hsafe 0 (by decide)
    have hb1 := hsafe 1 (by decide)
    have hb2 := hsafe 2 (by decide)
    have hb3 := hsafe 3 (by decide)
    omega
  intro j hj
  have hcases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases hcases with h | h | h | h <;> subst j <;> simp_all

/-- Original add_assign body, requiring every lane sum to fit in u32. -/
theorem add_assign_contract : ElicitContracts.c_20b5198a3591154d := by
  unfold ElicitContracts.c_20b5198a3591154d
  intro self other hsafe
  apply spec_result
  change algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign self other
    ⦃ out => ∀ i : Nat, i < 4 → (out.val[i]!).val = (self.val[i]!).val + (other.val[i]!).val ⦄
  unfold algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step* by
    simp_all [U32.max_eq]
    have hb0 := hsafe 0 (by decide)
    have hb1 := hsafe 1 (by decide)
    have hb2 := hsafe 2 (by decide)
    have hb3 := hsafe 3 (by decide)
    omega
  intro j hj
  have hcases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases hcases with h | h | h | h <;> subst j <;> simp_all

/-- Original default constructor succeeds and returns the Adler initial state. -/
theorem default_contract : ElicitContracts.c_2fa15a7d1ba5a684 := by
  unfold ElicitContracts.c_2fa15a7d1ba5a684
  intro _
  exact ⟨{ a := 1#u16, b := 0#u16 }, rfl, by decide⟩

/-- Checksum encoding covers every representable pair of u16 fields. -/
theorem checksum_contract : ElicitContracts.c_0a059b8c0885cf07 := by
  unfold ElicitContracts.c_0a059b8c0885cf07
  intro self _
  apply spec_result
  change Adler32.checksum self ⦃ out => out.val = 65536 * self.b.val + self.a.val ⦄
  unfold Adler32.checksum
  simp only [lift, bind_ok]
  step*
  have ha := U16.lt_succ_max self.a
  have hb := U16.lt_succ_max self.b
  have hshift : self.b.val <<< 16 < U32.size := by
    simp only [Nat.shiftLeft_eq, U32.size_eq]
    omega
  simp only [core.convert.num.FromU32U16.from_val_eq] at i1_post
  rw [UScalar.val_or, core.convert.num.FromU32U16.from_val_eq, i1_post,
    Nat.mod_eq_of_lt hshift, ← Nat.shiftLeft_add_eq_or_of_lt (i := 16) ha]
  simp [Nat.shiftLeft_eq, Nat.mul_comm]

/-- Decode every u32 without a normalization assumption. -/
theorem from_checksum_contract : ElicitContracts.c_9a31b7b9cb8492b0 := by
  unfold ElicitContracts.c_9a31b7b9cb8492b0
  intro sum _
  apply spec_result
  change Adler32.from_checksum sum
    ⦃ out => out.a.val = sum.val % 65536 ∧ out.b.val = sum.val / 65536 ⦄
  unfold Adler32.from_checksum
  simp only [lift, bind_ok]
  step*

/-- Decoding and encoding preserve every checksum word. -/
theorem checksum_decode_roundtrip (sum : U32) :
    ∃ state, Adler32.from_checksum sum = ok state ∧ Adler32.checksum state = ok sum := by
  obtain ⟨state, hd, ha, hb⟩ := from_checksum_contract sum trivial
  obtain ⟨out, hc, hv⟩ := checksum_contract state trivial
  have heq : out = sum := by
    apply UScalar.eq_of_val_eq
    rw [hv, ha, hb]
    omega
  exact ⟨state, hd, heq ▸ hc⟩

/-- Encoding and decoding preserve every representable state. -/
theorem decode_checksum_roundtrip (state : Adler32) :
    ∃ sum, Adler32.checksum state = ok sum ∧ Adler32.from_checksum sum = ok state := by
  obtain ⟨sum, hc, hv⟩ := checksum_contract state trivial
  obtain ⟨restored, hd, ha, hb⟩ := from_checksum_contract sum trivial
  have habound := U16.lt_succ_max state.a
  have heqa : restored.a = state.a := by
    apply UScalar.eq_of_val_eq
    rw [ha, hv]
    omega
  have heqb : restored.b = state.b := by
    apply UScalar.eq_of_val_eq
    rw [hb, hv]
    omega
  have heq : restored = state := by
    cases state
    cases restored
    congr
  exact ⟨sum, hc, heq ▸ hd⟩

/-- Widen exactly the first four bytes; longer slices are allowed. -/
theorem from_bytes_contract : ElicitContracts.c_0dfc1ce11e75ed00 := by
  unfold ElicitContracts.c_0dfc1ce11e75ed00
  intro bytes hlen
  have h0 : 0 < bytes.val.length := by omega
  have h1 : 1 < bytes.val.length := by omega
  have h2 : 2 < bytes.val.length := by omega
  have h3 : 3 < bytes.val.length := by omega
  apply spec_result
  change algo.U32X4.from bytes
    ⦃ out => ∀ i : Nat, i < 4 → (out.val[i]!).val = (bytes.val[i]!).val ⦄
  unfold algo.U32X4.from
  simp only [lift, bind_ok]
  step*
  intro j hj
  have hcases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases hcases with h | h | h | h <;> subst j <;> simp_all

/-- Explicit finish method obligation, without asserting Hasher trait laws. -/
theorem finish_contract : ElicitContracts.c_375e1cdaf1f944d2 := by
  unfold ElicitContracts.c_375e1cdaf1f944d2
  intro self _
  obtain ⟨word, hc, hv⟩ := checksum_contract self trivial
  refine ⟨core.convert.num.FromU64U32.from word, ?_, ?_⟩
  · unfold Adler32.Insts.CoreHashHasher.finish
    rw [hc]
    simp only [bind_ok]
  · simpa using hv

end AdlerProof
