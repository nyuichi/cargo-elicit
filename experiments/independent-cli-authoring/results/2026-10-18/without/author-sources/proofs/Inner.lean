import Vector
open Aeneas Aeneas.Std Result ControlFlow
namespace AdlerProof
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096

abbrev Lanes := Fin 4 → Nat

def byteAt (s : Slice U8) (i : Fin 4) : Nat := s.val[i.val]!.val

theorem byteAt_bound (s : Slice U8) (i : Fin 4) : byteAt s i ≤ 255 := by
  have h := (s.val[i.val]!).hmax
  exact Nat.le_of_lt_succ h

def laneBound (k : Nat) (a b : Lanes) : Prop :=
  ∀ i, a i ≤ 65520 + 255*k ∧ 2*b i ≤ 131040 + 131040*k + 255*k*(k+1)

def laneRun : List (Slice U8) → Lanes → Lanes → Lanes × Lanes
 | [], a, b => (a,b)
 | c::cs, a, b =>
   let a' := fun i => a i + byteAt c i
   laneRun cs a' (fun i => b i + a' i)

theorem laneBound_step {k : Nat} {a b : Lanes} (h : laneBound k a b) (c : Slice U8) :
 laneBound (k+1) (fun i => a i + byteAt c i) (fun i => b i + (a i + byteAt c i)) := by
 intro i
 have hi := h i
 have hc := byteAt_bound c i
 constructor <;> dsimp <;> nlinarith

theorem laneBound_safe {k : Nat} {a b : Lanes} (hk : k < 5552) (h : laneBound k a b)
 (c : Slice U8) :
 (∀ i, a i + byteAt c i ≤ 4294967295) ∧
 (∀ i, b i + (a i + byteAt c i) ≤ 4294967295) := by
 have h' := laneBound_step h c
 have hk' : k+1 ≤ 5552 := by omega
 constructor
 · intro i; have hi := (h' i).1; nlinarith
 · intro i
   have hi := (h' i).2
   have hm : (k+1)*(k+2) ≤ 5552*5553 := by nlinarith
   nlinarith

@[step] theorem vector_from_lanes (s : Slice U8) (h : s.val.length = 4) :
 adler2.algo.U32X4.from s ⦃ v => view v = byteAt s ⦄ := by
 have hv := vector_from s h
 apply WP.spec_mono hv
 intro v hv
 funext i
 rw [hv i]
 simp [byteAt, List.getElem!_eq_getElem?_getD, List.getElem?_eq_getElem, h]

-- The loop theorem uses the iterator's actual list as its termination measure.
theorem inner_loop (cs : List (Slice U8)) (rem : Slice U8)
 (av bv : adler2.algo.U32X4) (k : Nat)
 (hcs : ∀ c ∈ cs, c.val.length = 4)
 (hlen : k + cs.length ≤ 5552) (hb : laneBound k (view av) (view bv)) :
 adler2.algo.Adler32.compute_loop0_loop0 ⟨cs, rem⟩ av bv
 ⦃ r => (view r.1, view r.2) = laneRun cs (view av) (view bv) ∧
   laneBound (k + cs.length) (view r.1) (view r.2) ⦄ := by
 induction cs generalizing av bv k with
 | nil =>
   unfold adler2.algo.Adler32.compute_loop0_loop0
   rw [loop]
   simpa [adler2.algo.Adler32.compute_loop0_loop0.body,
     core.slice.iter.IteratorChunksExact.next, laneRun, WP.spec_ok] using hb
 | cons c cs ih =>
   have hc := hcs c (by simp)
   have htail : ∀ x ∈ cs, x.val.length = 4 := by intro x hx; exact hcs x (by simp [hx])
   have hs := laneBound_safe (by simp at hlen; omega : k < 5552) hb c
   unfold adler2.algo.Adler32.compute_loop0_loop0
   rw [loop]
   simp only [adler2.algo.Adler32.compute_loop0_loop0.body,
     core.slice.iter.IteratorChunksExact.next]
   step as ⟨val, hval⟩
   step with vector_add av val (by simpa [hval] using hs.1) as ⟨av', ha⟩
   step with vector_add bv av' (by simpa [ha, hval] using hs.2) as ⟨bv', hb'⟩
   change adler2.algo.Adler32.compute_loop0_loop0 ⟨cs, rem⟩ av' bv' ⦃ _ ⦄
   have hbound : laneBound (k+1) (view av') (view bv') := by
     simpa [ha, hb', hval] using laneBound_step hb c
   have hit := ih av' bv' (k+1) htail (by simp at hlen; omega) hbound
   apply WP.spec_mono hit
   intro r hr
   simpa [laneRun, ha, hb', hval, Nat.add_assoc, Nat.add_comm, Nat.add_left_comm] using hr

theorem final_inner_loop (cs : List (Slice U8)) (rem : Slice U8)
 (av bv : adler2.algo.U32X4) (k : Nat)
 (hcs : ∀ c ∈ cs, c.val.length = 4)
 (hlen : k + cs.length ≤ 5552) (hb : laneBound k (view av) (view bv)) :
 adler2.algo.Adler32.compute_loop1 ⟨cs, rem⟩ av bv
 ⦃ r => (view r.1, view r.2) = laneRun cs (view av) (view bv) ∧
   laneBound (k + cs.length) (view r.1) (view r.2) ⦄ := by
  exact inner_loop cs rem av bv k hcs hlen hb
