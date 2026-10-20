import ElicitGenerated
open Aeneas Aeneas.Std
namespace AdlerSpec
abbrev modulus : Nat := 65521
abbrev State := Nat × Nat
/-- Normalize even when the input list is empty. -/
def normalize (s : State) : State := (s.1 % modulus, s.2 % modulus)
def step (s : State) (x : U8) : State :=
  let a := (s.1 + x.val) % modulus
  (a, (s.2 + a) % modulus)
def run (s : State) (xs : List U8) : State := xs.foldl step (normalize s)
def state (s : ElicitGenerated.Adler32) : State := (s.a.val, s.b.val)
def pack (s : State) : Nat := 65536 * s.2 + s.1
end AdlerSpec
