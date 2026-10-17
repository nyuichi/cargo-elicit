import ElicitContracts

open Aeneas Aeneas.Std Result
set_option maxHeartbeats 1000000

namespace Adler32Proofs

-- This is only the representability invariant. Modulo normalization is an
-- update postcondition, not a precondition on states decoded from checksums.
theorem representable : ∀ s : ElicitGenerated.Adler32,
    s.a.val < 65536 ∧ s.b.val < 65536 := by
  intro s
  exact ⟨s.a.hmax, s.b.hmax⟩

end Adler32Proofs

namespace U32X4Proofs

-- Lane values may be unreduced partial sums. Their full machine range is
-- intentional; each arithmetic method still needs its own safety obligation.
theorem representable : ∀ s : ElicitGenerated.algo.U32X4,
    s.val.length = 4 ∧ ∀ x ∈ s.val, x.val < 4294967296 := by
  intro s
  constructor
  · exact s.property
  · intro x _
    exact x.hmax

end U32X4Proofs

namespace U32X4Proofs

-- Total-correctness proof of the original translated method. The fixed array
-- shape lets simplification identify the four updated lanes individually.
theorem remainder_body (s : ElicitGenerated.algo.U32X4) (q : U32) (hq : q.val ≠ 0) :
  ElicitGenerated.algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign s q ⦃ out =>
    out.val.map (fun x => x.val) = s.val.map (fun x => x.val % q.val) ∧
    out.val.length = 4 ∧ ∀ x ∈ out.val, x.val < q.val ⦄ := by
  obtain ⟨x0, x1, x2, x3, hs⟩ :=
    List.length_eq_four.mp (show s.val.length = 4 from s.property)
  unfold ElicitGenerated.algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
  step*
  all_goals simp_all
  all_goals simp_all [Nat.mod_lt _ (Nat.pos_of_ne_zero hq)]

-- Total correctness excludes failure and divergence, so its result can also
-- be exposed with an explicit successful-execution equation.
theorem result_of_spec {α : Type} {m : Result α} {p : α → Prop}
    (h : WP.spec m p) : ∃ out, m = .ok out ∧ p out := by
  cases m using Result.cases with
  | ret out => exact ⟨out, rfl, (WP.spec_ok out).mp h⟩
  | vis eff k => exact False.elim ((WP.spec_vis eff k).mp h)
  | div => exact False.elim (WP.spec_div.mp h)

-- Law of the actual original-source trait dictionary, with an explicit method
-- proof dependency. The ledger cannot currently map the trait_impl target.
theorem remainder_trait_law :
    ∀ (s : ElicitGenerated.algo.U32X4) (q : U32), q.val ≠ 0 → ∃ out,
      (ElicitGenerated.algo.U32X4.Insts.CoreOpsArithRemAssignU32).rem_assign s q = .ok out ∧
      out.val.map (fun x => x.val) = s.val.map (fun x => x.val % q.val) ∧
      out.val.length = 4 ∧ ∀ x ∈ out.val, x.val < q.val := by
  intro s q hq
  exact result_of_spec (remainder_body s q hq)

-- This statement is definitionally the generated remainder contract. Spelling
-- it out allows method-only wrapper runs without unrelated contract constants.
theorem remainder_method :
    ∀ (s : ElicitGenerated.algo.U32X4) (q : U32), q.val ≠ 0 → ∃ out,
      ElicitGenerated.algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign s q = .ok out ∧
      out.val.map (fun x => x.val) = s.val.map (fun x => x.val % q.val) ∧
      out.val.length = 4 ∧ ∀ x ∈ out.val, x.val < q.val :=
  remainder_trait_law

-- Every lane must fit the checked u32 multiplication. No reduction assumption
-- is required, and zero and one multipliers remain included.
theorem multiply_body (s : ElicitGenerated.algo.U32X4) (r : U32)
    (hsafe : ∀ x ∈ s.val, x.val * r.val < 4294967296) :
  ElicitGenerated.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign s r ⦃ out =>
    out.val.map (fun x => x.val) = s.val.map (fun x => x.val * r.val) ∧
    out.val.length = 4 ⦄ := by
  obtain ⟨x0, x1, x2, x3, hs⟩ :=
    List.length_eq_four.mp (show s.val.length = 4 from s.property)
  simp only [hs, List.mem_cons, forall_eq_or_imp] at hsafe
  unfold ElicitGenerated.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
  step*
  all_goals simp_all
  all_goals scalar_tac

theorem multiply_method :
    ∀ (s : ElicitGenerated.algo.U32X4) (r : U32),
      (∀ x ∈ s.val, x.val * r.val < 4294967296) → ∃ out,
      ElicitGenerated.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign s r = .ok out ∧
      out.val.map (fun x => x.val) = s.val.map (fun x => x.val * r.val) ∧
      out.val.length = 4 := by
  intro s r hsafe
  exact result_of_spec (multiply_body s r hsafe)

theorem multiply_trait_law :
    ∀ (s : ElicitGenerated.algo.U32X4) (r : U32),
      (∀ x ∈ s.val, x.val * r.val < 4294967296) → ∃ out,
      (ElicitGenerated.algo.U32X4.Insts.CoreOpsArithMulAssignU32).mul_assign s r = .ok out ∧
      out.val.map (fun x => x.val) = s.val.map (fun x => x.val * r.val) ∧
      out.val.length = 4 :=
  multiply_method

end U32X4Proofs

namespace U32X4Proofs

theorem add_body (s t : ElicitGenerated.algo.U32X4)
    (hsafe : ∀ p ∈ s.val.zip t.val, p.1.val + p.2.val < 4294967296) :
  ElicitGenerated.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign s t ⦃ out =>
    out.val.map (fun x => x.val) = s.val.zipWith (fun x y => x.val + y.val) t.val ∧
    out.val.length = 4 ⦄ := by
  obtain ⟨x0, x1, x2, x3, hs⟩ :=
    List.length_eq_four.mp (show s.val.length = 4 from s.property)
  obtain ⟨y0, y1, y2, y3, ht⟩ :=
    List.length_eq_four.mp (show t.val.length = 4 from t.property)
  simp [hs, ht] at hsafe
  unfold ElicitGenerated.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step*
  all_goals simp_all
  all_goals scalar_tac

-- Explicit successful-execution proposition, definitionally matching the
-- generated method contract for both original translated arguments.
theorem add_method :
    ∀ (s t : ElicitGenerated.algo.U32X4),
      (∀ p ∈ s.val.zip t.val, p.1.val + p.2.val < 4294967296) → ∃ out,
      ElicitGenerated.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign s t = .ok out ∧
      out.val.map (fun x => x.val) = s.val.zipWith (fun x y => x.val + y.val) t.val ∧
      out.val.length = 4 := by
  intro s t hsafe
  exact result_of_spec (add_body s t hsafe)

theorem add_trait_law :
    ∀ (s t : ElicitGenerated.algo.U32X4),
      (∀ p ∈ s.val.zip t.val, p.1.val + p.2.val < 4294967296) → ∃ out,
      (ElicitGenerated.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4).add_assign s t = .ok out ∧
      out.val.map (fun x => x.val) = s.val.zipWith (fun x y => x.val + y.val) t.val ∧
      out.val.length = 4 :=
  add_method

end U32X4Proofs
