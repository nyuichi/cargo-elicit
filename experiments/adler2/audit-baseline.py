#!/usr/bin/env python3
"""Independent fresh extraction and kernel audit, after timed authoring."""
import argparse,hashlib,json,os,pathlib,subprocess,time
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--root',required=True)
parser.add_argument('--out',required=True)
args=parser.parse_args()
root=pathlib.Path(args.root).resolve()
out=pathlib.Path(args.out).resolve()
started=time.monotonic()
# This auditor reproduces the manually reviewed proof artifact from this trial.
expected_proof='3ccb39d081cdc66d10a3fd62856a9b271611a188d3879decfbf419c7ae9e2625'
if hashlib.sha256((root/'verification/Proofs.lean').read_bytes()).hexdigest()!=expected_proof:raise SystemExit('Recorded proof artifact changed; theorem semantics require a new review')
out.mkdir(parents=True,exist_ok=False)
backend=pathlib.Path(os.environ['AENEAS_HOME'])/'backends/lean'
commands=[]
def run(argv,cwd,env=None):
    i=len(commands);commands.append({'argv':[str(x) for x in argv],'cwd':str(cwd)})
    with (out/f'command-{i}.log').open('w') as f:
        tick=time.monotonic()
        r=subprocess.run([str(x) for x in argv],cwd=cwd,env=env,text=True,stdout=f,stderr=subprocess.STDOUT)
        commands[-1]['elapsed_seconds']=time.monotonic()-tick
    commands[-1]['exit']=r.returncode
    (out/'commands.json').write_text(json.dumps(commands,indent=2)+'\n')
    if r.returncode:raise SystemExit(f'Audit command {i} failed')
run(['charon','cargo','--preset=aeneas','--sysroot=default','--exclude','adler2::adler32','--dest',out,'--','--manifest-path','Cargo.toml','--target','x86_64-unknown-linux-gnu'],root)
run([pathlib.Path(os.environ['AENEAS_HOME'])/'bin/aeneas','-backend','lean','-dest',out,out/'adler2.llbc'],root)
model=(out/'Adler2.lean').read_bytes()
if model!=(root/'verification/selected/Adler2.lean').read_bytes():raise SystemExit('Fresh generated model differs from the proof model')
(out/'Proofs.lean').write_bytes((root/'verification/Proofs.lean').read_bytes())
search=subprocess.check_output(['lake','env','printenv','LEAN_PATH'],cwd=backend,text=True).strip()
env=os.environ.copy();env['LEAN_PATH']=str(out)+':'+search
run(['lean','-o',out/'Adler2.olean',out/'Adler2.lean'],out,env)
run(['lean','-o',out/'Proofs.olean',out/'Proofs.lean'],out,env)
roots=['new_correct','checksum_correct','from_checksum_correct','codec_state_roundtrip','codec_checksum_roundtrip','resume_equivalence','scalar_split_representable','adler32_slice_of_update','four_byte_loader','lane_chunk_bound','seed_chunk_bound']
(out/'Audit.lean').write_text('import Proofs\n'+''.join(f'#check @Verification.{r}\n#print axioms Verification.{r}\n' for r in roots))
run(['lean',out/'Audit.lean'],out,env)
log=(out/f'command-{len(commands)-1}.log').read_text()
import re
sets=re.findall(r"depends on axioms: \[(.*?)\]",log,re.S)
if len(sets)!=len(roots):raise SystemExit('Incomplete independent root audit')
allowed={'propext','Classical.choice','Quot.sound'}
for names in sets:
    if {s.strip() for s in names.split(',') if s.strip()}-allowed:raise SystemExit('Unexpected theorem axiom')
expected={'src/algo.rs':'932c2bc591d13fe4470185125617b5aaa660a3898f23b553acc85df0bf49dded','src/lib.rs':'4acd41668fe30daffa37084e7e223f268957b816afc1864ffb3f5d6d7adf0890','Cargo.toml':'8f30dbd092f3acc475b9d339736cd7b64c6489bc47cd234a7f2232fc52e2d490'}
source={p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in expected}
if source!=expected:raise SystemExit('Original sources changed')
llbc=json.loads((out/'adler2.llbc').read_text())
if llbc['has_errors']:raise SystemExit('Charon reported errors')
translated=llbc['translated']
if not translated['runtime_checks']['overflow_checks']:raise SystemExit('Overflow checks disabled')
for key in ['monomorphize','monomorphize_mut','skip_borrowck','no_typecheck']:
    if translated['options'].get(key):raise SystemExit(f'Unsafe extraction option: {key}')
for relative in ['src/algo.rs','src/lib.rs']:
    matches=[f for f in translated['files'] if f['name'].get('Local')==relative]
    if len(matches)!=1 or matches[0]['contents']!=(root/relative).read_text():raise SystemExit('LLBC original source mismatch')
for name in ['new','checksum','from_checksum','write_slice','adler32_slice']:
    matches=[f for f in translated['fun_decls'] if f and f['item_meta']['is_local'] and f['item_meta']['name'][-1].get('Ident',[None])[0]==name]
    if len(matches)!=1:raise SystemExit('Ambiguous original API mapping')
    f=matches[0]
    if f['item_meta']['is_extern'] or f['item_meta']['has_errors'] or 'Structured' not in f['body']:raise SystemExit('Nontransparent original API')
(out/'summary.json').write_text(json.dumps({'elapsed_seconds':time.monotonic()-started,'fresh_model_byte_identical':True,'overflow_checks':True,'original_api_bodies_transparent':True,'charon_has_errors':False,'source_hashes':source,'allowed_axioms':sorted(allowed),'audited_roots':['Verification.'+r for r in roots],'accepted_apis':['Adler32::new','Adler32::checksum','Adler32::from_checksum'],'unproved_apis':['Adler32::write_slice','adler32_slice'],'selected_coverage':'3/5','notes':['adler32_slice_of_update assumes unproved UpdateContract and is not unconditional API correctness.','scalar_split_representable is a mathematical specification law, not optimized implementation correctness.','resume_equivalence preserves exact effects but does not prove write_slice success.']},indent=2)+'\n')
