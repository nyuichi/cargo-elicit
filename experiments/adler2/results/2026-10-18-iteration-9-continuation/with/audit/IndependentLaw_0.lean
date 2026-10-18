import Proofs
open Aeneas Aeneas.Std Result ElicitGenerated
set_option pp.fullNames true
theorem IndependentLaw_0 : ∀ (sum : U32), ∃ (s : Adler32) (out : U32), Adler32.from_checksum sum = .ok s ∧ Adler32.checksum s = .ok out ∧ out = sum := AdlerProof.checksum_from_checksum_roundtrip
#print IndependentLaw_0
#print axioms IndependentLaw_0
