import Specs
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Codec
@[step] theorem new_spec : Adler32.new ⦃ out => out.a.val = 1 ∧ out.b.val = 0 ⦄ := by
  simp [Adler32.new, Adler32.Insts.CoreDefaultDefault.default]
@[step] theorem checksum_spec (self : Adler32) :
    self.checksum ⦃ out => out.val = 65536 * self.b.val + self.a.val ⦄ := by
  unfold Adler32.checksum
  simp only [lift]
  step*
  have hb : i1.val = self.b.val <<< 16 := by
    rw [i1_post, core.convert.num.FromU32U16.from_val_eq]
    apply Nat.mod_eq_of_lt
    simp only [Nat.shiftLeft_eq]
    scalar_tac
  rw [UScalar.val_or, hb, core.convert.num.FromU32U16.from_val_eq]
  have ha : self.a.val < 2^16 := self.a.bv.isLt
  rw [← Nat.shiftLeft_add_eq_or_of_lt ha]
  simp only [Nat.shiftLeft_eq]
  omega
@[step] theorem from_checksum_spec (sum : U32) :
    Adler32.from_checksum sum ⦃ out => out.a.val = sum.val % 65536 ∧ out.b.val = sum.val / 65536 ⦄ := by
  unfold Adler32.from_checksum
  step*
end Codec
