import Loops
import Chunks
import Algebra
open Aeneas Aeneas.Std Result ControlFlow
namespace Verification
open ElicitGenerated
set_option maxHeartbeats 2000000
set_option linter.unusedSimpArgs false
set_option linter.unnecessarySimpa false

theorem outer_loop (cs : List (Slice U8)) (rem : Slice U8)
    (a b : U32) (av bv : algo.U32X4)
    (ha : a.val ≤ 65535) (hb : b.val ≤ 65535) (hv : laneBounds 0 av bv)
    (hc : ∀ c ∈ cs, c.val.length = 22208) :
    algo.Adler32.compute_loop0 22208#usize ⟨cs, rem⟩ a b av bv
      ⦃ b' av' bv' => b'.val ≤ 65535 ∧ laneBounds 0 av' bv' ∧
        projection (a.val : Z) (b'.val : Z) (vectorZ av') (vectorZ bv') =
          zRun (projection (a.val : Z) (b.val : Z) (vectorZ av) (vectorZ bv))
            (cs.map Slice.val).flatten ⦄ := by
  induction cs generalizing b av bv with
  | nil =>
    unfold algo.Adler32.compute_loop0
    unfold loop
    simp [algo.Adler32.compute_loop0.body, core.slice.iter.IteratorChunksExact.next,
      hb, hv, zRun]
  | cons c cs ih =>
    have hcl : c.val.length = 22208 := hc c (by simp)
    have hcs : ∀ c ∈ cs, c.val.length = 22208 := by simp_all
    unfold algo.Adler32.compute_loop0
    unfold loop
    simp only [algo.Adler32.compute_loop0.body, core.slice.iter.IteratorChunksExact.next,
      bind_tc_ok]
    step with chunks_exact_correct as ⟨it, hfour, hflat, hrem⟩
    have hcount := chunks_count c it 4 ⟨hfour, hflat⟩
    have hlen : it.chunks.length ≤ 5552 := by omega
    step with vector_inner_loop it.chunks it.remainder av bv 0 hfour (by omega) hv
      as ⟨lanes, hv1, heq⟩
    rcases lanes with ⟨av1, bv1⟩
    step as ⟨i, hi⟩
    step as ⟨prod, hp⟩
    step as ⟨b1, hb1⟩
    step with vector_rem_lane as ⟨av2, hav2⟩ by simp [algo.Adler32.compute.MOD]
    step with vector_rem_lane as ⟨bv2, hbv2⟩ by simp [algo.Adler32.compute.MOD]
    step as ⟨b2, hb2⟩ by simp [algo.Adler32.compute.MOD]
    have hbb : b2.val ≤ 65535 := by
      have := Nat.mod_lt b1.val (by decide : 0 < 65521)
      simp_all [algo.Adler32.compute.MOD]
      omega
    have hvv : laneBounds 0 av2 bv2 := by
      intro j
      have h1 := Nat.mod_lt (lane av1 j) (by decide : 0 < 65521)
      have h2 := Nat.mod_lt (lane bv1 j) (by decide : 0 < 65521)
      simp_all [algo.Adler32.compute.MOD]
      omega
    have hn : it.chunks.length = 5552 := by omega
    have hrem0 : it.remainder.val = [] := List.length_eq_zero_iff.mp (by omega)
    have hflat0 : (it.chunks.map Slice.val).flatten = c.val := by simpa [hrem0] using hflat
    have hzav : vectorZ av2 = vectorZ av1 := by
      funext j
      simp [vectorZ, hav2, algo.Adler32.compute.MOD]
    have hzbv : vectorZ bv2 = vectorZ bv1 := by
      funext j
      simp [vectorZ, hbv2, algo.Adler32.compute.MOD]
    have hzbb : (b2.val : Z) = (b.val : Z) + 22208*(a.val : Z) := by
      simp [hb2, hb1, hp, hi, algo.Adler32.compute.MOD]
    have hproj : projection (a.val : Z) (b2.val : Z) (vectorZ av2) (vectorZ bv2) =
        zRun (projection (a.val : Z) (b.val : Z) (vectorZ av) (vectorZ bv)) c.val := by
      simpa only [hzav, hzbv, hzbb, hn, hflat0, Nat.reduceMul, Nat.cast_ofNat] using
        lanes_projection it.chunks av bv av1 bv1 (a.val : Z) (b.val : Z) hfour heq
    change algo.Adler32.compute_loop0 22208#usize ⟨cs, rem⟩ a b2 av2 bv2 ⦃ _ ⦄
    apply WP.spec_mono (ih b2 av2 bv2 hbb hvv hcs)
    intro r hr
    rcases r with ⟨bb, aa, vv⟩
    change _ ∧ _ ∧ _ at hr
    change _ ∧ _ ∧ _
    refine ⟨hr.1, hr.2.1, ?_⟩
    rw [hr.2.2, hproj]
    simp [zRun, List.foldl_append]

#print axioms outer_loop
end Verification
