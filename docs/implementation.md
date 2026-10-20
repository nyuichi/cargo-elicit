# Implementation

The CLI manages selected goals, observations and native command results. It does
not certify theorem meaning, axiom policy, full-input coverage or correspondence
between Rust and Lean. The old contract generator, Aeneas proof admission,
transitive axiom audit and backend-specific coverage gates have been removed.

`init` saves schema-3 configuration and inventories the original Rust crate.
`status` observes inputs without running verification. `status --saved` is a
pure saved read, explicitly unable to assert current workspace freshness.
`verify` runs configured argv in its configured directory and records exit code,
output, duration, input binding and diagnostics. Equal argv/directory pairs run
once per request; associations with selected APIs are declared by the author.
Per-goal commands override the project default. A missing command cannot succeed.

Success means exit code zero, no timeout/interruption, and unchanged observed
inputs before/after execution. No stdout parser determines mathematical success;
`sorry`, additional axioms, partial-input theorems and unrelated propositions
are governed by the native command, not rejected by cargo-elicit. Native tool
errors still fail through their exit status. Native build caches are tool-owned.

Ordinary verify can reuse current successful durable receipts. Reuse validates
receipt path, kind, input digest/map, project scope, goal/command binding, saved
result agreement, zero exit and successful outcome. Failed, missing, corrupted
or stale receipts run again. `--fresh` bypasses receipt reuse and reruns the
command; it does not perform a clean build or claim an independent audit.

Fingerprints cover local project/dependency files, scope, formal configuration,
CLI binary, resolved command executables, Rust/native version observations,
inherited environment hashes, and explicit `verification.inputs`. Editorial
descriptions do not invalidate results. `.git`, internal `.elicit`, Cargo target,
`.lake` and `.why3find` are excluded build/state boundaries. Externally installed
models, interpreter arguments, libraries and arbitrary script dependencies are
not exhaustively discovered. Add files/directories via `verification.inputs` to
bind additional dependencies, including files under otherwise excluded trees.
Input hashes establish observed freshness, not a hermetic execution guarantee.
No environment values are stored in receipts; only their hashes are recorded.

Native output logs are copied into the durable receipt directory before saving
results. An input race or missing input inspection cannot establish current
success. Signals/timeouts stop descendant processes. The existing project lock
serializes CLI mutations; it does not prevent external edits. `.elicit/history.jsonl`
records observations, outcomes, reuse and errors; bounded recent history is
navigation context, not extra evidence. There is no operational-notes directory,
handoff-file convention, approval command, scheduler or AI client.

Rust call graphs, SCCs, shared callees and Lean imports support planning only.
Unresolved calls and parser limitations are shown; they never gate native command
results. Completed goals have current successful command results; they are hidden
by default, retained in saved history, and reappear when inputs become stale.

## Observation cache

`.elicit/cache/analysis.json` stores Cargo package metadata, source entry points,
cfg, inventory/call edges and analysis limitations. A checksum validates the
serialized cache payload; missing, outdated or corrupt cache data falls back to
ordinary discovery without deleting command receipts. No language-server process
survives a CLI invocation. Cache persistence failures do not prevent observation.

Reuse is keyed by content hashes, selected package/features/target/backend,
compiler and executable identities, inherited environment and Cargo configuration.
Workspace manifest scanning detects new members selected by Cargo globs. Cached
dependency manifests and local dependency sources participate in invalidation.
Pre/post content observations reject input races. Full Cargo metadata is compared
when rebuilding, rather than checking only package scope and cfg.

Lean source files and native-only configuration can be separated from the Rust
analysis key when a conservative source/metadata screen finds no build scripts,
proc macros, macro invocations or custom module paths. This exemption never
removes those files from native-command input fingerprints. Builds that can
read arbitrary files retain the full input binding for analysis too. This is
an optimization of advisory analysis, not a new proof-admission policy.

Warm status/verify can therefore skip Cargo metadata and rust-analyzer startup.
Proof import observations are still refreshed from current Lean sources. On a
cache miss, source indexing runs first; rust-analyzer starts only if definition
or alias queries are needed. Version observations and content checks remain
current, and the normal saved-only status path still performs neither.
