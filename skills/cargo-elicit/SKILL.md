---
name: cargo-elicit
description: Use cargo-elicit to inspect crate verification progress and record native verification command results.
---

Use cargo-elicit alongside ordinary native proof tools. It tracks selected APIs,
command results and input changes. Write specifications and proofs normally;
assess their meaning and assumptions as part of that work.

## Setup

Inspect the requested properties, native project and existing `elicit.toml`.
For a new project, select APIs and the command normally used to validate them:

```sh
cargo elicit init --api identity --directory verification --command lake build
```

Repeat `--api` for more targets. Put other init options before `--command`.
Commands are argv, not shell text; use a fail-fast script for multiple steps.
Existing projects can edit `verification.command` and `verification.directory`
in `elicit.toml`, with per-obligation overrides when needed. A shared command runs
once per request and its result is associated with the selected goals using it.
Add external semantic inputs to `verification.inputs`; dependencies inside native
build directories such as `.lake` are not automatically fingerprinted.
See [README](../../README.md) for configuration details.

## Work

1. Use `cargo elicit status` at the start, after interruption, and when deciding
   what to do next. It observes changes and shows unfinished goals, saved results
   and failure logs. `status API` focuses one goal; `--all` includes successful
   goals; `--inventory` lists APIs. `--saved` reads saved state without checking
   current files. Use JSON for programmatic processing.
2. Use `status [API ...] --plan` when dependency information helps. It shows
   callee-first SCCs and shared callees; `--details` expands the graph. These are
   planning hints, not mandatory proof prerequisites. Prove helpers as needed
   for the requested properties, rather than registering every helper.
3. Iterate with native tools. At useful milestones, save inputs and run
   `cargo elicit verify [API ...]`. Avoid a verify after every edit or an immediate
   status call when verify already provided enough information. Keep active
   inputs unchanged while verification runs. On failure, use its diagnostic/log
   and fix locally.
4. Successful unchanged results can be reused. Use `verify --fresh` only to force
   another command execution; native cache cleaning or stricter validation must
   be part of the configured command. Analysis caching is automatic and needs
   no routine maintenance.

## Results and workspace

Report what the native command established for the requested properties.
cargo-elicit records command success and freshness; it does not independently
validate theorem meaning, assumptions or API coverage. API/command associations
are user-declared.

Progress and execution history are automatic. Use ordinary comments or Markdown
for reasoning if useful. If memo edits should not invalidate results, keep the
memo outside the project tree; project-local comments and Markdown are tracked.
Build/proof code must not read untracked memos or internal CLI state. Preserve
`.elicit` explicitly when transferring a workspace if saved results are needed.
