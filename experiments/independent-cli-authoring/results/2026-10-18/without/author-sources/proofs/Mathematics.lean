import Chunks
import Mathlib.Data.ZMod.Basic
open Aeneas Aeneas.Std Result
namespace AdlerProof
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096
abbrev R := ZMod 65521
abbrev RState := R × R

def rstep (s : RState) (d : U8) : RState :=
 (s.1 + (d.val : R), s.2 + s.1 + (d.val : R))

def rfold (s : RState) (l : List U8) : RState := l.foldl rstep s

def sum4 (v : Lanes) : Nat := v 0 + v 1 + v 2 + v 3

def interp (a b : Nat) (av bv : Lanes) : RState :=
 ((a : R) + (sum4 av : R),
  (b : R) + 4*(sum4 bv : R) - ((av 1 : R) + 2*(av 2 : R) + 3*(av 3 : R)))

theorem vector_action (c : Slice U8) (hc : c.val.length = 4) (a b : Nat) (av bv : Lanes) :
 interp a (b+4*a) (fun i => av i + byteAt c i) (fun i => bv i + (av i + byteAt c i)) =
 rfold (interp a b av bv) c.val := by
 obtain ⟨d0,d1,d2,d3,hd⟩ := List.length_eq_four.mp hc
 ext <;> simp [rfold, rstep, interp, sum4, byteAt, hd] <;> ring

theorem rfold_append (s : RState) (l r : List U8) :
 rfold s (l++r) = rfold (rfold s l) r := by simp [rfold]

theorem laneRun_action (cs : List (Slice U8)) (hc : ∀ c ∈ cs, c.val.length = 4)
 (a b : Nat) (av bv : Lanes) :
 interp a (b+4*cs.length*a) (laneRun cs av bv).1 (laneRun cs av bv).2 =
 rfold (interp a b av bv) (cs.map Slice.val).flatten := by
 induction cs generalizing b av bv with
 | nil => simp [laneRun, rfold]
 | cons c cs ih =>
   have hhead := hc c (by simp)
   have htail : ∀ c ∈ cs, c.val.length = 4 := by intro c hm; exact hc c (by simp [hm])
   have hlen : b+4*(c::cs).length*a = (b+4*a)+4*cs.length*a := by simp; ring
   simp only [laneRun, hlen]
   rw [ih htail, vector_action c hhead]
   simp [rfold_append]

theorem interp_mod (a b : Nat) (av bv : Lanes) :
 interp a (b % 65521) (fun i => av i % 65521) (fun i => bv i % 65521) = interp a b av bv := by
 ext <;> simp [interp, sum4, Nat.cast_add]
