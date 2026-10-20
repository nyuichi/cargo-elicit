import Codec
import Recurrence
import ElicitAdapter
open Aeneas Aeneas.Std Result ElicitGenerated
namespace CodecLaws
 theorem state_inj {s t : Adler32} (h : AdlerSpec.state s = AdlerSpec.state t) : s = t := by
  cases s with
  | mk a b =>
    cases t with
    | mk c d =>
      have ha : a = c := UScalar.eq_of_val_eq (congrArg Prod.fst h)
      have hb : b = d := UScalar.eq_of_val_eq (congrArg Prod.snd h)
      subst c; subst d; rfl
@[step] theorem encode_decode (code : U32) :
    (do let s ← Adler32.from_checksum code; s.checksum) ⦃ out => out = code ⦄ := by
  step as ⟨s, ha, hb⟩
  step as ⟨out, hout⟩
  apply UScalar.eq_of_val_eq
  rw [hout, ha, hb]
  omega
@[step] theorem decode_encode (s : Adler32) :
    (do let code ← s.checksum; Adler32.from_checksum code) ⦃ out => out = s ⦄ := by
  step as ⟨code, hcode⟩
  step as ⟨out, ha, hb⟩
  apply state_inj
  simp only [AdlerSpec.state]
  have hsa : s.a.val < 65536 := s.a.bv.isLt
  have hsb : s.b.val < 65536 := s.b.bv.isLt
  apply Prod.ext <;> omega
 theorem save_restore_resume (s : Adler32) (bytes : Slice U8) :
    (do let code ← s.checksum
        let restored ← Adler32.from_checksum code
        restored.write_slice bytes) = s.write_slice bytes := by
  obtain ⟨code, hcode, hp⟩ := ElicitAdapter.spec_to_contract (Codec.checksum_spec s)
  obtain ⟨restored, hrestored, ha, hb⟩ := ElicitAdapter.spec_to_contract (Codec.from_checksum_spec code)
  have heq : restored = s := by
    apply state_inj
    have hsa : s.a.val < 65536 := s.a.bv.isLt
    have hsb : s.b.val < 65536 := s.b.bv.isLt
    apply Prod.ext <;> dsimp [AdlerSpec.state] <;> omega
  simp [hcode, hrestored, heq]
end CodecLaws
