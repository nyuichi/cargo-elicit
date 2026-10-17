import Proofs

-- Non-scored audit observation: reuse an existing proof constant without a new proof script.
elicit_contract AuditNewGoal for ElicitGenerated.Adler32.new requires (by intros; exact True) ensures (fun (out : ElicitGenerated.Adler32) => out.a.val = 1 ∧ out.b.val = 0)
theorem AuditDefaultAsNew : AuditNewGoal := AdlerProof.default_contract
#print AuditNewGoal
#print axioms AuditDefaultAsNew
