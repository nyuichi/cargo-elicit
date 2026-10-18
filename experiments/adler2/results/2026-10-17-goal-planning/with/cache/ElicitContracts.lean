import Specs
import ElicitAdapter
import ElicitGenerated
open Aeneas Aeneas.Std Result ElicitGenerated
elicit_contract ElicitContracts.c_8484fdd92cfcd3e7 for ElicitGenerated.adler32_slice requires (by intros; exact True) ensures (fun data out => let s := data.val.foldl (fun (s : Nat × Nat) x => let a := (s.1 + x.val) % 65521; (a, (s.2 + a) % 65521)) (1, 0); out.val = 65536 * s.2 + s.1)
elicit_contract ElicitContracts.c_0dfc1ce11e75ed00 for ElicitGenerated.algo.U32X4.from requires (fun bytes => 4 ≤ bytes.val.length) ensures (fun bytes out => out.val.map (fun x => x.val) = (bytes.val.take 4).map (fun x => x.val))
elicit_contract ElicitContracts.c_f321f3834d6c15c0 for ElicitGenerated.algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign requires (fun self quotient => quotient.val ≠ 0) ensures (fun self quotient out => ∀ i < 4, out.val[i]!.val = self.val[i]!.val % quotient.val)
elicit_contract ElicitContracts.c_6c7496278e307fe7 for ElicitGenerated.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign requires (fun self rhs => ∀ i < 4, self.val[i]!.val * rhs.val < 4294967296) ensures (fun self rhs out => ∀ i < 4, out.val[i]!.val = self.val[i]!.val * rhs.val)
elicit_contract ElicitContracts.c_20b5198a3591154d for ElicitGenerated.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign requires (fun self other => ∀ i < 4, self.val[i]!.val + other.val[i]!.val < 4294967296) ensures (fun self other out => ∀ i < 4, out.val[i]!.val = self.val[i]!.val + other.val[i]!.val)
elicit_contract ElicitContracts.c_a058c96663922dc4 for ElicitGenerated.algo.Adler32.compute requires (by intros; exact True) ensures (fun self bytes out => let s := AdlerModel.update self.a.val self.b.val bytes.val; out.a.val = s.1 ∧ out.b.val = s.2)
elicit_contract ElicitContracts.c_0a059b8c0885cf07 for ElicitGenerated.Adler32.checksum requires (by intros; exact True) ensures (fun self out => out.val = 65536 * self.b.val + self.a.val)
elicit_contract ElicitContracts.c_9a31b7b9cb8492b0 for ElicitGenerated.Adler32.from_checksum requires (by intros; exact True) ensures (fun sum out => out.a.val = sum.val % 65536 ∧ out.b.val = sum.val / 65536)
elicit_contract ElicitContracts.c_8055bec449b87b3d for ElicitGenerated.Adler32.new requires (by intros; exact True) ensures (fun out => out.a.val = 1 ∧ out.b.val = 0)
elicit_contract ElicitContracts.c_98f3436459f22f5d for ElicitGenerated.Adler32.write_slice requires (by intros; exact True) ensures (fun self bytes out => let s := AdlerModel.update self.a.val self.b.val bytes.val; out.a.val = s.1 ∧ out.b.val = s.2)
