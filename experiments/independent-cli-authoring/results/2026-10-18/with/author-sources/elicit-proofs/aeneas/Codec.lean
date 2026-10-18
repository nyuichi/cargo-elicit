import ElicitContracts
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Proofs

theorem new_spec : ElicitContracts.c_8055bec449b87b3d := by
  intro _
  exact ⟨⟨1#u16, 0#u16⟩, rfl, rfl, rfl⟩

theorem from_checksum_spec : ElicitContracts.c_9a31b7b9cb8492b0 := by
  intro sum _
  apply ElicitAdapter.spec_to_contract
  unfold Adler32.from_checksum
  step*

theorem checksum_spec : ElicitContracts.c_0a059b8c0885cf07 := by
  intro s _
  apply ElicitAdapter.spec_to_contract
  unfold Adler32.checksum
  simp only [lift, bind_ok]
  step*
  simp only [UScalar.val_or, core.convert.num.FromU32U16.from_val_eq,
    Specs.encode, Specs.state] at *
  have ha : s.a.val < 2^16 := by scalar_tac
  have hb : s.b.val < 65536 := by scalar_tac
  rw [i1_post]
  simp only [Nat.shiftLeft_eq, U32.size, U32.numBits, UScalarTy.numBits]
  rw [Nat.mod_eq_of_lt (by omega)]
  simpa [Nat.mul_comm] using (Nat.two_pow_add_eq_or_of_lt ha s.b.val).symm

end Proofs
