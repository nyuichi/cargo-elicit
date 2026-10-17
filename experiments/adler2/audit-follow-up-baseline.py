#!/usr/bin/env python3
"""Independent post-trial audit; no proof hints are supplied to live workers."""
import hashlib,json,os,pathlib,re,subprocess,time
import argparse
p=argparse.ArgumentParser(description=__doc__); p.add_argument('--root',required=True); p.add_argument('--out',required=True);args=p.parse_args()
root=pathlib.Path(args.root).resolve();out=pathlib.Path(args.out).resolve()
expected_proofs={'Proofs': '95127784a4c09a4ecae1625ccaa03f60ae8093d5cbce0e69a5533c41b91badd8', 'CoreLemmas': '41d2412a676a062dd74734b9dda911f9a0f1622035b64cd80ea09574a5c8e3bb', 'Empty': 'b718466f526a393ba2b9b3eeaad226277800967e1bdf6ab4226e8afea944d3b4', 'LaneAlgebra': 'e5106d2c140e33e8b51340fbdf13b6d65d12936eb690722c23622427b5c02a19'}
for module,digest in expected_proofs.items():
 assert hashlib.sha256((root/f'verification/{module}.lean').read_bytes()).hexdigest()==digest, 'Recorded proof changed; new semantic review required'
out.mkdir(parents=True,exist_ok=False)
commands=[]; start=time.monotonic()
def run(argv,cwd,env=None):
 i=len(commands); c={'argv':list(map(str,argv)),'cwd':str(cwd)}; commands.append(c)
 with (out/f'command-{i}.log').open('w') as f:
  t=time.monotonic(); p=subprocess.run(list(map(str,argv)),cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
 c.update(exit=p.returncode,seconds=time.monotonic()-t)
 (out/'commands.json').write_text(json.dumps(commands,indent=2)+'\n')
 assert p.returncode==0,c
expected={'src/algo.rs': '932c2bc591d13fe4470185125617b5aaa660a3898f23b553acc85df0bf49dded', 'src/lib.rs': '4acd41668fe30daffa37084e7e223f268957b816afc1864ffb3f5d6d7adf0890', 'Cargo.toml': '8f30dbd092f3acc475b9d339736cd7b64c6489bc47cd234a7f2232fc52e2d490'}
actual={p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in expected}; assert expected==actual
run(['charon','cargo','--preset=aeneas','--sysroot=default','--exclude','adler2::adler32','--exclude','adler2::{core::fmt::Debug for adler2::Adler32}','--dest',out,'--','--manifest-path','Cargo.toml','--target','x86_64-unknown-linux-gnu','--lib'],root)
run([pathlib.Path(os.environ['AENEAS_HOME'])/'bin/aeneas','-backend','lean','-dest',out,'-no-progress-bar',out/'adler2.llbc'],root)
llbc=json.loads((out/'adler2.llbc').read_text());assert not llbc['has_errors'];t=llbc['translated']
assert t['runtime_checks']['overflow_checks']
for key in ['monomorphize','monomorphize_mut','skip_borrowck','no_typecheck']:assert not t['options'].get(key),key
for relative in ['src/algo.rs','src/lib.rs']:
 matches=[f for f in t['files'] if f['name'].get('Local')==relative]
 assert len(matches)==1 and matches[0]['contents']==(root/relative).read_text()
for name in ['new','checksum','from_checksum','write_slice','adler32_slice']:
 matches=[f for f in t['fun_decls'] if f and f['item_meta']['is_local'] and f['item_meta']['name'][-1].get('Ident',[None])[0]==name]
 assert len(matches)==1,name
 f=matches[0]; assert not f['item_meta']['is_extern'] and not f['item_meta']['has_errors'] and 'Structured' in f['body'],name
assert (out/'Adler2.lean').read_bytes()==(root/'verification/generated/Adler2.lean').read_bytes()
modules=['Proofs','CoreLemmas','Empty','LaneAlgebra']
for m in modules:
 data=(root/f'verification/{m}.lean').read_bytes(); assert not re.search(rb'\b(sorry|admit|axiom)\b',data); (out/f'{m}.lean').write_bytes(data)
backend=pathlib.Path(os.environ['AENEAS_HOME'])/'backends/lean'
search=subprocess.check_output(['lake','env','printenv','LEAN_PATH'],cwd=backend,text=True).strip()
env=os.environ.copy(); env['LEAN_PATH']=str(out)+':'+search
for m in ['Adler2']+modules: run(['lean','-o',out/f'{m}.olean',out/f'{m}.lean'],out,env)
roots=['Verification.new_correct', 'Verification.from_checksum_eq', 'Verification.from_checksum_correct', 'Verification.checksum_eq', 'Verification.encode_val', 'Verification.checksum_correct', 'Verification.codec_decode_encode', 'Verification.codec_encode_decode', 'Verification.checksum_save_restore', 'Verification.checksum_restore_save', 'Verification.save_restore_resume', 'Verification.normalize_idem', 'Verification.scalarStep_normalized', 'Verification.scalarAdler_normalized', 'Verification.scalarAdler_split', 'Verification.loader_correct', 'Verification.vector_rem_correct', 'Verification.vector_add_correct', 'Verification.vector_mul_correct', 'Verification.lane_accumulator_bound', 'Verification.scalar_chunk_seed_bound', 'Verification.convenience_of_write', 'Verification.split_of_write', 'Verification.chunk_loop0_empty', 'Verification.chunk_loop1_empty', 'Verification.four_lane_step', 'Verification.complement_correction']
(out/'IndependentAudit.lean').write_text(''.join(f'import {m}\n' for m in modules)+''.join(f'#check @{r}\n#print axioms {r}\n' for r in roots))
run(['lean',out/'IndependentAudit.lean'],out,env)
log=(out/f'command-{len(commands)-1}.log').read_text(); allowed={'propext','Classical.choice','Quot.sound'}; axioms={}
for r in roots:
 match=re.search(re.escape("'"+r+"'")+r' depends on axioms: \[([^\]]*)\]',log)
 if match: names={n.strip() for n in match[1].split(',') if n.strip()}
 else: assert "'"+r+"' does not depend on any axioms" in log,r; names=set()
 assert names<=allowed,(r,names); axioms[r]=sorted(names)
summary={'elapsed_seconds':time.monotonic()-start,'fresh_model_byte_identical':True,'charon_has_errors':False,'original_sources_unchanged':True,'source_hashes':actual,'audited_roots':roots,'axioms':axioms,'accepted_apis':['Adler32::new','Adler32::checksum','Adler32::from_checksum'],'unproved_apis':['Adler32::write_slice','adler32_slice'],'selected_coverage':'3/5','notes':['All-state checksum and from_checksum theorems include successful Result outcomes and exact numeric results.','save_restore_resume preserves exact effects but does not establish write_slice success.','convenience_of_write and split_of_write assume the unproved WriteSliceCorrect contract.','Empty iterator exit and mathematical lane identities are support results, not unrestricted optimized update correctness.']}
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n'); print(json.dumps(summary,indent=2))
