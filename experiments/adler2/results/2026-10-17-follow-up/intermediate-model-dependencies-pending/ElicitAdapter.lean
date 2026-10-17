import Aeneas
import Lean
open Lean Meta Elab Command Term

elab "elicit_contract " target:ident " for " original:ident " requires " pre:term " ensures " post:term : command => do
  let info ← getConstInfo original.getId
  let value ← liftTermElabM do
    let f := Lean.mkConst info.name (info.levelParams.map Level.param)
    forallTelescope info.type fun args resultType => do
      unless resultType.isAppOfArity ``Aeneas.Std.Result 1 do
        throwError "expected an Aeneas Result return type, got {resultType}"
      let outputType := resultType.getAppArgs[0]!
      withLocalDeclD `result outputType fun result => do
        let prop := mkSort Level.zero
        let preType ← mkForallFVars args prop
        let preValue ← Term.elabTermEnsuringType pre preType
        let postType ← mkForallFVars (args.push result) prop
        let postValue ← Term.elabTermEnsuringType post postType
        Term.synthesizeSyntheticMVarsNoPostponing
        let equal ← mkEq (mkAppN f args) (← mkAppM ``Aeneas.Std.Result.ok #[result])
        let both ← mkAppM ``And #[equal, mkAppN postValue (args.push result)]
        let existsResult ← mkAppM ``Exists #[← mkLambdaFVars #[result] both]
        let implication ← mkArrow (mkAppN preValue args) existsResult
        instantiateMVars (← mkForallFVars args implication)
  liftCoreM <| addAndCompile <| .defnDecl {
    «name» := target.getId
    levelParams := info.levelParams
    type := mkSort Level.zero
    value := value
    hints := .abbrev
    safety := .safe
  }
