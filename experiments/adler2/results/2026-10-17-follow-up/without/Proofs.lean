import Adler2
open Aeneas Aeneas.Std Result
set_option maxRecDepth 2048
namespace Verification

theorem new_correct : adler2.Adler32.new = ok ⟨1#u16, 0#u16⟩ := by rfl

def decode (s : U32) : adler2.Adler32 :=
  ⟨s.cast .U16, ⟨(s.bv >>> 16).setWidth 16⟩⟩

theorem from_checksum_eq (s : U32) :
    adler2.Adler32.from_checksum s = ok (decode s) := by
  simp only [adler2.Adler32.from_checksum, lift, bind_ok]
  have hs : (s >>> 16#i32) = ok (⟨s.bv >>> 16⟩ : U32) := by rfl
  rw [hs]
  simp only [bind_ok]
  rfl

theorem from_checksum_correct (s : U32) :
    ∃ t, adler2.Adler32.from_checksum s = ok t ∧
      t.a.val = s.val % 65536 ∧ t.b.val = s.val / 65536 := by
  refine ⟨decode s, from_checksum_eq s, ?_, ?_⟩
  · simp [decode, UScalar.cast_val_eq]
  · change ((s.bv >>> 16).setWidth 16).toNat = s.bv.toNat / 65536
    simp only [BitVec.toNat_setWidth, BitVec.toNat_ushiftRight, Nat.shiftRight_eq_div_pow]
    have h := s.bv.isLt
    change s.bv.toNat / 65536 % 65536 = s.bv.toNat / 65536
    omega

def encode (t : adler2.Adler32) : U32 :=
  ⟨(t.b.bv.setWidth 32 <<< 16) ||| t.a.bv.setWidth 32⟩

theorem checksum_eq (t : adler2.Adler32) :
    adler2.Adler32.checksum t = ok (encode t) := by
  simp only [adler2.Adler32.checksum, lift, bind_ok]
  have hs : (core.convert.num.FromU32U16.from t.b <<< 16#i32) =
      ok (⟨t.b.bv.setWidth 32 <<< 16⟩ : U32) := by rfl
  rw [hs]
  simp only [bind_ok]
  rfl

theorem encode_val (t : adler2.Adler32) :
    (encode t).val = 65536 * t.b.val + t.a.val := by
  change ((t.b.bv.setWidth 32 <<< 16) ||| t.a.bv.setWidth 32).toNat = _
  simp only [BitVec.toNat_or, BitVec.toNat_shiftLeft, BitVec.toNat_setWidth,
    Nat.shiftLeft_eq]
  have ha := t.a.bv.isLt
  have hb := t.b.bv.isLt
  change t.a.bv.toNat < 65536 at ha
  change t.b.bv.toNat < 65536 at hb
  rw [Nat.mod_eq_of_lt (by omega : t.b.bv.toNat < 2^32),
      Nat.mod_eq_of_lt (by omega : t.a.bv.toNat < 2^32)]
  rw [Nat.mod_eq_of_lt (by omega : t.b.bv.toNat * 2^16 < 2^32)]
  have hor := Nat.shiftLeft_add_eq_or_of_lt (i := 16) ha t.b.bv.toNat
  simp only [Nat.shiftLeft_eq] at hor
  rw [← hor]
  change t.b.val * 65536 + t.a.val = _
  exact congrArg (fun n => n + t.a.val) (Nat.mul_comm _ _)

theorem checksum_correct (t : adler2.Adler32) :
    ∃ s, adler2.Adler32.checksum t = ok s ∧ s.val = 65536*t.b.val+t.a.val :=
  ⟨encode t, checksum_eq t, encode_val t⟩

theorem codec_decode_encode (t : adler2.Adler32) : decode (encode t) = t := by
  have h := from_checksum_correct (encode t)
  rcases h with ⟨u, hu, ha, hb⟩
  rw [from_checksum_eq] at hu
  have he : decode (encode t) = u := by exact Result.ok_injective hu
  rw [← he, encode_val] at ha hb
  have hA := t.a.bv.isLt
  have hB := t.b.bv.isLt
  change t.a.val < 65536 at hA
  change t.b.val < 65536 at hB
  have hda : (decode (encode t)).a = t.a := by
    apply UScalar.eq_of_val_eq
    omega
  have hdb : (decode (encode t)).b = t.b := by
    apply UScalar.eq_of_val_eq
    omega
  have ext (v w : adler2.Adler32) (ha : v.a = w.a) (hb : v.b = w.b) : v = w := by
    cases v
    cases w
    simp only at ha hb
    cases ha
    cases hb
    rfl
  exact ext _ _ hda hdb

theorem codec_encode_decode (s : U32) : encode (decode s) = s := by
  rcases from_checksum_correct s with ⟨t, ht, ha, hb⟩
  rw [from_checksum_eq] at ht
  have he : decode s = t := Result.ok_injective ht
  apply UScalar.eq_of_val_eq
  rw [encode_val, he, ha, hb]
  omega

theorem checksum_save_restore (t : adler2.Adler32) :
    (do let s ← adler2.Adler32.checksum t; adler2.Adler32.from_checksum s) = ok t := by
  rw [checksum_eq]
  simp only [bind_ok, from_checksum_eq, codec_decode_encode]

theorem checksum_restore_save (s : U32) :
    (do let t ← adler2.Adler32.from_checksum s; adler2.Adler32.checksum t) = ok s := by
  rw [from_checksum_eq]
  simp only [bind_ok, checksum_eq, codec_encode_decode]

/-- Save/restore preserves any subsequent call, without assuming that it succeeds. -/
theorem save_restore_resume (t : adler2.Adler32) (xs : Slice U8) :
    (do let s ← adler2.Adler32.checksum t
        let u ← adler2.Adler32.from_checksum s
        adler2.Adler32.write_slice u xs) = adler2.Adler32.write_slice t xs := by
  rw [checksum_eq]
  simp only [bind_ok, from_checksum_eq, codec_decode_encode]

#print axioms new_correct
#print axioms from_checksum_correct
#print axioms checksum_correct
#print axioms checksum_save_restore
#print axioms checksum_restore_save
#print axioms save_restore_resume
end Verification
