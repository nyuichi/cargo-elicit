import Api
open Aeneas Aeneas.Std Result WP
namespace Verification
open adler2

/-- Total correctness in the effect model entails a successful returned value. -/
theorem spec_returns {T : Type} (m : Result T) (P : T → Prop) (h : m ⦃ P ⦄) :
    ∃ t, m = ok t ∧ P t := by
  cases m using Result.cases with
  | ret t => exact ⟨t,rfl,(spec_ok t).mp h⟩
  | vis e k => exact False.elim ((spec_vis e k).mp h)
  | div => exact False.elim (spec_div.mp h)

theorem write_slice_returns (self : Adler32) (bytes : Slice U8) :
    ∃ s, self.write_slice bytes = ok s ∧
      (s.a.val,s.b.val) = scalar (self.a.val,self.b.val) bytes.val :=
  spec_returns _ _ (write_slice_correct self bytes)

theorem adler32_slice_returns (bytes : Slice U8) :
    ∃ c, adler32_slice bytes = ok c ∧
      c.val = 65536*(scalar (1,0) bytes.val).2 + (scalar (1,0) bytes.val).1 :=
  spec_returns _ _ (adler32_slice_correct bytes)

lemma cast_residues (s : State) :
    (((residues s).1 : F),((residues s).2 : F)) = s := by
  simp only [residues,ZMod.natCast_zmod_val,Prod.mk.eta]

lemma scalar_split (a b : Nat) (xs ys : List U8) :
    scalar (a,b) (xs++ys) = scalar (scalar (a,b) xs) ys := by
  rw [scalar_eq]
  change residues (run (a,b) (xs++ys)) =
    scalar ((scalar (a,b) xs).1,(scalar (a,b) xs).2) ys
  rw [scalar_eq,scalar_eq,cast_residues,run_append]

/-- Concatenation must itself fit the standard Slice model. -/
def concatSlice (xs ys : Slice U8)
    (hcat : xs.val.length + ys.val.length ≤ Usize.max) : Slice U8 :=
  Slice.from (xs.val++ys.val) (by simpa only [List.length_append] using hcat)

 theorem split_update (self : Adler32) (xs ys : Slice U8)
    (hcat : xs.val.length + ys.val.length ≤ Usize.max) :
    (do let s ← self.write_slice xs; s.write_slice ys) =
      self.write_slice (concatSlice xs ys hcat) := by
  obtain ⟨sx,hex,hx⟩ := spec_returns _ _ (write_slice_correct self xs)
  obtain ⟨sxy,hexy,hxy⟩ := spec_returns _ _ (write_slice_correct sx ys)
  obtain ⟨whole,hew,hw⟩ := spec_returns _ _ (write_slice_correct self (concatSlice xs ys hcat))
  rw [hex,bind_ok,hexy,hew]
  congr 1
  have hvals : (sxy.a.val,sxy.b.val) = (whole.a.val,whole.b.val) := by
    rw [hxy,hw,hx]
    simp only [concatSlice,Slice.from_val]
    exact (scalar_split _ _ _ _).symm
  exact state_ext sxy whole (congrArg Prod.fst hvals) (congrArg Prod.snd hvals)

 theorem codec_state_execution (s : Adler32) :
    (do let c ← s.checksum; Adler32.from_checksum c) = ok s := by
  rw [checksum_exec,bind_ok,from_checksum_exec,codec_state]

 theorem codec_word_execution (c : U32) :
    (do let s ← Adler32.from_checksum c; s.checksum) = ok c := by
  rw [from_checksum_exec,bind_ok,checksum_exec,codec_word]

 theorem save_restore_resume (s : Adler32) (ys : Slice U8) :
    (do let c ← s.checksum; let restored ← Adler32.from_checksum c; restored.write_slice ys) =
      s.write_slice ys := by
  rw [checksum_exec,bind_ok,from_checksum_exec,bind_ok,codec_state]

 theorem split_save_restore_resume (self : Adler32) (xs ys : Slice U8)
    (hcat : xs.val.length + ys.val.length ≤ Usize.max) :
    (do
      let s ← self.write_slice xs
      let c ← s.checksum
      let restored ← Adler32.from_checksum c
      restored.write_slice ys) =
      self.write_slice (concatSlice xs ys hcat) := by
  simp only [save_restore_resume]
  exact split_update self xs ys hcat
end Verification
