import Vector
import Algebra
open Aeneas Aeneas.Std Result WP
set_option maxHeartbeats 1500000
namespace Verification
open adler2
structure Q where
  x0 : U32
  x1 : U32
  x2 : U32
  x3 : U32

def Q.arr (q : Q) : algo.U32X4 := vec q.x0 q.x1 q.x2 q.x3

def Q.bound (q : Q) (n : Nat) : Prop :=
  q.x0.val ≤ n ∧ q.x1.val ≤ n ∧ q.x2.val ≤ n ∧ q.x3.val ≤ n

def Q.adds (c a b : Q) : Prop :=
  c.x0.val = a.x0.val + b.x0.val ∧ c.x1.val = a.x1.val + b.x1.val ∧
  c.x2.val = a.x2.val + b.x2.val ∧ c.x3.val = a.x3.val + b.x3.val

def Q.mods (c a : Q) : Prop :=
  c.x0.val = a.x0.val % 65521 ∧ c.x1.val = a.x1.val % 65521 ∧
  c.x2.val = a.x2.val % 65521 ∧ c.x3.val = a.x3.val % 65521

def combineQ (a b : F) (x y : Q) : State :=
  combine a b x.x0.val x.x1.val x.x2.val x.x3.val y.x0.val y.x1.val y.x2.val y.x3.val

def Q.bytes (q : Q) (s : Slice U8) : Prop :=
  ∃ d0 d1 d2 d3, s.val = [d0,d1,d2,d3] ∧
    q.x0.val = d0.val ∧ q.x1.val = d1.val ∧ q.x2.val = d2.val ∧ q.x3.val = d3.val

@[step] theorem q_add (a b : Q) (na nb : Nat) (ha : a.bound na) (hb : b.bound nb)
    (hmax : na + nb ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign a.arr b.arr ⦃ r =>
      ∃ c : Q, r = c.arr ∧ c.adds a b ⦄ := by
  rcases ha with ⟨ha0,ha1,ha2,ha3⟩
  rcases hb with ⟨hb0,hb1,hb2,hb3⟩
  apply WP.spec_mono (vec_add a.x0 a.x1 a.x2 a.x3 b.x0 b.x1 b.x2 b.x3
    (by omega) (by omega) (by omega) (by omega))
  intro r hr
  rcases hr with ⟨c0,c1,c2,c3,hr,h0,h1,h2,h3⟩
  exact ⟨⟨c0,c1,c2,c3⟩, hr, h0,h1,h2,h3⟩

@[step] theorem q_rem (a : Q) :
    algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign a.arr 65521#u32 ⦃ r =>
      ∃ c : Q, r = c.arr ∧ c.mods a ∧ c.bound 65520 ⦄ := by
  apply WP.spec_mono (vec_rem a.x0 a.x1 a.x2 a.x3 65521#u32 (by decide))
  intro r hr
  rcases hr with ⟨c0,c1,c2,c3,hr,h0,h1,h2,h3⟩
  refine ⟨⟨c0,c1,c2,c3⟩, hr, ⟨h0,h1,h2,h3⟩, ?_⟩
  change c0.val ≤ 65520 ∧ c1.val ≤ 65520 ∧ c2.val ≤ 65520 ∧ c3.val ≤ 65520
  change c0.val = a.x0.val % 65521 at h0
  change c1.val = a.x1.val % 65521 at h1
  change c2.val = a.x2.val % 65521 at h2
  change c3.val = a.x3.val % 65521 at h3
  omega

lemma byte_le (x : U8) : x.val ≤ 255 := by
  have h := x.hmax
  change x.val < 256 at h
  omega

@[step] theorem q_from (s : Slice U8) (hs : s.val.length = 4) :
    algo.U32X4.from s ⦃ r => ∃ c : Q, r = c.arr ∧ c.bytes s ∧ c.bound 255 ⦄ := by
  apply WP.spec_mono (vec_from s hs)
  intro r hr
  rcases hr with ⟨c0,c1,c2,c3,hr,h0,h1,h2,h3⟩
  have he : s.val = [s.val[0],s.val[1],s.val[2],s.val[3]] := by
    apply List.ext_getElem
    · simp [hs]
    · intro i h1 h2
      have hi : i < 4 := by simpa [hs] using h1
      interval_cases i <;> rfl
  refine ⟨⟨c0,c1,c2,c3⟩, hr, ?_, ?_⟩
  · exact ⟨_,_,_,_,he,h0,h1,h2,h3⟩
  · dsimp [Q.bound]
    rw [h0,h1,h2,h3]
    exact ⟨byte_le _, byte_le _, byte_le _, byte_le _⟩

lemma Q.add_bound {c a b : Q} (hc : c.adds a b) (ha : a.bound na) (hb : b.bound nb) :
    c.bound (na+nb) := by
  rcases hc with ⟨h0,h1,h2,h3⟩
  rcases ha with ⟨ha0,ha1,ha2,ha3⟩
  rcases hb with ⟨hb0,hb1,hb2,hb3⟩
  dsimp [Q.bound]
  omega

lemma combineQ_mod (a b : F) (x y x' y' : Q) (hx : x'.mods x) (hy : y'.mods y) :
    combineQ a b x' y' = combineQ a b x y := by
  rcases hx with ⟨h0,h1,h2,h3⟩
  rcases hy with ⟨g0,g1,g2,g3⟩
  simp only [combineQ, h0,h1,h2,h3,g0,g1,g2,g3]
  simp only [ZMod.natCast_mod]

lemma combineQ_four (a b : F) (x y d x' y' : Q) (s : Slice U8)
    (hd : d.bytes s) (hx : x'.adds x d) (hy : y'.adds y x') :
    combineQ a (b+4*a) x' y' = run (combineQ a b x y) s.val := by
  rcases hd with ⟨d0,d1,d2,d3,hs,h0,h1,h2,h3⟩
  rcases hx with ⟨e0,e1,e2,e3⟩
  rcases hy with ⟨f0,f1,f2,f3⟩
  simp only [combineQ, f0,f1,f2,f3,e0,e1,e2,e3,h0,h1,h2,h3,hs, Nat.cast_add]
  exact combine_four ..
end Verification
