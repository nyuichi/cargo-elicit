import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_14 : ∀ (s : ElicitGenerated.Adler32) (xs ys : Aeneas.Std.Slice Aeneas.Std.U8),
    (do let s1 ← ElicitGenerated.Adler32.write_slice s xs
        let restored ← (do let c ← ElicitGenerated.Adler32.checksum s1; ElicitGenerated.Adler32.from_checksum c)
        ElicitGenerated.Adler32.write_slice restored ys) =
    (do let s1 ← ElicitGenerated.Adler32.write_slice s xs; ElicitGenerated.Adler32.write_slice s1 ys) := checksum_resume_equiv
#check @checksum_resume_equiv
#print IndependentLemma_14
#print axioms IndependentLemma_14
