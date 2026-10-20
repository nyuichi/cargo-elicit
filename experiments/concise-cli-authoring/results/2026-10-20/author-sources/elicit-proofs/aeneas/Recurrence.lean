import Specs
open Aeneas Aeneas.Std
namespace AdlerSpec
@[simp] theorem normalize_idem (s : State) : normalize (normalize s) = normalize s := by
  simp [normalize]
@[simp] theorem normalize_step (s : State) (x : U8) : normalize (step s x) = step s x := by
  simp [normalize, step]
theorem fold_normalized (xs : List U8) (s : State) (h : normalize s = s) :
    normalize (xs.foldl step s) = xs.foldl step s := by
  induction xs generalizing s with
  | nil => exact h
  | cons x xs ih => exact ih _ (normalize_step s x)
@[simp] theorem normalize_run (s : State) (xs : List U8) : normalize (run s xs) = run s xs :=
  fold_normalized xs _ (normalize_idem s)
@[simp] theorem run_nil (s : State) : run s [] = normalize s := rfl
theorem run_append (s : State) (xs ys : List U8) :
    run s (xs ++ ys) = run (run s xs) ys := by
  simp only [run, List.foldl_append]
  rw [fold_normalized xs _ (normalize_idem s)]
theorem run_bounds (s : State) (xs : List U8) :
    (run s xs).1 < modulus ∧ (run s xs).2 < modulus := by
  have h := normalize_run s xs
  have h1 := congrArg Prod.fst h
  have h2 := congrArg Prod.snd h
  simp only [normalize] at h1 h2
  have := Nat.mod_lt (run s xs).1 (show 0 < modulus by decide)
  have := Nat.mod_lt (run s xs).2 (show 0 < modulus by decide)
  omega
end AdlerSpec
