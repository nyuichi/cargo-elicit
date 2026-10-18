import ElicitGenerated
open Aeneas Aeneas.Std ElicitGenerated

namespace AdlerSpec
/-- The scalar Adler recurrence over mathematical naturals. Initial normalization
also specifies empty updates for all representable, possibly noncanonical seeds. -/
def run (a b : Nat) (bytes : List U8) : Nat × Nat :=
  bytes.foldl (fun s x =>
    let a' := (s.1 + x.val) % 65521
    (a', (s.2 + a') % 65521)) (a % 65521, b % 65521)

def packed (s : Nat × Nat) : Nat := s.1 + 65536 * s.2
end AdlerSpec
