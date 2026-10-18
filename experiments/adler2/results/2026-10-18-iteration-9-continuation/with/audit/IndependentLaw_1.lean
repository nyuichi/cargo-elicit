import Proofs
open Aeneas Aeneas.Std Result ElicitGenerated
set_option pp.fullNames true
theorem IndependentLaw_1 : ∀ (s : Adler32), ∃ (sum : U32) (restored : Adler32), Adler32.checksum s = .ok sum ∧ Adler32.from_checksum sum = .ok restored ∧ restored = s := AdlerProof.from_checksum_checksum_roundtrip
#print IndependentLaw_1
#print axioms IndependentLaw_1
