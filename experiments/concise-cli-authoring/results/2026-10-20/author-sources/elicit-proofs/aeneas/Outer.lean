import Algebra
open Aeneas Aeneas.Std Result ElicitGenerated
set_option maxHeartbeats 2000000
namespace Outer
open Vector LaneAlgebra
abbrev normalized (v : V) : Prop := ∀ i, lane v i ≤ 65520
 theorem reduce_lanes (v w : V) (h : ∀ i, lane w i = lane v i % 65521) :
    normalized w ∧ lanes w = lanes v := by
  constructor
  · intro i
    rw [h i]
    have := Nat.mod_lt (lane v i) (by decide : 0 < 65521)
    omega
  · funext i
    simp only [lanes, h i, cast_mod]
@[step] theorem outer_spec (cs : List (Slice U8)) (rem : Slice U8) (a b : U32) (av bv : V)
    (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
    (hav : normalized av) (hbv : normalized bv)
    (hlen : ∀ c ∈ cs, c.val.length = 22208) :
    algo.Adler32.compute_loop0 22208#usize ⟨cs, rem⟩ a b av bv
      ⦃ out => out.1.val ≤ 65535 ∧ normalized out.2.1 ∧ normalized out.2.2 ∧
        combine (a.val : Z) (out.1.val : Z) (lanes out.2.1) (lanes out.2.2) =
          run (combine (a.val : Z) (b.val : Z) (lanes av) (lanes bv)) (Chunks.values cs) ⦄ := by
  induction cs generalizing b av bv with
  | nil =>
    unfold algo.Adler32.compute_loop0
    rw [loop]
    dsimp only
    simp [algo.Adler32.compute_loop0.body, core.slice.iter.IteratorChunksExact.next, hb, run]
    exact ⟨hav, hbv⟩
  | cons c cs ih =>
    have hc := hlen c (by simp)
    have ht : ∀ d ∈ cs, d.val.length = 22208 := by
      intro d hd
      exact hlen d (by simp [hd])
    unfold algo.Adler32.compute_loop0
    rw [loop]
    dsimp only
    rw [algo.Adler32.compute_loop0.body, core.slice.iter.IteratorChunksExact.next]
    step with Chunks.four_spec c (by simp [hc]) as ⟨it, hvals, hlens, hnumber⟩
    have hcount : it.chunks.length = 5552 := by simpa [hc] using hnumber
    have hbud : ∀ i, Inner.budget it.chunks.length (lane av i) (lane bv i) := by
      intro i
      exact Inner.budget_initial _ _ _ (by omega) (hav i) (hbv i)
    step with Inner.inner_spec it.chunks it.remainder av bv hlens hbud as ⟨vout, hinner⟩
    rcases vout with ⟨av1, bv1⟩
    simp only [lift, algo.Adler32.compute.MOD]
    step as ⟨prod, hprod⟩ <;> try scalar_tac
    step as ⟨b1, hb1⟩ <;> try scalar_tac
    step as ⟨av2, hav2⟩
    step as ⟨bv2, hbv2⟩
    step as ⟨b2, hb2⟩
    have hav2' : ∀ i, lane av2 i = lane av1 i % 65521 := hav2
    have hbv2' : ∀ i, lane bv2 i = lane bv1 i % 65521 := hbv2
    have hva := reduce_lanes av1 av2 hav2'
    have hvb := reduce_lanes bv1 bv2 hbv2'
    have hbound : b2.val ≤ 65535 := by
      have : b2.val = b1.val % 65521 := hb2
      omega
    have heq : combine (a.val : Z) (b2.val : Z) (lanes av2) (lanes bv2) =
        run (combine (a.val : Z) (b.val : Z) (lanes av) (lanes bv)) c.val := by
      rw [hva.2, hvb.2]
      have hbc : (b2.val : Z) = (b.val : Z) + 4 * (it.chunks.length : Z) * (a.val : Z) := by
        simp only [hb2, cast_mod, hb1, Nat.cast_add, hprod, Nat.cast_mul]
        rw [hcount]
        have hcast : (UScalar.cast .U32 22208#usize).val = 22208 := by
          have h := UScalar.cast_inBounds_spec .U32 22208#usize (by scalar_tac)
          simpa [lift, WP.spec_ok] using h
        rw [hcast]
        norm_num
      rw [hbc]
      rw [inner_combine it.chunks (a.val : Z) (b.val : Z) av bv av1 bv1 hlens hinner, hvals]
    have hi := ih b2 av2 bv2 hbound hva.1 hvb.1 ht
    apply WP.spec_mono (by
      simpa only [algo.Adler32.compute_loop0, algo.Adler32.compute_loop0.body,
        core.slice.iter.IteratorChunksExact.next, algo.Adler32.compute.MOD, lift] using hi)
    intro out hout
    refine ⟨hout.1, hout.2.1, hout.2.2.1, ?_⟩
    rw [hout.2.2.2, heq, Chunks.values_cons, run_append]

end Outer
