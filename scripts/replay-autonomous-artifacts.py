#!/usr/bin/env python3
"""Investigator-only saved Adler2 artifact replay; never feed these to a fresh author.

No proof search: ordinary-arm adaptation only renames imports/namespaces and
adds direct total-WP/export wrappers around the saved theorem statements.
All preparation, staging, status and collection are included in total cost.
"""
import argparse
import hashlib
import json
import pathlib
import re
import shutil
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--source', type=pathlib.Path, required=True)
parser.add_argument('--output', type=pathlib.Path, required=True)
parser.add_argument('--binary', type=pathlib.Path, required=True)
parser.add_argument('--layout', choices=['product', 'ordinary'], required=True)
a = parser.parse_args()
source, out, binary = a.source.resolve(), a.output.resolve(), a.binary.resolve()
assert not out.exists(), 'Use a fresh output directory'
start = time.monotonic()
root = out / 'case'
root.mkdir(parents=True)
expected = {'src/algo.rs': '932c2bc591d13fe4470185125617b5aaa660a3898f23b553acc85df0bf49dded', 'src/lib.rs': '4acd41668fe30daffa37084e7e223f268957b816afc1864ffb3f5d6d7adf0890', 'Cargo.toml': '8f30dbd092f3acc475b9d339736cd7b64c6489bc47cd234a7f2232fc52e2d490'}
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
assert {p: sha(source / p) for p in expected} == expected
for path in source.iterdir():
    if path.name in {'.git', '.elicit', 'target', 'bin', 'verification', 'elicit-proofs', 'elicit.toml'}:
        continue
    dest = root / path.name
    if path.is_dir():
        shutil.copytree(path, dest)
    else:
        shutil.copy2(path, dest)
proof = root / 'elicit-proofs/aeneas'
proof.mkdir(parents=True)
original_artifacts = {}
if a.layout == 'product':
    for path in (source / 'elicit-proofs/aeneas').iterdir():
        if path.suffix in {'.lean', '.toml'}:
            original_artifacts[path.name] = sha(path)
            shutil.copy2(path, proof / path.name)
else:
    for path in (source / 'verification').glob('*.lean'):
        if path.name == 'lakefile.lean':
            continue
        original_artifacts[path.name] = sha(path)
        text = path.read_text().replace('import Adler2', 'import ElicitGenerated')
        text = re.sub(r'\badler2\b', 'ElicitGenerated', text)
        text = text.replace('import Proofs\n', 'import SeedProofs\n')
        (proof / ('SeedProofs.lean' if path.name == 'Proofs.lean' else path.name)).write_text(text)
    (proof / 'Specs.lean').write_text('import Spec\n')
    ids = {'new': 'adler2::impl[ff6e69000b04949b]::new::Method', 'checksum': 'adler2::impl[ff6e69000b04949b]::checksum::Method', 'from_checksum': 'adler2::impl[ff6e69000b04949b]::from_checksum::Method', 'write_slice': 'adler2::impl[ff6e69000b04949b]::write_slice::Method', 'adler32_slice': 'adler2::adler32_slice::Function'}
    ensures = {'new': 'fun out => out = { a := 1#u16, b := 0#u16 }', 'checksum': 'fun self out => out.val = 65536*self.b.val + self.a.val', 'from_checksum': 'fun sum out => out.a.val = sum.val % 65536 ∧ out.b.val = sum.val / 65536', 'write_slice': 'fun self bytes out => (out.a.val, out.b.val) = Verification.scalar (self.a.val, self.b.val) bytes.val', 'adler32_slice': 'fun bytes out => out.val = 65536*(Verification.scalar (1,0) bytes.val).2 + (Verification.scalar (1,0) bytes.val).1'}
    manifest = 'schema_version = 1\n'
    bridges = 'import APIs\nimport ElicitContracts\nopen Aeneas Aeneas.Std Result ElicitGenerated\nnamespace Replay\n'
    for name, identity in ids.items():
        contract = 'c_' + hashlib.sha256(identity.encode()).hexdigest()[:16]
        manifest += f'\n[contracts."{identity}"]\nensures = {json.dumps(ensures[name], ensure_ascii=False)}\ntheorem = "Replay.{name}"\n'
        bridges += f'theorem {name} : ElicitContracts.{contract} := by\n'
        if name == 'new':
            bridges += '  intro _\n  exact ⟨_, Verification.new_correct, rfl⟩\n'
        else:
            args = {'checksum': 'self', 'from_checksum': 'sum', 'write_slice': 'self bytes', 'adler32_slice': 'bytes'}[name]
            bridges += f'  intro {args} _\n  exact ElicitAdapter.spec_to_contract (Verification.{name}_correct {args})\n'
    (proof / 'contracts.toml').write_text(manifest)
    (proof / 'Proofs.lean').write_text(bridges + 'end Replay\n')
(root / 'elicit.toml').write_text('[agent]\nprovider="codex"\n[verification]\ntool="aeneas"\nrequired=true\n')
commands = []
def run(args):
    index = len(commands)
    begin = time.monotonic()
    log = out / f'command-{index}.log'
    with log.open('w') as f:
        result = subprocess.run([str(binary), *args], cwd=root, stdout=f, stderr=subprocess.STDOUT)
    commands.append({'args': args, 'exit': result.returncode, 'elapsed_seconds': time.monotonic()-begin, 'log': log.name})
    (out / 'commands.json').write_text(json.dumps(commands, indent=2)+'\n')
    assert result.returncode == 0, f'Command failed: {args}; inspect {log}'
    return log
selectors = ['new', 'checksum', 'from_checksum', 'write_slice', 'adler32_slice']
init = ['init', '--tool', 'aeneas']
for selector in selectors:
    init += ['--api', selector]
run(init)
run(['status', '--json'])
run(['agent', 'stage', '--all', '--note', 'Investigator replay: previously reviewed saved arbitrary-input contracts; no new authoring'])
run(['status', '--json'])
run(['collect', '--seconds', '600'])
status = json.loads(run(['status', '--json']).read_text())
assert status['complete'] and status['selected_goals_verified']
assert {p: sha(root / p) for p in expected} == expected
summary = {'layout': a.layout, 'elapsed_seconds': time.monotonic()-start, 'binary_sha256': sha(binary), 'original_source_sha256': expected, 'original_artifact_sha256': original_artifacts, 'commands': commands, 'status': status, 'note': 'Fixed-artifact acceptance cost including format adaptation, init, staging and three status calls. No LLM authoring, no cross-run project object cache, no speedup estimate.'}
(out / 'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
print(json.dumps({'elapsed_seconds': summary['elapsed_seconds'], 'complete': status['complete'], 'layout': a.layout}))
