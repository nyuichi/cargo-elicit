import Specs
import ElicitAdapter
import ElicitGenerated
open Aeneas Aeneas.Std Result ElicitGenerated
elicit_contract ElicitContracts.c_0dfc1ce11e75ed00 for ElicitGenerated.algo.U32X4.from requires (fun bytes => 4 ≤ bytes.val.length) ensures (fun bytes after => ∀ i (hi : i < after.val.length) (hj : i < bytes.val.length), after.val[i].val = bytes.val[i].val)
elicit_contract ElicitContracts.c_f321f3834d6c15c0 for ElicitGenerated.algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign requires (fun _ divisor => divisor.val ≠ 0) ensures (fun before divisor after => after.val.map (fun x => x.val) = before.val.map (fun x => x.val % divisor.val))
elicit_contract ElicitContracts.c_6c7496278e307fe7 for ElicitGenerated.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign requires (fun before rhs => ∀ lane ∈ before.val, lane.val * rhs.val ≤ 4294967295) ensures (fun before rhs after => after.val.map (fun x => x.val) = before.val.map (fun x => x.val * rhs.val))
elicit_contract ElicitContracts.c_20b5198a3591154d for ElicitGenerated.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign requires (fun before other => ∀ i (hi : i < before.val.length) (hj : i < other.val.length), before.val[i].val + other.val[i].val ≤ 4294967295) ensures (fun before other after => after.val.map (fun x => x.val) = List.zipWith (fun x y => x.val + y.val) before.val other.val)
elicit_contract ElicitContracts.c_a058c96663922dc4 for ElicitGenerated.algo.Adler32.compute requires (by intros; exact True) ensures (fun before bytes after => (after.a.val, after.b.val) = AdlerSpec.ScalarAdler (before.a.val, before.b.val) bytes.val ∧ AdlerSpec.Normalized after.a.val after.b.val)
elicit_contract ElicitContracts.c_2fa15a7d1ba5a684 for ElicitGenerated.Adler32.Insts.CoreDefaultDefault.default requires (by intros; exact True) ensures (fun after => after.a.val = 1 ∧ after.b.val = 0)
elicit_contract ElicitContracts.c_375e1cdaf1f944d2 for ElicitGenerated.Adler32.Insts.CoreHashHasher.finish requires (by intros; exact True) ensures (fun before result => result.val = 65536 * before.b.val + before.a.val)
elicit_contract ElicitContracts.c_0a059b8c0885cf07 for ElicitGenerated.Adler32.checksum requires (by intros; exact True) ensures (fun before result => result.val = 65536 * before.b.val + before.a.val)
elicit_contract ElicitContracts.c_9a31b7b9cb8492b0 for ElicitGenerated.Adler32.from_checksum requires (by intros; exact True) ensures (fun sum after => after.a.val = sum.val % 65536 ∧ after.b.val = sum.val / 65536)
elicit_contract ElicitContracts.c_8055bec449b87b3d for ElicitGenerated.Adler32.new requires (by intros; exact True) ensures (fun after => after.a.val = 1 ∧ after.b.val = 0)
