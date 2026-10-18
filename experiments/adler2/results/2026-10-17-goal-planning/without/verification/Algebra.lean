import Loops
import Mathlib.Data.ZMod.Basic
import Mathlib.Tactic.Ring
set_option maxHeartbeats 2000000
open Aeneas Aeneas.Std Result
namespace Verification
abbrev Z := ZMod 65521

def zStep (s : Z × Z) (x : U8) : Z × Z :=
  (s.1 + (x.val : Z), s.2 + (s.1 + (x.val : Z)))
def zRun (s : Z × Z) (xs : List U8) := xs.foldl zStep s

def toZ (s : Nat × Nat) : Z × Z := ((s.1 : Z), (s.2 : Z))

@[simp] theorem toZ_normalize (s : Nat × Nat) : toZ (normalize s) = toZ s := by
  simp [toZ, normalize]

theorem zRun_raw (s : Nat × Nat) (xs : List U8) :
    toZ (raw s xs) = zRun (toZ s) xs := by
  induction xs generalizing s with
  | nil => rfl
  | cons x xs ih =>
    simp only [raw, List.foldl_cons]
    rw [show List.foldl rawStep (rawStep s x) xs = raw (rawStep s x) xs from rfl, ih]
    simp [zRun, toZ, rawStep, zStep]

theorem zRun_scalar (s : Nat × Nat) (xs : List U8) :
    toZ (scalar s xs) = zRun (toZ s) xs := by
  rw [← normalize_raw, toZ_normalize, zRun_raw]

def projection (a b : Z) (av bv : Fin 4 → Z) : Z × Z :=
  (a + av 0 + av 1 + av 2 + av 3,
   b + 4*(bv 0 + bv 1 + bv 2 + bv 3) - av 1 - 2*av 2 - 3*av 3)

def zLanes (cs : List (Slice U8)) (av bv : Fin 4 → Z) : (Fin 4 → Z) × (Fin 4 → Z) :=
  cs.foldl (fun s c =>
    let av' := fun j => s.1 j + (c.val[j.val]!.val : Z)
    (av', fun j => s.2 j + av' j)) (av, bv)

theorem projection_step (a b : Z) (av bv : Fin 4 → Z) (c : Slice U8)
    (hc : c.val.length = 4) :
    projection a (b + 4*a)
      (fun j => av j + (c.val[j.val]!.val : Z))
      (fun j => bv j + (av j + (c.val[j.val]!.val : Z))) =
    zRun (projection a b av bv) c.val := by
  have hex : ∃ x0 x1 x2 x3, c.val = [x0,x1,x2,x3] := by
    rcases h : c.val with _ | ⟨x0, xs⟩ <;> simp_all
    rcases xs with _ | ⟨x1, xs⟩ <;> simp_all
    rcases xs with _ | ⟨x2, xs⟩ <;> simp_all
    rcases xs with _ | ⟨x3, xs⟩ <;> simp_all
  rcases hex with ⟨x0,x1,x2,x3,hc⟩
  simp [projection, zRun, zStep, hc]
  constructor <;> ring

theorem projection_fold (a b : Z) (av bv : Fin 4 → Z) (cs : List (Slice U8))
    (hc : ∀ c ∈ cs, c.val.length = 4) :
    projection a (b + (4*cs.length : Nat)*a) (zLanes cs av bv).1 (zLanes cs av bv).2 =
      zRun (projection a b av bv) (cs.map Slice.val).flatten := by
  induction cs generalizing av bv b with
  | nil => simp [zLanes, zRun]
  | cons c cs ih =>
    have hcc := hc c (by simp)
    have ht : ∀ c ∈ cs, c.val.length = 4 := by simp_all
    have ih' := ih (b := b+4*a)
      (av := fun j => av j + (c.val[j.val]!.val : Z))
      (bv := fun j => bv j + (av j + (c.val[j.val]!.val : Z))) ht
    rw [projection_step a b av bv c hcc] at ih'
    simpa [zLanes, zRun, List.foldl_append, Nat.add_mul, Nat.mul_add,
      add_assoc, add_comm, add_left_comm, add_mul] using ih'

theorem zLanes_laneFold (cs : List (Slice U8)) (av bv : Fin 4 → Nat) :
    zLanes cs (fun j => (av j : Z)) (fun j => (bv j : Z)) =
      ((fun j => ((laneFold j (av j, bv j) cs).1 : Z)),
       (fun j => ((laneFold j (av j, bv j) cs).2 : Z))) := by
  induction cs generalizing av bv with
  | nil => rfl
  | cons c cs ih =>
    have ih' := ih (av := fun j => av j + c.val[j.val]!.val)
      (bv := fun j => bv j + (av j + c.val[j.val]!.val))
    simpa only [zLanes, laneFold, List.foldl_cons, laneStep,
      Nat.cast_add] using ih'

def vectorZ (av : adler2.algo.U32X4) : Fin 4 → Z := fun j => (lane av j : Z)

theorem lanes_projection (cs : List (Slice U8))
    (av bv av' bv' : adler2.algo.U32X4) (a b : Z)
    (hlen : ∀ c ∈ cs, c.val.length = 4)
    (heq : ∀ j, (lane av' j, lane bv' j) = laneFold j (lane av j, lane bv j) cs) :
    projection a (b + (4*cs.length : Nat)*a) (vectorZ av') (vectorZ bv') =
      zRun (projection a b (vectorZ av) (vectorZ bv)) (cs.map Slice.val).flatten := by
  have hz := zLanes_laneFold cs (lane av) (lane bv)
  have hza : (zLanes cs (vectorZ av) (vectorZ bv)).1 = vectorZ av' := by
    rw [show zLanes cs (vectorZ av) (vectorZ bv) = _ from hz]
    funext j
    exact congrArg (fun n : Nat => (n : Z)) (congrArg Prod.fst (heq j)).symm
  have hzb : (zLanes cs (vectorZ av) (vectorZ bv)).2 = vectorZ bv' := by
    rw [show zLanes cs (vectorZ av) (vectorZ bv) = _ from hz]
    funext j
    exact congrArg (fun n : Nat => (n : Z)) (congrArg Prod.snd (heq j)).symm
  simpa only [hza, hzb] using projection_fold a b (vectorZ av) (vectorZ bv) cs hlen

theorem vector_loop_projection (cs : List (Slice U8)) (rem : Slice U8)
    (av bv : adler2.algo.U32X4) (a b : Z)
    (hlen : ∀ c ∈ cs, c.val.length = 4)
    (hn : cs.length ≤ 5552) (hb : laneBounds 0 av bv) :
    adler2.algo.Adler32.compute_loop1 ⟨cs, rem⟩ av bv
      ⦃ av' bv' => laneBounds cs.length av' bv' ∧
        projection a (b + (4*cs.length : Nat)*a) (vectorZ av') (vectorZ bv') =
          zRun (projection a b (vectorZ av) (vectorZ bv)) (cs.map Slice.val).flatten ⦄ := by
  apply WP.spec_mono (vector_loop cs rem av bv 0 hlen (by omega) hb)
  intro r hr
  rcases r with ⟨av', bv'⟩
  change _ ∧ _ at hr
  change _ ∧ _
  constructor
  · simpa using hr.1
  · have hz := zLanes_laneFold cs (lane av) (lane bv)
    have hza : (zLanes cs (vectorZ av) (vectorZ bv)).1 = vectorZ av' := by
      rw [show zLanes cs (vectorZ av) (vectorZ bv) = _ from hz]
      funext j
      have hh := congrArg Prod.fst (hr.2 j)
      simp only [vectorZ]
      exact congrArg (fun n : Nat => (n : Z)) hh.symm
    have hzb : (zLanes cs (vectorZ av) (vectorZ bv)).2 = vectorZ bv' := by
      rw [show zLanes cs (vectorZ av) (vectorZ bv) = _ from hz]
      funext j
      have hh := congrArg Prod.snd (hr.2 j)
      simp only [vectorZ]
      exact congrArg (fun n : Nat => (n : Z)) hh.symm
    simpa only [hza, hzb] using projection_fold a b (vectorZ av) (vectorZ bv) cs hlen

#print axioms vector_loop_projection
theorem normalize_equal_of_toZ_equal (s t : Nat × Nat) (h : toZ s = toZ t) :
    normalize s = normalize t := by
  have h1 := congrArg (fun p : Z × Z => p.1.val) h
  have h2 := congrArg (fun p : Z × Z => p.2.val) h
  simp [toZ, ZMod.val_natCast] at h1 h2
  exact Prod.ext h1 h2

#print axioms normalize_equal_of_toZ_equal
theorem correction_cast (x : Nat) (hx : x ≤ 65521) :
    ((65521-x : Nat) : Z) = -(x : Z) := by
  rw [Nat.cast_sub hx, Nat.cast_ofNat]
  have hz : (65521 : Z) = 0 := by exact ZMod.natCast_self 65521
  rw [hz, zero_sub]

theorem combine_projection (a b aa bb : Nat) (av bv : Fin 4 → Nat)
    (h1 : av 1 ≤ 65521) (h2 : av 2 ≤ 65521) (h3 : av 3 ≤ 65521)
    (ha : aa = a + av 0 + av 1 + av 2 + av 3)
    (hb : bb = b + 4*bv 0 + (4*bv 1 + (65521-av 1)) +
      (4*bv 2 + (65521-av 2)*2) + (4*bv 3 + (65521-av 3)*3)) :
    toZ (aa, bb) = projection (a : Z) (b : Z)
      (fun j => (av j : Z)) (fun j => (bv j : Z)) := by
  simp only [toZ, ha, hb, projection, Nat.cast_add, Nat.cast_mul, Nat.cast_ofNat]
  rw [correction_cast _ h1, correction_cast _ h2, correction_cast _ h3]
  apply Prod.ext
  · rfl
  · ring

#print axioms combine_projection
end Verification
