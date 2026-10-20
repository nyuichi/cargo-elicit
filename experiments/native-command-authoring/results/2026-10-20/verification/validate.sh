#!/usr/bin/env bash
set -euo pipefail
source /tmp/elicit-implementation-env
cd "$(dirname "$0")/.."
sha256sum --check verification/original.sha256
mkdir -p target/native-validation
charon cargo --preset=aeneas --sysroot=default \
  --start-from adler2::adler32_slice --start-from adler2::Adler32::from_checksum \
  --dest-file target/native-validation/adler2.llbc \
  -- --manifest-path Cargo.toml --target x86_64-unknown-linux-gnu
aeneas -backend lean -no-progress-bar -dest target/native-validation target/native-validation/adler2.llbc
cmp verification/Generated/Adler2.lean target/native-validation/Adler2.lean
sha256sum --check verification/original.sha256
cd verification
lake clean adlerVerification
lake build
lake env lean Audit.lean > .lake/axioms.txt
cat .lake/axioms.txt
python3 audit_axioms.py .lake/axioms.txt
