import Mathematics
open Aeneas Aeneas.Std Result
namespace AdlerProof
set_option maxHeartbeats 2000000

-- This is the scalar recurrence, with seed normalization also on empty updates.
def scalarStep (s : Nat × Nat) (d : U8) : Nat × Nat :=
 let a := (s.1+d.val) % 65521
 (a,(s.2+a) % 65521)

def scalar (a b : Nat) (l : List U8) : Nat × Nat :=
 l.foldl scalarStep (a % 65521,b % 65521)

theorem scalarStep_cast (s : Nat × Nat) (d : U8) :
 (((scalarStep s d).1:R), ((scalarStep s d).2:R)) = rstep ((s.1:R),(s.2:R)) d := by
 ext <;> simp [scalarStep,rstep] <;> ring

theorem scalarFold_cast (l : List U8) (s : Nat × Nat) :
 (((l.foldl scalarStep s).1:R), ((l.foldl scalarStep s).2:R)) = rfold ((s.1:R),(s.2:R)) l := by
 induction l generalizing s with
 | nil => rfl
 | cons d ds ih =>
   rw [List.foldl_cons, ih]
   simp only [rfold, List.foldl_cons]
   rw [← scalarStep_cast]

theorem scalar_cast (a b : Nat) (l : List U8) :
 (((scalar a b l).1:R), ((scalar a b l).2:R)) = rfold ((a:R),(b:R)) l := by
 unfold scalar
 rw [scalarFold_cast]
 simp

theorem scalarFold_bounds (l : List U8) (s : Nat × Nat)
 (hs : s.1 < 65521 ∧ s.2 < 65521) :
 (l.foldl scalarStep s).1 < 65521 ∧ (l.foldl scalarStep s).2 < 65521 := by
 induction l generalizing s with
 | nil => exact hs
 | cons d ds ih =>
   rw [List.foldl_cons]
   apply ih
   constructor <;> exact Nat.mod_lt _ (by decide)

theorem scalar_bounds (a b : Nat) (l : List U8) :
 (scalar a b l).1 < 65521 ∧ (scalar a b l).2 < 65521 := by
 apply scalarFold_bounds
 constructor <;> exact Nat.mod_lt _ (by decide)

theorem cast_injective_normalized (a b c d : Nat)
 (ha : a < 65521) (hb : b < 65521) (hc : c < 65521) (hd : d < 65521)
 (h : ((a:R),(b:R)) = ((c:R),(d:R))) : (a,b) = (c,d) := by
 have h0 := congrArg (fun s : RState => s.1.val) h
 have h1 := congrArg (fun s : RState => s.2.val) h
 simp only [ZMod.val_natCast, Nat.mod_eq_of_lt ha, Nat.mod_eq_of_lt hb,
   Nat.mod_eq_of_lt hc, Nat.mod_eq_of_lt hd] at h0 h1
 exact Prod.ext h0 h1

theorem scalar_append (a b : Nat) (l r : List U8) :
 scalar a b (l++r) = scalar (scalar a b l).1 (scalar a b l).2 r := by
 have hl := scalar_bounds a b l
 simp only [scalar, List.foldl_append]
 simp only [scalar] at hl
 rw [Nat.mod_eq_of_lt hl.1,Nat.mod_eq_of_lt hl.2]
