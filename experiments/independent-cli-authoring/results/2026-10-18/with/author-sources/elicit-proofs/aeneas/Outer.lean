import Chunks
open Aeneas Aeneas.Std Result ElicitGenerated AdlerMath
namespace Proofs

def reduced (av bv : Vec) : Prop :=
  ∀ j < 4, lane av j < 65521 ∧ lane bv j < 65521

theorem reduced_bounds (av bv : Vec) (h : reduced av bv) : laneBounds 0 av bv := by
  intro j hj
  have := h j hj
  simp only [Nat.zero_mul, Nat.add_zero]
  omega

theorem rem_reduced (av bv ar br : Vec)
    (ha : ∀ j < 4, lane ar j = lane av j % 65521)
    (hb : ∀ j < 4, lane br j = lane bv j % 65521) : reduced ar br := by
  intro j hj
  rw [ha j hj, hb j hj]
  exact ⟨Nat.mod_lt _ (by decide), Nat.mod_lt _ (by decide)⟩

theorem combine_reduce (a b : F) (av bv ar br : Vec)
    (ha : ∀ j < 4, lane ar j = lane av j % 65521)
    (hb : ∀ j < 4, lane br j = lane bv j % 65521) :
    combine a b ar br = combine a b av bv := by
  have ha0 := ha 0 (by decide); have ha1 := ha 1 (by decide)
  have ha2 := ha 2 (by decide); have ha3 := ha 3 (by decide)
  have hb0 := hb 0 (by decide); have hb1 := hb 1 (by decide)
  have hb2 := hb 2 (by decide); have hb3 := hb 3 (by decide)
  simp [combine, ha0, ha1, ha2, ha3, hb0, hb1, hb2, hb3, ZMod.natCast_mod]

def outerPost (it : Chunks) (a b : U32) (av bv : Vec)
    (out : U32 × Vec × Vec) : Prop :=
  out.1.val ≤ 65535 ∧ reduced out.2.1 out.2.2 ∧
  combine a.val out.1.val out.2.1 out.2.2 =
    scalar (combine a.val b.val av bv) (flatten it.chunks)

@[step] theorem outer_spec (i : Usize) (it : Chunks) (a b : U32) (av bv : Vec)
    (hi : i.val = 22208) (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
    (hv : reduced av bv) (hc : ∀ c ∈ it.chunks, c.val.length = 22208) :
    algo.Adler32.compute_loop0 i it a b av bv ⦃ outerPost it a b av bv ⦄ := by
  unfold algo.Adler32.compute_loop0
  rw [loop]
  cases he : it.chunks with
  | nil =>
    simp [algo.Adler32.compute_loop0.body,
      core.slice.iter.IteratorChunksExact.next, he, outerPost,
      flatten, scalar, hb, hv]
  | cons c cs =>
    have hcl : c.val.length = 22208 := hc c (by simp [he])
    have hcs : ∀ c ∈ cs, c.val.length = 22208 := by
      intro d hd; exact hc d (by simp [he, hd])
    simp only [algo.Adler32.compute_loop0.body,
      core.slice.iter.IteratorChunksExact.next, he, bind_ok]
    step as ⟨iter, hf⟩
    have hd := chunks_divisible c 4 iter hf (by omega)
    have hl := chunkFacts_length c 4 iter hf
    have hcount : iter.chunks.length = 5552 := by
      simp only [hd.1, List.length_nil, Nat.add_zero] at hl
      omega
    have hinner := inner_spec iter av bv 0 (by omega) (reduced_bounds av bv hv) hf.1
    step with hinner as ⟨out, hout⟩
    rcases out with ⟨ao, bo⟩
    obtain ⟨hbound, hcombine⟩ := hout
    simp only [lift, bind_ok, algo.Adler32.compute.MOD]
    have hm : (UScalar.cast .U32 i).val = 22208 := by simp [UScalar.cast_val_eq, hi]
    step as ⟨mul, hmul⟩
    step as ⟨b', hb'⟩
    step as ⟨ar, har⟩
    step as ⟨br, hbr⟩
    step as ⟨b'', hb''⟩
    have hred := rem_reduced ao bo ar br har hbr
    have hbb : b''.val ≤ 65535 := by omega
    have hrec := outer_spec i ⟨cs, it.remainder⟩ a b'' ar br hi ha hbb hred hcs
    unfold algo.Adler32.compute_loop0 algo.Adler32.compute_loop0.body at hrec
    simp only [algo.Adler32.compute.MOD, core.slice.iter.IteratorChunksExact.next, lift, bind_ok] at hrec
    step with hrec as ⟨res, hres⟩
    obtain ⟨hrb, hrv, hre⟩ := hres
    refine ⟨hrb, hrv, ?_⟩
    simp only [outerPost, he, flatten, List.flatMap_cons, scalar, List.foldl_append] at *
    rw [hre]
    have hcomb : combine a.val b''.val ar br = scalar (combine a.val b.val av bv) c.val := by
      rw [combine_reduce a.val b''.val ao bo ar br har hbr]
      rw [hb'', hb', hmul, hm]
      simp only [ZMod.natCast_mod, Nat.cast_add, Nat.cast_mul]
      simpa [hcount, hd.2, ZMod.natCast_self, scalar] using hcombine (a.val : F) (b.val : F)
    rw [hcomb]
    rfl
termination_by it.chunks.length
decreasing_by simp_all

end Proofs
