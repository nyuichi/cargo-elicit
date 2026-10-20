import FinishMath
open Aeneas Aeneas.Std Result ElicitGenerated
set_option maxHeartbeats 4000000
set_option maxRecDepth 4096
namespace Compute
open Vector LaneAlgebra
@[step] theorem compute_correct (self : Adler32) (bytes : Slice U8) :
    algo.Adler32.compute self bytes ⦃ out =>
      AdlerSpec.state out = AdlerSpec.run (AdlerSpec.state self) bytes.val ⦄ := by
  unfold algo.Adler32.compute
  simp only [lift]
  step (config := { grind := false, scalarTac := true }) as ⟨r4, hr4⟩
  step (config := { grind := false, scalarTac := true }) as ⟨aligned, haligned⟩
  step (config := { grind := false, scalarTac := true }) as ⟨bulk, tail, hp_len, ht_len, hp_val, ht_val⟩
  step (config := { grind := false, scalarTac := true }) with chunk_size_spec as ⟨chunk_size, hchunk_size⟩
  subst chunk_size
  step (config := { grind := false, scalarTac := true }) with Chunks.exact_spec bulk 22208#usize (by scalar_tac) as ⟨chunks, hpartition, hchunk_lens, hrem_lt⟩
  simp only [core.slice.iter.ChunksExact.getRemainder]
  have ha : (core.convert.num.FromU32U16.from self.a).val ≤ 65535 := by
    rw [core.convert.num.FromU32U16.from_val_eq]
    scalar_tac
  have hb : (core.convert.num.FromU32U16.from self.b).val ≤ 65535 := by
    rw [core.convert.num.FromU32U16.from_val_eq]
    scalar_tac
  step (config := { grind := false, scalarTac := true }) with Outer.outer_spec chunks.chunks chunks.remainder _ _ _ _ ha hb zeros_normalized zeros_normalized hchunk_lens
    as ⟨b0, av0, bv0, hb0, hav0, hbv0, houter⟩

  simp only [Slice.length, Slice.len_val] at hr4 haligned hp_len ht_len
  have hbulk4 : bulk.val.length % 4 = 0 := by
    omega
  have hcslen := Chunks.values_length chunks.chunks 22208 hchunk_lens
  have hpartlen := congrArg List.length hpartition
  simp only [List.length_append] at hpartlen
  have hrem4 : chunks.remainder.val.length % 4 = 0 := by omega
  step (config := { grind := false, scalarTac := true }) with Chunks.four_spec chunks.remainder hrem4 as ⟨it, hvals, hlens, hcount⟩
  have hbud : ∀ i, Inner.budget it.chunks.length (lane av0 i) (lane bv0 i) := by
    intro i
    exact Inner.budget_initial _ _ _ (by omega) (hav0 i) (hbv0 i)
  step (config := { grind := false, scalarTac := true }) with Inner.remainder_spec it.chunks it.remainder av0 bv0 hlens hbud as ⟨av1, bv1, hinner⟩
  have hcast : (UScalar.cast .U32 chunks.remainder.len).val = chunks.remainder.val.length := by
    have h := UScalar.cast_inBounds_spec .U32 chunks.remainder.len (by scalar_tac)
    simpa only [lift, WP.spec_ok, Slice.len_val] using h
  step (config := { grind := false, scalarTac := true }) as ⟨prod, hprod⟩
  step (config := { grind := false, scalarTac := true }) as ⟨b1, hb1⟩
  simp only [algo.Adler32.compute.MOD]
  step (config := { grind := false, scalarTac := true }) as ⟨av2, hav2⟩
  step (config := { grind := false, scalarTac := true }) as ⟨bv2, hbv2⟩
  step (config := { grind := false, scalarTac := true }) as ⟨b2, hb2⟩
  have hva := Outer.reduce_lanes av1 av2 hav2
  have hvb := Outer.reduce_lanes bv1 bv2 hbv2
  have hbc : (b2.val : Z) = (b0.val : Z) + 4 * (it.chunks.length : Z) * (self.a.val : Z) := by
    simp only [hb2, cast_mod, hb1, Nat.cast_add, hprod, Nat.cast_mul, hcast,
      core.convert.num.FromU32U16.from_val_eq]
    have hlen : chunks.remainder.val.length = 4*it.chunks.length := by omega
    rw [hlen]
    push_cast
    ring
  have hcombined : combine (self.a.val : Z) (b2.val : Z) (lanes av2) (lanes bv2) =
      run (toZ (AdlerSpec.state self)) bulk.val := by
    rw [hva.2, hvb.2, hbc]
    rw [inner_combine it.chunks (self.a.val : Z) (b0.val : Z) av0 bv0 av1 bv1 hlens hinner]
    rw [hvals]
    have ho := houter
    simp only [core.convert.num.FromU32U16.from_val_eq, zeros_lanes] at ho
    have hz : combine (self.a.val : Z) (self.b.val : Z) (fun _ => 0) (fun _ => 0) =
        toZ (AdlerSpec.state self) := by
      simp [combine, LaneAlgebra.sum, weight, LaneAlgebra.toZ, AdlerSpec.state]
    rw [hz] at ho
    rw [← hpartition, run_append, ← ho]
  have hv2bound : ∀ i, lane bv2 i * (4#u32).val ≤ 4294967295 := by
    intro i
    have := hvb.1 i
    scalar_tac
  step (config := { grind := false, scalarTac := true }) with Vector.mul_spec bv2 4#u32 hv2bound as ⟨bv3, hbv3⟩
  have ha0 := hva.1 0; have ha1 := hva.1 1; have ha2 := hva.1 2; have ha3 := hva.1 3
  have hm0 := hbv3 0; have hm1 := hbv3 1; have hm2 := hbv3 2; have hm3 := hbv3 3
  have hn0 := hvb.1 0; have hn1 := hvb.1 1; have hn2 := hvb.1 2; have hn3 := hvb.1 3
  step (config := { grind := false, scalarTac := true }) as ⟨ai1, hai1⟩
  step (config := { grind := false, scalarTac := true }) as ⟨d1, hd1⟩ by (simp_all [lane, Array.set_val_eq] <;> scalar_tac)
  step (config := { grind := false, scalarTac := true }) as ⟨bi1, hbi1⟩
  step (config := { grind := false, scalarTac := true }) as ⟨c1, hc1⟩ by (simp_all [lane, Array.set_val_eq] <;> scalar_tac)
  step (config := { grind := false, scalarTac := true }) as ⟨v1, hv1⟩
  step (config := { grind := false, scalarTac := true }) as ⟨ai2, hai2⟩
  step (config := { grind := false, scalarTac := true }) as ⟨d2, hd2⟩ by (simp_all [lane, Array.set_val_eq] <;> scalar_tac)
  step (config := { grind := false, scalarTac := true }) as ⟨e2, he2⟩ by scalar_tac
  step (config := { grind := false, scalarTac := true }) as ⟨bi2, hbi2⟩
  step (config := { grind := false, scalarTac := true }) as ⟨c2, hc2⟩ by (simp_all [lane, Array.set_val_eq] <;> scalar_tac)
  step (config := { grind := false, scalarTac := true }) as ⟨v2, hv2⟩
  step (config := { grind := false, scalarTac := true }) as ⟨ai3, hai3⟩
  step (config := { grind := false, scalarTac := true }) as ⟨d3, hd3⟩ by (simp_all [lane, Array.set_val_eq] <;> scalar_tac)
  step (config := { grind := false, scalarTac := true }) as ⟨e3, he3⟩ by scalar_tac
  step (config := { grind := false, scalarTac := true }) as ⟨bi3, hbi3⟩
  step (config := { grind := false, scalarTac := true }) as ⟨c3, hc3⟩ by (simp_all [lane, Array.set_val_eq] <;> scalar_tac)
  step (config := { grind := false, scalarTac := true }) as ⟨v3, hv3⟩
  have hsum_a : (core.convert.num.FromU32U16.from self.a).val + Tail.total ⟨av2.to_slice, 0⟩ ≤ 400000 := by
    rw [total_array]
    omega
  have hsum_v3 : Tail.total ⟨v3.to_slice, 0⟩ = lane bv3 0 + c1.val + c2.val + c3.val := by
    simp [total_array, lane, hv3, hv2, hv1]
  have hsum_b : b2.val + Tail.total ⟨v3.to_slice, 0⟩ ≤ 2000000 := by
    rw [hsum_v3]
    simp [lane] at hm0 hm1 hm2 hm3 hn0 hn1 hn2 hn3
    simp [hv2, hv1] at hbi2 hbi3
    have hbb : b2.val < 65521 := by omega
    simp only [lane, Fin.val_zero, hc1, hc2, hc3, hbi1, hbi2, hbi3, hd1, he2, he3, hd2, hd3]
    omega
  simp only [core.slice.Slice.iter]
  step (config := { grind := false, scalarTac := true }) with Tail.sum_spec ⟨av2.to_slice, 0⟩ (core.convert.num.FromU32U16.from self.a) (by omega) as ⟨asum, hasum⟩
  step (config := { grind := false, scalarTac := true }) with Tail.sum_b_spec ⟨v3.to_slice, 0⟩ b2 (by omega) as ⟨bsum, hbsum⟩
  have hasum_bound : asum.val ≤ 400000 := by omega
  have hbsum_bound : bsum.val ≤ 2000000 := by omega
  have htail3 : tail.val.length ≤ 3 := by omega
  have hserial : Inner.budget tail.val.length asum.val bsum.val := by
    have hna : tail.val.length * asum.val ≤ 3*400000 := Nat.mul_le_mul htail3 hasum_bound
    have hnn : tail.val.length * (tail.val.length+1) ≤ 3*4 := Nat.mul_le_mul htail3 (by omega)
    constructor <;> dsimp [Inner.max32] <;> nlinarith
  step (config := { grind := false, scalarTac := true }) with Tail.serial_spec ⟨tail, 0⟩ asum bsum (by simpa [Tail.rest] using hserial)
    as ⟨afinal, bfinal, hfinal⟩
  step (config := { grind := false, scalarTac := true }) as ⟨ared, hared⟩
  step (config := { grind := false, scalarTac := true }) as ⟨bred, hbred⟩
  have hd1z : (d1.val : Z) = -lanes av2 1 := by
    apply correction_cast av2 1 d1 (by omega)
    simpa [lane, hai1] using hd1
  have hd2z : (d2.val : Z) = -lanes av2 2 := by
    apply correction_cast av2 2 d2 (by omega)
    simpa [lane, hai2] using hd2
  have hd3z : (d3.val : Z) = -lanes av2 3 := by
    apply correction_cast av2 3 d3 (by omega)
    simpa [lane, hai3] using hd3
  have hbi2' : bi2 = bv3.val[2] := by simpa [hv1] using hbi2
  have hbi3' : bi3 = bv3.val[3] := by simpa [hv2, hv1] using hbi3
  have hlanes3 : lanes bv3 = fun i => lanes bv2 i * 4 := by
    funext i
    simp only [lanes, hbv3, Nat.cast_mul, Nat.cast_ofNat]
  have hcorrection : (Tail.total ⟨v3.to_slice, 0⟩ : Z) =
      4*LaneAlgebra.sum (lanes bv2) - weight (lanes av2) := by
    simp only [hsum_v3, hc1, hc2, hc3, he2, he3, hbi1, hbi2', hbi3', Nat.cast_add, Nat.cast_mul]
    change lanes bv3 0 + (lanes bv3 1 + (d1.val : Z)) +
      (lanes bv3 2 + (d2.val : Z)*2) + (lanes bv3 3 + (d3.val : Z)*3) = _
    rw [hlanes3, hd1z, hd2z, hd3z]
    simp only [LaneAlgebra.sum, weight]
    ring
  have hbase : ((asum.val : Z), (bsum.val : Z)) =
      combine (self.a.val : Z) (b2.val : Z) (lanes av2) (lanes bv2) := by
    apply Prod.ext
    · simp only [hasum, Nat.cast_add, core.convert.num.FromU32U16.from_val_eq, cast_total, combine]
    · simp only [hbsum, Nat.cast_add, hcorrection, combine]
      ring
  have hresult : ((afinal.val : Z), (bfinal.val : Z)) =
      run (toZ (AdlerSpec.state self)) bytes.val := by
    rw [hfinal, hbase, hcombined, ← run_append]
    simp only [Tail.rest, List.drop_zero]
    rw [hp_val, ht_val, List.take_append_drop]
  have hacast : (UScalar.cast .U16 ared).val = ared.val := by
    have h := UScalar.cast_inBounds_spec .U16 ared (by scalar_tac)
    simpa only [lift, WP.spec_ok] using h
  have hbcast : (UScalar.cast .U16 bred).val = bred.val := by
    have h := UScalar.cast_inBounds_spec .U16 bred (by scalar_tac)
    simpa only [lift, WP.spec_ok] using h
  apply LaneAlgebra.cast_inj
  · simp only [AdlerSpec.state, hacast, hbcast]
    constructor <;> omega
  · exact AdlerSpec.run_bounds _ _
  · rw [cast_run]
    simp only [LaneAlgebra.toZ, AdlerSpec.state, hacast, hbcast, hared, hbred, cast_mod]
    exact hresult
end Compute
