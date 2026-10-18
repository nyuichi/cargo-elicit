#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
export PATH=/workspace/toolchain/elan/bin:$PATH
cd /workspace/toolchain/aeneas-src/backends/lean
LEAN_PATH="$ROOT/generated:$ROOT${LEAN_PATH:+:$LEAN_PATH}" lake env lean --root="$ROOT" "$ROOT/${1:-Proofs}.lean" -o "$ROOT/${1:-Proofs}.olean"
