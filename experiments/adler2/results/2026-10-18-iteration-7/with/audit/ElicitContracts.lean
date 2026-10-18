import Specs
import ElicitAdapter
import ElicitGenerated
open Aeneas Aeneas.Std Result ElicitGenerated
elicit_contract ElicitContracts.c_8484fdd92cfcd3e7 for ElicitGenerated.adler32_slice requires (by intros; exact True) ensures (fun data out => out.val = AdlerSpec.packed (AdlerSpec.update 1 0 data.val))
elicit_contract ElicitContracts.c_0dfc1ce11e75ed00 for ElicitGenerated.algo.U32X4.from requires (fun bytes => 4 ≤ bytes.val.length) ensures (fun bytes out => out.val.map (fun x => x.val) = (bytes.val.take 4).map (fun x => x.val))
elicit_contract ElicitContracts.c_a058c96663922dc4 for ElicitGenerated.algo.Adler32.compute requires (by intros; exact True) ensures (fun self bytes out => (out.a.val, out.b.val) = AdlerSpec.update self.a.val self.b.val bytes.val)
elicit_contract ElicitContracts.c_0a059b8c0885cf07 for ElicitGenerated.Adler32.checksum requires (by intros; exact True) ensures (fun self out => out.val = 65536*self.b.val + self.a.val)
elicit_contract ElicitContracts.c_9a31b7b9cb8492b0 for ElicitGenerated.Adler32.from_checksum requires (by intros; exact True) ensures (fun sum out => out.a.val = sum.val % 65536 ∧ out.b.val = sum.val / 65536)
elicit_contract ElicitContracts.c_8055bec449b87b3d for ElicitGenerated.Adler32.new requires (by intros; exact True) ensures (fun out => out.a.val = 1 ∧ out.b.val = 0)
