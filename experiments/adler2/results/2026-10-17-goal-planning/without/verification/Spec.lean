import Adler2
open Aeneas Aeneas.Std Result
namespace Verification

def normalize (s : Nat × Nat) : Nat × Nat := (s.1 % 65521, s.2 % 65521)
def scalarStep (s : Nat × Nat) (x : U8) : Nat × Nat :=
  let a := (s.1 + x.val) % 65521
  (a, (s.2 + a) % 65521)
def scalar (s : Nat × Nat) (xs : List U8) : Nat × Nat :=
  xs.foldl scalarStep (normalize s)

theorem normalize_idempotent (s : Nat × Nat) :
    normalize (normalize s) = normalize s := by simp [normalize]

theorem scalar_normalized (s : Nat × Nat) (xs : List U8) :
    normalize (scalar s xs) = scalar s xs := by
  unfold scalar
  have aux : ∀ (xs : List U8) (t : Nat × Nat), normalize t = t →
      normalize (xs.foldl scalarStep t) = xs.foldl scalarStep t := by
    intro xs
    induction xs with
    | nil => simp
    | cons x xs ih =>
      intro t ht
      simp only [List.foldl_cons]
      apply ih
      simp [normalize, scalarStep]
  exact aux xs _ (normalize_idempotent s)

/-- The model-level split law explicitly assumes the concatenation is representable.
It is not yet a theorem about Rust write_slice. -/
theorem scalar_split (s : Nat × Nat) (xs ys : List U8)
    (_representable : (xs ++ ys).length ≤ Usize.max) :
    scalar s (xs ++ ys) = scalar (scalar s xs) ys := by
  simp only [scalar, List.foldl_append]
  rw [show normalize (List.foldl scalarStep (normalize s) xs) =
      List.foldl scalarStep (normalize s) xs from scalar_normalized s xs]

def rawStep (s : Nat × Nat) (x : U8) : Nat × Nat :=
  (s.1 + x.val, s.2 + (s.1 + x.val))
def raw (s : Nat × Nat) (xs : List U8) : Nat × Nat :=
  xs.foldl rawStep s

theorem normalize_rawStep (s : Nat × Nat) (x : U8) :
    normalize (rawStep s x) = scalarStep (normalize s) x := by
  simp [normalize, rawStep, scalarStep, Nat.add_mod, Nat.mod_mod]

theorem normalize_raw (s : Nat × Nat) (xs : List U8) :
    normalize (raw s xs) = scalar s xs := by
  induction xs generalizing s with
  | nil => simp [raw, scalar]
  | cons x xs ih =>
    simp only [raw, List.foldl_cons]
    rw [show List.foldl rawStep (rawStep s x) xs = raw (rawStep s x) xs from rfl]
    rw [ih]
    simp only [scalar, List.foldl_cons, normalize_rawStep]

#print axioms scalar_split
#print axioms normalize_raw
end Verification
