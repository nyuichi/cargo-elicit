import Adler2
set_option linter.unusedSimpArgs false
set_option linter.unusedTactic false
set_option linter.unnecessarySeqFocus false
open Aeneas Aeneas.Std Result
namespace Verification
open adler2

theorem new_correct : Adler32.new = ok ⟨1#u16, 0#u16⟩ := by rfl

theorem from_checksum_correct (s : U32) :
    Adler32.from_checksum s ⦃ h =>
      h.a.val = s.val % 65536 ∧ h.b.val = s.val / 65536 ⦄ := by
  unfold Adler32.from_checksum
  simp only [lift, bind_tc_ok]
  step
  simp only [lift, bind_tc_ok, WP.spec_ok, UScalar.cast_val_eq]
  simp_all [Nat.shiftRight_eq_div_pow]
  scalar_tac

theorem checksum_correct (h : Adler32) :
    Adler32.checksum h ⦃ s => s.val = 65536 * h.b.val + h.a.val ⦄ := by
  unfold Adler32.checksum
  simp only [lift, bind_tc_ok]
  step
  simp only [WP.spec_ok, UScalar.val_or]
  simp_all [Nat.shiftLeft_eq]
  have hb : h.b.val * 65536 < U32.size := by scalar_tac
  rw [Nat.mod_eq_of_lt hb]
  have ha : h.a.val < 2^16 := by scalar_tac
  have hor := Nat.shiftLeft_add_eq_or_of_lt ha h.b.val
  simpa [Nat.shiftLeft_eq, Nat.mul_comm] using hor.symm

attribute [step] from_checksum_correct checksum_correct

theorem decode_encode (h : Adler32) :
    (do let s ← h.checksum; Adler32.from_checksum s) ⦃ h' => h' = h ⦄ := by
  step
  step
  have ha : h.a.val < 65536 := by scalar_tac
  have hb : h.b.val < 65536 := by scalar_tac
  have : h'.a = h.a := by apply UScalar.eq_of_val_eq; omega
  have : h'.b = h.b := by apply UScalar.eq_of_val_eq; omega
  cases h; cases h'; simp_all

theorem encode_decode (s : U32) :
    (do let h ← Adler32.from_checksum s; h.checksum) ⦃ s' => s' = s ⦄ := by
  step
  step
  apply UScalar.eq_of_val_eq
  omega

#print axioms new_correct
#print axioms checksum_correct
#print axioms from_checksum_correct
#print axioms decode_encode
#print axioms encode_decode

def vals (v : algo.U32X4) : List Nat := v.val.map UScalar.val

theorem vector_add (v w : algo.U32X4)
    (hb : ∀ j (hj : j < 4), (v.val[j]'(by simpa using hj)).val + (w.val[j]'(by simpa using hj)).val ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign v w
      ⦃ r => vals r = List.zipWith (· + ·) (vals v) (vals w) ⦄ := by
  have h0 := hb 0 (by omega)
  have h1 := hb 1 (by omega)
  have h2 := hb 2 (by omega)
  have h3 := hb 3 (by omega)
  unfold algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  simp_all [vals]
  apply List.ext_getElem
  · simp
  · intro j hj hj
    have hj4 : j < 4 := by simpa using hj
    have cases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
    rcases cases with rfl | rfl | rfl | rfl <;> simp_all [List.getElem_set]


theorem vector_rem (v : algo.U32X4) (m : U32) (hm : m.val ≠ 0) :
    algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign v m
      ⦃ r => vals r = (vals v).map (· % m.val) ⦄ := by
  unfold algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  simp_all [vals]
  apply List.ext_getElem
  · simp
  · intro j hj hj
    have hj4 : j < 4 := by simpa using hj
    have cases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
    rcases cases with rfl | rfl | rfl | rfl <;> simp_all [List.getElem_set]

theorem vector_mul (v : algo.U32X4) (m : U32)
    (hb : ∀ j (hj : j < 4),
      (v.val[j]'(by simpa using hj)).val * m.val ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign v m
      ⦃ r => vals r = (vals v).map (· * m.val) ⦄ := by
  have h0 := hb 0 (by omega)
  have h1 := hb 1 (by omega)
  have h2 := hb 2 (by omega)
  have h3 := hb 3 (by omega)
  unfold algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  step <;> try (solve | simp_all [Array.set, List.getElem_set] | scalar_tac)
  simp_all [vals]
  apply List.ext_getElem
  · simp
  · intro j hj hj
    have hj4 : j < 4 := by simpa using hj
    have cases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
    rcases cases with rfl | rfl | rfl | rfl <;> simp_all [List.getElem_set]

theorem vector_from (bytes : Slice U8) (hb : bytes.val.length = 4) :
    algo.U32X4.from bytes ⦃ r => vals r = bytes.val.map UScalar.val ⦄ := by
  unfold algo.U32X4.from
  simp only [lift]
  step
  step
  step
  step
  simp_all [vals]
  apply List.ext_getElem
  · simp_all
  · intro j hj hj
    have hj4 : j < 4 := by simpa [hb] using hj
    have cases : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
    rcases cases with rfl | rfl | rfl | rfl <;> simp_all

#print axioms vector_add
#print axioms vector_rem
#print axioms vector_mul
#print axioms vector_from

