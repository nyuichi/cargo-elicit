import Core
open Aeneas Aeneas.Std Result WP
namespace Verification
open adler2

@[step] theorem write_slice_correct (self : Adler32) (bytes : Slice U8) :
    self.write_slice bytes ⦃ s =>
      (s.a.val,s.b.val) = scalar (self.a.val,self.b.val) bytes.val ⦄ := by
  exact compute_correct self bytes

@[step] theorem adler32_slice_correct (bytes : Slice U8) :
    adler32_slice bytes ⦃ c =>
      c.val = 65536 * (scalar (1,0) bytes.val).2 + (scalar (1,0) bytes.val).1 ⦄ := by
  unfold adler32_slice
  rw [new_correct,bind_ok]
  step with write_slice_correct ⟨1#u16,0#u16⟩ bytes as ⟨s,hs⟩
  rw [checksum_exec]
  simp only [spec_ok]
  rw [encode_val]
  have hs1 := congrArg Prod.fst hs
  have hs2 := congrArg Prod.snd hs
  change s.a.val = (scalar (1,0) bytes.val).1 at hs1
  change s.b.val = (scalar (1,0) bytes.val).2 at hs2
  rw [hs1,hs2]
end Verification
