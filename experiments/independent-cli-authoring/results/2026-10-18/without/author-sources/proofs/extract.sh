#!/usr/bin/env bash
set -euo pipefail
proof_dir="$(cd "$(dirname "$0")" && pwd)"
repo_dir="$(dirname "$proof_dir")"
export PATH=/workspace/toolchain/cargo/bin:$PATH
export RUSTUP_HOME=/workspace/toolchain/rustup
export CARGO_HOME=/workspace/toolchain/cargo
export AENEAS_HOME=/workspace/toolchain/aeneas-src
cd "$repo_dir"
mkdir -p "$proof_dir/generated" "$proof_dir/logs"
sha256sum -c "$proof_dir/runtime.sha256"
# Remove only the generated artifacts from this task so a failed extraction cannot reuse them.
python3 - "$proof_dir" <<'PY'
from pathlib import Path
import sys
for name in ['adler2.llbc', 'Adler2.lean', 'Adler2.olean']:
    (Path(sys.argv[1]) / 'generated' / name).unlink(missing_ok=True)
PY
"$AENEAS_HOME/charon/charon/target/release/charon" cargo \
  --preset=aeneas --sysroot=default --error-on-warnings \
  --start-from 'adler2::Adler32::new' \
  --start-from 'adler2::Adler32::checksum' \
  --start-from 'adler2::Adler32::from_checksum' \
  --start-from 'adler2::Adler32::write_slice' \
  --start-from 'adler2::adler32_slice' \
  --dest-file "$proof_dir/generated/adler2.llbc" \
  -- --manifest-path "$repo_dir/Cargo.toml" --target x86_64-unknown-linux-gnu \
  > "$proof_dir/logs/charon.log" 2>&1
test -s "$proof_dir/generated/adler2.llbc"
"$AENEAS_HOME/bin/aeneas" -backend lean -no-progress-bar \
  -dest "$proof_dir/generated" "$proof_dir/generated/adler2.llbc" \
  > "$proof_dir/logs/aeneas.log" 2>&1
test -s "$proof_dir/generated/Adler2.lean"
