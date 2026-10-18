import Api
import SpecReview
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Proofs

/-- The combined list must itself be representable as a Rust slice. -/
theorem split_update (s : Adler32) (xs ys : Slice U8)
    (representable : (xs.val ++ ys.val).length ≤ Usize.max) :
    (do let t ← Adler32.write_slice s xs; Adler32.write_slice t ys) =
      Adler32.write_slice s (Slice.from (xs.val ++ ys.val) representable) := by
  obtain ⟨t,ht,htv⟩ := write_slice_spec s xs trivial
  obtain ⟨u,hu,huv⟩ := write_slice_spec t ys trivial
  obtain ⟨v,hv,hvv⟩ := write_slice_spec s (Slice.from (xs.val ++ ys.val) representable) trivial
  rw [ht, bind_ok, hu, hv]
  congr 1
  apply state_injective
  rw [huv, htv, hvv]
  simp only [Slice.from_val]
  exact (scalar_update_append _ _ _).symm

/-- Saving and restoring the checksum preserves every representable state. -/
theorem save_restore_resume (s : Adler32) (bytes : Slice U8) :
    (do let c ← Adler32.checksum s
        let t ← Adler32.from_checksum c
        Adler32.write_slice t bytes) = Adler32.write_slice s bytes := by
  obtain ⟨c,hc,_⟩ := checksum_spec s trivial
  obtain ⟨t,ht,_⟩ := from_checksum_spec c trivial
  have he := decode_encode s
  rw [hc, bind_ok, ht] at he
  have hs : t = s := Result.ok_injective he
  simp only [hc, bind_ok, ht, hs]

/-- Saving between pieces agrees with a single combined update, when representable. -/
theorem split_save_restore_resume (s : Adler32) (xs ys : Slice U8)
    (representable : (xs.val ++ ys.val).length ≤ Usize.max) :
    (do let t ← Adler32.write_slice s xs
        let c ← Adler32.checksum t
        let restored ← Adler32.from_checksum c
        Adler32.write_slice restored ys) =
      Adler32.write_slice s (Slice.from (xs.val ++ ys.val) representable) := by
  obtain ⟨t,ht,_⟩ := write_slice_spec s xs trivial
  have he := split_update s xs ys representable
  rw [ht, bind_ok] at he ⊢
  rw [save_restore_resume]
  exact he

end Proofs
