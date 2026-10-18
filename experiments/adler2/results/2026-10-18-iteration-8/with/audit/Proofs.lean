import Lemmas
open Aeneas Aeneas.Std ElicitGenerated

theorem adler32_new_correct : ElicitContracts.c_8055bec449b87b3d := by
  intro _
  exact ⟨⟨1#u16, 0#u16⟩, rfl, rfl, rfl⟩

elicit_spec adler32_new_spec from adler32_new_correct

private theorem checksum_execution (self : Adler32) :
    Adler32.checksum self ⦃ fun out => out.val = 65536 * self.b.val + self.a.val ⦄ := by
  unfold Adler32.checksum
  simp only [lift]
  step*
  have ha : self.a.val < 65536 := by scalar_tac
  have hb : self.b.val < 65536 := by scalar_tac
  simp only [UScalar.val_or, i1_post, core.convert.num.FromU32U16.from_val_eq,
    Nat.shiftLeft_eq, U32.size, U32.numBits]
  norm_num
  rw [Nat.mod_eq_of_lt (by omega)]
  simpa [Nat.mul_comm] using (Nat.two_pow_add_eq_or_of_lt (i := 16) ha self.b.val).symm

theorem adler32_checksum_correct : ElicitContracts.c_0a059b8c0885cf07 := by
  intro self _
  exact ElicitAdapter.spec_to_contract (checksum_execution self)

elicit_spec adler32_checksum_spec from adler32_checksum_correct
