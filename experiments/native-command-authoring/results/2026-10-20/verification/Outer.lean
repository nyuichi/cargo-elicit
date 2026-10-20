import Chunks
open Aeneas Aeneas.Std Result WP
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096
namespace Verification
open adler2

lemma cast32_val {ty : UScalarTy} (x : UScalar ty) :
    (UScalar.cast .U32 x).val = x.val % 4294967296 := by
  change (x.bv.setWidth 32).toNat = _
  exact BitVec.toNat_setWidth 32 x.bv

@[step] theorem outer_body_correct (c : Slice U8) (cs : List (Slice U8)) (rem : Slice U8)
    (a b : U32) (x y : Q) (hc : c.val.length = 22208)
    (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
    (hx : x.bound 65520) (hy : y.bound 65520) :
    algo.Adler32.compute_loop0.body 22208#usize a ⟨c::cs,rem⟩ b x.arr y.arr ⦃ r =>
      ∃ b' : U32, ∃ x' y' : Q,
        r = .cont (⟨cs,rem⟩, b', x'.arr, y'.arr) ∧
        b'.val ≤ 65535 ∧ x'.bound 65520 ∧ y'.bound 65520 ∧
        combineQ (a.val:F) b'.val x' y' = run (combineQ a.val b.val x y) c.val ⦄ := by
  unfold algo.Adler32.compute_loop0.body
  simp only [core.slice.iter.IteratorChunksExact.next, bind_ok, Std.uncurry, lift]
  step with chunks_spec c 4#usize (by decide) as ⟨it,hcs,hf,hr⟩
  have hcomplete := four_complete c it hcs hf hr (by omega)
  step with inner_aligned c it x y hcs hcomplete.1 (by omega) hx hy as ⟨ra,rb,u,v,hea,heb,hm⟩
  subst ra rb
  try simp only [lift, bind_ok, Std.uncurry]
  step as ⟨p,hp⟩
  have hp' : p.val = 22208*a.val := by scalar_tac
  step as ⟨b1,hb1⟩
  simp only [algo.Adler32.compute.MOD]
  step with q_rem u as ⟨ru,u',heu,hu',hub⟩
  subst ru
  step with q_rem v as ⟨rv,v',hev,hv',hvb⟩
  subst rv
  step as ⟨b2,hb2⟩
  try simp only [spec_ok]
  refine ⟨b2,u',v',rfl,?_,hub,hvb,?_⟩
  · change b2.val = b1.val % 65521 at hb2
    omega
  · have hp' : p.val = 22208*a.val := by scalar_tac
    have hb2' : b2.val = (b.val+22208*a.val)%65521 := by
      change b2.val = b1.val % 65521 at hb2
      rw [hb2,hb1,hp']
    rw [combineQ_mod _ _ _ _ _ _ hu' hv']
    rw [hb2']
    simp only [ZMod.natCast_mod, Nat.cast_add, Nat.cast_mul, Nat.cast_ofNat]
    simpa only [hc, Nat.cast_ofNat] using hm (a.val:F) (b.val:F)

def outerPost (cs : List (Slice U8)) (a b : U32) (x y : Q)
    (r : U32 × algo.U32X4 × algo.U32X4) : Prop :=
  ∃ b' : U32, ∃ x' y' : Q, r = (b',x'.arr,y'.arr) ∧
    b'.val ≤ 65535 ∧ x'.bound 65520 ∧ y'.bound 65520 ∧
    combineQ a.val b'.val x' y' = run (combineQ a.val b.val x y) (flat cs)

@[step] theorem outer_correct (cs : List (Slice U8)) (rem : Slice U8)
    (a b : U32) (x y : Q) (hcs : ∀ c ∈ cs, c.val.length = 22208)
    (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
    (hx : x.bound 65520) (hy : y.bound 65520) :
    algo.Adler32.compute_loop0 22208#usize ⟨cs,rem⟩ a b x.arr y.arr ⦃ outerPost cs a b x y ⦄ := by
  induction cs generalizing b x y with
  | nil =>
    unfold algo.Adler32.compute_loop0
    rw [loop]
    simp only [algo.Adler32.compute_loop0.body, core.slice.iter.IteratorChunksExact.next,
      bind_ok, Std.uncurry, spec_ok]
    try simp only [bind_ok, spec_ok]
    refine ⟨b,x,y,rfl,hb,hx,hy,?_⟩
    rfl
  | cons c cs ih =>
    have hc := hcs c (by simp)
    have hcs' : ∀ c ∈ cs, c.val.length = 22208 := by intro c h; exact hcs c (by simp [h])
    unfold algo.Adler32.compute_loop0
    rw [loop]
    step with outer_body_correct c cs rem a b x y hc ha hb hx hy as
      ⟨r,b',x',y',he,hb',hx',hy',hm⟩
    subst r
    change algo.Adler32.compute_loop0 22208#usize ⟨cs,rem⟩ a b' x'.arr y'.arr ⦃ _ ⦄
    apply WP.spec_mono (ih b' x' y' hcs' hb' hx' hy')
    intro r hr
    rcases hr with ⟨b'',x'',y'',hr,hb'',hx'',hy'',hm'⟩
    refine ⟨b'',x'',y'',hr,hb'',hx'',hy'',?_⟩
    rw [hm',hm]
    exact (run_append _ _ _).symm
end Verification
