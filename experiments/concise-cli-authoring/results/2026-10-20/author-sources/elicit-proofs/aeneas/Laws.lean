import API
import CodecLaws
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Laws
/-- The length condition makes the concatenated Rust slice representable. -/
def concat (xs ys : Slice U8) (h : xs.val.length + ys.val.length ≤ Usize.max) : Slice U8 :=
  Slice.from (xs.val ++ ys.val) (by simpa using h)
 theorem split_update (s : Adler32) (xs ys : Slice U8)
    (h : xs.val.length + ys.val.length ≤ Usize.max) :
    ∃ out, s.write_slice (concat xs ys h) = ok out ∧
      (do let mid ← s.write_slice xs; mid.write_slice ys) = ok out := by
  obtain ⟨whole, hw, hpw⟩ := ElicitAdapter.spec_to_contract (API.write_correct s (concat xs ys h))
  obtain ⟨mid, hm, hpm⟩ := ElicitAdapter.spec_to_contract (API.write_correct s xs)
  obtain ⟨out, ho, hpo⟩ := ElicitAdapter.spec_to_contract (API.write_correct mid ys)
  have heq : whole = out := by
    apply CodecLaws.state_inj
    rw [hpw, hpo, hpm]
    simp only [concat, Slice.from_val, AdlerSpec.run_append]
  exact ⟨out, heq ▸ hw, by simp [hm, ho]⟩
 theorem save_restore_resume (s : Adler32) (bytes : Slice U8) :
    ∃ out, s.write_slice bytes = ok out ∧
      (do let code ← s.checksum
          let restored ← Adler32.from_checksum code
          restored.write_slice bytes) = ok out := by
  obtain ⟨out, hout, hp⟩ := ElicitAdapter.spec_to_contract (API.write_correct s bytes)
  exact ⟨out, hout, (CodecLaws.save_restore_resume s bytes).trans hout⟩
 theorem split_checksum_resume (s : Adler32) (xs ys : Slice U8)
    (h : xs.val.length + ys.val.length ≤ Usize.max) :
    ∃ out, s.write_slice (concat xs ys h) = ok out ∧
      (do let mid ← s.write_slice xs
          let code ← mid.checksum
          let restored ← Adler32.from_checksum code
          restored.write_slice ys) = ok out := by
  obtain ⟨out, hw, ho⟩ := split_update s xs ys h
  obtain ⟨mid, hm, hp⟩ := ElicitAdapter.spec_to_contract (API.write_correct s xs)
  refine ⟨out, hw, ?_⟩
  simp only [hm, bind_ok] at ho ⊢
  exact (CodecLaws.save_restore_resume mid ys).trans ho
end Laws
