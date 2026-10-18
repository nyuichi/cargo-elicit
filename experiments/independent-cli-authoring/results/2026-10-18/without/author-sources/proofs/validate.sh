#!/usr/bin/env bash
set -euo pipefail
proof_dir="$(cd "$(dirname "$0")" && pwd)"
repo_dir="$(dirname "$proof_dir")"
cd "$repo_dir"
"$proof_dir/extract.sh"
"$proof_dir/lean.sh" --root="$proof_dir/generated" \
  -o "$proof_dir/generated/Adler2.olean" "$proof_dir/generated/Adler2.lean" \
  > "$proof_dir/logs/generated.log" 2>&1
modules=(Codec Vector Inner Chunks Mathematics Outer Sums Tail Combine Specification Compute API Laws)
for module in "${modules[@]}"; do
  echo "Checking $module"
  "$proof_dir/lean.sh" --root="$proof_dir" -o "$proof_dir/$module.olean" \
    "$proof_dir/$module.lean" > "$proof_dir/logs/$module.log" 2>&1
done
"$proof_dir/lean.sh" --root="$proof_dir" "$proof_dir/Audit.lean" \
  > "$proof_dir/logs/axioms.log" 2>&1
python3 - "$proof_dir" <<'PY'
from pathlib import Path
import re, sys
p = Path(sys.argv[1])
files = list(p.glob('*.lean')) + list((p/'generated').glob('*.lean'))
for f in files:
    text = f.read_text()
    assert not re.search(r'\b(sorry|admit|native_decide)\b|^\s*axiom\s', text, re.M), f
allowed = {'propext', 'Classical.choice', 'Quot.sound'}
audit = (p/'logs/axioms.log').read_text()
lines = [l for l in audit.splitlines() if 'depends on axioms:' in l or 'does not depend on any axioms' in l]
assert len(lines) == 21, lines
for line in lines:
    if 'depends on axioms:' in line:
        names = line.split('[',1)[1].split(']',1)[0].split(',')
        assert {n.strip() for n in names} <= allowed, line
print('All 21 audited theorem roots use only the allowed axioms (or none).')
PY
export PATH=/workspace/toolchain/cargo/bin:$PATH
export RUSTUP_HOME=/workspace/toolchain/rustup
export CARGO_HOME=/workspace/toolchain/cargo
cargo test --locked --target x86_64-unknown-linux-gnu > "$proof_dir/logs/cargo-test.log" 2>&1
sha256sum -c "$proof_dir/runtime.sha256"
sha256sum "$proof_dir/generated/Adler2.lean" "$proof_dir/generated/adler2.llbc" > "$proof_dir/logs/generated.sha256"
echo 'Final validation passed.'
