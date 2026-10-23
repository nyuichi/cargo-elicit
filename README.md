# cargo-elicit

A Rust CLI for people and AI tools to track native verification work across a
crate. It records command results, changed inputs and dependency planning hints.
It never starts an AI or decides whether a theorem proves an API correct.

## Three commands

| Command | Purpose |
| --- | --- |
| `init --api API --command PROGRAM ARG...` | Select goals, discover APIs and save the native verification command |
| `status [API]` | Observe inputs and show remaining work, changes, diagnostics and saved results |
| `verify [API ...]` | Run configured native commands and save input-bound results; reuse unchanged successful results |

```sh
cargo install cargo-elicit --locked
cd your-crate
cargo elicit init --api identity --directory verification --command lake build
# Write specifications/proofs normally in your native project.
cargo elicit verify
cargo elicit status
```

Put other init options before `--command`, whose remaining arguments are native
command arguments. No dedicated contract comments, generated propositions,
wrapper proofs, theorem registration, handoff files or approval commands are
required. Existing native Aeneas, Lean, Kani and Creusot workflows can be invoked
through their ordinary commands or a project script. `init` does not generate,
translate or compile proofs.

## Native command configuration

`elicit.toml` uses schema 3:

```toml
schema = 3

[verification]
tool = "aeneas" # inventory/backend label, not a pinned toolchain requirement
command = ["lake", "build"]
directory = "verification" # relative to the crate root; default "."
# Add external libraries/scripts/toolchain files used by your command if needed.
inputs = []

[[obligations]]
id = "identity"
target = "identity"

[[obligations]]
id = "other"
target = "other"
command = ["sh", "check-other.sh"] # optional per-goal override
# directory = "." # otherwise inherits verification.directory
```

Commands are argv arrays, executed without an implicit shell. Use
`["sh", "verify.sh"]` for a native script; use a fail-fast script when several
steps must succeed. Configure what you would normally run without cargo-elicit.
A shared command/directory runs once per request and its result is associated
with all selected goals that use it. Those associations are user-declared; the
CLI does not inspect whether the command actually covers each API. Missing
commands are reported, never treated as successful verification.

Optional `statement`, `theorem` and `description` fields are user metadata.
There is no required proposition shape, full-input coverage gate, Rust/theorem
correspondence check, or axiom/`sorry` policy. A successful native command with
`sorry` warnings is recorded as successful. Choose stricter native commands when
that is the policy you want. The saved result means **this command exited zero
on these observed inputs**, not that cargo-elicit established API correctness.

## Daily use

- `status` refreshes inputs and shows unfinished selected goals. It never runs the verification command.
- `status --saved` reads saved state immediately; current workspace freshness is unknown.
- `status API` shows a goal's command, source, result, log and changed inputs.
- `status --inventory` shows discovered APIs; `status [API ...] --plan` shows callee-first SCCs and shared dependencies. `--plan --details` expands the graph.
- `status --all` includes goals with current successful command results.
- `verify` reuses matching successful durable receipts. Failed, missing, corrupt or stale results run again.
- `verify --fresh` forces the native command to run again. It does **not** clean native tool caches or impose an independent audit. Put clean-build policy in the command if desired.

Cargo discovery and Rust API/call-graph observations are cached on disk; no
background process or daemon runs. Unchanged Rust observations reuse the cache
while native proof inputs are still checked for freshness. Lean-only edits can
reuse Rust analysis when build scripts, macros and custom module paths do not
make their separation uncertain. Other cases retain conservative invalidation.
Rust/configuration/dependency/environment changes rebuild the relevant saved
observation. Sources needing no definition or alias queries skip rust-analyzer
entirely, including after Rust edits. JSON `analysis_reused` describes the last
input observation; a saved-only read does not perform new analysis.

Input checks cover crate/local-dependency files, configuration, the CLI binary,
resolved command executables, Rust/native-tool observations, inherited
environment hashes and explicit `verification.inputs`. Native build locations
`target`, `.lake`, `.why3find`, and internal `.elicit` state are excluded.
External scripts, toolchain installations and dependencies are not exhaustively
inferred; list additional semantic inputs explicitly. A command that changes
tracked inputs cannot produce a current successful result. Native caches remain
owned by the native tools; cargo-elicit does not validate their contents.

Verification records in `.elicit/evidence/*.json` are intended for Git. They
contain command results, relative project input paths and hashes, separate source
and environment identities, and log references with content hashes. They are
immutable, content-addressed records, not certificates of mathematical correctness
or authenticated worker attestations. A checksum detects damage, not forgery.

```sh
git add src verification elicit.toml .elicit/.gitignore .elicit/evidence
git commit -m "Checkpoint verification"
# In another checkout:
cargo elicit status
```

Use your project's actual proof directory, and commit its toolchain/lock files as
usual. `init` (and `verify` when missing) creates `.elicit/.gitignore`: evidence
JSON and that ignore file are tracked; logs, caches, runtime state and local
history are ignored. `status` never writes Git-tracked CLI records. No additional
handwritten progress file or command is required.

`.elicit/cache/snapshot.json` is a disposable view. Without it, ordinary `status`
rebuilds the view from source, configuration and evidence; `--saved` requires an
existing local snapshot and does not inspect current inputs/environment. Status
separately reports historical outcome, unchanged/changed project inputs, and
matching/different environment. Relocating a checkout preserves history but
conservatively requires verification again. Exact local input/environment binding
is still required for successful receipt reuse. External inputs remain part of
the environment identity and are conservatively rechecked.

Raw command output and diagnostic excerpts live in `.elicit/logs/`, outside Git.
Evidence references their relative paths and hashes; logs can be archived by a
cloud runner without committing them. Missing logs do not erase results or prevent
rechecking; status marks unavailable diagnostic logs. This CLI does not upload
artifacts or manage cloud jobs. `.elicit/history.jsonl` is optional local operation
history, not the source of verification records. There is no notes feature.
Use comments or Markdown as needed; memos outside the project tree do not
invalidate evidence. Project-local comments and Markdown remain tracked. Build
and proof code must not read untracked memos or internal CLI state. Existing
schema-3 evidence/snapshots are not migrated; rerun verification for new records.

Exit codes: 0 for successful requested commands/reads, 1 for command failure,
timeout or changed inputs, 2 for configuration/environment errors, 130 for
interruption. Rust 1.86+ builds the CLI. Discovery uses Cargo, rustc and
rust-analyzer. See [implementation](docs/implementation.md),
[planning](docs/planning.md), [testing](docs/testing.md), and the
[Codex skill](skills/cargo-elicit/SKILL.md). Earlier experiment reports describe
historical implementations, not this command-result interface.
