#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
export PATH=/workspace/toolchain/elan/bin:$PATH
cd /workspace/toolchain/aeneas-src/backends/lean
lake env lean --root="$ROOT/generated" "$ROOT/generated/Adler2.lean" -o "$ROOT/generated/Adler2.olean" > "$ROOT/logs/check-model.log" 2>&1
for module in Proofs Spec Chunks Loops Tail Algebra Outer Micro Core APIs; do
  LEAN_PATH="$ROOT/generated:$ROOT${LEAN_PATH:+:$LEAN_PATH}" lake env lean --root="$ROOT" "$ROOT/$module.lean" -o "$ROOT/$module.olean" > "$ROOT/logs/check-$module.log" 2>&1
  printf '%s checked\n' "$module"
done
