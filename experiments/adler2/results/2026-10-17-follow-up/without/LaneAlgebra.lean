import CoreLemmas
import Mathlib.Tactic.Ring
import Mathlib.Data.ZMod.Basic

namespace Verification

/-- Algebraic interpretation of four lanes, before the final reduction. -/
def combineLanes {R : Type*} [CommRing R]
    (a b a0 a1 a2 a3 b0 b1 b2 b3 : R) : R × R :=
  (a+a0+a1+a2+a3, b+4*(b0+b1+b2+b3)-a1-2*a2-3*a3)

def ringStep {R : Type*} [CommRing R] (s : R × R) (x : R) : R × R :=
  (s.1+x, s.2+s.1+x)

/-- One four-byte group of the lane algorithm satisfies four scalar steps.
The base contribution may be accumulated per group or deferred to a chunk boundary.
This is algebra only; connecting machine operations and extracted loops remains open. -/
theorem four_lane_step {R : Type*} [CommRing R]
    (a b a0 a1 a2 a3 b0 b1 b2 b3 x0 x1 x2 x3 : R) :
    combineLanes a (b+4*a)
      (a0+x0) (a1+x1) (a2+x2) (a3+x3)
      (b0+a0+x0) (b1+a1+x1) (b2+a2+x2) (b3+a3+x3) =
    [x0,x1,x2,x3].foldl ringStep (combineLanes a b a0 a1 a2 a3 b0 b1 b2 b3) := by
  apply Prod.ext <;> dsimp [combineLanes, ringStep, List.foldl] <;> ring

/-- In the residue ring, complement corrections implement subtraction exactly. -/
theorem complement_correction (a1 a2 a3 b0 b1 b2 b3 : ZMod 65521) :
    4*b0 + (4*b1+(65521-a1)) + (4*b2+2*(65521-a2)) +
      (4*b3+3*(65521-a3)) =
    4*(b0+b1+b2+b3)-a1-2*a2-3*a3 := by
  have h : (65521 : ZMod 65521) = 0 := ZMod.natCast_self 65521
  rw [h]
  ring

#print axioms four_lane_step
#print axioms complement_correction
end Verification
