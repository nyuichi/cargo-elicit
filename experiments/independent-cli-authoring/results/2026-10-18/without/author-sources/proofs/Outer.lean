import Mathematics
open Aeneas Aeneas.Std Result ControlFlow
namespace AdlerProof
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096

@[step] theorem cast_u32 {ty} (x : UScalar ty) (hx : x.val ≤ 4294967295) :
 lift (UScalar.cast .U32 x) ⦃ y => y.val = x.val ⦄ := by
 apply UScalar.cast_inBounds_spec
 simpa [UScalar.max, U32.max, U32.numBits] using hx

@[simp] theorem mod_value : adler2.algo.Adler32.compute.MOD.val = 65521 := by
 simp [adler2.algo.Adler32.compute.MOD]

theorem normalized_lanes (a b : Lanes) :
 laneBound 0 (fun i => a i % 65521) (fun i => b i % 65521) := by
 intro i
 have h0 := Nat.mod_lt (a i) (by decide : 0 < 65521)
 have h1 := Nat.mod_lt (b i) (by decide : 0 < 65521)
 constructor <;> dsimp <;> omega

theorem outer_loop (cs : List (Slice U8)) (rem : Slice U8)
 (a b : U32) (av bv : adler2.algo.U32X4)
 (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
 (hcs : ∀ c ∈ cs, c.val.length = 22208)
 (hv : laneBound 0 (view av) (view bv)) :
 adler2.algo.Adler32.compute_loop0 22208#usize ⟨cs,rem⟩ a b av bv
 ⦃ r => r.1.val ≤ 65535 ∧ laneBound 0 (view r.2.1) (view r.2.2) ∧
   interp a.val r.1.val (view r.2.1) (view r.2.2) =
   rfold (interp a.val b.val (view av) (view bv)) (cs.map Slice.val).flatten ⦄ := by
 induction cs generalizing b av bv with
 | nil =>
   unfold adler2.algo.Adler32.compute_loop0
   rw [loop]
   simp [adler2.algo.Adler32.compute_loop0.body,
     core.slice.iter.IteratorChunksExact.next, WP.spec_ok, rfold, hb, hv]
 | cons c cs ih =>
   have hc := hcs c (by simp)
   have ht : ∀ x ∈ cs, x.val.length = 22208 := by intro x hx; exact hcs x (by simp [hx])
   unfold adler2.algo.Adler32.compute_loop0
   rw [loop]
   simp only [adler2.algo.Adler32.compute_loop0.body,
     core.slice.iter.IteratorChunksExact.next]
   step with chunks_four c (by omega) as ⟨it, hi, hrem, hflat, hlen⟩
   step with inner_loop it.chunks it.remainder av bv 0 hi (by omega) hv as ⟨r, hr, hv'⟩
   rcases r with ⟨av', bv'⟩
   dsimp only at *
   step as ⟨n, hn⟩
   step as ⟨prod, hprod⟩
   step as ⟨b', hb'⟩
   step with vector_rem av' adler2.algo.Adler32.compute.MOD (by simp) as ⟨av'', hav⟩
   step with vector_rem bv' adler2.algo.Adler32.compute.MOD (by simp) as ⟨bv'', hbv⟩
   step with U32.rem_spec b' (y := adler2.algo.Adler32.compute.MOD) (by simp) as ⟨b'', hb''⟩
   simp only [mod_value] at hav hbv hb''
   have hnorm : laneBound 0 (view av'') (view bv'') := by
     rw [hav, hbv]
     exact normalized_lanes _ _
   have hbb : b''.val ≤ 65535 := by
     have hh := Nat.mod_lt b'.val (by decide : 0 < 65521)
     omega
   have haction := laneRun_action it.chunks hi a.val b.val (view av) (view bv)
   rw [← hr, hflat] at haction
   have hbase : b'.val = b.val + 4*it.chunks.length*a.val := by
     rw [hb', hprod, hn, hlen, hc]
   have hinterp : interp a.val b''.val (view av'') (view bv'') =
       rfold (interp a.val b.val (view av) (view bv)) c.val := by
     rw [hav, hbv, hb'', interp_mod, hbase]
     exact haction
   change adler2.algo.Adler32.compute_loop0 22208#usize ⟨cs,rem⟩ a b'' av'' bv'' ⦃ _ ⦄
   apply WP.spec_mono (ih b'' av'' bv'' hbb ht hnorm)
   intro r hr'
   refine ⟨hr'.1, hr'.2.1, ?_⟩
   rw [hr'.2.2, hinterp]
   simp [rfold_append]
