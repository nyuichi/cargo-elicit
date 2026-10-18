"""Independent exact-statement and transitive-axiom audits of requested laws."""
import pathlib,os,subprocess,json,time,re
out=pathlib.Path('/workspace/elicit-comparison/round9-continuation-product-audit')
assert (out/'summary.json').exists(), 'Run after the independent module/API audit'
backend=pathlib.Path(os.environ['AENEAS_HOME'])/'backends/lean'
env=os.environ.copy();env['LEAN_PATH']=str(out)+':'+subprocess.check_output(['lake','env','printenv','LEAN_PATH'],cwd=backend,text=True).strip()
statements={
'checksum_from_checksum_roundtrip':'''∀ (sum : U32), ∃ (s : Adler32) (out : U32), Adler32.from_checksum sum = .ok s ∧ Adler32.checksum s = .ok out ∧ out = sum''',
'from_checksum_checksum_roundtrip':'''∀ (s : Adler32), ∃ (sum : U32) (restored : Adler32), Adler32.checksum s = .ok sum ∧ Adler32.from_checksum sum = .ok restored ∧ restored = s''',
'split_update':'''∀ (s : Adler32) (x y xy : Slice U8), x.val.length + y.val.length ≤ Usize.max → xy.val = x.val ++ y.val → ∃ (sx sxy whole : Adler32), Adler32.write_slice s x = .ok sx ∧ Adler32.write_slice sx y = .ok sxy ∧ Adler32.write_slice s xy = .ok whole ∧ sxy = whole''',
'save_restore_resume':'''∀ (s : Adler32) (x y xy : Slice U8), x.val.length + y.val.length ≤ Usize.max → xy.val = x.val ++ y.val → ∃ (sx : Adler32) (saved : U32) (restored resumed whole : Adler32), Adler32.write_slice s x = .ok sx ∧ Adler32.checksum sx = .ok saved ∧ Adler32.from_checksum saved = .ok restored ∧ Adler32.write_slice restored y = .ok resumed ∧ Adler32.write_slice s xy = .ok whole ∧ resumed = whole''',
}
allowed={'propext','Classical.choice','Quot.sound'};rows={};start=time.monotonic()
for i,(name,statement) in enumerate(statements.items()):
 root=f'IndependentLaw_{i}';src=out/f'{root}.lean';src.write_text('import Proofs\nopen Aeneas Aeneas.Std Result ElicitGenerated\nset_option pp.fullNames true\n'+f'theorem {root} : {statement} := AdlerProof.{name}\n#print {root}\n#print axioms {root}\n')
 tick=time.monotonic();r=subprocess.run(['lean','--root='+str(out),str(src)],cwd=out,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);log=out/f'{root}.log';log.write_text(r.stdout)
 match=re.search(re.escape("'"+root+"'")+r' depends on axioms: \[([^\]]*)\]',r.stdout);ax={v.strip() for v in match[1].split(',') if v.strip()} if match else set() if "does not depend on any axioms" in r.stdout else None
 rows[name]={'statement':statement,'kernel_checked':r.returncode==0 and ax is not None and ax<=allowed,'axioms':sorted(ax) if ax is not None else None,'elapsed_seconds':time.monotonic()-tick,'exit':r.returncode,'log':log.name}
summary={'elapsed_seconds':time.monotonic()-start,'laws':rows,'notes':['Independent exact theorem assignments on freshly rebuilt modules; no new proof authoring.','Split/update and save/restore/resume require representable concatenation and an explicitly matching concatenated slice.']};(out/'additional-laws-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2));assert all(r['kernel_checked'] for r in rows.values())
