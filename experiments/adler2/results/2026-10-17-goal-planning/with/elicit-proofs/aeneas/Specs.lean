import ElicitGenerated
open Aeneas Aeneas.Std

namespace AdlerModel
/-- Scalar Adler recurrence, including normalization on an empty update. -/
def update (a b : Nat) (bytes : List U8) : Nat × Nat :=
  bytes.foldl (fun s x => let a' := (s.1 + x.val) % 65521; (a', (s.2 + a') % 65521))
    (a % 65521, b % 65521)
end AdlerModel
