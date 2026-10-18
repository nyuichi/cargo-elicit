import APIs
import ElicitContracts
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Replay
theorem new : ElicitContracts.c_8055bec449b87b3d := by
  intro _
  exact ⟨_, Verification.new_correct, rfl⟩
theorem checksum : ElicitContracts.c_0a059b8c0885cf07 := by
  intro self _
  exact ElicitAdapter.spec_to_contract (Verification.checksum_correct self)
theorem from_checksum : ElicitContracts.c_9a31b7b9cb8492b0 := by
  intro sum _
  exact ElicitAdapter.spec_to_contract (Verification.from_checksum_correct sum)
theorem write_slice : ElicitContracts.c_98f3436459f22f5d := by
  intro self bytes _
  exact ElicitAdapter.spec_to_contract (Verification.write_slice_correct self bytes)
theorem adler32_slice : ElicitContracts.c_8484fdd92cfcd3e7 := by
  intro bytes _
  exact ElicitAdapter.spec_to_contract (Verification.adler32_slice_correct bytes)
end Replay
