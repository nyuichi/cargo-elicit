import Specs
import ElicitAdapter
import ElicitGenerated
open Aeneas Aeneas.Std Result ElicitGenerated
elicit_contract ElicitContracts.c_8484fdd92cfcd3e7 for ElicitGenerated.adler32_slice requires (by intros; exact True) ensures (fun data out => out.val = ((data.val.foldl (fun s x => let a := (s.1 + x.val) % 65521; (a, (s.2 + a) % 65521)) (1, 0)).2 * 65536 + (data.val.foldl (fun s x => let a := (s.1 + x.val) % 65521; (a, (s.2 + a) % 65521)) (1, 0)).1))
elicit_contract ElicitContracts.c_0a059b8c0885cf07 for ElicitGenerated.Adler32.checksum requires (by intros; exact True) ensures (fun self out => out.val = 65536 * self.b.val + self.a.val)
elicit_contract ElicitContracts.c_8055bec449b87b3d for ElicitGenerated.Adler32.new requires (by intros; exact True) ensures (fun out => out.a.val = 1 ∧ out.b.val = 0)
elicit_contract ElicitContracts.c_98f3436459f22f5d for ElicitGenerated.Adler32.write_slice requires (by intros; exact True) ensures (fun self bytes out => (out.a.val, out.b.val) = AdlerSpec.update (self.a.val, self.b.val) bytes.val)
