import Specs
import ElicitAdapter
import ElicitGenerated
open Aeneas Aeneas.Std Result ElicitGenerated
elicit_contract ElicitContracts.c_8484fdd92cfcd3e7 for ElicitGenerated.adler32_slice requires (by intros; exact True) ensures (fun bytes out => out.val = 65536*(Verification.scalar (1,0) bytes.val).2 + (Verification.scalar (1,0) bytes.val).1)
elicit_contract ElicitContracts.c_0a059b8c0885cf07 for ElicitGenerated.Adler32.checksum requires (by intros; exact True) ensures (fun self out => out.val = 65536*self.b.val + self.a.val)
elicit_contract ElicitContracts.c_9a31b7b9cb8492b0 for ElicitGenerated.Adler32.from_checksum requires (by intros; exact True) ensures (fun sum out => out.a.val = sum.val % 65536 ∧ out.b.val = sum.val / 65536)
elicit_contract ElicitContracts.c_8055bec449b87b3d for ElicitGenerated.Adler32.new requires (by intros; exact True) ensures (fun out => out = { a := 1#u16, b := 0#u16 })
elicit_contract ElicitContracts.c_98f3436459f22f5d for ElicitGenerated.Adler32.write_slice requires (by intros; exact True) ensures (fun self bytes out => (out.a.val, out.b.val) = Verification.scalar (self.a.val, self.b.val) bytes.val)
