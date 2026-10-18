# Autonomous authoring acceptance validation

This directory records one frozen-candidate replay of two saved Adler2 proof layouts. It contains no new LLM authoring trial. See the [implementation report](../../../../docs/experiments/autonomous-authoring-implementation.md) and [predeclared cost gates](../../../../docs/experiments/adler2-parity-plan.md).

- `frozen-candidate.json`: release executable hash, exact Rust/script source hashes, toolchain pins and cache policy.
- `ordinary/`: saved ordinary Codex proof sources mechanically adapted to the existing Aeneas contract interface, commands/timings, three status observations, fresh generated sources and final ledger.
- `product/`: saved iteration-9 product proof terms, same acceptance path and evidence.
- `validation/`: regular test suite, final targeted checks, real Lean module/model/axiom test, Clippy, MSRV and formatting.

The regular suite passed 91 tests with 10 opt-in native tests skipped. After final wrapper error-classification and author-context changes, targeted unit/ledger, unmapped/environment rejection and whole-goal protocol checks were rerun; the native collector test passed on the final collector/backend code. Its subsequent author-payload addition was checked by the goal-set protocol test. This distinguishes the full-suite checkpoint from the final targeted reruns rather than claiming an unperformed extra full-suite run.

Original runtime source and manifest hashes are asserted before and after each replay. No project compiled objects are reused. Installed pinned toolchain libraries/digest caches are warm. Replays run sequentially after local test/compiler workloads stop. Proof import/namespace renaming and the direct contract-export wrappers are included in preparation and kernel acceptance; no new mathematical helper proof or weaker input domain is supplied.

`summary.json` contains total elapsed time from preparation through final status, plus every command duration. Five selected APIs are complete; `inventory_complete=false` honestly retains unsupported whole-package discovery/trait/type work. Saved sources, staged review and actual current evidence are separate fields in the status logs. No raw AI conversation or credentials are recorded.

## Measured cost

| Saved artifacts | Total processing | `collect` command | Selected API completion | Conditional 28.40 min + gross cost |
| --- | --- | --- | --- | --- |
| ordinary saved layout | 99.747 s (1.66 min) | 77.546 s | 5/5 | 30.07 min (+5.85%) |
| product saved layout | 216.298 s (3.60 min) | 181.434 s | 5/5 | 32.01 min (+12.69%) |

Both layouts pass the +20% gross-cost screen under equal-authoring arithmetic. The ordinary layout also fits +10%; the slow product layout does not. These are necessary-condition screens, not observed end-to-end ratios or forecasts. No authoring improvement has been measured here. Avoid subtracting the ordinary final compile unless a future workflow actually replaces it; the table conservatively appends the entire replay cost.
