import Generated.Adler2
open Aeneas Aeneas.Std Result
set_option maxHeartbeats 2000000
namespace Verification
open adler2
 theorem new_correct : Adler32.new = ok ⟨1#u16, 0#u16⟩ := by rfl

def encode (s : Adler32) : U32 :=
  ⟨(s.b.bv.setWidth 32 <<< 16) ||| s.a.bv.setWidth 32⟩
def decode (c : U32) : Adler32 :=
  ⟨⟨c.bv.setWidth 16⟩, ⟨(c.bv >>> 16).setWidth 16⟩⟩

theorem checksum_exec (s : Adler32) : s.checksum = ok (encode s) := by
  simp only [Adler32.checksum, lift, bind_ok]
  change (do let x ← UScalar.shiftLeft_IScalar (core.convert.num.FromU32U16.from s.b) 16#i32; ok (x ||| core.convert.num.FromU32U16.from s.a)) = _
  have h : UScalar.shiftLeft_IScalar (core.convert.num.FromU32U16.from s.b) 16#i32 =
      ok ⟨s.b.bv.setWidth 32 <<< 16⟩ := by rfl
  rw [h]
  simp only [bind_tc_ok, bind_ok]
  rfl

theorem from_checksum_exec (c : U32) : Adler32.from_checksum c = ok (decode c) := by
  simp only [Adler32.from_checksum, lift, bind_ok]
  have h : UScalar.shiftRight_IScalar c 16#i32 = ok ⟨c.bv >>> 16⟩ := by rfl
  change (do let x ← UScalar.shiftRight_IScalar c 16#i32; ok (Adler32.mk (UScalar.cast .U16 c) (UScalar.cast .U16 x))) = _
  rw [h]
  simp only [bind_tc_ok, bind_ok]
  rfl

theorem encode_val (s : Adler32) : (encode s).val = 65536 * s.b.val + s.a.val := by
  have ha : s.a.val < 65536 := by scalar_tac
  have hb : s.b.val < 65536 := by scalar_tac
  change (((s.b.bv.setWidth 32) <<< 16) ||| s.a.bv.setWidth 32).toNat = _
  simp only [BitVec.toNat_or, BitVec.toNat_shiftLeft, BitVec.toNat_setWidth, Nat.shiftLeft_eq]
  change ((s.b.val % 4294967296 * 65536) % 4294967296 ||| s.a.val % 4294967296) = _
  rw [Nat.mod_eq_of_lt (by omega), Nat.mod_eq_of_lt (by omega),
    Nat.mod_eq_of_lt (by omega)]
  have h := Nat.shiftLeft_add_eq_or_of_lt (i := 16) (by omega : s.a.val < 2^16) s.b.val
  simpa [Nat.shiftLeft_eq, Nat.mul_comm] using h.symm

theorem decode_val (c : U32) :
    (decode c).a.val = c.val % 65536 ∧ (decode c).b.val = c.val / 65536 := by
  have hc : c.val < 4294967296 := by scalar_tac
  change (c.bv.setWidth 16).toNat = _ ∧ ((c.bv >>> 16).setWidth 16).toNat = _
  simp only [BitVec.toNat_setWidth, BitVec.toNat_ushiftRight, Nat.shiftRight_eq_div_pow]
  change c.val % 65536 = _ ∧ (c.val / 65536) % 65536 = _
  constructor
  · rfl
  · apply Nat.mod_eq_of_lt; omega

theorem state_ext (s t : Adler32) (ha : s.a.val = t.a.val) (hb : s.b.val = t.b.val) : s = t := by
  have ha' := UScalar.eq_imp s.a t.a ha
  have hb' := UScalar.eq_imp s.b t.b hb
  cases s
  cases t
  cases ha'
  cases hb'
  rfl

theorem codec_state (s : Adler32) : decode (encode s) = s := by
  have hd := decode_val (encode s)
  rw [encode_val] at hd
  have ha : s.a.val < 65536 := by scalar_tac
  apply state_ext
  · omega
  · omega

theorem codec_word (c : U32) : encode (decode c) = c := by
  apply UScalar.eq_imp
  rw [encode_val, (decode_val c).1, (decode_val c).2]
  omega

 theorem checksum_correct (s : Adler32) :
    ∃ c, s.checksum = ok c ∧ c.val = 65536 * s.b.val + s.a.val :=
  ⟨encode s, checksum_exec s, encode_val s⟩
 theorem from_checksum_correct (c : U32) :
    ∃ s, Adler32.from_checksum c = ok s ∧
      s.a.val = c.val % 65536 ∧ s.b.val = c.val / 65536 :=
  ⟨decode c, from_checksum_exec c, decode_val c⟩
end Verification
