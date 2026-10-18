import Adler2
open Aeneas Aeneas.Std Result
namespace AdlerProof
set_option maxHeartbeats 2000000

theorem new_correct : adler2.Adler32.new = ok ⟨1#u16, 0#u16⟩ := by rfl

theorem shifted_half (b : U16) : (b.bv.setWidth 32 <<< 16).toNat = b.val * 65536 := by
  rw [BitVec.toNat_shiftLeft, BitVec.toNat_setWidth]
  simp only [Nat.shiftLeft_eq]
  have h := b.hmax
  change b.bv.toNat < 65536 at h
  norm_num only [Nat.reducePow]
  rw [Nat.mod_eq_of_lt (by omega : b.bv.toNat < 4294967296)]
  rw [Nat.mod_eq_of_lt (by omega : b.bv.toNat * 65536 < 4294967296)]
  rfl

theorem checksum_correct (s : adler2.Adler32) :
    ∃ c, adler2.Adler32.checksum s = ok c ∧ c.val = 65536 * s.b.val + s.a.val := by
  simp only [adler2.Adler32.checksum, HShiftLeft.hShiftLeft,
    UScalar.shiftLeft_IScalar, UScalar.shiftLeft]
  simp [IScalar.toNat, lift]
  change ((s.b.bv.setWidth 32 <<< 16).toNat ||| s.a.val) = _
  rw [shifted_half]
  have h := Nat.shiftLeft_add_eq_or_of_lt (i := 16) s.a.hmax s.b.val
  simp only [Nat.shiftLeft_eq] at h
  rw [← h]
  omega

theorem from_checksum_correct (c : U32) :
    ∃ s, adler2.Adler32.from_checksum c = ok s ∧
      s.a.val = c.val % 65536 ∧ s.b.val = c.val / 65536 := by
  simp only [adler2.Adler32.from_checksum, HShiftRight.hShiftRight,
    UScalar.shiftRight_IScalar, UScalar.shiftRight]
  simp [IScalar.toNat, UScalar.cast, lift]
  change (c.bv.setWidth 16).toNat = c.bv.toNat % 65536 ∧
    ((c.bv >>> 16).setWidth 16).toNat = c.bv.toNat / 65536
  simp only [BitVec.toNat_setWidth, BitVec.toNat_ushiftRight]
  have h := c.hmax
  change c.bv.toNat < 4294967296 at h
  constructor
  · trivial
  · change (c.bv.toNat >>> 16) % 65536 = c.bv.toNat / 65536
    rw [Nat.shiftRight_eq_div_pow]
    exact Nat.mod_eq_of_lt (by omega)
