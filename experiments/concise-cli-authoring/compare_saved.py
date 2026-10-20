#!/usr/bin/env python3
"""Read exactly one unchanged saved snapshot with old and concise binaries.

Counts UTF-8 bytes/characters/lines, not model tokens. Never runs a verifier.
"""
import argparse, hashlib, json, pathlib, subprocess
p=argparse.ArgumentParser()
p.add_argument('--old',required=True,type=pathlib.Path)
p.add_argument('--new',required=True,type=pathlib.Path)
p.add_argument('--case',required=True,type=pathlib.Path)
p.add_argument('--out',required=True,type=pathlib.Path)
a=p.parse_args(); a.out.mkdir(parents=True,exist_ok=False)
snapshot=a.case/'.elicit/snapshot.json'
before=hashlib.sha256(snapshot.read_bytes()).hexdigest()
records=[]
for label,binary in [('previous',a.old),('concise',a.new)]:
 for name,argv in [('status',['status']),('status-json',['status','--json']),('plan',['plan']),('plan-json',['plan','--json'])]:
  r=subprocess.run([str(binary.resolve()),*argv],cwd=a.case,capture_output=True,check=True)
  (a.out/f'{label}-{name}.stdout').write_bytes(r.stdout)
  (a.out/f'{label}-{name}.stderr').write_bytes(r.stderr)
  text=r.stdout.decode()
  records.append({'binary':label,'view':name,'argv':argv,'utf8_bytes':len(r.stdout),'characters':len(text),'lines':len(text.splitlines()),'output_sha256':hashlib.sha256(r.stdout).hexdigest()})
assert hashlib.sha256(snapshot.read_bytes()).hexdigest()==before
summary={'unit':'bytes/characters/lines; no tokenizer estimate or model-token claim',
 'saved_snapshot_sha256':before,'snapshot_unchanged':True,
 'binary_sha256':{k:hashlib.sha256(v.read_bytes()).hexdigest() for k,v in [('previous',a.old),('concise',a.new)]},'records':records}
(a.out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
