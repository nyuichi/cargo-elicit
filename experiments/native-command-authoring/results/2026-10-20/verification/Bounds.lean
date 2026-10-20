import Lanes
open Aeneas Aeneas.Std
namespace Verification

def ab (k : Nat) := 65520 + 255*k
def bb (k : Nat) := 131040 + 131040*k + 255*k*(k+1)
def Q.twiceBound (q : Q) (n : Nat) : Prop :=
  2*q.x0.val ≤ n ∧ 2*q.x1.val ≤ n ∧ 2*q.x2.val ≤ n ∧ 2*q.x3.val ≤ n

lemma ab_step (k : Nat) : ab k + 255 = ab (k+1) := by unfold ab; omega
lemma bb_step (k : Nat) : bb k + 2 * ab (k+1) = bb (k+1) := by unfold bb ab; ring
lemma ab_small {k : Nat} (h : k ≤ 5552) : ab k ≤ 1481280 := by unfold ab; omega
lemma bb_small {k : Nat} (h : k ≤ 5551) : bb k ≤ 8586417840 := by
  have hh := Nat.mul_self_le_mul_self h
  unfold bb
  nlinarith

lemma Q.twice_add_bound {c a b : Q} (hc : c.adds a b)
    (ha : a.twiceBound n) (hb : b.bound m) : c.twiceBound (n+2*m) := by
  rcases hc with ⟨h0,h1,h2,h3⟩
  rcases ha with ⟨ha0,ha1,ha2,ha3⟩
  rcases hb with ⟨hb0,hb1,hb2,hb3⟩
  dsimp [Q.twiceBound]
  omega

lemma Q.initial_ab (q : Q) (h : q.bound 65520) : q.bound (ab 0) := h
lemma Q.initial_bb (q : Q) (h : q.bound 65520) : q.twiceBound (bb 0) := by
  rcases h with ⟨h0,h1,h2,h3⟩
  dsimp [Q.twiceBound, bb]
  omega

lemma Q.bound_mono {q : Q} (h : q.bound n) (hn : n ≤ m) : q.bound m := by
  rcases h with ⟨h0,h1,h2,h3⟩
  dsimp [Q.bound]; omega

lemma Q.twice_mono {q : Q} (h : q.twiceBound n) (hn : n ≤ 2*m) : q.bound m := by
  rcases h with ⟨h0,h1,h2,h3⟩
  dsimp [Q.bound]; omega
end Verification
