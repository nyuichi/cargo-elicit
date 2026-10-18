import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_9 : ∀ (bytes : Slice U8) (cs : List (Slice U8)) (rem : Slice U8) (a b : algo.U32X4) (hlen : 4 ≤ bytes.val.length) (ha : ∀ i : Nat, i < 4 → a.val[i]!.val + bytes.val[i]!.val ≤ U32.max) (hb : ∀ i : Nat, i < 4 → b.val[i]!.val + (a.val[i]!.val + bytes.val[i]!.val) ≤ U32.max), algo.Adler32.compute_loop0_loop0.body { chunks := bytes :: cs, remainder := rem } a b ⦃ fun r => ∃ av bv, r = .cont ({ chunks := cs, remainder := rem }, av, bv) ∧ (∀ i : Nat, i < 4 → av.val[i]!.val = a.val[i]!.val + bytes.val[i]!.val) ∧ (∀ i : Nat, i < 4 → bv.val[i]!.val = b.val[i]!.val + (a.val[i]!.val + bytes.val[i]!.val)) ⦄ := AdlerProof.inner_step
#check @AdlerProof.inner_step
#print IndependentLemma_9
#print axioms IndependentLemma_9
