import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_13 : ∀ (bytes : Aeneas.Std.Slice Aeneas.Std.U8), 4 ≤ bytes.val.length → ElicitGenerated.algo.U32X4.from bytes ⦃ fun out => ∀ i : Nat, i < 4 → out.val[i]!.val = bytes.val[i]!.val ⦄ := AdlerSpec.vector_from_spec
#check @AdlerSpec.vector_from_spec
#print IndependentLemma_13
#print axioms IndependentLemma_13
