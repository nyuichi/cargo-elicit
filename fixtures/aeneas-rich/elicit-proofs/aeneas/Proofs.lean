import ElicitContracts
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Proofs

theorem new_spec : ElicitContracts.c_c77515a93681f6e2 := by
  intro _
  exact ⟨{ value := 0#u32 }, rfl, rfl⟩

theorem value_spec : ElicitContracts.c_8cdaa63cb55ad731 := by
  intro state _
  exact ⟨state.value, rfl, rfl⟩

theorem set_spec : ElicitContracts.c_33f76b115873a09e := by
  intro state value _
  exact ⟨{ value := value }, rfl, rfl⟩

theorem len_spec : ElicitContracts.c_d319ad4cf801c760 := by
  intro state bytes _
  exact ⟨bytes.len, rfl, rfl⟩

theorem roundtrip_spec : ElicitContracts.c_af80c86f2878b00e := by
  intro value _
  exact ⟨value, rfl, rfl⟩

end Proofs
