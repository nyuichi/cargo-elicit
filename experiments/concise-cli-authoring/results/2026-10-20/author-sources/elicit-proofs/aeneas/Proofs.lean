import ElicitContracts
import API
import ExtraAudit
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Compute
theorem compute_spec : ElicitContracts.c_a058c96663922dc4 := by
  intro self bytes _
  exact ElicitAdapter.spec_to_contract (compute_correct self bytes)
end Compute
namespace Proofs
theorem new_spec : ElicitContracts.c_8055bec449b87b3d := by
  intro _
  exact ElicitAdapter.spec_to_contract Codec.new_spec
theorem checksum_spec : ElicitContracts.c_0a059b8c0885cf07 := by
  intro self _
  exact ElicitAdapter.spec_to_contract (Codec.checksum_spec self)
theorem from_checksum_spec : ElicitContracts.c_9a31b7b9cb8492b0 := by
  intro sum _
  exact ElicitAdapter.spec_to_contract (Codec.from_checksum_spec sum)
theorem write_slice_spec : ElicitContracts.c_98f3436459f22f5d := by
  intro self bytes _
  exact ElicitAdapter.spec_to_contract (API.write_correct self bytes)
theorem adler32_slice_spec : ElicitContracts.c_8484fdd92cfcd3e7 := by
  intro bytes _
  exact ElicitAdapter.spec_to_contract (API.slice_correct bytes)
end Proofs
