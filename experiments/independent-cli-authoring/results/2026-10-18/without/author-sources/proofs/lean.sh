#!/usr/bin/env bash
set -euo pipefail
proof_dir="$(cd "$(dirname "$0")" && pwd)"
export PATH=/workspace/toolchain/elan/bin:$PATH
export ELAN_HOME=/workspace/toolchain/elan
caller_dir="$PWD"
cd /workspace/toolchain/aeneas-src/backends/lean
export LEAN_PATH="$proof_dir/generated:$proof_dir:$(lake env printenv LEAN_PATH)"
cd "$caller_dir"
exec lean "$@"
