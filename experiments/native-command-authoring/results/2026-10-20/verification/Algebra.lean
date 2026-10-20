import Mathlib.Data.ZMod.Basic
import Mathlib.Tactic
import Generated.Adler2
open Aeneas Aeneas.Std Result
namespace Verification
abbrev F := ZMod 65521
abbrev State := F × F

def advance (s : State) (x : U8) : State :=
  (s.1 + (x.val : F), s.2 + s.1 + (x.val : F))
def run (s : State) (xs : List U8) : State := xs.foldl advance s

def combine (a b x0 x1 x2 x3 y0 y1 y2 y3 : F) : State :=
  (a + x0 + x1 + x2 + x3,
   b + 4*(y0+y1+y2+y3) - x1 - 2*x2 - 3*x3)

theorem combine_four (a b x0 x1 x2 x3 y0 y1 y2 y3 : F) (d0 d1 d2 d3 : U8) :
    combine a (b + 4*a)
      (x0+d0.val) (x1+d1.val) (x2+d2.val) (x3+d3.val)
      (y0+(x0+d0.val)) (y1+(x1+d1.val)) (y2+(x2+d2.val)) (y3+(x3+d3.val)) =
    run (combine a b x0 x1 x2 x3 y0 y1 y2 y3) [d0,d1,d2,d3] := by
  simp only [combine, run, List.foldl_cons, List.foldl_nil, advance]
  apply Prod.ext <;> dsimp <;> ring

theorem run_append (s : State) (xs ys : List U8) : run s (xs ++ ys) = run (run s xs) ys := by
  exact List.foldl_append

/-- The empty update is normalized by taking the canonical residues. -/
def residues (s : State) : Nat × Nat := (s.1.val, s.2.val)

def scalarStep (s : Nat × Nat) (x : U8) : Nat × Nat :=
  let a := (s.1 + x.val) % 65521
  (a, (s.2 + a) % 65521)
def scalar (s : Nat × Nat) (xs : List U8) : Nat × Nat :=
  xs.foldl scalarStep (s.1 % 65521, s.2 % 65521)

theorem residues_advance (s : State) (x : U8) :
    residues (advance s x) = scalarStep (residues s) x := by
  apply Prod.ext <;> simp only [residues, advance, scalarStep, ZMod.val_add, ZMod.val_natCast] <;> omega

theorem residues_run (s : State) (xs : List U8) :
    residues (run s xs) = xs.foldl scalarStep (residues s) := by
  induction xs generalizing s with
  | nil => rfl
  | cons x xs ih =>
    simp only [run, List.foldl_cons]
    exact (ih (advance s x)).trans (congrArg (fun z => xs.foldl scalarStep z) (residues_advance s x))

theorem scalar_eq (a b : Nat) (xs : List U8) :
    scalar (a,b) xs = residues (run ((a:F),(b:F)) xs) := by
  rw [residues_run]
  rfl
end Verification
