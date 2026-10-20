import Vector
import Chunks
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Inner
open Vector
abbrev max32 : Nat := 4294967295
def budget (n a b : Nat) : Prop :=
  a + 255*n ≤ max32 ∧ 2*b + 2*n*a + 255*n*(n+1) ≤ 2*max32
 theorem budget_step (n a b d : Nat) (hd : d ≤ 255) (h : budget (n+1) a b) :
    a+d ≤ max32 ∧ b+(a+d) ≤ max32 ∧ budget n (a+d) (b+(a+d)) := by
  unfold budget at *
  have hn : 0 ≤ n := Nat.zero_le n
  have hnd : n*d ≤ n*255 := Nat.mul_le_mul_left n hd
  constructor
  · nlinarith [h.1]
  constructor
  · nlinarith [h.2]
  constructor <;> nlinarith [h.1, h.2]
 theorem budget_initial (n a b : Nat) (hn : n ≤ 5552) (ha : a ≤ 65520) (hb : b ≤ 65520) :
    budget n a b := by
  have hna : n*a ≤ 5552*65520 := Nat.mul_le_mul hn ha
  have hnn : n*(n+1) ≤ 5552*5553 := Nat.mul_le_mul hn (by omega)
  constructor <;> dsimp [max32] <;> nlinarith

def laneStep (i : Fin 4) (s : Nat × Nat) (c : Slice U8) : Nat × Nat :=
  (s.1 + byteLane c i, s.2 + (s.1 + byteLane c i))
def model (cs : List (Slice U8)) (a b : V) (i : Fin 4) : Nat × Nat :=
  cs.foldl (laneStep i) (lane a i, lane b i)
 theorem byte_bound (c : Slice U8) (i : Fin 4) : byteLane c i ≤ 255 := by
  unfold byteLane
  scalar_tac

@[step] theorem inner_spec (cs : List (Slice U8)) (rem : Slice U8) (a b : V)
    (hlen : ∀ c ∈ cs, c.val.length = 4)
    (hbudget : ∀ i, budget cs.length (lane a i) (lane b i)) :
    algo.Adler32.compute_loop0_loop0 ⟨cs, rem⟩ a b
      ⦃ out => ∀ i, (lane out.1 i, lane out.2 i) = model cs a b i ⦄ := by
  induction cs generalizing a b with
  | nil =>
    unfold algo.Adler32.compute_loop0_loop0
    rw [loop]
    simp [algo.Adler32.compute_loop0_loop0.body, core.slice.iter.IteratorChunksExact.next, model]
  | cons c cs ih =>
    have hc : c.val.length = 4 := hlen c (by simp)
    have ht : ∀ c ∈ cs, c.val.length = 4 := by
      intro d hd
      exact hlen d (by simp [hd])
    have hb := fun i => budget_step cs.length (lane a i) (lane b i) (byteLane c i) (byte_bound c i) (hbudget i)
    unfold algo.Adler32.compute_loop0_loop0
    rw [loop]
    simp only [algo.Adler32.compute_loop0_loop0.body, core.slice.iter.IteratorChunksExact.next,
      bind_tc_ok]
    step as ⟨v, hv⟩
    step as ⟨a', ha'⟩
    step as ⟨b', hb'⟩
    have hnext : ∀ i, budget cs.length (lane a' i) (lane b' i) := by
      intro i
      simpa [ha', hb', hv] using (hb i).2.2
    have hi := ih a' b' ht hnext
    change algo.Adler32.compute_loop0_loop0 ⟨cs, rem⟩ a' b' ⦃ _ ⦄
    apply WP.spec_mono hi
    intro out hout i
    rw [hout i]
    simp only [model, List.foldl_cons, laneStep]
    simp only [ha', hb', hv]

@[step] theorem remainder_spec (cs : List (Slice U8)) (rem : Slice U8) (a b : V)
    (hlen : ∀ c ∈ cs, c.val.length = 4)
    (hbudget : ∀ i, budget cs.length (lane a i) (lane b i)) :
    algo.Adler32.compute_loop1 ⟨cs, rem⟩ a b
      ⦃ out => ∀ i, (lane out.1 i, lane out.2 i) = model cs a b i ⦄ := by
  exact inner_spec cs rem a b hlen hbudget
end Inner
