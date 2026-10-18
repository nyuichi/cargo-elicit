#!/usr/bin/env python3
"""Archive final evidence, preserving raw bytes; never includes auth files or build caches."""
import argparse,gzip,hashlib,json,pathlib,shutil,tarfile
p=argparse.ArgumentParser();p.add_argument('--run',required=True,type=pathlib.Path);p.add_argument('--output',required=True,type=pathlib.Path);a=p.parse_args();run=a.run.resolve();out=a.output.resolve();out.mkdir(parents=True,exist_ok=True)
def copy(source,target):
 if source.is_file():target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,target)
def gz(source,target):
 if not source.is_file():return
 target.parent.mkdir(parents=True,exist_ok=True)
 with source.open('rb') as src,target.open('wb') as dst,gzip.GzipFile(filename='',mode='wb',fileobj=dst,mtime=0) as comp:shutil.copyfileobj(src,comp)
def archive(directory,target,select):
 target.parent.mkdir(parents=True,exist_ok=True)
 with target.open('wb') as dst,gzip.GzipFile(filename='',mode='wb',fileobj=dst,mtime=0) as comp,tarfile.open(fileobj=comp,mode='w') as tar:
  for f in sorted(directory.rglob('*')):
   if f.is_symlink():continue
   if f.is_file() and select(f):
    info=tar.gettarinfo(str(f),arcname=str(f.relative_to(directory)));info.uid=info.gid=0;info.uname=info.gname='';info.mtime=0
    with f.open('rb') as src:tar.addfile(info,src)
for name in ['preparation.json','binary-path-repair.json','temporary-isolation.json','protocol-sha256.json','observer-tests.log','release-build.log','with-upstream-tests.log','without-upstream-tests.log','with-upstream-tests-reprepared.log']:
 copy(run/name,out/name)
copy(run/'readiness/status.json',out/'readiness-summary.json')
for arm in ['with','without']:
 case=run/'cases'/arm;results=run/'results'/arm;dest=out/arm
 if not (results/'metrics.json').exists():continue
 for name in ['metrics.json','resolved-settings.json','observations.json','author-messages.json','terminal-snapshot.json','termination.json','check-audit-summary.json','input-file-changes.json','cache-summary.json']:
  copy(results/name,dest/name)
 for f in results.glob('status-*'):copy(f,dest/f.name)
 for name in ['telemetry.jsonl','commands.json','command-analysis.json','terminal-events.jsonl']:
  gz(results/name,dest/(name+'.gz'))
 for name in ['FINAL_REPORT.md','PROGRESS.md','verification-inventory.json','elicit.toml','BENCHMARK_TASK.md']:
  copy(case/name,dest/name)
 for folder in ['elicit-proofs','proofs','verification','scripts']:
  source=case/folder
  if source.is_dir():
   for f in sorted(source.rglob('*')):
    if f.is_file() and not f.is_symlink() and not any(part in ['.lake','.git','.elicit','target'] for part in f.relative_to(source).parts) and f.suffix in ['.lean','.toml','.json','.md','.sh','.py','.sha256']:
     copy(f,dest/'author-sources'/f.relative_to(case))
 if (case/'proofs').is_dir():archive(case/'proofs',dest/'native-authoring-logs.tar.gz',lambda f:'logs' in f.relative_to(case/'proofs').parts or f.suffix=='.llbc')
 if (case/'.elicit/evidence').is_dir():archive(case/'.elicit/evidence',dest/'product-evidence.tar.gz',lambda f:True)
 temp=results/'temporary-artifacts'
 if temp.is_dir():archive(temp,dest/'temporary-sources.tar.gz',lambda f:f.suffix in ['.lean','.sh','.py'])
 inv=run/'investigator'/arm
 if inv.is_dir():archive(inv,dest/'investigator-audit.tar.gz',lambda f:f.suffix not in ['.olean','.ilean','.o','.c'] and 'target' not in f.parts)
 for name in ['review.json','timing.json','supplementary-summary.json','summary.json']:
  copy(inv/name,dest/'investigator'/name)
# Invalid environment attempt is retained, never promoted to a completed treatment.
invalid=run/'results/invalid-binary-path-with'
if invalid.is_dir():
 for name in ['metrics.json','resolved-settings.json','observations.json']:copy(invalid/name,out/'invalid-binary-path-with'/name)
 gz(invalid/'telemetry.jsonl',out/'invalid-binary-path-with/telemetry.jsonl.gz')
for name in ['numeric-summary.json','comparison.json']:
 copy(run/'results'/name,out/name)
for name in ['PRODUCTIVITY_REVIEW.md','analysis-usage.json']:
 copy(run/'posthoc'/name,out/name)
# Original crate is tiny and independently hashable; no old proof solution is included.
case=run/'cases/without';copy(case/'Cargo.toml',out/'original/Cargo.toml');copy(case/'Cargo.lock',out/'original/Cargo.lock');copy(case/'Cargo.toml.orig',out/'original/Cargo.toml.orig')
for f in (case/'src').rglob('*'):
 if f.is_file():copy(f,out/'original/src'/f.relative_to(case/'src'))
manifest={str(f.relative_to(out)):hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted(out.rglob('*')) if f.is_file() and f.name!='SHA256SUMS.json'}
(out/'SHA256SUMS.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'files':len(manifest),'bytes':sum(f.stat().st_size for f in out.rglob('*') if f.is_file())}))
