import ElicitContracts
import Lemmas
open Aeneas Aeneas.Std Result ElicitGenerated
namespace AdlerProof
theorem new_correct : ElicitContracts.c_8055bec449b87b3d := by
  intro _
  refine ⟨{ a := 1#u16, b := 0#u16 }, rfl, ?_⟩
  decide
elicit_spec new_spec from new_correct


theorem checksum_correct : ElicitContracts.c_0a059b8c0885cf07 := by
  intro self _
  apply ElicitAdapter.spec_to_contract
  unfold Adler32.checksum
  simp only [lift]
  step* +scalarTac
  change (i1.bv ||| (core.convert.num.FromU32U16.from self.a).bv).toNat = _
  rw [BitVec.toNat_or]
  change i1.val ||| (core.convert.num.FromU32U16.from self.a).val = _
  simp only [core.convert.num.FromU32U16.from_val_eq] at *
  rw [i1_post]
  have hb : self.b.val < 65536 := by scalar_tac
  have ha : self.a.val < 65536 := by scalar_tac
  have hmod : self.b.val <<< 16 < U32.size := by
    simp only [Nat.shiftLeft_eq]
    scalar_tac
  rw [Nat.mod_eq_of_lt hmod, ← Nat.shiftLeft_add_eq_or_of_lt (by omega : self.a.val < 2^16)]
  simp [Nat.shiftLeft_eq, Nat.add_comm, Nat.mul_comm]

elicit_spec checksum_spec from checksum_correct
end AdlerProof

namespace AdlerProof
/-- The original convenience function follows from the complete arbitrary-input
update contract. This explicit implication is not a proof of that prerequisite. -/
theorem slice_of_update (update_contract : ElicitContracts.c_98f3436459f22f5d) :
    ElicitContracts.c_8484fdd92cfcd3e7 := by
  intro data _
  obtain ⟨seed, hseed, ha, hb⟩ := new_correct trivial
  obtain ⟨updated, hupdate, hstate⟩ := update_contract seed data trivial
  obtain ⟨out, hout, hpack⟩ := checksum_correct updated trivial
  refine ⟨out, ?_, ?_⟩
  · simp [adler32_slice, hseed, hupdate, hout]
  · have hstate' : (updated.a.val, updated.b.val) =
        data.val.foldl (fun (s : Nat × Nat) x =>
          let a := (s.1 + x.val) % 65521
          (a, (s.2 + a) % 65521)) (1, 0) := by
      simpa [AdlerSpec.run, ha, hb] using hstate
    have hfirst := congrArg Prod.fst hstate'
    have hsecond := congrArg Prod.snd hstate'
    simp only [Prod.fst, Prod.snd] at hfirst hsecond
    simpa only [hfirst, hsecond] using hpack
end AdlerProof

namespace AdlerProof
theorem from_correct : ElicitContracts.c_0dfc1ce11e75ed00 := by
  intro bytes h
  exact ElicitAdapter.spec_to_contract (from_wp bytes h)
elicit_spec from_spec from from_correct

theorem add_correct : ElicitContracts.c_20b5198a3591154d := by
  intro self other h
  exact ElicitAdapter.spec_to_contract (add_wp self other h)
elicit_spec add_spec from add_correct
end AdlerProof
