import Compute
import Specification
open Aeneas Aeneas.Std Result
namespace AdlerProof
set_option maxHeartbeats 2000000

@[step] theorem new_spec : adler2.Adler32.new ⦃ s => s.a = 1#u16 ∧ s.b = 0#u16 ⦄ := by
 simp [new_correct]

@[step] theorem checksum_spec (s : adler2.Adler32) : adler2.Adler32.checksum s
 ⦃ c => c.val = 65536*s.b.val+s.a.val ⦄ := by
 obtain ⟨c,hc,hv⟩ := checksum_correct s
 simp [hc,hv]

@[step] theorem from_checksum_spec (c : U32) : adler2.Adler32.from_checksum c
 ⦃ s => s.a.val = c.val % 65536 ∧ s.b.val = c.val / 65536 ⦄ := by
 obtain ⟨s,hs,ha,hb⟩ := from_checksum_correct c
 simp [hs,ha,hb]

@[step] theorem write_slice_correct (s : adler2.Adler32) (bytes : Slice U8) :
 adler2.Adler32.write_slice s bytes ⦃ out =>
 (out.a.val,out.b.val) = scalar s.a.val s.b.val bytes.val ⦄ := by
 unfold adler2.Adler32.write_slice
 apply WP.spec_mono (compute_correct_ring s bytes)
 intro out ⟨ha,hb,hr⟩
 have hs := scalar_bounds s.a.val s.b.val bytes.val
 apply cast_injective_normalized _ _ _ _ ha hb hs.1 hs.2
 rw [hr,scalar_cast]

@[step] theorem adler32_slice_correct (bytes : Slice U8) :
 adler2.adler32_slice bytes ⦃ c =>
 c.val = 65536*(scalar 1 0 bytes.val).2 + (scalar 1 0 bytes.val).1 ⦄ := by
 unfold adler2.adler32_slice
 step as ⟨s,ha,hb⟩
 step as ⟨out,hout⟩
 step as ⟨c,hc⟩
 have heq : (out.a.val,out.b.val) = scalar 1 0 bytes.val := by simpa [ha,hb] using hout
 have h0 := congrArg Prod.fst heq
 have h1 := congrArg Prod.snd heq
 change out.a.val = (scalar 1 0 bytes.val).1 at h0
 change out.b.val = (scalar 1 0 bytes.val).2 at h1
 simpa only [h0,h1] using hc

theorem write_slice_empty (s : adler2.Adler32) :
 adler2.Adler32.write_slice s (Slice.from [] (by simp)) ⦃ out =>
 out.a.val = s.a.val % 65521 ∧ out.b.val = s.b.val % 65521 ⦄ := by
 apply WP.spec_mono (write_slice_correct s _)
 intro out h
 simpa [scalar] using h
