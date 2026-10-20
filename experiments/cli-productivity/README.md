# CLI productivity and interruption evaluation protocol

Status: protocol prepared; no new authoring or interruption trial has been run.
Historical runs are references, not a matched baseline for this change.

## Matched authoring comparison

Compare ordinary Codex using native Charon/Aeneas/Lean (without cargo-elicit)
against the same task with the frozen CLI and its skill. Start both arms without
proofs, generated models, ledgers or prior solutions. Pin crate bytes, selected
APIs and mathematical properties, backend versions, model and effective settings,
permissions, installed libraries and time budget. Freeze CLI/skill identities.
Give both arms equal reporting requirements and equivalent trust requirements;
do not require an artificial CLI-shaped workflow from the native arm. No hints
or solution transfer between arms. Alternate/randomize arm order for repetitions.

Use the existing external authoring telemetry runner, not a product AI launcher.
Retain end-to-end authoring wall time, provider cached/noncached input, total
output and its reasoning subset, completed command items, command output bytes,
and exit codes. Record receipt/check/audit counts separately. Failed searches
are not proof failures; command items are not subprocesses. Do not sum overlapping
command intervals into a model-time decomposition. Independently audit exact
properties, original-source correspondence and transitive axioms outside the
authoring clock. Compare only equally accepted coverage. Report setup failures
and incomplete runs, not only successes. One pair cannot establish a stable gain.

## Separate interruption comparison

Use each arm's own saved workspace at the first successful native compilation of
a useful partial proof, before full coverage. Define the corresponding theorem
and checkpoint criterion before running either arm. If no such checkpoint is
reached, report that outcome; do not choose a favorable later state. Preserve all
files and original telemetry and record the checkpoint time and hashes. Fork the
checkpoint into an interrupted branch and an uninterrupted continuation where
practical. Keep the original long-lived session only for the uninterrupted branch.

Start a genuinely new session for the interrupted branch. Supply the original
task, workspace location and the same arm-specific tool instructions; supply no
old chat, summary or investigator hints. Both arms retain their own saved notes,
proofs and native build artifacts. The CLI arm retains its saved evidence too.
Do not seed either branch with a final proof from a completed run.

Measure from the new session's first task delivery:

- Time and command items to a correct saved-state assessment: completed properties,
  unresolved work, relevant files and next concrete task. Score against checkpoint
  artifacts using a prewritten checklist; do not count an unsupported assertion.
- Time to the next genuinely new accepted property and to equal final coverage.
- Input/output token counters with the same splits as authoring.
- Recompilation/rechecking of unchanged artifacts, repeated work and any lost or
  incorrectly assumed evidence. Distinguish necessary freshness checks from waste.

Compare interrupted versus uninterrupted continuation within each arm before
comparing interruption penalties between arms. Do not combine checkpoint work
and recovery timing into a fresh-authoring result. A quicker self-report is not
proof of more reliable recovery. Publish curated metrics and review judgments;
exclude raw private conversations, credentials and inherited workspace archives.

## Cheap diagnostics gate before another full trial

Run focused Rust tests using compiler-error fixtures: the normal check summary
must expose one concrete source-location error and a durable phase-log path,
deduplicate a shared failure, retain per-goal JSON outcomes, and preserve old
receipt deserialization. No full adler2 verification is required for this gate.

## Diagnostics implementation validation (2026-10-20)

The failure hint is derived from the unsuccessful backend output, prefers the
first error over preceding warnings, and links to the matching saved phase log
(falling back to `verifier.log`). The path is relative to the durable receipt
artifact directory, not the disposable backend cache. Shared log/reason pairs
are displayed once in the bounded operation summary; per-goal JSON retains the
result, phase and diagnostic. Existing receipt diagnostics deserialize without
the new optional fields. Extraction, kernel/axiom checks, input binding and
acceptance criteria are unchanged.

Validation: 22 library tests and 10 mock-backend workflow tests passed;
`cargo clippy --all-targets -- -D warnings` passed. These are software regression
checks, not fresh proof-authoring, adler2 verification or recovery benchmarks.
No productivity improvement is inferred from these tests.

## Four-command and automation follow-up

The current interface has only `init`, `status`, `verify`, and `review`.
Observation/inventory/detail/planning are status modes; use `--saved` for a pure,
fast read. Old subcommands, including hidden entry points, were removed.
Initialization can generate unproved correspondence-ready Lean modules and a
translated type reference. Verification reuses valid input-bound receipts and
records missing requested goals; `--fresh` bypasses reuse for authored targets.
The CLI journals operational milestones so handoffs need only author intent.

[Curated software-smoke measurements](results/2026-10-20-interface-smoke.json)
record 41 normal tests, two small real-Lean cases, source/binary hashes and native
command timings. They are not a new adler2 run or a productivity comparison. The
remaining runtime/observation cost is visible even when no proof stages execute.

Historical authoring runners and tasks retain their frozen interface. Before a
new trial, adapt their command arguments to the current interface and use
`status --saved` for investigator observations, so the observer does not invoke
input discovery or mutate the worker snapshot. Preserve the original task,
coverage and trust policy when comparing arms.

The no-daemon observation-cache latency smoke is recorded in
[`2026-10-20-observation-cache-smoke.json`](results/2026-10-20-observation-cache-smoke.json)
and [its explanation](../../docs/experiments/observation-cache-latency.md).
It measures CLI overhead on tiny fixtures, not proof-authoring productivity.
