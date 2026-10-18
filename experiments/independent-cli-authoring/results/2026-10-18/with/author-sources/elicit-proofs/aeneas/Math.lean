import Specs
import Mathlib.Data.ZMod.Basic
import Mathlib.Tactic.Ring
open Aeneas Aeneas.Std ElicitGenerated
namespace AdlerMath
abbrev F := ZMod 65521
abbrev S := F × F
def scalarStep (s : S) (x : U8) : S := (s.1 + x.val, s.2 + s.1 + x.val)
def scalar (s : S) (xs : List U8) : S := xs.foldl scalarStep s
def castState (s : Nat × Nat) : S := (s.1, s.2)

theorem cast_normalize (s : Nat × Nat) :
    castState (Specs.normalize s) = castState s := by
  simp [castState, Specs.normalize, Specs.modulus, ZMod.natCast_mod]

theorem cast_step (s : Nat × Nat) (x : U8) :
    castState (Specs.step s x) = scalarStep (castState s) x := by
  simp [castState, Specs.step, scalarStep, Specs.modulus, Nat.cast_add,
    ZMod.natCast_mod, add_assoc]

theorem cast_fold (s : Nat × Nat) (xs : List U8) :
    castState (xs.foldl Specs.step s) = scalar (castState s) xs := by
  induction xs generalizing s with
  | nil => rfl
  | cons x xs ih => simp only [List.foldl_cons, scalar]; rw [ih, cast_step]; rfl

theorem cast_update (s : Nat × Nat) (xs : List U8) :
    castState (Specs.update s xs) = scalar (castState s) xs := by
  rw [Specs.update, cast_fold, cast_normalize]

theorem update_normalized (s : Nat × Nat) (xs : List U8) :
    (Specs.update s xs).1 < 65521 ∧ (Specs.update s xs).2 < 65521 := by
  have h : ∀ t : Nat × Nat, t.1 < 65521 ∧ t.2 < 65521 →
      (xs.foldl Specs.step t).1 < 65521 ∧ (xs.foldl Specs.step t).2 < 65521 := by
    induction xs with
    | nil => simp
    | cons x xs ih =>
      intro t ht
      apply ih
      exact ⟨Nat.mod_lt _ (by decide), Nat.mod_lt _ (by decide)⟩
  apply h
  exact ⟨Nat.mod_lt _ (by decide), Nat.mod_lt _ (by decide)⟩

theorem castState_inj {s t : Nat × Nat}
    (hs : s.1 < 65521 ∧ s.2 < 65521) (ht : t.1 < 65521 ∧ t.2 < 65521)
    (h : castState s = castState t) : s = t := by
  have h1 := congrArg (fun p : S => p.1.val) h
  have h2 := congrArg (fun p : S => p.2.val) h
  simp [castState, ZMod.val_natCast, Nat.mod_eq_of_lt, hs.1, hs.2, ht.1, ht.2] at h1 h2
  exact Prod.ext h1 h2

end AdlerMath
