#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/verification-env.sh
sha256sum -c elicit-proofs/runtime.sha256
cargo test --locked --target x86_64-unknown-linux-gnu
cargo elicit audit
cargo elicit status
sha256sum -c elicit-proofs/runtime.sha256
