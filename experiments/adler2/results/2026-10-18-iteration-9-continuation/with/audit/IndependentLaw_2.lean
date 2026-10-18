import Proofs
open Aeneas Aeneas.Std Result ElicitGenerated
set_option pp.fullNames true
theorem IndependentLaw_2 : ∀ (s : Adler32) (x y xy : Slice U8), x.val.length + y.val.length ≤ Usize.max → xy.val = x.val ++ y.val → ∃ (sx sxy whole : Adler32), Adler32.write_slice s x = .ok sx ∧ Adler32.write_slice sx y = .ok sxy ∧ Adler32.write_slice s xy = .ok whole ∧ sxy = whole := AdlerProof.split_update
#print IndependentLaw_2
#print axioms IndependentLaw_2
