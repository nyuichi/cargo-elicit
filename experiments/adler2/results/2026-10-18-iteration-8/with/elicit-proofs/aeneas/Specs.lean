import Aeneas
open Aeneas Aeneas.Std

namespace AdlerSpec

def step (s : Nat × Nat) (x : U8) : Nat × Nat :=
  let a := (s.1 + x.val) % 65521
  (a, (s.2 + a) % 65521)

/-- Normalize both halves even when no bytes are consumed. -/
def update (s : Nat × Nat) (xs : List U8) : Nat × Nat :=
  xs.foldl step (s.1 % 65521, s.2 % 65521)

end AdlerSpec

namespace AdlerSpec

def normalize (s : Nat × Nat) : Nat × Nat := (s.1 % 65521, s.2 % 65521)

/-- Unbounded scalar arithmetic, used only as a logical model. -/
def rawStep (s : Nat × Nat) (x : U8) : Nat × Nat :=
  (s.1 + x.val, s.2 + s.1 + x.val)

end AdlerSpec
