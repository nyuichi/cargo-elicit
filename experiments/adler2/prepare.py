#!/usr/bin/env python3
"""Prepare identical, unproved Adler-32 cases. Never overwrite previous runs."""
import argparse,hashlib,json,pathlib,shutil,subprocess,tarfile
p=argparse.ArgumentParser();p.add_argument('workdir',type=pathlib.Path);args=p.parse_args()
r=args.workdir.resolve()
if r.exists() and any(r.iterdir()):raise SystemExit('Refusing to overwrite a nonempty workdir')
r.mkdir(parents=True,exist_ok=True)
url='https://static.crates.io/crates/adler2/adler2-2.0.1.crate'
checksum='320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa'
archive=r/'adler2-2.0.1.crate'
subprocess.run(['curl','-fLsS',url,'-o',str(archive)],check=True)
if hashlib.sha256(archive.read_bytes()).hexdigest()!=checksum:raise SystemExit('Release checksum mismatch')
with tarfile.open(archive) as tar:tar.extractall(r/'upstream',filter='data')
source=r/'upstream/adler2-2.0.1'
subprocess.run(['cargo','generate-lockfile','--manifest-path',str(source/'Cargo.toml')],check=True)
local=pathlib.Path(__file__).resolve().parent
for arm in ['with','without']:
 case=r/f'adler2-{arm}';shutil.copytree(source,case)
 (case/'BENCHMARK_TASK.md').write_text((local/'TASK.md').read_text())
 (case/'AGENTS.md').write_text('Read BENCHMARK_TASK.md before starting. Follow its common goals, preservation rules and experiment boundaries. This is the '+arm+' cargo-elicit arm.\n')
 (case/'bin').mkdir();shutil.copy2(local/'codex-proxy.py',case/'bin/codex');(case/'bin/codex').chmod(0o755)
 if arm=='with':(case/'elicit.toml').write_text('[agent]\nprovider = "codex"\n\n[verification]\ntool = "aeneas"\nrequired = true\n')
 subprocess.run(['git','init','-q',str(case)],check=True)
 subprocess.run(['git','-C',str(case),'add','.'],check=True)
 subprocess.run(['git','-C',str(case),'-c','user.name=Experiment Setup','-c','user.email=setup@example.invalid','commit','-qm','Unmodified upstream and common experiment instructions'],check=True)
files={str(p.relative_to(source)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(source.rglob('*')) if p.is_file()}
(r/'preparation.json').write_text(json.dumps({'crate':'adler2','version':'2.0.1','archive_sha256':checksum,'source_url':url,'files':files,'trial_order':['without','with'],'status':'prepared; no authoring trials run'},indent=2)+'\n')
