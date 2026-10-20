import Recurrence
import Vector
import Inner
import Mathlib.Data.ZMod.Basic
import Mathlib.Tactic.Ring
open Aeneas Aeneas.Std Result ElicitGenerated
namespace LaneAlgebra
abbrev Z := ZMod 65521
abbrev S := Z × Z
def toZ (s : AdlerSpec.State) : S := (s.1, s.2)
def step (s : S) (x : U8) : S := (s.1 + (x.val : Z), s.2 + (s.1 + (x.val : Z)))
def run (s : S) (xs : List U8) : S := xs.foldl step s
@[simp] theorem cast_mod (a : Nat) : ((a % 65521 : Nat) : Z) = (a : Z) := by
  exact (ZMod.natCast_eq_natCast_iff' _ _ _).2 (by simp)
@[simp] theorem cast_normalize (s : AdlerSpec.State) : toZ (AdlerSpec.normalize s) = toZ s := by
  simp [toZ, AdlerSpec.normalize, AdlerSpec.modulus]
@[simp] theorem cast_step (s : AdlerSpec.State) (x : U8) :
    toZ (AdlerSpec.step s x) = step (toZ s) x := by
  simp [toZ, AdlerSpec.step, AdlerSpec.modulus, step, Nat.cast_add]
 theorem cast_fold (xs : List U8) (s : AdlerSpec.State) :
    toZ (xs.foldl AdlerSpec.step s) = run (toZ s) xs := by
  induction xs generalizing s with
  | nil => rfl
  | cons x xs ih => simp only [List.foldl_cons, run] at *; rw [ih, cast_step]
 theorem cast_run (s : AdlerSpec.State) (xs : List U8) :
    toZ (AdlerSpec.run s xs) = run (toZ s) xs := by
  rw [AdlerSpec.run, cast_fold, cast_normalize]
 theorem cast_inj {s t : AdlerSpec.State}
    (hs : s.1 < 65521 ∧ s.2 < 65521) (ht : t.1 < 65521 ∧ t.2 < 65521)
    (h : toZ s = toZ t) : s = t := by
  have h1 := congrArg Prod.fst h
  have h2 := congrArg Prod.snd h
  simp only [toZ, Prod.fst, Prod.snd, ZMod.natCast_eq_natCast_iff'] at h1 h2
  rw [Nat.mod_eq_of_lt hs.1, Nat.mod_eq_of_lt ht.1] at h1
  rw [Nat.mod_eq_of_lt hs.2, Nat.mod_eq_of_lt ht.2] at h2
  exact Prod.ext h1 h2
@[simp] theorem run_append (s : S) (xs ys : List U8) : run s (xs ++ ys) = run (run s xs) ys := by
  exact List.foldl_append

def sum (v : Fin 4 → Z) : Z := v 0 + v 1 + v 2 + v 3
def weight (v : Fin 4 → Z) : Z := v 1 + 2*v 2 + 3*v 3
def combine (a b : Z) (av bv : Fin 4 → Z) : S := (a + sum av, b + 4*sum bv - weight av)
def lanes (v : Vector.V) : Fin 4 → Z := fun i => (Vector.lane v i : Z)
def bytes (c : Slice U8) : Fin 4 → Z := fun i => (Vector.byteLane c i : Z)
 theorem block (a b : Z) (av bv : Fin 4 → Z) (c : Slice U8) (hc : c.val.length = 4) :
    combine a (b+4*a) (fun i => av i + bytes c i) (fun i => bv i + (av i + bytes c i)) =
      run (combine a b av bv) c.val := by
  obtain ⟨x0, x1, x2, x3, heq⟩ := List.length_eq_four.mp hc
  apply Prod.ext <;> simp [combine, sum, weight, run, step, bytes, Vector.byteLane, heq] <;> ring
def parallel (cs : List (Slice U8)) (av bv : Fin 4 → Z) : (Fin 4 → Z) × (Fin 4 → Z) :=
  match cs with
  | [] => (av, bv)
  | c :: cs => parallel cs (fun i => av i + bytes c i) (fun i => bv i + (av i + bytes c i))
 theorem parallel_combine (cs : List (Slice U8)) (a b : Z) (av bv : Fin 4 → Z)
    (hlen : ∀ c ∈ cs, c.val.length = 4) :
    combine a (b + 4*(cs.length : Z)*a) (parallel cs av bv).1 (parallel cs av bv).2 =
      run (combine a b av bv) (Chunks.values cs) := by
  induction cs generalizing b av bv with
  | nil => simp [parallel, combine, run]
  | cons c cs ih =>
    have hc := hlen c (by simp)
    have ht : ∀ c ∈ cs, c.val.length = 4 := by
      intro d hd
      exact hlen d (by simp [hd])
    simp only [parallel, List.length_cons, Nat.cast_add, Nat.cast_one, Chunks.values_cons, run_append]
    rw [show b + 4 * ((cs.length : Z) + 1) * a = (b+4*a) + 4*(cs.length : Z)*a by ring]
    rw [ih _ _ _ ht, block a b av bv c hc]
 theorem cast_lane_fold (cs : List (Slice U8)) (s : Nat × Nat) (i : Fin 4)
    (av bv : Fin 4 → Z) (ha : av i = (s.1 : Z)) (hb : bv i = (s.2 : Z)) :
    ((cs.foldl (Inner.laneStep i) s).1 : Z) = (parallel cs av bv).1 i ∧
    ((cs.foldl (Inner.laneStep i) s).2 : Z) = (parallel cs av bv).2 i := by
  induction cs generalizing s av bv with
  | nil => simpa [parallel] using And.intro ha.symm hb.symm
  | cons c cs ih =>
    simp only [List.foldl_cons, parallel]
    apply ih
    · simp [Inner.laneStep, ha, bytes, Nat.cast_add]
    · simp [Inner.laneStep, ha, hb, bytes, Nat.cast_add]
 theorem inner_combine (cs : List (Slice U8)) (a b : Z) (av bv av' bv' : Vector.V)
    (hlen : ∀ c ∈ cs, c.val.length = 4)
    (h : ∀ i, (Vector.lane av' i, Vector.lane bv' i) = Inner.model cs av bv i) :
    combine a (b + 4*(cs.length : Z)*a) (lanes av') (lanes bv') =
      run (combine a b (lanes av) (lanes bv)) (Chunks.values cs) := by
  have ha : lanes av' = (parallel cs (lanes av) (lanes bv)).1 := by
    funext i
    have hi := congrArg Prod.fst (h i)
    dsimp [Inner.model] at hi
    exact (congrArg (fun n : Nat => (n : Z)) hi).trans (cast_lane_fold cs _ i _ _ rfl rfl).1
  have hb : lanes bv' = (parallel cs (lanes av) (lanes bv)).2 := by
    funext i
    have hi := congrArg Prod.snd (h i)
    dsimp [Inner.model] at hi
    exact (congrArg (fun n : Nat => (n : Z)) hi).trans (cast_lane_fold cs _ i _ _ rfl rfl).2
  rw [ha, hb]
  exact parallel_combine cs a b (lanes av) (lanes bv) hlen
end LaneAlgebra
