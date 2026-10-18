import ElicitContracts
open Aeneas Aeneas.Std

namespace AdlerSpec

theorem fold_normalized (xs : List U8) (s : Nat × Nat)
    (ha : s.1 < 65521) (hb : s.2 < 65521) :
    (xs.foldl step s).1 < 65521 ∧ (xs.foldl step s).2 < 65521 := by
  induction xs generalizing s with
  | nil => exact ⟨ha, hb⟩
  | cons x xs ih =>
    apply ih
    · exact Nat.mod_lt _ (by decide)
    · exact Nat.mod_lt _ (by decide)

theorem update_normalized (a b : Nat) (xs : List U8) :
    (update a b xs).1 < 65521 ∧ (update a b xs).2 < 65521 := by
  apply fold_normalized
  · exact Nat.mod_lt _ (by decide)
  · exact Nat.mod_lt _ (by decide)

theorem update_append (a b : Nat) (xs ys : List U8) :
    update a b (xs ++ ys) =
      update (update a b xs).1 (update a b xs).2 ys := by
  have h := update_normalized a b xs
  simp only [update] at h
  simp only [update, List.foldl_append]
  rw [Nat.mod_eq_of_lt h.1, Nat.mod_eq_of_lt h.2]

end AdlerSpec

namespace AdlerSpec
 theorem chunks_partition {α : Type} (n : Nat) (hn : 0 < n) (xs : List α) :
    (List.toChunksExact n hn xs).1.flatten ++ (List.toChunksExact n hn xs).2 = xs ∧
    (∀ c ∈ (List.toChunksExact n hn xs).1, c.length = n) ∧
    (List.toChunksExact n hn xs).2.length < n := by
  unfold List.toChunksExact
  split
  · rename_i h
    simp_all
  · rename_i h
    have ih := chunks_partition n hn (xs.drop n)
    dsimp only at *
    constructor
    · simp only [List.flatten_cons, List.append_assoc]
      rw [ih.1]
      exact List.take_append_drop n xs
    · constructor
      · intro c hc
        simp only [List.mem_cons] at hc
        rcases hc with rfl | hc
        · simp only [List.length_take]
          omega
        · exact ih.2.1 c hc
      · exact ih.2.2
termination_by xs.length
 decreasing_by simp [List.length_drop]; omega
@[step] theorem chunks_exact_spec {α : Type} (s : Slice α) (n : Usize)
    (hn : 0 < n.val) :
    core.slice.Slice.chunks_exact s n ⦃ it =>
      (it.chunks.map Slice.val).flatten ++ it.remainder.val = s.val ∧
      (∀ c ∈ it.chunks, c.val.length = n.val) ∧
      it.remainder.val.length < n.val ⦄ := by
  unfold core.slice.Slice.chunks_exact
  simp only [dite_eq_left hn]
  apply (Std.WP.spec_ok _).mpr
  have h := chunks_partition n.val hn s.val
  constructor
  · simpa using h.1
  constructor
  · intro c hc
    simp only [List.mem_map] at hc
    obtain ⟨x, hx, heq⟩ := hc
    subst c
    simpa using h.2.1 x.val x.property
  · simpa using h.2.2
@[step] theorem get_remainder_spec {α : Type} (it : core.slice.iter.ChunksExact α) :
    core.slice.iter.ChunksExact.getRemainder it ⦃ r => r = it.remainder ⦄ := by
  unfold core.slice.iter.ChunksExact.getRemainder
  exact (Std.WP.spec_ok _).mpr rfl
end AdlerSpec

open ElicitGenerated
namespace AdlerSpec
@[step] theorem lane_add_spec (a b : algo.U32X4)
    (h : ∀ i : Fin 4, (a.val[i.val]'(by simpa using i.isLt)).val + (b.val[i.val]'(by simpa using i.isLt)).val ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign a b ⦃ out =>
      out.val.map (fun x => x.val) = List.zipWith (fun x y : U32 => x.val + y.val) a.val b.val ⦄ := by
  have h0 := h ⟨0, by decide⟩
  have h1 := h ⟨1, by decide⟩
  have h2 := h ⟨2, by decide⟩
  have h3 := h ⟨3, by decide⟩
  have al : a.val.length = 4 := by simp
  have bl : b.val.length = 4 := by simp
  unfold algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step*
  all_goals simp_all [Array.set_val_eq]
  apply List.ext_getElem
  · simp
  · intro j hj hk
    have hj4 : j < 4 := by simpa using hj
    have cases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
    rcases cases with rfl | rfl | rfl | rfl <;> simp_all
end AdlerSpec

theorem lane_from_correct : ElicitContracts.c_0dfc1ce11e75ed00 := by
  intro bytes h
  apply ElicitAdapter.spec_to_contract
  unfold algo.U32X4.from
  simp only [lift]
  step*
  simp only [Array.make_val, List.map_cons, List.map_nil,
    core.convert.num.FromU32U8.from_val_eq]
  apply List.ext_getElem
  · simp only [List.length_cons, List.length_nil, List.length_map, List.length_take]
    omega
  · intro j hj hk
    have hj4 : j < 4 := by simpa using hj
    have cases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
    rcases cases with rfl | rfl | rfl | rfl <;> simp_all

elicit_spec lane_from_rule from lane_from_correct

namespace AdlerSpec
abbrev vals (a : algo.U32X4) := a.val.map (fun x => x.val)
def laneStep (s : List Nat × List Nat) (c : Slice U8) : List Nat × List Nat :=
  let a := List.zipWith Nat.add s.1 (c.val.map (fun x => x.val))
  (a, List.zipWith Nat.add s.2 a)
def laneModel (s : List Nat × List Nat) (cs : List (Slice U8)) := cs.foldl laneStep s

def aBound (t : Nat) := 65520 + 255*t
def bBound (t : Nat) := 65520 + 65520*t + 255*(t*(t+1)/2)

theorem bounds_step (t : Nat) (ht : t < 5552) :
    aBound t + 255 = aBound (t+1) ∧
    bBound t + aBound (t+1) = bBound (t+1) ∧
    aBound (t+1) ≤ U32.max ∧ bBound (t+1) ≤ U32.max := by
  have hprod : (t+1)*(t+2) ≤ 5552*5553 :=
    Nat.mul_le_mul (by omega) (by omega)
  unfold aBound bBound
  have hmax : U32.max = 4294967295 := by scalar_tac
  rw [hmax]
  simp only [Nat.mul_add, Nat.add_mul, Nat.mul_one, Nat.one_mul] at *
  omega
theorem add_at (a b out : algo.U32X4)
    (h : vals out = List.zipWith (fun x y : U32 => x.val + y.val) a.val b.val)
    (i : Fin 4) :
    (out.val[i.val]'(by simpa using i.isLt)).val =
      (a.val[i.val]'(by simpa using i.isLt)).val +
      (b.val[i.val]'(by simpa using i.isLt)).val := by
  have hi := congrArg (fun l : List Nat => l[i.val]!) h
  simpa [vals, List.getElem!_eq_getElem?_getD, List.getElem?_eq_getElem, i.isLt] using hi
abbrev laneVal (a : algo.U32X4) (i : Fin 4) : Nat :=
  (a.val[i.val]'(by simpa using i.isLt)).val

@[step] theorem inner_loop (cs : List (Slice U8)) (rem : Slice U8)
    (a b : algo.U32X4) (t : Nat)
    (hlen : ∀ c ∈ cs, c.val.length = 4)
    (ha : ∀ i, laneVal a i ≤ aBound t)
    (hb : ∀ i, laneVal b i ≤ bBound t)
    (ht : t + cs.length ≤ 5552) :
    algo.Adler32.compute_loop0_loop0 { chunks := cs, remainder := rem } a b ⦃ out =>
      (vals out.1, vals out.2) = laneModel (vals a, vals b) cs ∧
      (∀ i, laneVal out.1 i ≤ aBound (t + cs.length)) ∧
      (∀ i, laneVal out.2 i ≤ bBound (t + cs.length)) ⦄ := by
  induction cs generalizing a b t with
  | nil =>
    rw [algo.Adler32.compute_loop0_loop0, loop]
    simp [algo.Adler32.compute_loop0_loop0.body,
      core.slice.iter.IteratorChunksExact.next, laneModel, ha, hb]
  | cons c cs ih =>
    have hc : c.val.length = 4 := hlen c (by simp)
    have ht' : t < 5552 := by simp only [List.length_cons] at ht; omega
    have hbnds := bounds_step t ht'
    rw [algo.Adler32.compute_loop0_loop0, loop]
    simp only [algo.Adler32.compute_loop0_loop0.body,
      core.slice.iter.IteratorChunksExact.next, bind_tc_ok]
    step as ⟨v, hv⟩
    have hv' : vals v = c.val.map (fun x => x.val) := by
      simpa [List.take_of_length_le (by omega : c.val.length ≤ 4)] using hv
    have hvi (i : Fin 4) : laneVal v i = (c.val[i.val]'(by omega)).val := by
      have hi := congrArg (fun l : List Nat => l[i.val]!) hv'
      simpa [vals, laneVal, List.getElem!_eq_getElem?_getD,
        List.getElem?_eq_getElem, i.isLt, hc] using hi
    have hva (i : Fin 4) : laneVal v i ≤ 255 := by rw [hvi]; scalar_tac
    step as ⟨a1, ha1⟩
    · intro i
      have h := ha i
      have hvb := hva i
      change laneVal a i + laneVal v i ≤ U32.max
      omega
    have ha1b (i : Fin 4) : laneVal a1 i ≤ aBound (t+1) := by
      have he := add_at a v a1 ha1 i
      have h := ha i
      have hvb := hva i
      change laneVal a1 i = laneVal a i + laneVal v i at he
      omega
    step as ⟨b1, hb1⟩
    · intro i
      have h := hb i
      have h' := ha1b i
      change laneVal b i + laneVal a1 i ≤ U32.max
      omega
    have hb1b (i : Fin 4) : laneVal b1 i ≤ bBound (t+1) := by
      have he := add_at b a1 b1 hb1 i
      have h := hb i
      have h' := ha1b i
      change laneVal b1 i = laneVal b i + laneVal a1 i at he
      omega
    have hh := ih a1 b1 (t+1) (by intro x hx; exact hlen x (by simp [hx])) ha1b hb1b (by simp only [List.length_cons] at ht; omega)
    change algo.Adler32.compute_loop0_loop0 { chunks := cs, remainder := rem } a1 b1 ⦃ _ ⦄
    apply Std.WP.spec_mono hh
    intro out hout
    have heA : vals a1 = List.zipWith Nat.add (vals a) (vals v) := by
      simpa [vals, List.zipWith_map] using ha1
    have heB : vals b1 = List.zipWith Nat.add (vals b) (vals a1) := by
      simpa [vals, List.zipWith_map] using hb1
    have heS : (vals a1, vals b1) = laneStep (vals a, vals b) c := by
      simp only [laneStep]
      rw [← hv', ← heA, ← heB]
    constructor
    · simpa only [laneModel, List.foldl_cons, heS] using hout.1
    · simpa only [List.length_cons, Nat.add_assoc, Nat.add_comm 1] using hout.2
end AdlerSpec

namespace AdlerSpec
@[step] theorem lane_rem_spec (a : algo.U32X4) (q : U32) (hq : 0 < q.val) :
    algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign a q ⦃ out =>
      vals out = (vals a).map (fun x => x % q.val) ⦄ := by
  unfold algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
  step*
  all_goals simp_all [Array.set_val_eq, vals]
  apply List.ext_getElem
  · simp
  · intro j hj hk
    have hj4 : j < 4 := by simpa using hj
    have cases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
    rcases cases with rfl | rfl | rfl | rfl <;> simp_all
end AdlerSpec

namespace AdlerSpec
theorem flatten_length (cs : List (Slice U8)) (n : Nat)
    (h : ∀ c ∈ cs, c.val.length = n) :
    (cs.map Slice.val).flatten.length = n*cs.length := by
  induction cs with
  | nil => simp
  | cons c cs ih =>
    simp only [List.map_cons, List.flatten_cons, List.length_append,
      List.length_cons, h c (by simp)]
    rw [ih (by intro c hc; exact h c (by simp [hc]))]
    simp only [Nat.mul_add, Nat.mul_one]; omega
@[step] theorem four_chunks_spec (s : Slice U8) (hs : s.val.length ≤ 22208) :
    core.slice.Slice.chunks_exact s 4#usize ⦃ it =>
      (it.chunks.map Slice.val).flatten ++ it.remainder.val = s.val ∧
      (∀ c ∈ it.chunks, c.val.length = 4) ∧
      it.remainder.val.length < 4 ∧ it.chunks.length ≤ 5552 ⦄ := by
  apply Std.WP.spec_mono (chunks_exact_spec s 4#usize (by decide))
  intro it hit
  have hlen := congrArg List.length hit.1
  have hc := flatten_length it.chunks 4 hit.2.1
  refine ⟨hit.1, hit.2.1, hit.2.2, ?_⟩
  simp only [List.length_append] at hlen
  omega
theorem mod_lane_bounds (v : algo.U32X4) (xs : List Nat)
    (h : vals v = xs.map (fun x => x % 65521)) :
    ∀ i, laneVal v i ≤ 65520 := by
  intro i
  have hm : laneVal v i ∈ vals v := by
    exact List.mem_map.mpr ⟨v.val[i.val]'(by simpa using i.isLt), List.getElem_mem _, rfl⟩
  rw [h] at hm
  obtain ⟨x, hx, he⟩ := List.mem_map.mp hm
  have hb := Nat.mod_lt x (by decide : 0 < 65521)
  omega
end AdlerSpec

namespace AdlerSpec
theorem outer_body (c : Slice U8) (cs : List (Slice U8)) (rem : Slice U8)
    (a b : U32) (av bv : algo.U32X4)
    (hc : c.val.length = 22208) (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
    (hav : ∀ i, laneVal av i ≤ 65520) (hbv : ∀ i, laneVal bv i ≤ 65520) :
    algo.Adler32.compute_loop0.body 22208#usize a {chunks := c::cs, remainder := rem} b av bv ⦃ out =>
      ∃ (small : List (Slice U8)) (b' : U32) (av' bv' : algo.U32X4),
        (∀ s ∈ small, s.val.length = 4) ∧
        (small.map Slice.val).flatten = c.val ∧
        out = ControlFlow.cont ({chunks := cs, remainder := rem}, b', av', bv') ∧
        b'.val = (b.val + 22208*a.val) % 65521 ∧
        vals av' = (laneModel (vals av, vals bv) small).1.map (fun x => x%65521) ∧
        vals bv' = (laneModel (vals av, vals bv) small).2.map (fun x => x%65521) ⦄ := by
  unfold algo.Adler32.compute_loop0.body
  simp only [core.slice.iter.IteratorChunksExact.next, lift,
    algo.Adler32.compute.MOD]
  step with four_chunks_spec c (by omega) as ⟨it, hit, hlen, hrem, hcount⟩
  have hzero : it.remainder.val = [] := by
    have hlen' := congrArg List.length hit
    have hf := flatten_length it.chunks 4 hlen
    have hz : it.remainder.val.length = 0 := by simp only [List.length_append] at hlen'; omega
    exact List.eq_nil_of_length_eq_zero hz
  have hflat : (it.chunks.map Slice.val).flatten = c.val := by simpa [hzero] using hit
  step with inner_loop it.chunks it.remainder av bv 0 hlen hav hbv (by omega) as ⟨res, hres, hresa, hresb⟩
  step*

end AdlerSpec

namespace AdlerSpec
def blockRep (small : List (Slice U8)) (c : Slice U8) :=
  (∀ s ∈ small, s.val.length = 4) ∧ (small.map Slice.val).flatten = c.val
def outerStep (a : Nat) (s : Nat × List Nat × List Nat) (small : List (Slice U8)) :=
  let lanes := laneModel (s.2.1, s.2.2) small
  ((s.1 + 22208*a)%65521, lanes.1.map (fun x => x%65521), lanes.2.map (fun x => x%65521))

@[step] theorem outer_loop (cs : List (Slice U8)) (rem : Slice U8)
    (a b : U32) (av bv : algo.U32X4)
    (hlen : ∀ c ∈ cs, c.val.length = 22208) (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
    (hav : ∀ i, laneVal av i ≤ 65520) (hbv : ∀ i, laneVal bv i ≤ 65520) :
    algo.Adler32.compute_loop0 22208#usize {chunks := cs, remainder := rem} a b av bv ⦃ out =>
      ∃ groups : List (List (Slice U8)),
        List.Forall₂ blockRep groups cs ∧
        (out.1.val, vals out.2.1, vals out.2.2) = groups.foldl (outerStep a.val) (b.val, vals av, vals bv) ∧
        out.1.val ≤ 65535 ∧
        (∀ i, laneVal out.2.1 i ≤ 65520) ∧ (∀ i, laneVal out.2.2 i ≤ 65520) ⦄ := by
  induction cs generalizing b av bv with
  | nil =>
    rw [algo.Adler32.compute_loop0, loop]
    simp [algo.Adler32.compute_loop0.body, core.slice.iter.IteratorChunksExact.next]
    exact ⟨hb, hav, hbv⟩
  | cons c cs ih =>
    have hc := hlen c (by simp)
    rw [algo.Adler32.compute_loop0, loop]
    step with outer_body c cs rem a b av bv hc ha hb hav hbv as ⟨r, small, b', av', bv', hs, hf, hr, hb', hav', hbv'⟩
    simp only [hr]
    have hbn : b'.val ≤ 65535 := by have := Nat.mod_lt (b.val + 22208*a.val) (by decide : 0 < 65521); omega
    have han := mod_lane_bounds av' _ hav'
    have hbvn := mod_lane_bounds bv' _ hbv'
    have hi := ih b' av' bv' (by intro c hc; exact hlen c (by simp [hc])) hbn han hbvn
    change algo.Adler32.compute_loop0 22208#usize {chunks := cs, remainder := rem} a b' av' bv' ⦃ _ ⦄
    apply Std.WP.spec_mono hi
    intro out hout
    obtain ⟨groups, hg, ho, hbo, hao, hbvo⟩ := hout
    refine ⟨small::groups, .cons ⟨hs, hf⟩ hg, ?_, hbo, hao, hbvo⟩
    simpa only [List.foldl_cons, outerStep, ← hb', ← hav', ← hbv'] using ho
end AdlerSpec

namespace AdlerSpec
@[step] theorem final_lane_loop (cs : List (Slice U8)) (rem : Slice U8)
    (a b : algo.U32X4) (t : Nat)
    (hlen : ∀ c ∈ cs, c.val.length = 4)
    (ha : ∀ i, laneVal a i ≤ aBound t)
    (hb : ∀ i, laneVal b i ≤ bBound t)
    (ht : t + cs.length ≤ 5552) :
    algo.Adler32.compute_loop1 { chunks := cs, remainder := rem } a b ⦃ out =>
      (vals out.1, vals out.2) = laneModel (vals a, vals b) cs ∧
      (∀ i, laneVal out.1 i ≤ aBound (t + cs.length)) ∧
      (∀ i, laneVal out.2 i ≤ bBound (t + cs.length)) ⦄ := by
  exact inner_loop cs rem a b t hlen ha hb ht

@[step] theorem lane_mul_spec (a : algo.U32X4) (q : U32)
    (h : ∀ i, laneVal a i * q.val ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign a q ⦃ out =>
      vals out = (vals a).map (fun x => x*q.val) ⦄ := by
  have h0 := h ⟨0, by decide⟩
  have h1 := h ⟨1, by decide⟩
  have h2 := h ⟨2, by decide⟩
  have h3 := h ⟨3, by decide⟩
  unfold algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
  step*
  all_goals simp_all [Array.set_val_eq, vals, laneVal]
  apply List.ext_getElem
  · simp
  · intro j hj hk
    have hj4 : j < 4 := by simpa using hj
    have cases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
    rcases cases with rfl | rfl | rfl | rfl <;> simp_all
end AdlerSpec

namespace AdlerSpec
@[step] theorem sum_loop (s : Slice U32) (i : Nat) (a : U32)
    (h : a.val + ((s.val.drop i).map (fun x => x.val)).sum ≤ U32.max) :
    algo.Adler32.compute_loop2 {slice := s, i := i} a ⦃ out =>
      out.val = a.val + ((s.val.drop i).map (fun x => x.val)).sum ⦄ := by
  rw [algo.Adler32.compute_loop2, loop]
  simp only [algo.Adler32.compute_loop2.body, core.slice.iter.IteratorSliceIter.next]
  split
  · rename_i hi
    have hi' : i < s.val.length := by simpa using hi
    have hd : s.val.drop i = s.val[i] :: s.val.drop (i+1) := by
      exact List.drop_eq_getElem_cons hi'
    simp only [hd, List.map_cons, List.sum_cons] at h
    step as ⟨a1, ha1⟩
    change a1.val = a.val + (s.val[i]'hi').val at ha1
    have hh := sum_loop s (i+1) a1 (by omega)
    change algo.Adler32.compute_loop2 {slice := s, i := i+1} a1 ⦃ _ ⦄
    apply Std.WP.spec_mono hh
    intro out ho
    simp only [hd, List.map_cons, List.sum_cons]
    omega
  · rename_i hi
    have hd : s.val.drop i = [] := List.drop_eq_nil_iff.mpr (by simpa using Nat.le_of_not_gt hi)
    simp [hd]
termination_by s.val.length - i
decreasing_by simp_all; omega
end AdlerSpec
namespace AdlerSpec
@[step] theorem sum_loop_b (s : Slice U32) (i : Nat) (a : U32)
    (h : a.val + ((s.val.drop i).map (fun x => x.val)).sum ≤ U32.max) :
    algo.Adler32.compute_loop3 {slice := s, i := i} a ⦃ out =>
      out.val = a.val + ((s.val.drop i).map (fun x => x.val)).sum ⦄ := by
  exact sum_loop s i a h
end AdlerSpec

open Aeneas Aeneas.Std Result ElicitGenerated

theorem new_correct : ElicitContracts.c_8055bec449b87b3d := by
  intro _
  refine ⟨{a := 1#u16, b := 0#u16}, rfl, ?_, ?_⟩ <;> rfl
elicit_spec new_rule from new_correct

theorem checksum_correct : ElicitContracts.c_0a059b8c0885cf07 := by
  intro self _
  apply ElicitAdapter.spec_to_contract
  unfold Adler32.checksum
  simp only [lift]
  step*
  have hi : i1.val = 65536*self.b.val := by
    simp only [core.convert.num.FromU32U16.from_val_eq, Nat.shiftLeft_eq] at i1_post
    scalar_tac
  simp only [UScalar.val_or, core.convert.num.FromU32U16.from_val_eq, hi]
  exact (Nat.two_pow_add_eq_or_of_lt (i := 16) (by scalar_tac) self.b.val).symm
elicit_spec checksum_rule from checksum_correct

theorem from_checksum_correct : ElicitContracts.c_9a31b7b9cb8492b0 := by
  intro sum _
  apply ElicitAdapter.spec_to_contract
  unfold Adler32.from_checksum
  simp only [lift]
  step*
elicit_spec from_checksum_rule from from_checksum_correct

theorem checksum_restore_roundtrip (sum : U32) :
    (do let h ← Adler32.from_checksum sum; Adler32.checksum h) ⦃ out => out = sum ⦄ := by
  step*

theorem checksum_state_roundtrip (s : Adler32) :
    (do let c ← Adler32.checksum s; Adler32.from_checksum c) ⦃ out => out = s ⦄ := by
  step*
  cases s
  cases out
  congr <;> scalar_tac

theorem checksum_state_eq (s : Adler32) :
    (do let c ← Adler32.checksum s; Adler32.from_checksum c) = ok s := by
  obtain ⟨out, heq, hs⟩ := ElicitAdapter.spec_to_contract (checksum_state_roundtrip s)
  simpa only [hs] using heq

theorem checksum_resume_equiv (s : Adler32) (xs ys : Slice U8) :
    (do let s1 ← Adler32.write_slice s xs
        let restored ← (do let c ← Adler32.checksum s1; Adler32.from_checksum c)
        Adler32.write_slice restored ys) =
    (do let s1 ← Adler32.write_slice s xs; Adler32.write_slice s1 ys) := by
  simp only [checksum_state_eq, bind_tc_ok]
  rfl
