import Outer
import Tail
import Micro
open Aeneas Aeneas.Std Result ControlFlow
namespace Verification
open ElicitGenerated
set_option maxHeartbeats 4000000
set_option maxRecDepth 4096
set_option linter.unusedSimpArgs false
set_option linter.unnecessarySimpa false

theorem vals_four (v : algo.U32X4) :
    vals v = [lane v 0, lane v 1, lane v 2, lane v 3] := by
  apply List.ext_getElem
  · simp [vals]
  · intro j hj hright
    have hj4 : j < 4 := by simpa [vals] using hj
    have hc : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
    rcases hc with rfl | rfl | rfl | rfl <;> simp [vals, lane]

theorem chunk_size : algo.Adler32.compute.CHUNK_SIZE = ok 22208#usize := by
  unfold algo.Adler32.compute.CHUNK_SIZE
  simp [HMul.hMul, UScalar.mul, UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds]
  have hb : Mul.mul 5552 4 < 2 ^ System.Platform.numBits := by
    have hs : 22208 ≤ Usize.max := by scalar_tac
    simp only [Usize.max, Usize.numBits, UScalarTy.numBits] at hs
    change 22208 < 2 ^ System.Platform.numBits
    omega
  simp only [hb, _root_.dite_true]


theorem compute_correct (self : Adler32) (bytes : Slice U8) :
    algo.Adler32.compute self bytes ⦃ r =>
      (r.a.val, r.b.val) = scalar (self.a.val, self.b.val) bytes.val ⦄ := by
  unfold algo.Adler32.compute
  rw [chunk_size]
  simp only [lift, bind_ok, bind_tc_ok, algo.Adler32.compute.MOD]
  step as ⟨nmod, hmod⟩
  step as ⟨n, hn⟩
  step as ⟨head, tail, hlen0, hlen1, hval0, hval1⟩
  step with chunks_exact_correct as ⟨it, hchunks, hflat, hrem⟩
  simp only [core.slice.iter.ChunksExact.getRemainder, bind_ok, bind_tc_ok]
  have hzero : laneBounds 0 (Array.repeat 4#usize 0#u32) (Array.repeat 4#usize 0#u32) := by
    unfold laneBounds
    intro j
    have hc : j.val = 0 ∨ j.val = 1 ∨ j.val = 2 ∨ j.val = 3 := by omega
    rcases hc with hc | hc | hc | hc <;> simp [lane, Array.repeat, hc]
  step with outer_loop as ⟨b0, av0, bv0, hb0, hv0, hproj0⟩ by
    solve | assumption | scalar_tac
  step with chunks_exact_correct as ⟨it4, hfour, hflat4, hrem4⟩
  have hcount0 := chunks_count head it 22208 ⟨hchunks, hflat⟩
  have hcount4 := chunks_count it.remainder it4 4 ⟨hfour, hflat4⟩
  have hlen4 : it4.chunks.length ≤ 5552 := by omega
  step with vector_loop_projection it4.chunks it4.remainder av0 bv0
    (self.a.val : Z) (b0.val : Z) hfour hlen4 hv0 as ⟨av1, bv1, hv1, hproj1⟩
  have hcastlen : (UScalar.cast .U32 it.remainder.len).val = it.remainder.val.length := by
    rw [UScalar.cast_val_eq]
    simp only [Slice.len_val]
    exact Nat.mod_eq_of_lt (by scalar_tac)
  step as ⟨offset, hoff⟩ by scalar_tac
  step as ⟨b1, hb1⟩ by scalar_tac
  step with vector_rem_lane as ⟨av2, hav2⟩ by simp [algo.Adler32.compute.MOD]
  step with vector_rem_lane as ⟨bv2, hbv2⟩ by simp [algo.Adler32.compute.MOD]
  step as ⟨b2, hb2⟩ by simp [algo.Adler32.compute.MOD]
  have havbound : ∀ j, lane av2 j < 65521 := by
    intro j
    simpa [hav2, algo.Adler32.compute.MOD] using Nat.mod_lt (lane av1 j) (by decide : 0 < 65521)
  have hbvbound : ∀ j, lane bv2 j < 65521 := by
    intro j
    simpa [hbv2, algo.Adler32.compute.MOD] using Nat.mod_lt (lane bv1 j) (by decide : 0 < 65521)
  step with vector_mul_lane as ⟨bv3, hbv3⟩ by
    intro j
    have := hbvbound j
    scalar_tac
  have a0b := havbound 0
  have a1b := havbound 1
  have a2b := havbound 2
  have a3b := havbound 3
  have b0b := hbvbound 0
  have b1b := hbvbound 1
  have b2b := hbvbound 2
  have b3b := hbvbound 3
  have bv30 := hbv3 0
  have bv31 := hbv3 1
  have bv32 := hbv3 2
  have bv33 := hbv3 3
  simp [lane] at a0b a1b a2b a3b b0b b1b b2b b3b bv30 bv31 bv32 bv33
  step as ⟨aidx1, hai1⟩
  step as ⟨sub1, hs1⟩ by scalar_tac
  step as ⟨bidx1, hbi1⟩
  step as ⟨new1, hnew1⟩ by scalar_tac
  step as ⟨bb1, hbb1⟩
  step as ⟨aidx2, hai2⟩
  step as ⟨sub2, hs2⟩ by scalar_tac
  step as ⟨times2, ht2⟩ by scalar_tac
  step as ⟨bidx2, hbi2⟩
  step as ⟨new2, hnew2⟩ by
    simp_all [Array.set, List.getElem_set, algo.Adler32.compute.MOD]
    scalar_tac
  step as ⟨bb2, hbb2⟩
  step as ⟨aidx3, hai3⟩
  step as ⟨sub3, hs3⟩ by scalar_tac
  step as ⟨times3, ht3⟩ by scalar_tac
  step as ⟨bidx3, hbi3⟩
  step as ⟨new3, hnew3⟩ by
    simp_all [Array.set, List.getElem_set, algo.Adler32.compute.MOD]
    scalar_tac
  step as ⟨bb3, hbb3⟩
  have hsumA : (vals av2).sum ≤ 262080 := by
    rw [vals_four]
    simp only [List.sum_cons, List.sum_nil]
    have h0 := havbound 0
    have h1 := havbound 1
    have h2 := havbound 2
    have h3 := havbound 3
    omega
  have hvalsB : vals bb3 = [lane bv3 0, new1.val, new2.val, new3.val] := by
    simp only [hbb3, hbb2, hbb1, vals, Array.set_val_eq, List.map_set]
    rw [show bv3.val.map UScalar.val = _ from vals_four bv3]
    simp
  have hbi2base : bidx2.val = lane bv3 2 := by
    rw [hbi2, hbb1]
    exact get_set_one_two bv3 new1
  have hbi3base : bidx3.val = lane bv3 3 := by
    rw [hbi3, hbb2, hbb1]
    exact get_set_two_three bv3 new1 new2
  have hnew1eq : new1.val = 4*lane bv2 1 + (65521-lane av2 1) := by
    rw [hnew1, hbi1, hs1, hai1]
    simpa [lane, Nat.mul_comm] using congrArg (fun n => n + (65521-lane av2 1)) bv31
  have hnew2eq : new2.val = 4*lane bv2 2 + (65521-lane av2 2)*2 := by
    rw [hnew2, hbi2base, ht2, hs2, hai2, hbv3]
    simp [lane, Nat.mul_comm]
  have hnew3eq : new3.val = 4*lane bv2 3 + (65521-lane av2 3)*3 := by
    rw [hnew3, hbi3base, ht3, hs3, hai3, hbv3]
    simp [lane, Nat.mul_comm]
  have hnew1b : new1.val ≤ 500000 := by
    rw [hnew1eq]
    have := hbvbound 1
    omega
  have hnew2b : new2.val ≤ 500000 := by
    rw [hnew2eq]
    have := hbvbound 2
    omega
  have hnew3b : new3.val ≤ 500000 := by
    rw [hnew3eq]
    have := hbvbound 3
    omega
  have hsumB : (vals bb3).sum ≤ 2000000 := by
    rw [hvalsB]
    simp only [List.sum_cons, List.sum_nil]
    change lane bv3 0 + (new1.val + (new2.val + (new3.val+0))) ≤ 2000000
    have := hbv3 0
    have := hbvbound 0
    omega
  simp only [core.slice.Slice.iter, bind_ok, bind_tc_ok]
  step with sum_loop as ⟨aa, haa⟩ by
    simp only [tailSum, List.drop_zero, Array.to_slice, Slice.from_val]
    change (core.convert.num.FromU32U16.from self.a).val + (vals av2).sum ≤ U32.max
    scalar_tac
  step with sum_loop_b as ⟨bb, hbb⟩ by
    simp only [tailSum, List.drop_zero, Array.to_slice, Slice.from_val]
    change b2.val + (vals bb3).sum ≤ U32.max
    scalar_tac
  have haa' : aa.val = self.a.val + (vals av2).sum := by
    simpa [tailSum, vals, Array.to_slice] using haa
  have hbb' : bb.val = b2.val + (vals bb3).sum := by
    simpa [tailSum, vals, Array.to_slice] using hbb
  have haab : aa.val ≤ 327615 := by scalar_tac
  have hbbb : bb.val ≤ 2065520 := by
    have := Nat.mod_lt b1.val (by decide : 0 < 65521)
    omega
  have htail : tail.val.length < 4 := by scalar_tac
  have htailA : aa.val + 255*tail.val.length ≤ U32.max := by scalar_tac
  have htailB : bb.val + tail.val.length*(aa.val+255*tail.val.length) ≤ U32.max := by
    have ht : tail.val.length ≤ 3 := by omega
    have ha : aa.val+255*tail.val.length ≤ 328380 := by omega
    have hp : tail.val.length*(aa.val+255*tail.val.length) ≤ 3*328380 := Nat.mul_le_mul ht ha
    scalar_tac
  step with tail_loop as ⟨aend, bend, hend⟩ by
    simp only [List.drop_zero]
    first | exact htailA | exact htailB
  step as ⟨afinal, hafinal⟩
  step as ⟨bfinal, hbfinal⟩
  have hcomb : toZ (aa.val, bb.val) =
      projection (self.a.val : Z) (b2.val : Z) (vectorZ av2) (vectorZ bv2) := by
    apply combine_projection self.a.val b2.val aa.val bb.val (lane av2) (lane bv2)
      (by have := havbound 1; omega) (by have := havbound 2; omega) (by have := havbound 3; omega)
    · simp only [haa', vals_four, List.sum_cons, List.sum_nil]
      omega
    · rw [hbb', hvalsB]
      simp only [List.sum_cons, List.sum_nil, hnew1eq, hnew2eq, hnew3eq, hbv3]
      ring
  have hhead4 : head.val.length % 4 = 0 := by
    simp only [Slice.len_val] at hmod hn
    change head.val.length = n.val at hlen0
    omega
  have hrem40 : it4.remainder.val = [] := by
    apply List.length_eq_zero_iff.mp
    omega
  have hnrem : 4*it4.chunks.length = it.remainder.val.length := by
    have hh := congrArg List.length hrem40
    simp only [List.length_nil] at hh
    omega
  have hflat40 : (it4.chunks.map Slice.val).flatten = it.remainder.val := by
    simpa only [hrem40, List.append_nil] using hflat4
  have hzav2 : vectorZ av2 = vectorZ av1 := by
    funext j
    simp [vectorZ, hav2]
  have hzbv2 : vectorZ bv2 = vectorZ bv1 := by
    funext j
    simp [vectorZ, hbv2]
  have hzbb2 : (b2.val : Z) = (b0.val : Z) +
      (it.remainder.val.length : Z)*(self.a.val : Z) := by
    simp [hb2, hb1, hoff, hcastlen]
  have hproj2 : projection (self.a.val : Z) (b2.val : Z) (vectorZ av2) (vectorZ bv2) =
      zRun (projection (self.a.val : Z) (b0.val : Z) (vectorZ av0) (vectorZ bv0)) it.remainder.val := by
    simpa only [hzav2, hzbv2, hzbb2, hnrem, hflat40] using hproj1
  have hz0 : vectorZ (Array.repeat 4#usize 0#u32) = (fun _ => 0) := by
    funext j
    have hc : j.val = 0 ∨ j.val = 1 ∨ j.val = 2 ∨ j.val = 3 := by omega
    rcases hc with hc | hc | hc | hc <;> simp [vectorZ, lane, Array.repeat, hc]
  have hproj0' : projection (self.a.val : Z) (b0.val : Z) (vectorZ av0) (vectorZ bv0) =
      zRun (toZ (self.a.val, self.b.val)) (it.chunks.map Slice.val).flatten := by
    simpa [hz0, projection, toZ] using hproj0
  have hmid : toZ (aa.val, bb.val) = zRun (toZ (self.a.val, self.b.val)) head.val := by
    rw [hcomb, hproj2, hproj0']
    change zRun (zRun (toZ (self.a.val, self.b.val)) _) _ = _
    have happ : ∀ (s : Z × Z) xs ys, zRun (zRun s xs) ys = zRun s (xs ++ ys) := by
      intros; simp [zRun, List.foldl_append]
    rw [happ, hflat]
  have hbytes : head.val ++ tail.val = bytes.val := by
    rw [hval0, hval1]
    exact List.take_append_drop n.val bytes.val
  have hzfinal : toZ (aend.val, bend.val) =
      toZ (scalar (self.a.val, self.b.val) bytes.val) := by
    rw [hend]
    simp only [List.drop_zero]
    rw [zRun_raw, hmid, zRun_scalar]
    simp only [zRun, ← List.foldl_append, hbytes]
  have heq := normalize_equal_of_toZ_equal _ _ hzfinal
  rw [scalar_normalized] at heq
  have hafb : afinal.val < 65536 := by
    have := Nat.mod_lt aend.val (by decide : 0 < 65521)
    omega
  have hbfb : bfinal.val < 65536 := by
    have := Nat.mod_lt bend.val (by decide : 0 < 65521)
    omega
  have hacast : (UScalar.cast .U16 afinal).val = afinal.val := by
    rw [UScalar.cast_val_eq]
    simpa using Nat.mod_eq_of_lt hafb
  have hbcast : (UScalar.cast .U16 bfinal).val = bfinal.val := by
    rw [UScalar.cast_val_eq]
    simpa using Nat.mod_eq_of_lt hbfb
  rw [hacast, hbcast, hafinal, hbfinal]
  simpa only [normalize] using heq

#print axioms compute_correct
end Verification
