use crate::{analysis, fingerprint,model::*,process,store::Store};
use anyhow::{Context,Result,bail};
use serde_json::Value;
use std::{collections::BTreeSet,fs,path::{Path,PathBuf},process::Command,time::{Duration,Instant}};
use quote::ToTokens;
use syn::{spanned::Spanned,visit::Visit};

pub fn syntax(store:&Store,s:&State,seconds:u64)->Result<String>{
    let version=fingerprint::tool_version(s.tool)?;
    let before=fingerprint::input(&s.scope,s.tool,&version)?;
    let cache=store.root.join(".elicit/cache");fs::create_dir_all(&cache)?;
    let temp=tempfile::tempdir_in(&cache)?;
    let mut cmd=compile_command(s,temp.path(),false,&version)?;cmd.current_dir(&store.root);
    let o=process::run(&mut cmd,Duration::from_secs(seconds))?;
    if o.interrupted {bail!("interrupted");}if o.timeout {bail!("syntax/type processing timed out");}
    if o.code!=Some(0){bail!("verifier could not process the current source (exit {:?}); diagnostics were not persisted to avoid recording secrets",o.code);}
    if fingerprint::input(&s.scope,s.tool,&version)?!=before{bail!("input changed during syntax/type processing; retry on stable sources");}
    Ok(before)
}
fn compile_command(s:&State,out:&Path,prove:bool,version:&str)->Result<Command>{
    let mut c=Command::new("cargo");
    match s.tool{
        Tool::Kani=>{c.args(["kani","-Z","unstable-options","-Z","function-contracts"]);if prove{c.arg("--export-json").arg(out.join("kani.json"));}else{c.arg("--only-codegen");}c.args(s.scope.cargo_args());},
        Tool::Creusot=>{c.args(["creusot","--only","coma","--output-dir"]).arg(out).arg("--erasure-check-dir").arg(out.join("erasure")).arg("--").args(s.scope.cargo_args());c.env("CARGO_TARGET_DIR",Path::new(&s.scope.manifest_path).parent().unwrap().join(".elicit/cache/creusot-build"));let _=version;}
    }Ok(c)
}
fn base(s:&State,id:&str,version:&str)->Verification{
    Verification{id:format!("{}-{}-{}",s.revision,id,s.results.len()),target:id.into(),units:vec!["contract".into()],obligations:vec![],scope:Coverage::Unknown,assumptions:vec![],restrictions:vec![],input:s.input.clone(),tool:s.tool,tool_version:version.into(),trusted_base:vec![],reusable:false,outcome:Outcome::Interrupted,summary:"verification started, not finalized".into(),elapsed_ms:0}
}
pub fn verify(store:&Store,ids:&[String],revision:Option<u64>,seconds:u64)->Result<Vec<Verification>>{
    // A live worker holds this lock until its turn has stopped. Official proof
    // collection cannot race the next worker turn, including standalone CLI use.
    let _worker=store.lock("worker")?;let _verify=store.lock("verification")?;
    let mut s=store.load()?;if revision.is_some_and(|r|r!=s.revision){bail!("revision conflict");}
    for id in ids{if !s.targets.contains_key(id){bail!("unknown target {id}");}}
    let version=fingerprint::tool_version(s.tool)?;
    let before=fingerprint::input(&s.scope,s.tool,&version)?;
    if before!=s.input{bail!("source or verifier inputs changed; reconcile with status before verifying");}
    let mut pending=vec![];for id in ids{pending.push(base(&s,id,&version));}
    store.transaction(Some(s.revision),|state|{state.results.extend(pending.clone());Ok(())})?;
    s=store.load()?;
    let started=Instant::now();let cache=store.root.join(".elicit/cache");fs::create_dir_all(&cache)?;
    let dir=tempfile::Builder::new().prefix("verification-").tempdir_in(&cache)?;
    let execute=(||->Result<(process::Output,Option<Value>)>{
        if s.tool==Tool::Creusot{
            // Force fresh compiler output in our own build cache, never clean user artifacts.
            let mut clean=Command::new("cargo");
            if let Some(tc)=version.split_whitespace().find(|s|s.starts_with("nightly-")){clean.arg(format!("+{}",tc.trim_matches(['(',')',','])));}
            clean.args(["clean","--package",&s.scope.package,"--manifest-path",&s.scope.manifest_path]).env("CARGO_TARGET_DIR",Path::new(&s.scope.manifest_path).parent().unwrap().join(".elicit/cache/creusot-build")).current_dir(&store.root);
            let o=process::run(&mut clean,Duration::from_secs(seconds))?;
            if o.code!=Some(0){return Ok((o,None));}
        }
        let mut cmd=compile_command(&s,dir.path(),true,&version)?;cmd.current_dir(&store.root);
        let mut o=process::run(&mut cmd,Duration::from_secs(seconds.saturating_sub(started.elapsed().as_secs())))?;
        let evidence=match s.tool{
            Tool::Kani=>fs::read(dir.path().join("kani.json")).ok().and_then(|b|serde_json::from_slice(&b).ok()),
            Tool::Creusot=>{
                if o.code==Some(0){let mut cmd=why3_command(&store.root,dir.path())?;o=process::run(&mut cmd,Duration::from_secs(seconds.saturating_sub(started.elapsed().as_secs())))?;}None
            }
        };Ok((o,evidence))
    })();
    let after=fingerprint::input(&s.scope,s.tool,&version);
    let mut results=vec![];
    for p in pending{
        let mut r=p;
        match &execute{
            Err(_)=>{r.outcome=Outcome::EnvironmentError;r.summary="verifier setup or execution failed; run the verifier directly for diagnostics".into();},
            Ok((o,evidence))=>{
                r.elapsed_ms=started.elapsed().as_millis() as u64;
                if o.interrupted{r.outcome=Outcome::Interrupted;r.summary="verification interrupted".into();}
                else if o.timeout{r.outcome=Outcome::Timeout;r.summary="verification deadline exceeded".into();}
                else if o.code!=Some(0){r.outcome=if evidence.is_some(){Outcome::Unresolved}else{Outcome::EnvironmentError};r.summary=format!("verifier exited {:?}; no complete proof accepted",o.code);}
                else{r.outcome=Outcome::UnknownScope;r.summary="tool completed; contract coverage not established".into();match s.tool{Tool::Kani=>if let Some(e)=evidence{kani_evidence(&s,&s.targets[&r.target],e,&mut r)?;},Tool::Creusot=>creusot_evidence(&s,&s.targets[&r.target],dir.path(),&mut r)?,}}
            }
        }
        if after.as_ref().ok()!=Some(&before){r.outcome=Outcome::InputChanged;r.reusable=false;r.summary="verification inputs changed during execution; result is not current".into();}
        results.push(r);
    }
    // Keep only a typed, bounded summary in the ledger. Native tool output can
    // contain source excerpts or environment secrets and is deliberately not logged.
    store.transaction(None,|state|{for r in &results{if let Some(old)=state.results.iter_mut().find(|p|p.id==r.id){*old=r.clone();}}Ok(())})?;
    Ok(results)
}
fn safe_project(s:&State)->Result<bool>{
    let manifest:toml::Value=toml::from_str(&fs::read_to_string(&s.scope.manifest_path)?)?;
    let root=Path::new(&s.scope.manifest_path).parent().unwrap();
    if root.join("build.rs").exists() || manifest["package"].get("build").is_some() || manifest.get("lib").and_then(|v|v.get("proc-macro")).and_then(|v|v.as_bool())==Some(true){return Ok(false);}
    let allowed=match s.tool{Tool::Kani=>vec![],Tool::Creusot=>vec!["creusot-std"]};
    for table in ["dependencies","dev-dependencies","build-dependencies"] {if manifest.get(table).and_then(|v|v.as_table()).is_some_and(|t|t.keys().any(|k|!allowed.contains(&k.as_str()))){return Ok(false);}}
    if manifest.get("target").is_some() || manifest.get("patch").is_some() || manifest.get("replace").is_some(){return Ok(false);}
    if manifest.get("package").and_then(|v|v.get("metadata")).is_some(){return Ok(false);}
    Ok(true)
}
fn original_fn(t:&Target)->Result<syn::ItemFn>{
    // Find by the original definition location, not a duplicate function name.
    let f=syn::parse_file(&fs::read_to_string(&t.file)?)?;
    fn find(items:&[syn::Item],t:&Target)->Option<syn::ItemFn>{for i in items{match i{syn::Item::Fn(f) if f.sig.ident.to_string()==t.name && f.sig.ident.span().start().line==t.line=>return Some(f.clone()),syn::Item::Mod(m)=>if let Some((_,is))=&m.content{if let Some(f)=find(is,t){return Some(f);}},_=>{}}}None}
    find(&f.items,t).context("original free function not found")
}
fn attr_text(attrs:&[syn::Attribute])->String{attrs.iter().map(|a|a.to_token_stream().to_string()).collect::<Vec<_>>().join(" ")}
fn contract_closed(f:&syn::ItemFn,tool:Tool)->bool{
    let mut names=BTreeSet::new();for i in &f.sig.inputs{if let syn::FnArg::Typed(p)=i{if let syn::Pat::Ident(i)=p.pat.as_ref(){names.insert(i.ident.to_string());}}}names.insert("result".into());
    // Parse each contract's expression. Paths must be bound primitives; custom
    // models, macros, trait calls and implicit external input remain unknown.
    fn expression(e:&syn::Expr,names:&BTreeSet<String>)->bool{struct Check<'a>{names:&'a BTreeSet<String>,bad:bool}impl<'ast> Visit<'ast> for Check<'_>{fn visit_expr(&mut self,e:&'ast syn::Expr){match e{syn::Expr::Path(p) if p.path.segments.len()==1 && self.names.contains(&p.path.segments[0].ident.to_string())=>{},syn::Expr::Lit(_)|syn::Expr::Unary(_)|syn::Expr::Binary(_)|syn::Expr::Paren(_)|syn::Expr::Group(_)=>{},_=>self.bad=true}syn::visit::visit_expr(self,e);}}
        let mut c=Check{names,bad:false};c.visit_expr(e);!c.bad}
    let mut count=0;
    for a in &f.attrs{
        let m=if a.path().is_ident("cfg_attr"){
            use syn::parse::Parser;let syn::Meta::List(l)=&a.meta else{return false};let Ok(ms)=syn::punctuated::Punctuated::<syn::Meta,syn::Token![,]>::parse_terminated.parse2(l.tokens.clone())else{return false};if ms.len()!=2 || !matches!(&ms[0],syn::Meta::Path(p) if p.is_ident(&tool.to_string())){return false;}ms[1].clone()
        }else{a.meta.clone()};
        let name=m.path().segments.last().unwrap().ident.to_string();
        if name=="doc"{continue;} if !matches!(name.as_str(),"requires"|"ensures"){return false;}
        let syn::Meta::List(l)=m else{return false};let Ok(mut e)=syn::parse2::<syn::Expr>(l.tokens)else{return false};
        let mut bound=names.clone();if tool==Tool::Kani && name=="ensures"{if let syn::Expr::Closure(c)=e{if c.inputs.len()!=1{return false;}if let syn::Pat::Ident(i)=&c.inputs[0]{bound.insert(i.ident.to_string());}else{return false;}e=*c.body;}else{return false;}}
        if !expression(&e,&bound){return false;}if name=="ensures"{count+=1;}
    }
    count>0
}
fn kani_evidence(s:&State,t:&Target,e:&Value,r:&mut Verification)->Result<()>{
    // Populated from the pinned native export schema; no textual SUCCESS scan.
    r.trusted_base=vec!["Kani 0.68 compiler and contract transformation".into(),"CBMC bit-vector logic and standard Rust/Kani models".into()];
    let Some(harnesses)=e.get("harnesses").and_then(Value::as_array) else{return Ok(())};
    for h in harnesses{
        let text=h.to_string();if !text.contains(&t.name){continue;}
        r.obligations.push("native harness result (coverage pending)".into());
    }
    // Additional backend validation is defined below once the export is mapped.
    let _=(s,t);Ok(())
}
fn creusot_home()->Result<PathBuf>{
    if let Some(p)=std::env::var_os("CREUSOT_DATA_HOME"){return Ok(PathBuf::from(p));}
    if let Some(p)=std::env::var_os("XDG_DATA_HOME"){return Ok(PathBuf::from(p).join("creusot"));}
    Ok(PathBuf::from(std::env::var_os("HOME").context("HOME missing; set CREUSOT_DATA_HOME")?).join(".local/share/creusot"))
}
fn why3_command(root:&Path,out:&Path)->Result<Command>{
    if !root.join("why3find.json").is_file(){bail!("why3find.json required in package root; configure standard creusot package and provers");}
    let home=creusot_home()?;let mut c=Command::new(home.join("bin/why3find"));c.arg("prove");
    for entry in walkdir::WalkDir::new(out){let e=entry?;if e.path().extension().is_some_and(|e|e=="coma"){c.arg(e.path());}}
    c.current_dir(root).env("WHY3CONFIG",home.join("why3.conf")).env("DUNE_DIR_LOCATIONS",format!("{}",home.join("share/why3find").display()));Ok(c)
}
fn creusot_evidence(s:&State,t:&Target,out:&Path,r:&mut Verification)->Result<()>{
    let _=(s,t,out);r.trusted_base=vec!["Creusot 0.14 compiler and standard models".into(),"Why3 and configured SMT provers".into()];Ok(())
}
