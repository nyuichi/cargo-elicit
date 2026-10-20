#!/usr/bin/env python3
"""Freeze the native-command CLI/skill and a proof-free upstream adler2 crate."""
import argparse,hashlib,json,pathlib,shutil,subprocess,tarfile
p=argparse.ArgumentParser();p.add_argument('--destination',type=pathlib.Path,required=True);a=p.parse_args()
repo=pathlib.Path(__file__).resolve().parents[2];dest=a.destination.resolve();dest.mkdir(parents=True,exist_ok=False)
archive=repo/'experiments/independent-cli-authoring/results/2026-10-18/original/adler2-2.0.1.crate'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(archive)=='320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa'
installed=pathlib.Path('/workspace/toolchain/cargo/bin/cargo-elicit');frozen=dest/'frozen-bin';frozen.mkdir();shutil.copy2(installed,frozen/'cargo-elicit')
extract=dest/'extract';extract.mkdir()
with tarfile.open(archive) as t:t.extractall(extract,filter='data')
case=dest/'case';shutil.move(extract/'adler2-2.0.1',case);extract.rmdir();(dest/'reports').mkdir()
assert not list(case.rglob('*.lean')) and not (case/'.elicit').exists()
protocol=pathlib.Path(__file__).resolve().parent
shutil.copy2(protocol/'TASK.md',case/'BENCHMARK_TASK.md');shutil.copytree(protocol,dest/'protocol',ignore=shutil.ignore_patterns('__pycache__','results'))
skill=case/'.agents/skills/cargo-elicit';skill.mkdir(parents=True);shutil.copy2(repo/'skills/cargo-elicit/SKILL.md',skill/'SKILL.md')
shutil.copy2(repo/'README.md',case/'.agents/README.md');docs=case/'.agents/docs';docs.mkdir()
for name in ['aeneas.md','implementation.md','planning.md','resume.md']:shutil.copy2(repo/'docs'/name,docs/name)
with (dest/'upstream-tests.log').open('w') as f:subprocess.run(['cargo','test','--locked','--lib','--offline'],cwd=case,stdout=f,stderr=subprocess.STDOUT,check=True)
record={'proof_free':True,'product_local_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
 'binary_sha256':sha(installed),'skill_sha256':sha(skill/'SKILL.md'),'archive_sha256':sha(archive),
 'source_sha256':{str(p.relative_to(repo)):sha(p) for p in sorted((repo/'src').glob('*.rs'))},
 'original_sha256':{str(p.relative_to(case)):sha(p) for p in [case/'Cargo.toml',*sorted((case/'src').rglob('*.rs'))]},
 'comparison':'Fresh native-command CLI authoring versus historical no-CLI trial; not a new matched control or isolated causal estimate.'}
(dest/'preparation.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'destination':str(dest),'proof_free':True,'binary_sha256':record['binary_sha256']}))
