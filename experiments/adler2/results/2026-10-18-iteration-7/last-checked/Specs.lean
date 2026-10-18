import ElicitGenerated
open Aeneas Aeneas.Std

namespace AdlerSpec
def step (s : Nat × Nat) (x : U8) : Nat × Nat :=
  let a := (s.1 + x.val) % 65521
  (a, (s.2 + a) % 65521)

def update (a b : Nat) (xs : List U8) : Nat × Nat :=
  xs.foldl step (a % 65521, b % 65521)

def packed (s : Nat × Nat) : Nat := 65536 * s.2 + s.1
end AdlerSpec
