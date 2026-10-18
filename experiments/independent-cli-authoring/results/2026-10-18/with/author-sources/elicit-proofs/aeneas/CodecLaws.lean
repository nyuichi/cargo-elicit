import Codec
import Math
open Aeneas Aeneas.Std Result ElicitGenerated AdlerMath
namespace Proofs

theorem state_injective (s t : Adler32) (h : Specs.state s = Specs.state t) : s = t := by
  have ha := UScalar.eq_of_val_eq (congrArg Prod.fst h)
  have hb := UScalar.eq_of_val_eq (congrArg Prod.snd h)
  cases s; cases t
  simp only [Adler32.mk.injEq]
  exact ⟨ha,hb⟩

theorem decode_encode (s : Adler32) :
    (do let c ← Adler32.checksum s; Adler32.from_checksum c) = ok s := by
  obtain ⟨c,hc,hcv⟩ := checksum_spec s trivial
  obtain ⟨t,ht,hta,htb⟩ := from_checksum_spec c trivial
  rw [hc, bind_ok, ht]
  congr 1
  apply state_injective
  apply Prod.ext <;> simp only [Specs.state]
  · rw [hta, hcv]
    simp only [Specs.encode, Specs.state]
    have : s.a.val < 65536 := by scalar_tac
    omega
  · rw [htb, hcv]
    simp only [Specs.encode, Specs.state]
    have : s.a.val < 65536 := by scalar_tac
    omega

theorem encode_decode (c : U32) :
    (do let s ← Adler32.from_checksum c; Adler32.checksum s) = ok c := by
  obtain ⟨s,hs,hsa,hsb⟩ := from_checksum_spec c trivial
  obtain ⟨d,hd,hdv⟩ := checksum_spec s trivial
  rw [hs, bind_ok, hd]
  congr 1
  apply UScalar.eq_of_val_eq
  rw [hdv]
  simp only [Specs.encode, Specs.state, hsa, hsb]
  omega

theorem scalar_update_append (s : Nat × Nat) (xs ys : List U8) :
    Specs.update s (xs ++ ys) = Specs.update (Specs.update s xs) ys := by
  apply castState_inj (update_normalized _ _) (update_normalized _ _)
  simp only [cast_update, scalar, List.foldl_append]

end Proofs
