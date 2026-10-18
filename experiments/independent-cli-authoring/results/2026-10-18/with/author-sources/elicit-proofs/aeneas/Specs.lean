import ElicitGenerated
open Aeneas Aeneas.Std ElicitGenerated

namespace Specs
abbrev modulus : Nat := 65521
abbrev state (s : Adler32) : Nat × Nat := (s.a.val, s.b.val)
def normalize (s : Nat × Nat) : Nat × Nat := (s.1 % modulus, s.2 % modulus)
def step (s : Nat × Nat) (byte : U8) : Nat × Nat :=
  let a := (s.1 + byte.val) % modulus
  (a, (s.2 + a) % modulus)
/-- Normalize even the empty update, then run the scalar Adler recurrence. -/
def update (s : Nat × Nat) (bytes : List U8) : Nat × Nat :=
  bytes.foldl step (normalize s)
def encode (s : Nat × Nat) : Nat := 65536 * s.2 + s.1
end Specs
