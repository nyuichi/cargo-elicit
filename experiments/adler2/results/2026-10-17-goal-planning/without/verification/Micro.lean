import Loops
open Aeneas Aeneas.Std Result
namespace Verification
open adler2

theorem get_set_one_two (v : algo.U32X4) (x : U32) :
    ((v.set 1#usize x).val[2]'(by simp)).val = lane v 2 := by
  simp [Array.set, List.getElem_set, lane]

theorem get_set_two_three (v : algo.U32X4) (x y : U32) :
    (((v.set 1#usize x).set 2#usize y).val[3]'(by simp)).val = lane v 3 := by
  simp [Array.set, List.getElem_set, lane]
end Verification
