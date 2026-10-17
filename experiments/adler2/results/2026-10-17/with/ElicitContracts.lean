import ElicitAdapter
import ElicitGenerated
open Aeneas Aeneas.Std Result ElicitGenerated
elicit_contract ElicitContracts.c_f321f3834d6c15c0 for ElicitGenerated.algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign requires (fun _ q => q.val ≠ 0) ensures (fun s q out => out.val.map (fun x => x.val) = s.val.map (fun x => x.val % q.val) ∧ out.val.length = 4 ∧ (∀ x ∈ out.val, x.val < q.val))
elicit_contract ElicitContracts.c_6c7496278e307fe7 for ElicitGenerated.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign requires (fun s r => ∀ x ∈ s.val, x.val * r.val < 4294967296) ensures (fun s r out => out.val.map (fun x => x.val) = s.val.map (fun x => x.val * r.val) ∧ out.val.length = 4)
elicit_contract ElicitContracts.c_20b5198a3591154d for ElicitGenerated.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign requires (fun s t => ∀ p ∈ s.val.zip t.val, p.1.val + p.2.val < 4294967296) ensures (fun s t out => out.val.map (fun x => x.val) = s.val.zipWith (fun x y => x.val + y.val) t.val ∧ out.val.length = 4)
