---
name: cargo-elicit
description: Track crate verification progress and input-bound native command results with init, status and verify.
---

cargo-elicit is a normal utility for people and AI tools. It does not start an
AI, prescribe proof strategies, or judge whether a theorem proves an API correct.

1. Inspect the user's requested properties and native verification project. For
   a new project use `cargo elicit init --api API --command PROGRAM ARG...`;
   repeat `--api` to select more APIs. Put other init options before `--command`.
   Use `--directory PATH` if the native command runs in a subdirectory. Init
   discovers APIs and writes schema-3 configuration, not proof scaffolds.
2. Configure the ordinary native verification command you would use without
   cargo-elicit (for example `lake build` or a fail-fast translation/build script).
   `verification.command` is argv, not shell text. Per-obligation command and
   directory overrides are available. Shared commands run once per request.
   Add external semantic dependencies/scripts/toolchain files to
   `verification.inputs` as needed. Tool versions are not pinned by elicit.
3. On resumption use `cargo elicit status` for current input observations, saved
   command results, changed paths, failures/logs and recorded milestones.
   Current observations automatically reuse disk-cached Rust/Cargo analysis
   when its inputs match; no daemon runs. Lean-only edits can retain that cache
   while invalidating native command results. Build scripts/macros/custom module
   paths keep conservative analysis invalidation. Do not manually refresh or
   delete the cache as a routine step. `status --saved` is a fast saved read with
   unknown workspace freshness.
   Use `status API` for focused details, `status --inventory` for API discovery,
   and `status [API ...] --plan` for dependency-first SCCs and shared callees.
   `--plan --details` expands graph details. Call/import edges are advisory,
   not logical proof prerequisites; do not prove every helper automatically.
4. Author specifications and proofs normally with native tools. No elicit
   contract syntax, generated proposition, wrapper, theorem registration,
   approval text, or handoff file is required. Optional statement/theorem fields
   are metadata only. Whether the chosen theorem covers the requested API and
   whether its assumptions are acceptable remain ordinary verification work.
5. At useful milestones use `cargo elicit verify [API ...]` to execute native
   commands and save input-bound results. Valid successful receipts may be
   reused; `verify --fresh` forces execution without cleaning native caches.
   Native clean-build or strict-warning policies belong in the chosen command.
   Do not run verify after every edit or immediately follow its summary with
   another status call unless a new observation is needed.
6. Save inputs before verification and do not edit them while a command runs.
   On failure use the returned diagnostic/log, fix locally and retry at a useful
   milestone. Missing executables, nonzero exits, timeouts, interruptions, input
   races and invalid durable receipts remain explicit failures.
7. No handoff or notes format is read or required. Use source comments as useful;
   comments in tracked files count as input changes. If changing a memo should
   not invalidate results, put it in a separate Markdown file outside the project
   tree. Merely using Markdown inside the project does not exclude it. Never
   read untracked memos or internal CLI state from build/proof code. Archive
   `.elicit` explicitly when transferring workspaces.
8. Describe results accurately: elicit records native command success, not an
   independent proof audit. It does not inspect `sorry`, custom axioms, theorem
   type correspondence or full Rust API coverage. Choose a stricter native
   command if required by the user; do not invent an additional CLI approval
   ritual. There is no `review` or `finish` command.

Only `init`, `status`, and `verify` exist. Saved receipt modes named `run` and
`rerun` distinguish ordinary and forced command runs; neither is an additional
command or a semantic certification. Status hides current successful goals by
default; use `--all` to include them. Use JSON for actual programmatic processing.
