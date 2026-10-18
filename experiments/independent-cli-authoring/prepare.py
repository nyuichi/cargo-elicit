#!/usr/bin/env python3
"""Make two source-identical, proof-free cases from a checksummed release."""
import argparse, hashlib, json, pathlib, shutil, subprocess, tarfile, time
p = argparse.ArgumentParser()
p.add_argument('--archive', required=True, type=pathlib.Path)
p.add_argument('--binary', required=True, type=pathlib.Path)
p.add_argument('--destination', required=True, type=pathlib.Path)
a = p.parse_args()
archive_hash = hashlib.sha256(a.archive.read_bytes()).hexdigest()
assert archive_hash == '320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa'
destination = a.destination.resolve()
destination.mkdir(parents=True, exist_ok=False)
repo = pathlib.Path(__file__).resolve().parents[2]
protocol = pathlib.Path(__file__).resolve().parent
shutil.copytree(protocol, destination / 'protocol', ignore=shutil.ignore_patterns('__pycache__'))
binpath = destination / 'frozen-bin'
binpath.mkdir()
shutil.copyfile(a.binary, binpath / 'cargo-elicit')
(binpath / 'cargo-elicit').chmod(0o755)
record = {'archive_sha256': archive_hash, 'binary_sha256': hashlib.sha256(a.binary.read_bytes()).hexdigest(), 'product_local_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip(), 'arms': {}}
for arm in ['with', 'without']:
    temp = destination / ('extract-' + arm)
    temp.mkdir()
    with tarfile.open(a.archive) as archive:
        archive.extractall(temp, filter='data')
    root = destination / 'cases' / arm
    root.parent.mkdir(exist_ok=True)
    shutil.move(temp / 'adler2-2.0.1', root)
    temp.rmdir()
    assert not list(root.rglob('*.lean')) and not (root / '.elicit').exists()
    (root / 'BENCHMARK_TASK.md').write_bytes((protocol / 'TASK.md').read_bytes())
    if arm == 'with':
        skill = root / '.agents/skills/cargo-elicit'
        skill.mkdir(parents=True)
        shutil.copyfile(repo / 'skills/cargo-elicit/SKILL.md', skill / 'SKILL.md')
        docs = root / 'tool-docs'
        docs.mkdir()
        for source, target in [('README.md', 'README.md'), ('docs/aeneas.md', 'aeneas.md'), ('docs/implementation.md', 'implementation.md')]:
            shutil.copyfile(repo / source, docs / target)
    t = time.monotonic()
    with (destination / (arm + '-upstream-tests.log')).open('w') as log:
        result = subprocess.run(['cargo', 'test', '--locked', '--lib'], cwd=root, stdout=log, stderr=subprocess.STDOUT)
    assert result.returncode == 0, 'upstream preparation test failed'
    record['arms'][arm] = {'upstream_test_seconds': time.monotonic() - t, 'source_hashes': {str(f.relative_to(root)): hashlib.sha256(f.read_bytes()).hexdigest() for f in [root / 'Cargo.toml'] + sorted((root / 'src').glob('*.rs'))}}
assert record['arms']['with']['source_hashes'] == record['arms']['without']['source_hashes']
(destination / 'preparation.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({'destination': str(destination), 'source_identical': True, 'proof_free': True, 'binary_sha256': record['binary_sha256']}))
