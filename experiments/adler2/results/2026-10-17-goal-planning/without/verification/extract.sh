#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export PATH=/workspace/toolchain/cargo/bin:$PATH
export RUSTUP_HOME=/workspace/toolchain/rustup
export CARGO_HOME=/workspace/toolchain/cargo
/workspace/toolchain/aeneas-src/charon/charon/target/release/charon cargo \
  --preset=aeneas --sysroot=default \
  --start-from adler2::Adler32::new \
  --start-from adler2::Adler32::checksum \
  --start-from adler2::Adler32::from_checksum \
  --start-from adler2::Adler32::write_slice \
  --start-from adler2::adler32_slice \
  --dest verification/selected-model -- --manifest-path Cargo.toml --target x86_64-unknown-linux-gnu
/workspace/toolchain/aeneas-src/bin/aeneas -backend lean \
  -dest verification/selected-model -use-lean-modules false verification/selected-model/adler2.llbc
cp verification/selected-model/Adler2.lean verification/generated/Adler2.lean
cp verification/selected-model/adler2.llbc verification/generated/adler2.llbc
