import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_22 : ∀ (size : Aeneas.Std.Usize) (it : Aeneas.Std.core.slice.iter.ChunksExact Aeneas.Std.U8) (a b : Aeneas.Std.U32) (av bv : ElicitGenerated.algo.U32X4), size.val = 22208 → a.val ≤ 65535 → b.val ≤ 65535 → (∀ c ∈ it.chunks, c.val.length = 22208) → (∀ i : Nat, i < 4 → av.val[i]!.val ≤ 65520) → (∀ i : Nat, i < 4 → bv.val[i]!.val ≤ 65520) → ElicitGenerated.algo.Adler32.compute_loop0 size it a b av bv ⦃ fun out => out.1.val = it.chunks.foldl (fun z _ => (z + 22208 * a.val) % 65521) b.val ∧ out.1.val ≤ 65535 ∧ (∀ i : Nat, i < 4 → out.2.1.val[i]!.val ≤ 65520) ∧ (∀ i : Nat, i < 4 → out.2.2.val[i]!.val ≤ 65520) ⦄ := AdlerSpec.outer_execution_spec
#check @AdlerSpec.outer_execution_spec
#print IndependentLemma_22
#print axioms IndependentLemma_22
