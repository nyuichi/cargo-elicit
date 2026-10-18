import Core
open Aeneas Aeneas.Std Result
namespace Verification
open ElicitGenerated
set_option maxHeartbeats 2000000

theorem write_slice_correct (self : Adler32) (bytes : Slice U8) :
    self.write_slice bytes ⦃ r =>
      (r.a.val, r.b.val) = scalar (self.a.val, self.b.val) bytes.val ⦄ := by
  exact compute_correct self bytes

theorem adler32_slice_correct (bytes : Slice U8) :
    adler32_slice bytes ⦃ sum => sum.val =
      65536*(scalar (1,0) bytes.val).2 + (scalar (1,0) bytes.val).1 ⦄ := by
  unfold adler32_slice
  rw [new_correct]
  step with write_slice_correct as ⟨h, hh⟩
  step with checksum_correct as ⟨sum, hsum⟩
  have ha := congrArg Prod.fst hh
  have hb := congrArg Prod.snd hh
  simp_all

/-- The concatenation is explicitly required to fit in a Rust slice. -/
theorem split_updates_correct (self : Adler32) (xs ys : Slice U8)
    (hrepresentable : (xs.val ++ ys.val).length ≤ Usize.max) :
    (do let mid ← self.write_slice xs; mid.write_slice ys) ⦃ r =>
      (r.a.val, r.b.val) = scalar (self.a.val,self.b.val) (xs.val ++ ys.val) ⦄ := by
  step with write_slice_correct as ⟨mid, hmid⟩
  step with write_slice_correct as ⟨out, hout⟩
  rw [scalar_split _ _ _ hrepresentable, ← hmid]
  exact hout

theorem save_restore_resume_correct (self : Adler32) (xs ys : Slice U8)
    (hrepresentable : (xs.val ++ ys.val).length ≤ Usize.max) :
    (do
      let mid ← self.write_slice xs
      let sum ← mid.checksum
      let restored ← Adler32.from_checksum sum
      restored.write_slice ys) ⦃ r =>
      (r.a.val,r.b.val) = scalar (self.a.val,self.b.val) (xs.val ++ ys.val) ⦄ := by
  step with write_slice_correct as ⟨mid, hmid⟩
  rw [← Std.bind_assoc]
  apply WP.spec_bind (decode_encode mid)
  intro restored hrestored
  subst restored
  apply WP.spec_mono (write_slice_correct mid ys)
  intro r hr
  rw [scalar_split _ _ _ hrepresentable, ← hmid]
  exact hr

theorem state_eq_of_values (s t : Adler32)
    (h : (s.a.val,s.b.val) = (t.a.val,t.b.val)) : s = t := by
  have ha : s.a = t.a := UScalar.eq_of_val_eq (congrArg Prod.fst h)
  have hb : s.b = t.b := UScalar.eq_of_val_eq (congrArg Prod.snd h)
  cases s; cases t; simp_all

/-- Both execution paths succeed and return the same state. -/
theorem split_equivalence (self : Adler32) (xs ys whole : Slice U8)
    (hconcat : whole.val = xs.val ++ ys.val)
    (hrepresentable : (xs.val ++ ys.val).length ≤ Usize.max) :
    (do
      let one ← self.write_slice whole
      let mid ← self.write_slice xs
      let two ← mid.write_slice ys
      ok (one,two)) ⦃ one two => one = two ⦄ := by
  step with write_slice_correct as ⟨one, hone⟩
  step with write_slice_correct as ⟨mid, hmid⟩
  step with write_slice_correct as ⟨two, htwo⟩
  apply state_eq_of_values
  rw [hone, hconcat, scalar_split _ _ _ hrepresentable, ← hmid]
  exact htwo.symm

/-- Compare direct continuation with checksum-save/restore continuation. -/
theorem resume_equivalence (self : Adler32) (bytes : Slice U8) :
    (do
      let direct ← self.write_slice bytes
      let sum ← self.checksum
      let restored ← Adler32.from_checksum sum
      let resumed ← restored.write_slice bytes
      ok (direct,resumed)) ⦃ direct resumed => direct = resumed ⦄ := by
  step with write_slice_correct as ⟨direct, hdirect⟩
  rw [← Std.bind_assoc]
  apply WP.spec_bind (decode_encode self)
  intro restored hrestored
  subst restored
  step with write_slice_correct as ⟨resumed, hresumed⟩
  apply state_eq_of_values
  exact hdirect.trans hresumed.symm

#print axioms split_equivalence
#print axioms resume_equivalence
#print axioms write_slice_correct
#print axioms adler32_slice_correct
#print axioms split_updates_correct
#print axioms save_restore_resume_correct
end Verification
