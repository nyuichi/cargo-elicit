import Compute
import Codec
open Aeneas Aeneas.Std Result ElicitGenerated
namespace API
@[step] theorem write_correct (self : Adler32) (bytes : Slice U8) :
    self.write_slice bytes ⦃ out =>
      AdlerSpec.state out = AdlerSpec.run (AdlerSpec.state self) bytes.val ⦄ := by
  exact Compute.compute_correct self bytes
@[step] theorem slice_correct (bytes : Slice U8) :
    adler32_slice bytes ⦃ out => out.val = AdlerSpec.pack (AdlerSpec.run (1, 0) bytes.val) ⦄ := by
  unfold adler32_slice
  step with Codec.new_spec as ⟨s, ha, hb⟩
  step with write_correct s bytes as ⟨t, ht⟩
  step with Codec.checksum_spec t as ⟨out, hout⟩
  have hs : AdlerSpec.state s = (1, 0) := Prod.ext ha hb
  rw [hs] at ht
  have h1 := congrArg Prod.fst ht
  have h2 := congrArg Prod.snd ht
  simp only [AdlerSpec.state] at h1 h2
  simp only [AdlerSpec.pack]
  omega
end API
