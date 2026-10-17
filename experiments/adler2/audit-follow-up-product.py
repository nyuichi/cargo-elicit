#!/usr/bin/env python3
"""Rebuild a fresh native wrapper cache independently and inspect exact roots."""
import argparse,hashlib,json,os,pathlib,re,subprocess,time,tomllib
p=argparse.ArgumentParser();p.add_argument('--root',required=True);p.add_argument('--cache',required=True);p.add_argument('--out',required=True);a=p.parse_args();root=pathlib.Path(a.root);cache=pathlib.Path(a.cache);out=pathlib.Path(a.out);out.mkdir(parents=True,exist_ok=False);commands=[];start=time.monotonic()
def run(argv,env):
 n=len(commands);t=time.monotonic()
 with (out/f'command-{n}.log').open('w') as f:r=subprocess.run(list(map(str,argv)),cwd=out,env=env,stdout=f,stderr=subprocess.STDOUT)
 commands.append({'argv':list(map(str,argv)),'exit':r.returncode,'elapsed_seconds':time.monotonic()-t});(out/'commands.json').write_text(json.dumps(commands,indent=2)+'\n');assert r.returncode==0,commands[-1]
manifest=tomllib.loads((root/'elicit-proofs/aeneas/contracts.toml').read_text());roots={k:v['theorem'] for k,v in manifest['contracts'].items() if v.get('theorem')}
for module in ['Specs','ElicitAdapter','ElicitGenerated','ElicitContracts','Proofs']:
 (out/f'{module}.lean').write_bytes((cache/f'{module}.lean').read_bytes())
for m in ['Specs','Proofs']:assert (out/f'{m}.lean').read_bytes()==(root/f'elicit-proofs/aeneas/{m}.lean').read_bytes()
backend=pathlib.Path(os.environ['AENEAS_HOME'])/'backends/lean';search=subprocess.check_output(['lake','env','printenv','LEAN_PATH'],cwd=backend,text=True).strip();env=os.environ.copy();env['LEAN_PATH']=str(out)+':'+search
for module in ['ElicitGenerated','ElicitAdapter','Specs','ElicitContracts','Proofs']:run(['lean','-o',out/f'{module}.olean',out/f'{module}.lean'],env)
source='import Proofs\n'
for i,(target,theorem) in enumerate(roots.items()):
 contract='ElicitContracts.c_'+hashlib.sha256(target.encode()).hexdigest()[:16];source+=f'theorem IndependentAudit_{i} : {contract} := {theorem}\n#check @{contract}\n#print {contract}\n#print axioms IndependentAudit_{i}\n'
(out/'IndependentAudit.lean').write_text(source);run(['lean',out/'IndependentAudit.lean'],env);log=(out/f'command-{len(commands)-1}.log').read_text();allowed={'propext','Classical.choice','Quot.sound'};axioms={}
for i,(target,theorem) in enumerate(roots.items()):
 name=f'IndependentAudit_{i}';match=re.search(re.escape("'"+name+"'")+r' depends on axioms: \[([^\]]*)\]',log)
 if match:names={x.strip() for x in match[1].split(',') if x.strip()}
 else:assert "'"+name+"' does not depend on any axioms" in log;names=set()
 assert names<=allowed,(name,names);axioms[target]=sorted(names)
llbc=json.loads((cache/'crate.llbc').read_text());assert not llbc['has_errors'];t=llbc['translated'];assert t['runtime_checks']['overflow_checks']
for key in ['monomorphize','monomorphize_mut','skip_borrowck','no_typecheck']:assert not t['options'].get(key)
for rel in ['src/algo.rs','src/lib.rs']:
 files=[f for f in t['files'] if f['name'].get('Local')==rel];assert len(files)==1 and files[0]['contents']==(root/rel).read_text()
for name in ['new','checksum','from_checksum','write_slice','adler32_slice']:
 fs=[f for f in t['fun_decls'] if f and f['item_meta']['is_local'] and f['item_meta']['name'][-1].get('Ident',[None])[0]==name];assert len(fs)==1,name
 f=fs[0];assert not f['item_meta']['is_extern'] and not f['item_meta']['has_errors'] and 'Structured' in f['body']
summary={'elapsed_seconds':time.monotonic()-start,'fresh_cache':str(cache),'separate_kernel_rebuild':True,'llbc_original_sources_identical':True,'overflow_checks':True,'original_api_bodies_transparent':True,'audited_contracts':roots,'axioms':axioms,'proof_sha256':{f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in out.glob('*.lean')},'acceptance_note':'These roots prove their emitted exact contracts. API coverage still requires manual semantic review; theorem names alone are insufficient.'};(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
