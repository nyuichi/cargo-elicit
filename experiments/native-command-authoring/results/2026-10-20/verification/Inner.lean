import Bounds
open Aeneas Aeneas.Std Result WP
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096
namespace Verification
open adler2
abbrev flat (cs : List (Slice U8)) := cs.flatMap (fun s => s.val)

def innerPost (cs : List (Slice U8)) (k : Nat) (x y : Q)
    (r : algo.U32X4 × algo.U32X4) : Prop :=
  ∃ u v : Q, r = (u.arr, v.arr) ∧
    u.bound (ab (k+cs.length)) ∧ v.twiceBound (bb (k+cs.length)) ∧
    ∀ a b : F, combineQ a (b + (4*cs.length : Nat)*a) u v =
      run (combineQ a b x y) (flat cs)

@[step] theorem inner_correct (cs : List (Slice U8)) (rem : Slice U8) (k : Nat) (x y : Q)
    (hcs : ∀ c ∈ cs, c.val.length = 4) (hk : k + cs.length ≤ 5552)
    (hx : x.bound (ab k)) (hy : y.twiceBound (bb k)) :
    algo.Adler32.compute_loop0_loop0 ⟨cs,rem⟩ x.arr y.arr ⦃ innerPost cs k x y ⦄ := by
  induction cs generalizing k x y with
  | nil =>
    unfold algo.Adler32.compute_loop0_loop0
    rw [loop]
    simp only [algo.Adler32.compute_loop0_loop0.body,
      core.slice.iter.IteratorChunksExact.next, bind_ok, bind_tc_ok, spec_ok, innerPost]
    try dsimp only [Std.uncurry, WP.uncurry']
    try simp only [bind_ok, bind_tc_ok, spec_ok, Std.uncurry, WP.uncurry']
    change innerPost [] k x y (x.arr,y.arr)
    refine ⟨x,y,rfl,?_,?_,?_⟩
    · simpa using hx
    · simpa using hy
    · intro a b; simp [flat, combineQ, run]
  | cons c cs ih =>
    have hc := hcs c (by simp)
    have hcs' : ∀ c ∈ cs, c.val.length = 4 := by intro c h; exact hcs c (by simp [h])
    have hk' : k < 5552 := by simp only [List.length_cons] at hk; omega
    have hxa : x.bound 1481025 := Q.bound_mono hx (by unfold ab; omega)
    have hyb : y.bound 4293208920 := Q.twice_mono hy (bb_small (by omega))
    unfold algo.Adler32.compute_loop0_loop0
    rw [loop]
    simp only [algo.Adler32.compute_loop0_loop0.body,
      core.slice.iter.IteratorChunksExact.next, bind_ok, bind_tc_ok]
    step with q_from c hc as ⟨r, d, hr, hd, hdb⟩
    subst r
    step with q_add x d 1481025 255 hxa hdb (by scalar_tac) as ⟨r, x', hr, hx'⟩
    subst r
    have hxb : x'.bound (ab (k+1)) := by
      rw [← ab_step]
      exact Q.add_bound hx' hx hdb
    have hxb' := Q.bound_mono hxb (ab_small (by omega))
    step with q_add y x' 4293208920 1481280 hyb hxb' (by scalar_tac) as ⟨r, y', hr, hy'⟩
    subst r
    have hybb : y'.twiceBound (bb (k+1)) := by
      rw [← bb_step]
      exact Q.twice_add_bound hy' hy hxb
    try dsimp only [Std.uncurry, WP.uncurry']
    try simp only [bind_ok, bind_tc_ok, Std.uncurry, WP.uncurry']
    change algo.Adler32.compute_loop0_loop0 ⟨cs, rem⟩ x'.arr y'.arr ⦃ _ ⦄
    apply WP.spec_mono (ih (k+1) x' y' hcs' (by simpa [List.length_cons, Nat.add_assoc, Nat.add_comm, Nat.add_left_comm] using hk) hxb hybb)
    intro r hr
    rcases hr with ⟨u,v,hr,hu,hv,hsem⟩
    refine ⟨u,v,hr,?_,?_,?_⟩
    · simpa [List.length_cons, Nat.add_assoc, Nat.add_comm, Nat.add_left_comm] using hu
    · simpa [List.length_cons, Nat.add_assoc, Nat.add_comm, Nat.add_left_comm] using hv
    · intro a b
      have hm := hsem a (b+4*a)
      have hf := combineQ_four a b x y d x' y' c hd hx' hy'
      have he : b + (4 * (c::cs).length : Nat)*a = (b+4*a) + (4*cs.length : Nat)*a := by
        simp only [List.length_cons, Nat.cast_mul, Nat.cast_add, Nat.cast_ofNat]
        ring
      rw [he, hm, hf]
      simp only [flat, List.flatMap_cons, run_append]

 theorem inner_duplicate (i : core.slice.iter.ChunksExact U8) (a b : algo.U32X4) :
    algo.Adler32.compute_loop1 i a b = algo.Adler32.compute_loop0_loop0 i a b := by rfl
end Verification
