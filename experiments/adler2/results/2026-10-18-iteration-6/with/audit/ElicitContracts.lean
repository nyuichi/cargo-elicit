import Specs
import ElicitAdapter
import ElicitGenerated
open Aeneas Aeneas.Std Result ElicitGenerated
elicit_contract ElicitContracts.c_8484fdd92cfcd3e7 for ElicitGenerated.adler32_slice requires (by intros; exact True) ensures (fun data out => out.val = AdlerSpec.encode (AdlerSpec.run 1 0 data.val))
elicit_contract ElicitContracts.c_0dfc1ce11e75ed00 for ElicitGenerated.algo.U32X4.from requires (fun bytes => 4 ≤ bytes.length) ensures (fun bytes out => ∀ i : Fin 4, AdlerSpec.lane out i = bytes.val[i.val]!.val)
elicit_contract ElicitContracts.c_20b5198a3591154d for ElicitGenerated.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign requires (fun self other => ∀ i : Fin 4, AdlerSpec.lane self i + AdlerSpec.lane other i ≤ U32.max) ensures (fun self other out => ∀ i : Fin 4, AdlerSpec.lane out i = AdlerSpec.lane self i + AdlerSpec.lane other i)
elicit_contract ElicitContracts.c_a058c96663922dc4 for ElicitGenerated.algo.Adler32.compute requires (by intros; exact True) ensures (fun self bytes out => (out.a.val, out.b.val) = AdlerSpec.run self.a.val self.b.val bytes.val)
elicit_contract ElicitContracts.c_0a059b8c0885cf07 for ElicitGenerated.Adler32.checksum requires (by intros; exact True) ensures (fun self out => out.val = 65536 * self.b.val + self.a.val)
elicit_contract ElicitContracts.c_8055bec449b87b3d for ElicitGenerated.Adler32.new requires (by intros; exact True) ensures (fun out => out.a.val = 1 ∧ out.b.val = 0)
elicit_contract ElicitContracts.c_98f3436459f22f5d for ElicitGenerated.Adler32.write_slice requires (by intros; exact True) ensures (fun self bytes out => (out.a.val, out.b.val) = AdlerSpec.run self.a.val self.b.val bytes.val)
