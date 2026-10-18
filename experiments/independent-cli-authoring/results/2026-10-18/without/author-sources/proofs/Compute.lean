import Combine
open Aeneas Aeneas.Std Result ControlFlow
namespace AdlerProof
set_option maxHeartbeats 4000000
set_option maxRecDepth 4096

@[simp] theorem view_zero : view (Array.repeat 4#usize 0#u32) = fun _ => 0 := by
 funext i; fin_cases i <;> simp [view]

@[step] theorem chunk_size_correct : adler2.algo.Adler32.compute.CHUNK_SIZE ⦃ n => n = 22208#usize ⦄ := by
 unfold adler2.algo.Adler32.compute.CHUNK_SIZE
 step
 scalar_tac

theorem compute_correct_ring (self : adler2.Adler32) (bytes : Slice U8) :
 adler2.algo.Adler32.compute self bytes ⦃ out =>
 out.a.val < 65521 ∧ out.b.val < 65521 ∧
 ((out.a.val : R), (out.b.val : R)) = rfold ((self.a.val:R),(self.b.val:R)) bytes.val ⦄ := by
 unfold adler2.algo.Adler32.compute
 step as ⟨a, ha⟩
 step as ⟨b, hb⟩
 step as ⟨m, hm⟩
 step as ⟨n, hn⟩
 step as ⟨pre, tail, hp, ht, hptake, htdrop⟩
 step as ⟨csz, hcsz⟩
 subst csz
 step with chunks_exact_spec pre 22208#usize (by decide) as ⟨it, hchunks, hrem, hflat, hlen⟩
 simp only [core.slice.iter.ChunksExact.getRemainder]
 step with outer_loop it.chunks it.remainder a b (Array.repeat 4#usize 0#u32) (Array.repeat 4#usize 0#u32) (by scalar_tac) (by scalar_tac) hchunks
   (by simp [laneBound]) as ⟨b1,av,bv, hbb, hv, houter⟩
 have hprediv : pre.val.length % 4 = 0 := by
   simp only [Slice.length, Slice.len_val] at *
   omega
 have hrdiv : it.remainder.val.length % 4 = 0 := by omega
 step with chunks_four it.remainder hrdiv as ⟨small, hsizes, hsmallrem, hsmallflat, hsmalllen⟩
 step with final_inner_loop small.chunks small.remainder av bv 0 hsizes (by omega) hv as ⟨av1,bv1, hrun, hbound⟩
 step as ⟨rlen, hrlen⟩
 step as ⟨prod, hprod⟩
 step as ⟨b2, hb2⟩
 step with vector_rem av1 adler2.algo.Adler32.compute.MOD (by simp) as ⟨av2,hav2⟩
 step with vector_rem bv1 adler2.algo.Adler32.compute.MOD (by simp) as ⟨bv2,hbv2⟩
 step with U32.rem_spec b2 (y:=adler2.algo.Adler32.compute.MOD) (by simp) as ⟨b3,hb3⟩
 simp only [mod_value] at hav2 hbv2 hb3
 have hnormalized : laneBound 0 (view av2) (view bv2) := by
   rw [hav2,hbv2]; exact normalized_lanes _ _
 have hb3bound : b3.val < 65521 := by
   rw [hb3]; exact Nat.mod_lt _ (by decide)
 have haction := laneRun_action small.chunks hsizes a.val b1.val (view av) (view bv)
 rw [← hrun, hsmallflat] at haction
 have hbase : b2.val = b1.val + 4*small.chunks.length*a.val := by
   rw [hb2,hprod,hrlen,Slice.len_val,Slice.length,hsmalllen]
 have hpre : interp a.val b3.val (view av2) (view bv2) =
     rfold ((self.a.val:R),(self.b.val:R)) pre.val := by
   rw [hav2,hbv2,hb3,interp_mod,hbase,haction,houter]
   simp only [view_zero,interp,sum4,Nat.cast_zero,add_zero,mul_zero,sub_zero,ha,hb]
   rw [← rfold_append,hflat]
 step with vector_mul bv2 4#u32 (by
   intro i
   have h := (hnormalized i).2
   simp only [Nat.mul_zero,Nat.add_zero,Nat.zero_mul] at h
   simp; omega) as ⟨bv3,hbv3⟩
 have hna : ∀ i, view av2 i ≤ 65520 := by intro i; have h := (hnormalized i).1; simpa using h
 have hnb : ∀ i, view bv2 i ≤ 65520 := by intro i; have h := (hnormalized i).2; simp at h; omega
 have ha0 := hna 0; have ha1 := hna 1; have ha2 := hna 2; have ha3 := hna 3
 have hb0 := hnb 0; have hb1 := hnb 1; have hb2' := hnb 2; have hb3' := hnb 3
 step as ⟨x1,hx1⟩
 change x1.val = view av2 1 at hx1
 step as ⟨d1,hd1⟩
 · simp only [mod_value]; omega
 simp only [mod_value] at hd1
 step as ⟨y1,hy1⟩
 simp only [hbv3] at hy1
 change y1.val = view bv2 1 * 4 at hy1
 step as ⟨z1,hz1⟩
 step as ⟨v1,hv1⟩
 step as ⟨x2,hx2⟩
 change x2.val = view av2 2 at hx2
 step as ⟨d2,hd2⟩
 · simp only [mod_value]; omega
 simp only [mod_value] at hd2
 step as ⟨e2,he2⟩
 step as ⟨y2,hy2⟩
 simp only [hv1,view_set,hbv3] at hy2
 norm_num only at hy2
 change y2.val = view bv2 2 * 4 at hy2
 step as ⟨z2,hz2⟩
 step as ⟨v2,hv2⟩
 step as ⟨x3,hx3⟩
 change x3.val = view av2 3 at hx3
 step as ⟨d3,hd3⟩
 · simp only [mod_value]; omega
 simp only [mod_value] at hd3
 step as ⟨e3,he3⟩
 step as ⟨y3,hy3⟩
 simp only [hv2,hv1,view_set,hbv3] at hy3
 norm_num only at hy3
 change y3.val = view bv2 3 * 4 at hy3
 step as ⟨z3,hz3⟩
 step as ⟨v3,hv3⟩
 have hvadjusted : view v3 = adjusted (view av2) (view bv2) := by
   funext i
   fin_cases i <;> simp [hv3,hv2,hv1,view_set,hbv3,adjusted,hz1,hz2,hz3,
     hy1,hy2,hy3,he2,he3,hd1,hd2,hd3,hx1,hx2,hx3,Nat.mul_comm]
 have hsumA : sum4 (view av2) ≤ 262080 := by
   exact sum4_bound _ 65520 hna
 have hsumB : sum4 (view v3) ≤ 1834572 := by
   rw [hvadjusted]
   exact sum4_bound _ 458643 (adjusted_bound _ _ hna hnb)
 step as ⟨sA,hsA⟩
 step as ⟨iterA,hiterA⟩
 subst iterA
 step with sum_loop sA 0 a (by
   simp only [List.drop_zero,hsA,sumValues_array]
   have h := self.a.hmax
   change self.a.val < 65536 at h
   rw [ha]
   omega) as ⟨a5,ha5⟩
 simp only [List.drop_zero,hsA,sumValues_array] at ha5
 step as ⟨sB,hsB⟩
 step as ⟨iterB,hiterB⟩
 subst iterB
 step with sum_loop3 sB 0 b3 (by
   simp only [List.drop_zero,hsB,sumValues_array]
   omega) as ⟨b4,hb4⟩
 simp only [List.drop_zero,hsB,sumValues_array] at hb4
 have ha5bound : a5.val ≤ 327615 := by have h:= self.a.hmax; change self.a.val < 65536 at h; rw [ha] at ha5; omega
 have hb4bound : b4.val ≤ 1900092 := by omega
 have hcombine : ((a5.val:R),(b4.val:R)) = rfold ((self.a.val:R),(self.b.val:R)) pre.val := by
   rw [ha5,hb4,hvadjusted,combination_identity a.val b3.val (view av2) (view bv2) (by intro i; have h:=hna i; omega)]
   exact hpre
 have htailLen : tail.val.length ≤ 3 := by
   simp only [Slice.len_val,Slice.length] at hm hn ht
   omega
 step as ⟨iterTail,hiterTail⟩
 subst iterTail
 step with tail_loop tail 0 a5 b4 (by
   simp only [List.drop_zero]
   apply raw_safe_budget
   · omega
   · have ht0 : tail.val.length * (a5.val+255*tail.val.length) ≤ 3*(327615+255*3) :=
       Nat.mul_le_mul htailLen (by omega)
     omega) as ⟨a6,b5,htail⟩
 try simp only [List.drop_zero] at htail
 have hwhole : ((a6.val:R),(b5.val:R)) = rfold ((self.a.val:R),(self.b.val:R)) bytes.val := by
   have hr := rawFold_cast tail.val a5.val b4.val
   rw [← htail,hcombine,←rfold_append] at hr
   rw [hptake,htdrop,List.take_append_drop] at hr
   exact hr
 step with U32.rem_spec a6 (y:=adler2.algo.Adler32.compute.MOD) (by simp) as ⟨am,ham⟩
 simp only [mod_value] at ham
 step with cast_u16 am (by have h:=Nat.mod_lt a6.val (by decide : 0<65521); omega) as ⟨ao,hao⟩
 step with U32.rem_spec b5 (y:=adler2.algo.Adler32.compute.MOD) (by simp) as ⟨bm,hbm⟩
 simp only [mod_value] at hbm
 step with cast_u16 bm (by have h:=Nat.mod_lt b5.val (by decide : 0<65521); omega) as ⟨bo,hbo⟩
 refine ⟨?_,?_,?_⟩
 · change ao.val < 65521
   rw [hao,ham]; exact Nat.mod_lt _ (by decide)
 · change bo.val < 65521
   rw [hbo,hbm]; exact Nat.mod_lt _ (by decide)
 · change ((ao.val:R),(bo.val:R)) = _
   rw [hao,hbo,ham,hbm]
   simpa using hwhole
