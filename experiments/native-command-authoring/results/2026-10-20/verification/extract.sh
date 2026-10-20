#!/usr/bin/env bash
set -euo pipefail
source /tmp/elicit-implementation-env
cd "$(dirname "$0")/.."
mkdir -p verification/Generated
charon cargo --preset=aeneas --sysroot=default \
  --start-from adler2::adler32_slice --start-from adler2::Adler32::from_checksum \
  --dest-file verification/Generated/adler2.llbc \
  -- --manifest-path Cargo.toml --target x86_64-unknown-linux-gnu
aeneas -backend lean -no-progress-bar -dest verification/Generated verification/Generated/adler2.llbc
