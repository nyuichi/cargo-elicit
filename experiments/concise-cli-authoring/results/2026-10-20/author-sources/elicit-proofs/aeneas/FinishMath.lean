import Outer
import Tail
import ElicitAdapter
import Mathlib.Tactic.LinearCombination
open Aeneas Aeneas.Std Result ElicitGenerated
set_option maxHeartbeats 4000000
set_option maxRecDepth 4096
namespace Compute
open Vector LaneAlgebra
@[step] theorem chunk_size_spec : algo.Adler32.compute.CHUNK_SIZE ⦃ out => out = 22208#usize ⦄ := by
  unfold algo.Adler32.compute.CHUNK_SIZE
  step*
 theorem total_array (v : V) : Tail.total ⟨Array.to_slice v, 0⟩ =
    lane v 0 + lane v 1 + lane v 2 + lane v 3 := by
  have hl : v.val.length = 4 := v.property
  obtain ⟨x0, x1, x2, x3, hx⟩ := List.length_eq_four.mp hl
  simp [Tail.total, Tail.rest, Array.to_slice, hx, lane, Nat.add_assoc]
 theorem cast_total (v : V) : (Tail.total ⟨Array.to_slice v, 0⟩ : Z) = sum (lanes v) := by
  rw [total_array]
  simp only [Nat.cast_add, LaneAlgebra.sum, lanes]
 theorem zeros_normalized : Outer.normalized (Array.repeat 4#usize 0#u32) := by
  intro i
  fin_cases i <;> simp [lane]
 theorem zeros_lanes : lanes (Array.repeat 4#usize 0#u32) = fun _ => 0 := by
  funext i
  fin_cases i <;> simp [lanes, lane]

 theorem correction_cast (v : V) (i : Fin 4) (d : U32)
    (h : lane v i ≤ 65521) (hd : d.val = 65521 - lane v i) :
    (d.val : Z) = -lanes v i := by
  have heq : d.val + lane v i = 65521 := by omega
  have hz := congrArg (fun n : Nat => (n : Z)) heq
  simp only [Nat.cast_add] at hz
  norm_num at hz
  have hm : (65521 : Z) = 0 := by exact ZMod.natCast_self 65521
  rw [hm] at hz
  change (d.val : Z) = -(lane v i : Z)
  linear_combination hz

end Compute
