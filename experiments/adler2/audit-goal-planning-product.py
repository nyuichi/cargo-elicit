#!/usr/bin/env python3
"""Independent fresh extraction, kernel rebuild, API and checkpoint audits.

A kernel check here is not automatic acceptance of a user's natural-language goal.
Review the emitted exact contracts and propositions before assigning coverage.
"""
import argparse,hashlib,json,os,pathlib,re,shutil,subprocess,time,tomllib
p=argparse.ArgumentParser();p.add_argument('--root',required=True,type=pathlib.Path);p.add_argument('--cache',required=True,type=pathlib.Path);p.add_argument('--plan',required=True,type=pathlib.Path);p.add_argument('--out',required=True,type=pathlib.Path);a=p.parse_args()
root=a.root.resolve();cache=a.cache.resolve();out=a.out.resolve();out.mkdir(parents=True,exist_ok=False)
state=tomllib.loads(a.plan.read_text());scope=state['scope'];commands=[];tick=time.monotonic()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
proofroot=root/'elicit-proofs/aeneas';frozen={str(p.relative_to(root)):sha(p) for p in proofroot.rglob('*') if p.is_file()}
def run(argv,cwd,env=None):
 n=len(commands);t=time.monotonic();log=out/f'command-{n}.log'
 with log.open('w') as f:r=subprocess.run(list(map(str,argv)),cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
 commands.append({'argv':list(map(str,argv)),'exit':r.returncode,'elapsed_seconds':time.monotonic()-t,'log':log.name});(out/'commands.json').write_text(json.dumps(commands,indent=2)+'\n');return r.returncode,log
# Re-extract without using the product wrapper. Fresh model correspondence is
# checked separately from proof compilation and plan/ledger claims.
fresh=out/'fresh';fresh.mkdir();cargo=['--manifest-path',root/'Cargo.toml','--package',scope['package'],'--target',scope['target']]
if scope.get('all_features'):cargo+=['--all-features']
if scope.get('no_default_features'):cargo+=['--no-default-features']
if scope.get('features'):cargo+=['--features',','.join(scope['features'])]
env=os.environ.copy();env['CARGO_TARGET_DIR']=str(fresh/'target')
code,_=run(['charon','cargo','--preset=aeneas','--sysroot=default','--format=json','--no-dedup-serialized-ast','--dest-file',fresh/'crate.llbc','--']+cargo,root,env);assert code==0
code,_=run(['aeneas','-backend','lean','-namespace','ElicitGenerated','-dest',fresh,fresh/'crate.llbc'],root);assert code==0
models=list(fresh.glob('*.lean'));assert len(models)==1
assert models[0].read_bytes()==(cache/'ElicitGenerated.lean').read_bytes(),'fresh generated model differs'
llbc=json.loads((fresh/'crate.llbc').read_text());assert not llbc['has_errors'];translated=llbc['translated'];assert translated['runtime_checks']['overflow_checks']
for key in ['monomorphize','monomorphize_mut','skip_borrowck','no_typecheck']:assert not translated.get('options',{}).get(key)
for rel in ['src/algo.rs','src/lib.rs']:
 fs=[f for f in translated['files'] if f['name'].get('Local')==rel];assert len(fs)==1 and fs[0]['contents']==(root/rel).read_text()
for name in ['new','checksum','from_checksum','write_slice','adler32_slice']:
 fs=[f for f in translated['fun_decls'] if f and f['item_meta']['is_local'] and f['item_meta']['name'][-1].get('Ident',[None])[0]==name];assert len(fs)==1,name
 f=fs[0];m=f['item_meta'];assert not m['is_extern'] and not m['has_errors'] and m['opacity']=='Transparent' and 'Structured' in f['body']
for module in ['ElicitGenerated','ElicitAdapter','Specs','ElicitContracts','Lemmas','Proofs']:
 src=cache/f'{module}.lean'
 if src.exists():shutil.copy2(src,out/src.name)
 if module in ['Specs','Lemmas','Proofs'] and (proofroot/f'{module}.lean').exists():assert (out/f'{module}.lean').read_bytes()==(proofroot/f'{module}.lean').read_bytes()
backend=pathlib.Path(os.environ['AENEAS_HOME'])/'backends/lean'
search=subprocess.check_output(['lake','env','printenv','LEAN_PATH'],cwd=backend,text=True).strip();env=os.environ.copy();env['LEAN_PATH']=str(out)+':'+search
builds={}
for module in ['ElicitGenerated','ElicitAdapter','Specs','ElicitContracts','Lemmas','Proofs']:
 if not (out/f'{module}.lean').exists():continue
 code,_=run(['lean','--root='+str(out),'-o',out/f'{module}.olean',out/f'{module}.lean'],out,env);builds[module]=code==0
allowed={'propext','Classical.choice','Quot.sound'}
def audit(name,imports,statement,theorem):
 f=out/f'{name}.lean';f.write_text(imports+'\nset_option pp.fullNames true\nopen Aeneas Aeneas.Std Result ElicitGenerated\n'+f'theorem {name} : {statement} := {theorem}\n#check @{theorem}\n#print {name}\n#print axioms {name}\n')
 code,log=run(['lean','--root='+str(out),f],out,env);text=log.read_text();ax=None
 if code==0:
  match=re.search(re.escape("'"+name+"'")+r' depends on axioms: \[([^\]]*)\]',text)
  if match:ax={x.strip() for x in match[1].split(',') if x.strip()}
  elif "'"+name+"' does not depend on any axioms" in text:ax=set()
 return {'kernel_checked':code==0 and ax is not None and ax<=allowed,'axioms':sorted(ax) if ax is not None else None,'statement':statement,'theorem':theorem,'log':log.name}
manifest=tomllib.loads((proofroot/'contracts.toml').read_text());apis={}
for i,(target,c) in enumerate(manifest.get('contracts',{}).items()):
 if not c.get('theorem'):continue
 contract='ElicitContracts.c_'+hashlib.sha256(target.encode()).hexdigest()[:16]
 apis[target]=audit(f'IndependentAPI_{i}','import Proofs',contract,c['theorem'])
 # Print the proposition body independently of the named proof assignment.
 f=out/f'Contract_{i}.lean';f.write_text('import ElicitContracts\nset_option pp.fullNames true\n#print '+contract+'\n');run(['lean','--root='+str(out),f],out,env)
lemmas={}
for i,s in enumerate(state.get('proof_plan',{}).get('steps',[])):
 if s['kind']=='lemma':lemmas[s['id']]=audit(f'IndependentLemma_{i}','import Lemmas',s['statement'],s['theorem'])
assert frozen=={str(p.relative_to(root)):sha(p) for p in proofroot.rglob('*') if p.is_file()},'authoring artifacts changed during audit'
summary={'elapsed_seconds':time.monotonic()-tick,'separate_fresh_extraction':True,'byte_identical_generated_model':True,'original_sources_identical':True,'overflow_checks':True,'original_selected_bodies_transparent':True,'module_builds':builds,'api_contract_audits':apis,'lemma_audits':lemmas,'frozen_authoring_sha256':frozen,'coverage_note':'Kernel acceptance is relative to the exact emitted proposition. Manually review universal API contracts and conditional/helper results before counting selected goals. This audit is outside the authoring clock.'}
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
