import Tail
open Aeneas Aeneas.Std Result
namespace AdlerProof
set_option maxHeartbeats 2000000

@[step] theorem vector_index (v : adler2.algo.U32X4) (j : Usize) (hj : j.val < 4) :
 Std.Array.index_usize v j ⦃ x => x.val = view v ⟨j.val,hj⟩ ⦄ := by
 apply WP.spec_mono (Std.Array.index_usize_spec v j (by simpa using hj))
 intro x hx
 simp only [hx,view]

@[simp] theorem view_set (v : adler2.algo.U32X4) (j : Usize) (x : U32) (i : Fin 4) :
 view (v.set j x) i = if j.val = i.val then x.val else view v i := by
 simp [view, List.getElem_set]
 split <;> simp_all

@[simp] theorem sumValues_array (v : adler2.algo.U32X4) : sumValues v.val = sum4 (view v) := by
 obtain ⟨a,b,c,d,rfl⟩ := vec_cases v
 simp [sumValues,vec,sum4,view]
 omega

def adjusted (av bv : Lanes) : Lanes := fun i => 4*bv i+i.val*(65521-av i)

theorem combination_identity (a b : Nat) (av bv : Lanes) (h : ∀ i, av i ≤ 65521) :
 (((a+sum4 av : Nat):R), ((b+sum4 (adjusted av bv):Nat):R)) = interp a b av bv := by
 have h0 := h 0; have h1 := h 1; have h2 := h 2; have h3 := h 3
 ext <;> simp [interp,sum4,adjusted,Nat.cast_sub h0,Nat.cast_sub h1,
   Nat.cast_sub h2,Nat.cast_sub h3] <;> ring

theorem sum4_bound (v : Lanes) (n : Nat) (h : ∀ i, v i ≤ n) : sum4 v ≤ 4*n := by
 have h0:=h 0; have h1:=h 1; have h2:=h 2; have h3:=h 3
 unfold sum4; omega

theorem adjusted_bound (av bv : Lanes) (ha : ∀ i, av i ≤ 65520)
 (hb : ∀ i, bv i ≤ 65520) : ∀ i, adjusted av bv i ≤ 458643 := by
 intro i
 have hi : i.val ≤ 3 := by omega
 have hb' := hb i
 have hm : i.val * (65521-av i) ≤ 3*65521 :=
   Nat.mul_le_mul hi (Nat.sub_le _ _)
 unfold adjusted
 omega

@[step] theorem array_slice (v : adler2.algo.U32X4) :
 lift (Std.Array.to_slice v) ⦃ s => s.val = v.val ⦄ := by simp [lift]

@[step] theorem slice_iter {α : Type} (s : Slice α) :
 core.slice.Slice.iter s ⦃ it => it = ⟨s,0⟩ ⦄ := by simp [core.slice.Slice.iter]

@[step] theorem cast_u16 (x : U32) (hx : x.val ≤ 65535) :
 lift (UScalar.cast .U16 x) ⦃ y => y.val = x.val ⦄ := by
 apply UScalar.cast_inBounds_spec
 simpa [UScalar.max,U16.max,U16.numBits] using hx
