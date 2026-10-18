import Finish
open Aeneas Aeneas.Std Result ElicitGenerated AdlerMath
set_option maxHeartbeats 2000000
namespace Proofs
@[step] theorem chunkSize_spec : algo.Adler32.compute.CHUNK_SIZE ⦃ v => v.val = 22208 ⦄ := by
  unfold algo.Adler32.compute.CHUNK_SIZE
  step*


@[step] theorem compute_spec (self : Adler32) (bytes : Slice U8) :
    algo.Adler32.compute self bytes ⦃ out =>
      Specs.state out = Specs.update (Specs.state self) bytes.val ⦄ := by
  unfold algo.Adler32.compute
  simp only [lift, bind_ok]
  step as ⟨r4, hr4⟩
  step as ⟨aligned, haligned, halignedb⟩
  step as ⟨front, rem, hp, hr, hpv, hrv⟩
  simp only [Slice.len_val, Slice.length] at hr4 haligned halignedb hp hr
  step as ⟨chunkSize, hchunkSize⟩
  step as ⟨outer, houter⟩
  simp only [core.slice.iter.ChunksExact.getRemainder, bind_ok]
  have hv0 : reduced (Array.repeat 4#usize 0#u32) (Array.repeat 4#usize 0#u32) := by
    intro j hj
    have : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
    rcases this with rfl | rfl | rfl | rfl <;> simp [lane, Array.repeat]
  have hout := outer_spec chunkSize outer (core.convert.num.FromU32U16.from self.a)
    (core.convert.num.FromU32U16.from self.b) (Array.repeat 4#usize 0#u32)
    (Array.repeat 4#usize 0#u32) hchunkSize (by simp; scalar_tac) (by simp; scalar_tac) hv0 (by simpa [hchunkSize] using houter.1)
  step with hout as ⟨b0, av0, bv0, houtPost⟩
  obtain ⟨hb0, hv, heOuter⟩ := houtPost
  step as ⟨inner, hinner⟩
  have hpdiv : front.val.length % 4 = 0 := by omega
  have houterLen := chunkFacts_length front chunkSize.val outer houter
  have hrdiv : outer.remainder.val.length % 4 = 0 := by
    rw [hchunkSize] at houterLen
    omega
  have hinnerD := chunks_divisible outer.remainder 4 inner hinner hrdiv
  have hinnerLen := chunkFacts_length outer.remainder 4 inner hinner
  have hcount : inner.chunks.length ≤ 5552 := by
    have := houter.2.1
    rw [hchunkSize] at this
    simp only [hinnerD.1, List.length_nil, Nat.add_zero] at hinnerLen
    omega
  have hin := inner1_spec inner av0 bv0 0 (by omega) (reduced_bounds av0 bv0 hv) hinner.1
  step with hin as ⟨av1, bv1, hinnerPost⟩
  obtain ⟨hinnerBounds, heInner⟩ := hinnerPost
  simp only [lift, bind_ok, algo.Adler32.compute.MOD]
  have hremBound : outer.remainder.val.length < 22208 := by simpa [hchunkSize] using houter.2.1
  have hcast : (UScalar.cast .U32 (Slice.len outer.remainder)).val = outer.remainder.val.length := by
    simp [UScalar.cast_val_eq, Nat.mod_eq_of_lt (by omega : outer.remainder.val.length < 4294967296)]
  step as ⟨mul, hmul⟩
  step as ⟨b1, hb1⟩
  step as ⟨av2, hav2⟩
  step as ⟨bv2, hbv2⟩
  step as ⟨b2, hb2⟩
  have hred := rem_reduced av1 bv1 av2 bv2 hav2 hbv2
  have hfinish := finish_spec (core.convert.num.FromU32U16.from self.a) b2 av2 bv2 rem
    (by simp; scalar_tac) (by omega) hred (by omega)
  unfold finish at hfinish
  simp only [lift, bind_ok] at hfinish
  apply WP.spec_mono hfinish
  intro out ⟨hoa, hob, heFinish⟩
  apply castState_inj ⟨hoa,hob⟩ (update_normalized _ _)
  rw [cast_update]
  rw [heFinish]
  have hlen : 4 * inner.chunks.length = outer.remainder.val.length := by
    simp only [hinnerD.1, List.length_nil, Nat.add_zero] at hinnerLen
    omega
  have hbcast : (b2.val : F) = (b0.val : F) +
      (outer.remainder.val.length : F) * (self.a.val : F) := by
    rw [hb2, hb1, hmul, hcast]
    simp only [ZMod.natCast_mod, Nat.cast_add, Nat.cast_mul,
      core.convert.num.FromU32U16.from_val_eq]
  have hcomb : combine (self.a.val : F) b2.val av2 bv2 =
      scalar (castState (Specs.state self)) front.val := by
    rw [combine_reduce (self.a.val : F) b2.val av1 bv1 av2 bv2 hav2 hbv2, hbcast]
    rw [← hlen]
    rw [heInner (self.a.val : F) (b0.val : F), hinnerD.2]
    simp only [core.convert.num.FromU32U16.from_val_eq, Prod.fst, Prod.snd] at heOuter
    rw [heOuter]
    have hz : combine (self.a.val : F) (self.b.val : F)
        (Array.repeat 4#usize 0#u32) (Array.repeat 4#usize 0#u32) =
        castState (Specs.state self) := by
      simp [combine, lane, Array.repeat, castState, Specs.state]
    rw [hz]
    simp only [scalar, ← List.foldl_append]
    rw [houter.2.2]
  simp only [core.convert.num.FromU32U16.from_val_eq]
  rw [hcomb]
  simp only [scalar, ← List.foldl_append]
  rw [hpv, hrv, List.take_append_drop]

end Proofs
