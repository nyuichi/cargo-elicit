import Vectors
import Math
import Mathlib.Tactic.Linarith
open Aeneas Aeneas.Std Result ElicitGenerated
open AdlerMath
namespace Proofs

def combine (a b : F) (av bv : Vec) : S :=
  (a + lane av 0 + lane av 1 + lane av 2 + lane av 3,
   b + 4 * ((lane bv 0 : F) + lane bv 1 + lane bv 2 + lane bv 3) -
     lane av 1 - 2 * lane av 2 - 3 * lane av 3)

def laneBounds (k : Nat) (av bv : Vec) : Prop :=
  ∀ j < 4, lane av j ≤ 65520 + k * 255 ∧
    2 * lane bv j ≤ 2 * 65520 + 2 * k * 65520 + 255 * k * (k + 1)

theorem next_bounds {k : Nat} (hk : k < 5552) (av bv : Vec)
    (h : laneBounds k av bv) (dv : Vec)
    (hd : ∀ j < 4, lane dv j ≤ 255) :
    (∀ j < 4, lane av j + lane dv j ≤ 4294967295) ∧
    (∀ j < 4, lane bv j + (lane av j + lane dv j) ≤ 4294967295) := by
  constructor <;> intro j hj
  · have := (h j hj).1
    have := hd j hj
    omega
  · have hja := (h j hj).1
    have hjb := (h j hj).2
    have hjd := hd j hj
    have hpoly : k * k ≤ 5551 * k := Nat.mul_le_mul_right k (by omega)
    nlinarith

theorem advance_bounds {k : Nat} (av bv dv av' bv' : Vec)
    (h : laneBounds k av bv) (hd : ∀ j < 4, lane dv j ≤ 255)
    (ha : ∀ j < 4, lane av' j = lane av j + lane dv j)
    (hb : ∀ j < 4, lane bv' j = lane bv j + lane av' j) :
    laneBounds (k+1) av' bv' := by
  intro j hj
  have hja := (h j hj).1
  have hjb := (h j hj).2
  have hjd := hd j hj
  rw [hb j hj, ha j hj]
  constructor <;> nlinarith

theorem combine_four (a b : F) (av bv dv av' bv' : Vec) (s : Slice U8)
    (hs : s.val.length = 4)
    (hd : ∀ j < 4, lane dv j = (s.val[j]!).val)
    (ha : ∀ j < 4, lane av' j = lane av j + lane dv j)
    (hb : ∀ j < 4, lane bv' j = lane bv j + lane av' j) :
    combine a (b + 4*a) av' bv' = scalar (combine a b av bv) s.val := by
  have hl : ∃ x0 x1 x2 x3, s.val = [x0,x1,x2,x3] := by
    generalize hx : s.val = xs at hs ⊢
    rcases xs with _ | ⟨x0, _ | ⟨x1, _ | ⟨x2, _ | ⟨x3, _ | ⟨x4, xs⟩⟩⟩⟩⟩ <;>
      simp_all
  obtain ⟨x0,x1,x2,x3,hxs⟩ := hl
  have ha0 := ha 0 (by decide); have ha1 := ha 1 (by decide)
  have ha2 := ha 2 (by decide); have ha3 := ha 3 (by decide)
  have hb0 := hb 0 (by decide); have hb1 := hb 1 (by decide)
  have hb2 := hb 2 (by decide); have hb3 := hb 3 (by decide)
  have hd0 := hd 0 (by decide); have hd1 := hd 1 (by decide)
  have hd2 := hd 2 (by decide); have hd3 := hd 3 (by decide)
  simp only [hxs, List.getElem!_cons_zero, List.getElem!_cons_succ] at hd0 hd1 hd2 hd3
  simp only [combine, ha0, ha1, ha2, ha3, hb0, hb1, hb2, hb3,
    hd0, hd1, hd2, hd3, Nat.cast_add, hxs, scalar, List.foldl_cons,
    List.foldl_nil, scalarStep]
  apply Prod.ext <;> simp only <;> ring

abbrev Chunks := core.slice.iter.ChunksExact U8
def flatten (cs : List (Slice U8)) : List U8 := cs.flatMap Slice.val

def innerPost (it : Chunks) (k : Nat) (av bv : Vec) (out : Vec × Vec) : Prop :=
  laneBounds (k + it.chunks.length) out.1 out.2 ∧
  ∀ a b : F,
    combine a (b + (4 * it.chunks.length : Nat) * a) out.1 out.2 =
      scalar (combine a b av bv) (flatten it.chunks)

@[step] theorem inner_spec (it : Chunks) (av bv : Vec) (k : Nat)
    (hk : k + it.chunks.length ≤ 5552)
    (hb : laneBounds k av bv)
    (hc : ∀ c ∈ it.chunks, c.val.length = 4) :
    algo.Adler32.compute_loop0_loop0 it av bv ⦃ innerPost it k av bv ⦄ := by
  unfold algo.Adler32.compute_loop0_loop0
  rw [loop]
  cases he : it.chunks with
  | nil =>
    simp [algo.Adler32.compute_loop0_loop0.body,
      core.slice.iter.IteratorChunksExact.next, he, innerPost,
      flatten, scalar, hb]
  | cons c cs =>
    have hcl : c.val.length = 4 := hc c (by simp [he])
    have hcs : ∀ c ∈ cs, c.val.length = 4 := by
      intro d hd; exact hc d (by simp [he, hd])
    simp only [algo.Adler32.compute_loop0_loop0.body,
      core.slice.iter.IteratorChunksExact.next, he, bind_ok]
    step as ⟨dv, hd⟩
    have hdb : ∀ j < 4, lane dv j ≤ 255 := by
      intro j hj; rw [hd j hj]; scalar_tac
    have hsafe := next_bounds (by simp only [he, List.length_cons] at hk; omega) av bv hb dv hdb
    step as ⟨av', ha⟩
    step as ⟨bv', hb'⟩
    have hbounds := advance_bounds av bv dv av' bv' hb hdb ha hb'
    have hrec := inner_spec ⟨cs, it.remainder⟩ av' bv' (k+1) (by
      simp only [he, List.length_cons] at hk; simpa [Nat.add_assoc, Nat.add_comm, Nat.add_left_comm] using hk) hbounds hcs
    change algo.Adler32.compute_loop0_loop0 ⟨cs, it.remainder⟩ av' bv' ⦃ innerPost it k av bv ⦄
    step with hrec as ⟨out, hout⟩
    obtain ⟨ho, heq⟩ := hout
    simp only [innerPost, he, List.length_cons]
    constructor
    · simpa [Nat.add_assoc, Nat.add_comm, Nat.add_left_comm] using ho
    · intro a b
      have hr := heq a (b + 4*a)
      have hfour := combine_four a b av bv dv av' bv' c hcl hd ha hb'
      simp only [flatten, List.flatMap_cons, scalar, List.foldl_append]
      rw [hfour] at hr
      have hoff : b + 4*a + (4 * cs.length : Nat) * a =
          b + (4 * (cs.length + 1) : Nat) * a := by push_cast; ring
      rw [hoff] at hr
      exact hr
termination_by it.chunks.length
decreasing_by simp_all

theorem inner1_spec (it : Chunks) (av bv : Vec) (k : Nat)
    (hk : k + it.chunks.length ≤ 5552)
    (hb : laneBounds k av bv)
    (hc : ∀ c ∈ it.chunks, c.val.length = 4) :
    algo.Adler32.compute_loop1 it av bv ⦃ innerPost it k av bv ⦄ := by
  have heq : algo.Adler32.compute_loop1 = algo.Adler32.compute_loop0_loop0 := by
    rfl
  rw [heq]
  exact inner_spec it av bv k hk hb hc

end Proofs
