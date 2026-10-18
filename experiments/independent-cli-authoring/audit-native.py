#!/usr/bin/env python3
"""Rebuild hash-pinned, manually reviewed ordinary-arm proof artifacts.

The review JSON supplies module order, exact theorem names and model-generation
options. It never supplies new proof terms. Mathematical API coverage is reviewed
separately from kernel/axiom acceptance.
"""
import argparse,hashlib,json,os,pathlib,re,shutil,subprocess,time
p=argparse.ArgumentParser();p.add_argument('--root',type=pathlib.Path,required=True);p.add_argument('--review',type=pathlib.Path,required=True);p.add_argument('--out',type=pathlib.Path,required=True);a=p.parse_args();root=a.root.resolve();out=a.out.resolve();review=json.loads(a.review.read_text());out.mkdir(parents=True,exist_ok=False);commands=[];started=time.monotonic()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(argv,cwd,env=None):
 n=len(commands);t=time.monotonic();log=out/f'command-{n}.log'
 with log.open('w') as f:r=subprocess.run(list(map(str,argv)),cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
 commands.append({'argv':list(map(str,argv)),'exit':r.returncode,'elapsed_seconds':time.monotonic()-t,'log':log.name});(out/'commands.json').write_text(json.dumps(commands,indent=2)+'\n');return r.returncode,log
expected=review['expected_source_sha256']
assert {p:sha(root/p) for p in expected}==expected
for m in review['proof_modules']:assert sha(root/m['path'])==m['sha256'],'Proof changed after semantic review'
assert sha(root/review['generated_model'])==review['generated_sha256']
fresh=out/'fresh';fresh.mkdir();excludes=review.get('charon_excludes',[])
assert set(excludes)<={'adler2::adler32','adler2::{core::fmt::Debug for adler2::Adler32}'}
args=['charon','cargo','--preset=aeneas','--sysroot=default']
for e in excludes:args+=['--exclude',e]
for name in review.get('start_from',[]):args+=['--start-from',name]
args+=['--dest',fresh,'--','--manifest-path','Cargo.toml','--target','x86_64-unknown-linux-gnu','--lib']
env=os.environ.copy();env['CARGO_TARGET_DIR']=str(fresh/'target');assert run(args,root,env)[0]==0
args=['aeneas','-backend','lean','-dest',fresh,'-no-progress-bar']
if review.get('aeneas_namespace'):args+=['-namespace',review['aeneas_namespace']]
if review.get('use_lean_modules') is False:args+=['-use-lean-modules','false']
args+=[fresh/'adler2.llbc'];assert run(args,root)[0]==0
models=list(fresh.glob('*.lean'));assert len(models)==1
assert models[0].read_bytes()==(root/review['generated_model']).read_bytes(),'Fresh model differs from authoring model'
llbc=json.loads((fresh/'adler2.llbc').read_text());assert not llbc['has_errors'];tr=llbc['translated'];assert tr['runtime_checks']['overflow_checks']
for key in ['monomorphize','monomorphize_mut','skip_borrowck','no_typecheck']:assert not tr.get('options',{}).get(key)
for rel in ['src/algo.rs','src/lib.rs']:
 fs=[f for f in tr['files'] if f['name'].get('Local')==rel];assert len(fs)==1 and fs[0]['contents']==(root/rel).read_text()
for name in ['new','checksum','from_checksum','write_slice','adler32_slice']:
 fs=[f for f in tr['fun_decls'] if f and f['item_meta']['is_local'] and f['item_meta']['name'][-1].get('Ident',[None])[0]==name];assert len(fs)==1,name
 f=fs[0];assert f['item_meta']['opacity']=='Transparent' and not f['item_meta']['is_extern'] and not f['item_meta']['has_errors'] and 'Structured' in f['body']
module=review.get('generated_module','Adler2');dest=out/(module.replace('.','/')+'.lean');dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(models[0],dest)
for m in review['proof_modules']:
 dest=out/(m['module'].replace('.','/')+'.lean');dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(root/m['path'],dest)
backend=pathlib.Path(os.environ['AENEAS_HOME'])/'backends/lean';search=subprocess.check_output(['lake','env','printenv','LEAN_PATH'],cwd=backend,text=True).strip();env=os.environ.copy();env['LEAN_PATH']=str(out)+':'+search
builds={}
for name in [module]+[m['module'] for m in review['proof_modules']]:
 file=out/(name.replace('.','/')+'.lean');code,_=run(['lean','--root='+str(out),'-o',file.with_suffix('.olean'),file],out,env);builds[name]=code==0
imports=''.join('import '+m['module']+'\n' for m in review['proof_modules'] if builds.get(m['module']));allowed={'propext','Classical.choice','Quot.sound'};audits={}
for i,record in enumerate(review['theorem_roots']):
 name=record['name'] if isinstance(record,dict) else record
 root_imports='import '+record['module']+'\n' if isinstance(record,dict) else imports
 file=out/f'Audit_{i}.lean';file.write_text(root_imports+'\nset_option pp.fullNames true\n#check @'+name+'\n#print axioms '+name+'\n');code,log=run(['lean','--root='+str(out),file],out,env);text=log.read_text();axioms=None
 if code==0:
  match=re.search(re.escape("'"+name+"'")+r' depends on axioms: \[([^\]]*)\]',text)
  if match:axioms={x.strip() for x in match[1].split(',') if x.strip()}
  elif "'"+name+"' does not depend on any axioms" in text:axioms=set()
 audits[name]={'kernel_checked':code==0 and axioms is not None and axioms<=allowed,'axioms':sorted(axioms) if axioms is not None else None,'log':log.name}
file=out/'InspectDefinitions.lean';file.write_text(imports+''.join('#print '+n+'\n' for n in review.get('inspect_constants',[])));run(['lean','--root='+str(out),file],out,env)
for m in review['proof_modules']:assert sha(root/m['path'])==m['sha256'],'Authoring artifacts changed during audit'
summary={'elapsed_seconds':time.monotonic()-started,'fresh_model_byte_identical':True,'original_source_sha256':expected,'overflow_checks':True,'original_selected_bodies_transparent':True,'module_builds':builds,'theorem_audits':audits,'review':review,'coverage_note':'These kernel/axiom checks do not establish semantic API coverage by themselves. Read exact root types, inspected definitions and the separate investigator review.'};(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
