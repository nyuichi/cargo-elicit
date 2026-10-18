import Compute
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Proofs

elicit_spec new_wp from new_spec
elicit_spec checksum_wp from checksum_spec
elicit_spec from_checksum_wp from from_checksum_spec

theorem write_slice_spec : ElicitContracts.c_98f3436459f22f5d := by
  intro s bytes _
  apply ElicitAdapter.spec_to_contract
  exact compute_spec s bytes

elicit_spec write_slice_wp from write_slice_spec

theorem adler32_slice_spec : ElicitContracts.c_8484fdd92cfcd3e7 := by
  intro bytes _
  apply ElicitAdapter.spec_to_contract
  unfold adler32_slice
  step as ⟨s, ha, hb⟩
  step as ⟨t, ht⟩
  step as ⟨out, hout⟩
  rw [hout, ht]
  simp only [Specs.state, ha, hb]

elicit_spec adler32_slice_wp from adler32_slice_spec
end Proofs
