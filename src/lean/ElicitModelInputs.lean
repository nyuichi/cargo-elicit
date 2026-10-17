import ElicitContracts
import Lean
open Lean Elab Command

private def sourceModule (env : Environment) (declName : Name) : Name :=
  match env.getModuleIdxFor? declName with
  | some idx => env.allImportedModuleNames[idx.toNat]!
  | none => Name.anonymous

private partial def canonical (e : Expr) : Expr :=
  match e with
  | .app f a => .app (canonical f) (canonical a)
  | .lam _ t b bi => .lam Name.anonymous (canonical t) (canonical b) bi
  | .forallE _ t b bi => .forallE Name.anonymous (canonical t) (canonical b) bi
  | .letE _ t v b nondep => .letE Name.anonymous (canonical t) (canonical v) (canonical b) nondep
  | .mdata _ b => canonical b
  | .proj t i b => .proj t i (canonical b)
  | other => other

private def moduleDecls (env : Environment) (modName : Name) : Array Name := Id.run do
  for i in [:env.header.modules.size] do
    if env.header.modules[i]!.module == modName then
      return env.header.moduleData[i]!.constNames
  return #[]

private def declarationData (c : ConstantInfo) : Json :=
  let kind := match c with
    | .defnInfo _ => "definition"
    | .thmInfo _ => "theorem"
    | .opaqueInfo _ => "opaque"
    | .axiomInfo _ => "axiom"
    | _ => "unsupported"
  Json.arr #[toJson c.name.toString, toJson (reprStr (canonical c.type)),
    toJson (c.value? (allowOpaque := true) |>.map (reprStr ∘ canonical)),
    toJson (reprStr c.levelParams), toJson c.isUnsafe, toJson kind]

run_cmd do
  let env ← getEnv
  let mut supported := true
  let mut models : Array Json := #[]
  let mut modelTodo := (moduleDecls env `Specs).toList
  let mut modelSeen : NameSet := {}
  while let declName :: rest := modelTodo do
    modelTodo := rest
    if modelSeen.contains declName then continue
    modelSeen := modelSeen.insert declName
    if let some c := env.find? declName then
      if sourceModule env declName != `Specs then continue
      match c with
      | .defnInfo _ | .thmInfo _ | .opaqueInfo _ | .axiomInfo _ => pure ()
      | _ => supported := false
      if c.isUnsafe || c.isPartial then supported := false
      models := models.push (declarationData c)
      modelTodo := c.getUsedConstantsAsSet.toList ++ modelTodo
  let mut contracts : Array Json := #[]
  for declName in moduleDecls env `ElicitContracts do
    if let some c := env.find? declName then
     if declName.toString.startsWith "ElicitContracts.c_" then
       let mut todo := c.getUsedConstantsAsSet.toList
       let mut seen : NameSet := {}
       let mut inputs := #[declarationData c]
       while let current :: rest := todo do
         todo := rest
         if seen.contains current then continue
         seen := seen.insert current
         if let some info := env.find? current then
           if sourceModule env current == `Specs then
             inputs := inputs.push (declarationData info)
             todo := info.getUsedConstantsAsSet.toList ++ todo
       contracts := contracts.push (Json.arr #[toJson declName.toString, Json.arr inputs])
  logInfo ("ELICIT_MODEL_INPUTS=" ++ (Json.mkObj [("supported", toJson supported), ("models", Json.arr models), ("contracts", Json.arr contracts)]).compress)
