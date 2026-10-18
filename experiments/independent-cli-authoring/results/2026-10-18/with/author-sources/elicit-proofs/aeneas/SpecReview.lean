import CodecLaws
open Aeneas Aeneas.Std ElicitGenerated AdlerMath
namespace Proofs

theorem update_eq_normalize_fold (s : Nat × Nat) (xs : List U8) :
    Specs.update s xs = Specs.normalize (xs.foldl Specs.step s) := by
  have hn : (Specs.normalize (xs.foldl Specs.step s)).1 < 65521 ∧
      (Specs.normalize (xs.foldl Specs.step s)).2 < 65521 :=
    ⟨Nat.mod_lt _ (by decide), Nat.mod_lt _ (by decide)⟩
  apply castState_inj (update_normalized _ _) hn
  rw [cast_update, cast_normalize, cast_fold]

theorem update_cons (s : Nat × Nat) (x : U8) (xs : List U8) :
    Specs.update s (x :: xs) = Specs.update (Specs.step s x) xs := by
  apply castState_inj (update_normalized _ _) (update_normalized _ _)
  simp only [cast_update, cast_step, scalar, List.foldl_cons]

end Proofs
