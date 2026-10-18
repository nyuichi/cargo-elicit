import Proofs
import Lean
open Lean Meta Elab Command Term

elab "elicit_audit " index:num proposition:str proofName:str : command => do
  let saved ← get
  let row ← try
    let auditName := Name.str `ElicitBatchAudit ("root_" ++ toString index.getNat)
    if (← getEnv).contains auditName then throwError "audit name collision"
    let proofName := proofName.getString.toName
    unless (← getEnv).contains proofName do throwError "missing proof"
    let propSyntax ← match Parser.runParserCategory (← getEnv) `term proposition.getString with
      | .ok propSyntax => pure propSyntax
      | .error error => throwError error
    let (type, value, levels) ← liftTermElabM <| withoutErrToSorry do
      let type ← elabType propSyntax
      unless ← isProp type do throwError "expected a proposition"
      let value ← elabTermEnsuringType (mkIdent proofName) type
      unless ← isDefEq (← inferType value) type do throwError "proof type mismatch"
      synthesizeSyntheticMVarsNoPostponing
      let type ← instantiateMVars type
      let type ← levelMVarToParam type
      let value ← instantiateMVars value
      if type.hasMVar || value.hasMVar || type.hasFVar || value.hasFVar then
        throwError "unresolved proof variables"
      pure (type, value, ← getLevelNames)
    liftCoreM <| addDecl <| .thmDecl {
      «name» := auditName
      levelParams := levels
      type := type
      value := value
    }
    let some (.thmInfo info) := (← getEnv).checked.get.find? auditName
      | throwError "kernel theorem missing"
    unless info.type == type do throwError "unexpected theorem type"
    let axioms ← collectAxioms auditName
    let allowed := axioms.all fun ax => ax == `propext || ax == `Classical.choice || ax == `Quot.sound
    pure <| Json.mkObj [("index", toJson index.getNat), ("checked", toJson allowed),
      ("axioms", toJson (axioms.map Name.toString))]
  catch _ =>
    pure <| Json.mkObj [("index", toJson index.getNat), ("checked", toJson false), ("axioms", Json.null)]
  set saved
  logInfo ("ELICIT_AUDIT=" ++ row.compress)

open Aeneas Aeneas.Std Result ElicitGenerated
elicit_audit 0 "ElicitContracts.c_8484fdd92cfcd3e7" "adler32_slice_correct"
elicit_audit 1 "ElicitContracts.c_0dfc1ce11e75ed00" "lane_from_correct"
elicit_audit 2 "ElicitContracts.c_a058c96663922dc4" "compute_correct"
elicit_audit 3 "ElicitContracts.c_0a059b8c0885cf07" "checksum_correct"
elicit_audit 4 "ElicitContracts.c_9a31b7b9cb8492b0" "from_checksum_correct"
elicit_audit 5 "ElicitContracts.c_8055bec449b87b3d" "new_correct"
