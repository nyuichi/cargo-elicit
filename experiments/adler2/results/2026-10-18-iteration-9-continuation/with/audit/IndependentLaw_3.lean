import Proofs
open Aeneas Aeneas.Std Result ElicitGenerated
set_option pp.fullNames true
theorem IndependentLaw_3 : ∀ (s : Adler32) (x y xy : Slice U8), x.val.length + y.val.length ≤ Usize.max → xy.val = x.val ++ y.val → ∃ (sx : Adler32) (saved : U32) (restored resumed whole : Adler32), Adler32.write_slice s x = .ok sx ∧ Adler32.checksum sx = .ok saved ∧ Adler32.from_checksum saved = .ok restored ∧ Adler32.write_slice restored y = .ok resumed ∧ Adler32.write_slice s xy = .ok whole ∧ resumed = whole := AdlerProof.save_restore_resume
#print IndependentLaw_3
#print axioms IndependentLaw_3
