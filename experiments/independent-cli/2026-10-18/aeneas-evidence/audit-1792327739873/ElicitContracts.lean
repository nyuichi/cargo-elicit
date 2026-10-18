import Specs
import ElicitAdapter
import ElicitGenerated
open Aeneas Aeneas.Std Result ElicitGenerated
elicit_contract ElicitContracts.c_a944f8e645077fd5 for ElicitGenerated.identity requires (by intros; exact True) ensures (fun x result => result = FixtureModel.identity x)
