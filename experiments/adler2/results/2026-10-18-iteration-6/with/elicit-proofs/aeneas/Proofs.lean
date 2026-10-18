import ElicitContracts
import Lemmas
open Aeneas Aeneas.Std ElicitGenerated
namespace AdlerProof
 theorem new_body : ∃ s, Adler32.new = Result.ok s ∧ s.a.val = 1 ∧ s.b.val = 0 := by
   exact ⟨⟨1#u16, 0#u16⟩, rfl, rfl, rfl⟩
end AdlerProof

namespace AdlerProof
 theorem vector_from_correct : ElicitContracts.c_0dfc1ce11e75ed00 := vector_from_contract
 theorem vector_add_correct : ElicitContracts.c_20b5198a3591154d := vector_add_contract
end AdlerProof
