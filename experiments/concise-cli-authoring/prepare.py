#!/usr/bin/env python3
"""Prepare one proof-free concise-CLI trial; historical runs are references only."""
import argparse, hashlib, json, pathlib, shutil, subprocess, tarfile
p=argparse.ArgumentParser()
p.add_argument('--destination',required=True,type=pathlib.Path)
a=p.parse_args(); dest=a.destination.resolve(); dest.mkdir(exist_ok=False)
repo=pathlib.Path(__file__).resolve().parents[2]
archive=repo/'experiments/independent-cli-authoring/results/2026-10-18/original/adler2-2.0.1.crate'
assert hashlib.sha256(archive.read_bytes()).hexdigest()=='320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa'
source=repo/'target/release/cargo-elicit'
frozen=dest/'frozen-bin'; frozen.mkdir(); shutil.copy2(source,frozen/'cargo-elicit')
installed=pathlib.Path('/workspace/toolchain/cargo/bin/cargo-elicit')
shutil.copy2(installed,dest/'previous-installed-cargo-elicit')
shutil.copy2(source,installed)
extract=dest/'extract'; extract.mkdir()
with tarfile.open(archive) as tar: tar.extractall(extract,filter='data')
case=dest/'case'; shutil.move(extract/'adler2-2.0.1',case); extract.rmdir()
assert not list(case.rglob('*.lean')) and not (case/'.elicit').exists()
protocol=pathlib.Path(__file__).resolve().parent
(case/'BENCHMARK_TASK.md').write_bytes((protocol/'TASK.md').read_bytes())
skill=case/'.agents/skills/cargo-elicit'; skill.mkdir(parents=True)
shutil.copy2(repo/'skills/cargo-elicit/SKILL.md',skill/'SKILL.md')
docs=case/'tool-docs'; docs.mkdir()
for s in ['README.md','docs/aeneas.md','docs/implementation.md','docs/planning.md']:
 shutil.copy2(repo/s,docs/pathlib.Path(s).name)
shutil.copytree(protocol,dest/'protocol',ignore=shutil.ignore_patterns('__pycache__','results'))
with (dest/'upstream-tests.log').open('w') as log:
 subprocess.run(['cargo','test','--locked','--lib','--offline'],cwd=case,stdout=log,stderr=subprocess.STDOUT,check=True)
files=list((repo/'src').rglob('*'))+[repo/'Cargo.toml',repo/'Cargo.lock']
record={'proof_free':True,'binary_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
 'production_source_sha256':{str(f.relative_to(repo)):hashlib.sha256(f.read_bytes()).hexdigest() for f in files if f.is_file()},
 'archive_sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),
 'comparison':'One fresh concise trial; historical trials are unmatched references, not a new control arm.'}
(dest/'preparation.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps(record))
