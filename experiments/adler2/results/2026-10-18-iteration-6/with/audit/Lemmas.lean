import ElicitContracts
open Aeneas Aeneas.Std ElicitGenerated
namespace AdlerProof
 theorem lane_bound (n a b : Nat) (hn : n ≤ 5552) (ha : a ≤ 65520) (hb : b ≤ 65520) :
     b + n * a + 255 * (n * (n + 1) / 2) < 4294967296 := by
   have hna := Nat.mul_le_mul hn ha
   have hnn := Nat.mul_le_mul hn (Nat.add_le_add_right hn 1)
   have hdiv : n * (n + 1) / 2 ≤ 5552 * (5552 + 1) / 2 := Nat.div_le_div_right hnn
   have hsum := Nat.mul_le_mul_left 255 hdiv
   omega
 theorem fold_normalized (xs : List U8) (s : Nat × Nat)
     (ha : s.1 < 65521) (hb : s.2 < 65521) :
     (xs.foldl (fun s x => AdlerSpec.step s x.val) s).1 < 65521 ∧
     (xs.foldl (fun s x => AdlerSpec.step s x.val) s).2 < 65521 := by
   induction xs generalizing s with
   | nil => exact ⟨ha, hb⟩
   | cons x xs ih =>
     apply ih
     · exact Nat.mod_lt _ (by decide)
     · exact Nat.mod_lt _ (by decide)
 theorem run_append (a b : Nat) (xs ys : List U8) :
     AdlerSpec.run a b (xs ++ ys) =
     AdlerSpec.run (AdlerSpec.run a b xs).1 (AdlerSpec.run a b xs).2 ys := by
   have h := fold_normalized xs (a % 65521, b % 65521)
     (Nat.mod_lt _ (by decide)) (Nat.mod_lt _ (by decide))
   simp only [AdlerSpec.run, List.foldl_append]
   rw [Nat.mod_eq_of_lt h.1, Nat.mod_eq_of_lt h.2]
end AdlerProof
namespace AdlerProof
open ElicitGenerated
 theorem slice_of_update (data : Slice U8)
     (hw : ∃ s, Adler32.write_slice ⟨1#u16, 0#u16⟩ data = Result.ok s ∧
       (s.a.val, s.b.val) = AdlerSpec.run 1 0 data.val)
     (hc : ∀ s : Adler32, ∃ out, Adler32.checksum s = Result.ok out ∧
       out.val = 65536 * s.b.val + s.a.val) :
     ∃ out, adler32_slice data = Result.ok out ∧
       out.val = AdlerSpec.encode (AdlerSpec.run 1 0 data.val) := by
   obtain ⟨s, hs, hr⟩ := hw
   obtain ⟨out, ho, hv⟩ := hc s
   refine ⟨out, ?_, ?_⟩
   · simp [adler32_slice, Adler32.new, Adler32.Insts.CoreDefaultDefault.default, hs, ho]
   · rw [hv, ← hr]
     rfl
end AdlerProof

namespace AdlerProof
 theorem chunks_exact_lengths {α : Type} {n : Nat} (hn : 0 < n) (xs : List α) :
     ∀ c ∈ (List.toChunksExact n hn xs).1, c.length = n := by
   unfold List.toChunksExact
   split
   · simp
   · rename_i h
     simp only [List.mem_cons]
     intro c hc
     rcases hc with rfl | hc
     · simp only [List.length_take]
       omega
     · exact chunks_exact_lengths hn (xs.drop n) c hc
 termination_by xs.length
 decreasing_by simp [List.length_drop]; omega
 theorem chunks_remainder_lt {α : Type} {n : Nat} (hn : 0 < n) (xs : List α) :
     (List.toChunksExact n hn xs).2.length < n := by
   unfold List.toChunksExact
   split
   · assumption
   · exact chunks_remainder_lt hn (xs.drop n)
 termination_by xs.length
 decreasing_by simp [List.length_drop]; omega
 theorem chunks_reconstruct {α : Type} {n : Nat} (hn : 0 < n) (xs : List α) :
     (List.toChunksExact n hn xs).1.flatten ++ (List.toChunksExact n hn xs).2 = xs := by
   unfold List.toChunksExact
   split
   · rfl
   · simp only [List.flatten_cons, List.append_assoc]
     rw [chunks_reconstruct hn (xs.drop n)]
     exact List.take_append_drop n xs
 termination_by xs.length
 decreasing_by simp [List.length_drop]; omega
end AdlerProof
namespace AdlerProof
@[step] theorem chunks_exact_spec {α : Type} (s : Slice α) (n : Usize) (hn : 0 < n.val) :
    core.slice.Slice.chunks_exact s n ⦃ it =>
      it.chunks.map Slice.val = (List.toChunksExact n.val hn s.val).1 ∧
      it.remainder.val = (List.toChunksExact n.val hn s.val).2 ∧
      (∀ c ∈ it.chunks, c.length = n.val) ∧
      it.remainder.length < n.val ∧
      (it.chunks.map Slice.val).flatten ++ it.remainder.val = s.val ⦄ := by
  unfold core.slice.Slice.chunks_exact
  simp only [dite_eq_left hn, WP.spec_ok]
  simp only [List.map_map, Function.comp_def, Slice.from_val, List.attach_map_subtype_val, true_and]
  refine ⟨?_, ?_, ?_⟩
  · intro c hc
    simp only [List.mem_map, List.mem_attach, true_and] at hc
    obtain ⟨x, rfl⟩ := hc
    simpa only [Slice.length, Slice.from_val] using chunks_exact_lengths hn s.val x.val x.property
  · simpa only [Slice.length, Slice.from_val] using chunks_remainder_lt hn s.val
  · exact chunks_reconstruct hn s.val
end AdlerProof
namespace AdlerProof
@[step] theorem chunks_remainder_spec {α : Type} (it : core.slice.iter.ChunksExact α) :
    it.getRemainder ⦃ r => r = it.remainder ⦄ := by
  exact WP.spec_ok _ |>.mpr rfl
end AdlerProof


namespace AdlerProof
theorem vector_from_lane_spec (bytes : Slice U8) (h : 4 ≤ bytes.length) :
    algo.U32X4.from bytes ⦃ out => ∀ i : Fin 4, AdlerSpec.lane out i = bytes.val[i.val]!.val ⦄ := by
  change 4 ≤ bytes.val.length at h
  have hb0 : 0 < bytes.val.length := by omega
  have hb1 : 1 < bytes.val.length := by omega
  have hb2 : 2 < bytes.val.length := by omega
  have hb3 : 3 < bytes.val.length := by omega
  unfold algo.U32X4.from
  simp only [lift]
  step*
  intro ⟨i, hi⟩
  have hi' : i = 0 ∨ i = 1 ∨ i = 2 ∨ i = 3 := by omega
  rcases hi' with rfl | rfl | rfl | rfl <;> simp_all [Array.make_val, AdlerSpec.lane]
theorem vector_from_contract (bytes : Slice U8) (h : 4 ≤ bytes.length) :
    ∃ out, algo.U32X4.from bytes = Result.ok out ∧
      ∀ i : Fin 4, AdlerSpec.lane out i = bytes.val[i.val]!.val :=
  ElicitAdapter.spec_to_contract (vector_from_lane_spec bytes h)
elicit_spec vector_from_rule from vector_from_contract
end AdlerProof

namespace AdlerProof
theorem vector_add_spec (self other : algo.U32X4)
    (h : ∀ i : Fin 4, AdlerSpec.lane self i + AdlerSpec.lane other i ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign self other ⦃ out =>
      ∀ i : Fin 4, AdlerSpec.lane out i = AdlerSpec.lane self i + AdlerSpec.lane other i ⦄ := by
  have h0 := h ⟨0, by decide⟩
  have h1 := h ⟨1, by decide⟩
  have h2 := h ⟨2, by decide⟩
  have h3 := h ⟨3, by decide⟩
  simp only [AdlerSpec.lane] at h0 h1 h2 h3
  unfold algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step* by simp_all [Array.set_val_eq]
  intro ⟨i, hi⟩
  have hi' : i = 0 ∨ i = 1 ∨ i = 2 ∨ i = 3 := by omega
  rcases hi' with rfl | rfl | rfl | rfl <;> simp_all [AdlerSpec.lane]
theorem vector_add_contract (self other : algo.U32X4)
    (h : ∀ i : Fin 4, AdlerSpec.lane self i + AdlerSpec.lane other i ≤ U32.max) :
    ∃ out, algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign self other = Result.ok out ∧
      ∀ i : Fin 4, AdlerSpec.lane out i = AdlerSpec.lane self i + AdlerSpec.lane other i :=
  ElicitAdapter.spec_to_contract (vector_add_spec self other h)
elicit_spec vector_add_rule from vector_add_contract
end AdlerProof
namespace AdlerProof
 theorem inner_step_spec (chunk remainder : Slice U8) (chunks : List (Slice U8))
     (a b : algo.U32X4) (hc : 4 ≤ chunk.length)
     (ha : ∀ i : Fin 4, AdlerSpec.lane a i + chunk.val[i.val]!.val ≤ U32.max)
     (hb : ∀ i : Fin 4, AdlerSpec.lane b i + (AdlerSpec.lane a i + chunk.val[i.val]!.val) ≤ U32.max) :
     algo.Adler32.compute_loop0_loop0.body ⟨chunk :: chunks, remainder⟩ a b ⦃ r =>
       ∃ av bv, r = ControlFlow.cont (⟨chunks, remainder⟩, av, bv) ∧
       ∀ i : Fin 4,
         AdlerSpec.lane av i = AdlerSpec.lane a i + chunk.val[i.val]!.val ∧
         AdlerSpec.lane bv i = AdlerSpec.lane b i + (AdlerSpec.lane a i + chunk.val[i.val]!.val) ⦄ := by
   unfold algo.Adler32.compute_loop0_loop0.body core.slice.iter.IteratorChunksExact.next
   simp only [bind_tc_ok]
   step* by (intros; simp_all)
end AdlerProof


namespace AdlerProof
 theorem inner_loop_spec (chunks : List (Slice U8)) (remainder : Slice U8)
     (a b : algo.U32X4) (hs : AdlerSpec.safeLanes chunks (AdlerSpec.lane a) (AdlerSpec.lane b)) :
     algo.Adler32.compute_loop0_loop0 ⟨chunks, remainder⟩ a b ⦃ av bv =>
       (AdlerSpec.lane av, AdlerSpec.lane bv) =
         AdlerSpec.rawLanes chunks (AdlerSpec.lane a) (AdlerSpec.lane b) ⦄ := by
   induction chunks generalizing a b with
   | nil =>
     unfold algo.Adler32.compute_loop0_loop0
     rw [loop]
     simp [algo.Adler32.compute_loop0_loop0.body, core.slice.iter.IteratorChunksExact.next,
       AdlerSpec.rawLanes]
   | cons c cs ih =>
     obtain ⟨hc, ha, hb, ht⟩ := hs
     unfold algo.Adler32.compute_loop0_loop0
     rw [loop]
     apply WP.spec_bind (inner_step_spec c remainder cs a b hc ha hb)
     intro r hr
     obtain ⟨av, bv, rfl, hp⟩ := hr
     have hpa : AdlerSpec.lane av = fun i => AdlerSpec.lane a i + c.val[i.val]!.val := by
       funext i; exact (hp i).1
     have hpb : AdlerSpec.lane bv = fun i => AdlerSpec.lane b i + (AdlerSpec.lane a i + c.val[i.val]!.val) := by
       funext i; exact (hp i).2
     have ht' : AdlerSpec.safeLanes cs (AdlerSpec.lane av) (AdlerSpec.lane bv) := by
       simpa only [hpa, hpb] using ht
     have hrec := ih av bv ht'
     simpa only [algo.Adler32.compute_loop0_loop0, AdlerSpec.rawLanes, hpa, hpb] using hrec
end AdlerProof
namespace AdlerProof
 theorem triangular_succ (k : Nat) :
     (k + 1) * (k + 1 + 1) / 2 = k * (k + 1) / 2 + (k + 1) := by
   simp only [Nat.mul_add, Nat.add_mul, Nat.one_mul, Nat.mul_one]
   omega
 theorem safe_lanes_bound (chunks : List (Slice U8)) (k : Nat) (a b : Fin 4 → Nat)
     (hlen : k + chunks.length ≤ 5552)
     (hc : ∀ c ∈ chunks, 4 ≤ c.length)
     (ha : ∀ i, a i ≤ 65520 + k * 255)
     (hb : ∀ i, b i ≤ 65520 + k * 65520 + 255 * (k * (k + 1) / 2)) :
     AdlerSpec.safeLanes chunks a b := by
   induction chunks generalizing k a b with
   | nil => trivial
   | cons c cs ih =>
     have hx (i : Fin 4) : c.val[i.val]!.val ≤ 255 := by scalar_tac
     have ha' (i : Fin 4) : a i + c.val[i.val]!.val ≤ 65520 + (k + 1) * 255 := by
       have := ha i; have := hx i; omega
     have hb' (i : Fin 4) : b i + (a i + c.val[i.val]!.val) ≤
         65520 + (k + 1) * 65520 + 255 * ((k + 1) * (k + 1 + 1) / 2) := by
       rw [triangular_succ]
       have := hb i; have := ha i; have := hx i
       omega
     have hmax (i : Fin 4) : b i + (a i + c.val[i.val]!.val) ≤ U32.max := by
       have h := lane_bound (k + 1) 65520 65520 (by simp only [List.length_cons] at hlen; omega) (by omega) (by omega)
       have := hb' i
       scalar_tac
     refine ⟨hc c (by simp), ?_, hmax, ?_⟩
     · intro i
       have := ha' i
       simp only [List.length_cons] at hlen
       scalar_tac
     · apply ih (k + 1) (fun i => a i + c.val[i.val]!.val)
         (fun i => b i + (a i + c.val[i.val]!.val))
       · simp only [List.length_cons] at hlen
         omega
       · intro x hx; exact hc x (by simp [hx])
       · exact ha'
       · exact hb'
end AdlerProof
namespace AdlerProof
@[step] theorem inner_loop_bounded (chunks : List (Slice U8)) (remainder : Slice U8)
     (a b : algo.U32X4) (hn : chunks.length ≤ 5552)
     (hc : ∀ c ∈ chunks, 4 ≤ c.length)
     (ha : ∀ i, AdlerSpec.lane a i ≤ 65520)
     (hb : ∀ i, AdlerSpec.lane b i ≤ 65520) :
     algo.Adler32.compute_loop0_loop0 ⟨chunks, remainder⟩ a b ⦃ av bv =>
       (AdlerSpec.lane av, AdlerSpec.lane bv) =
         AdlerSpec.rawLanes chunks (AdlerSpec.lane a) (AdlerSpec.lane b) ⦄ := by
   apply inner_loop_spec
   apply safe_lanes_bound chunks 0 (AdlerSpec.lane a) (AdlerSpec.lane b)
   · simpa using hn
   · exact hc
   · simpa using ha
   · simpa using hb
@[step] theorem remainder_loop_bounded (chunks : List (Slice U8)) (remainder : Slice U8)
     (a b : algo.U32X4) (hn : chunks.length ≤ 5552)
     (hc : ∀ c ∈ chunks, 4 ≤ c.length)
     (ha : ∀ i, AdlerSpec.lane a i ≤ 65520)
     (hb : ∀ i, AdlerSpec.lane b i ≤ 65520) :
     algo.Adler32.compute_loop1 ⟨chunks, remainder⟩ a b ⦃ av bv =>
       (AdlerSpec.lane av, AdlerSpec.lane bv) =
         AdlerSpec.rawLanes chunks (AdlerSpec.lane a) (AdlerSpec.lane b) ⦄ := by
   exact inner_loop_bounded chunks remainder a b hn hc ha hb
end AdlerProof
namespace AdlerProof
 theorem chunks_count_bound {α : Type} {n : Nat} (hn : 0 < n) (xs : List α) :
     (List.toChunksExact n hn xs).1.length * n ≤ xs.length := by
   unfold List.toChunksExact
   split
   · simp
   · have ih := chunks_count_bound hn (xs.drop n)
     simp only [List.length_cons, Nat.add_mul, Nat.one_mul]
     simp only [List.length_drop] at ih
     omega
 termination_by xs.length
 decreasing_by simp [List.length_drop]; omega
end AdlerProof

namespace AdlerProof
@[step] theorem inner_iter_rule (iter : core.slice.iter.ChunksExact U8)
     (a b : algo.U32X4) (hn : iter.chunks.length ≤ 5552)
     (hc : ∀ c ∈ iter.chunks, 4 ≤ c.length)
     (ha : ∀ i, AdlerSpec.lane a i ≤ 65520)
     (hb : ∀ i, AdlerSpec.lane b i ≤ 65520) :
     algo.Adler32.compute_loop0_loop0 iter a b ⦃ av bv =>
       (AdlerSpec.lane av, AdlerSpec.lane bv) =
         AdlerSpec.rawLanes iter.chunks (AdlerSpec.lane a) (AdlerSpec.lane b) ⦄ := by
   exact inner_loop_bounded iter.chunks iter.remainder a b hn hc ha hb
@[step] theorem remainder_iter_rule (iter : core.slice.iter.ChunksExact U8)
     (a b : algo.U32X4) (hn : iter.chunks.length ≤ 5552)
     (hc : ∀ c ∈ iter.chunks, 4 ≤ c.length)
     (ha : ∀ i, AdlerSpec.lane a i ≤ 65520)
     (hb : ∀ i, AdlerSpec.lane b i ≤ 65520) :
     algo.Adler32.compute_loop1 iter a b ⦃ av bv =>
       (AdlerSpec.lane av, AdlerSpec.lane bv) =
         AdlerSpec.rawLanes iter.chunks (AdlerSpec.lane a) (AdlerSpec.lane b) ⦄ := by
   exact remainder_loop_bounded iter.chunks iter.remainder a b hn hc ha hb
end AdlerProof
