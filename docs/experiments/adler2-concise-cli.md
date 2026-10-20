# Concise CLI feedback: saved-output comparison and fresh adler2 proof authoring

The concise interface substantially reduces output for the same saved state. One new proof-free adler2 trial completed all five selected APIs and optional laws in **31 min 54 s**, with a separate fresh investigator audit accepting all five. This trial does **not** establish improved end-to-end time or token efficiency: relative to the historical CLI trial, noncached input fell 2.1%, cached input rose 9.5%, and output rose 42.1%.

The two measurements answer different questions. The saved-state comparison isolates output formatting. The fresh trial observes a complete proof trajectory with the new utility/skill; historical runs are unmatched references and predate the SCC, notes and concise changes.

## Implementation and saved-state evidence

`check`/`audit` now return requested outcomes, remaining-work counts and structural candidates together. Ordinary status and planning output is bounded; `--verbose`, `show ID`, `plan --details` and JSON expose details. JSON no longer duplicates internal input-file hash maps inside each last-evidence entry; durable snapshots and receipts retain them. The skill prefers concise text for interactive work and discourages immediate redundant status/plan calls after a sufficient check summary. Differential `status --since` checkpoints are deferred.

The [same-snapshot comparison](../../experiments/concise-cli-authoring/results/saved-output/summary.json) uses the previous notes-version executable and the frozen concise executable against exactly the same unchanged saved adler2 snapshot:

| Output | Previous UTF-8 bytes | Concise UTF-8 bytes | Reduction |
| --- | ---: | ---: | ---: |
| `status` | 1,698 | 859 | 49.4% |
| `plan` | 18,708 | 1,078 | 94.2% |
| `status --json` | 52,181 | 16,870 | 67.7% |
| `plan --json` | 86,993 | 49,753 | 42.8% |

These are byte counts, **not model-token estimates**. Output depends on the snapshot; this completed-proof state is not representative of every working phase. A programmatic comparison confirmed equality of all other JSON facts after removing the duplicated input hash maps and additive candidate labels. No proof or verification policy was changed.

## Fresh authoring protocol

[Protocol and scripts](../../experiments/concise-cli-authoring/README.md), [task](../../experiments/concise-cli-authoring/TASK.md), and [curated results](../../experiments/concise-cli-authoring/results/2026-10-20/) are preserved. The worker started from the checksummed upstream adler2 2.0.1 release without proof files or an elicit snapshot. It received no historical solutions, subagents or human proof hints, and had a three-hour maximum. The timed trial used frozen implementation commit `0c53a65f5752eba6016664ac843d051ed59a22f9`, binary SHA-256 `982d97121a9c3d07383022e41b1d3bc0337447de74458db0d89a7ffb575809e8`, and the copied skill/docs. Product/binary contents were unchanged during the trial.

The model was `gpt-6.1-sol`, provider `openai`, reasoning effort/service tier inherited as null; the thread used workspace-write sandboxing and was ephemeral. Setup failures are preserved separately: two read-only-runtime startup failures and a model transport/routing failure with no observed live model response. A writable SQLite location and appropriate runtime/network permissions resolved setup without reading/copying credentials. The successful trial used a fresh proof-free workspace and did not resume a failed proof trajectory.

Targets and scope match the historical common task: successful execution and total functional correctness for `new`, `checksum`, `from_checksum`, `write_slice` and `adler32_slice`, with arbitrary representable seeds/slices, empty-update normalization and no artificial input-length bound. Optional codec roundtrips, split-update and save/restore/resume laws were also attempted. Source and Cargo-manifest bytes were preserved.

## Numerical results

Historical columns are references, **not a new matched control group**:

| Metric | Historical CLI | New concise CLI | Historical ordinary Codex |
| --- | ---: | ---: | ---: |
| Authoring wall time | 29:37 | **31:54** | 31:19 |
| Independently accepted selected APIs | 5/5 | **5/5** | 5/5 |
| Noncached input tokens | 250,342 | **244,984** | 272,186 |
| Cached input tokens | 20,608,384 | **22,572,928** | 15,347,456 |
| Output tokens, including reasoning | 55,163 | **78,399** | 54,142 |
| Reasoning output subset | 15,652 | **35,608** | 18,341 |
| Completed command-tool items | 143 | **152** | 109 |
| Command output UTF-8 bytes | 424,209 | **345,246** | 322,998 |

Provider counters are taken once from the final cumulative usage; cached input and reasoning output are subsets and must not be added twice. Command items are not subprocess or model-request counts. The new trial recorded 68 nonzero-exit items, including expected empty searches as well as proof failures; this is not a count of failed API validations. Usage notifications are not an exact model-request count.

Total command output fell 18.6% versus the historical CLI run, while overall output tokens increased. Of the 23,236 additional output tokens, 19,956 (85.9%) were reported as reasoning. This locates most output growth in the provider counters; it does not prove why reasoning increased. The mathematical trajectory differed, including repairing an early `bv_decide` proof rejected by the axiom policy and troubleshooting the final scalar/loop correspondence. Model reasoning and proof search remain substantial work after shortening utility output.

## Actual utility use and evidence

The timed author produced seven backend `check` receipts and one fresh `audit` receipt. It used concise check feedback during work and made one explicit final `status --all --json` read for programmatic report validation. External observer status reads are recorded separately and were never sent to the author. Commands containing CLI text can also contain heredocs, scripts or source inspection; the lexical classification in the numerical summary is explicitly heuristic.

The author reported that init/plan/show clarified scope and exact contracts, and that concise check summaries avoided repeated status reads. It also reported that axiom rejection and input-race detection helped preserve correctness. One check raced with semantic source edits and was rejected. The final report/inventory lived in `.elicit/notes/`; final operational-report edits did not cause another author audit. These are observed behaviors and a self-assessment, not proof of general productivity gains.

The investigator then independently inspected the specifications and freshly audited the saved proofs outside the authoring clock. All five APIs passed full-contract checking, with no extra assumptions or restrictions. The specifications use exact natural-valued fields: `run` first normalizes both halves modulo 65521, including empty inputs, then folds the byte recurrence using the updated `a`. All API preconditions are True. Both split laws explicitly require concatenated slice representability. The five optional theorem roots were kernel-checked with only `propext`, `Classical.choice` and `Quot.sound`.

The [investigator review](../../experiments/concise-cli-authoring/results/2026-10-20/investigator-review.json), native acceptance records/axiom logs, source hashes, authored Lean sources/contracts and author report are preserved. Finite runtime tests supplement the proofs; they are not counted as proof. Generic I/O, Hasher laws, derived traits, no_std, other targets and whole-crate correctness remain outside coverage.

## Post-trial correction and limitations

Independent inspection found that the first concise implementation applied the remaining-work filter to `check/audit --json` after all goals had accepted semantic reviews. Requested-operation results correctly reported 5/5, but the separate `goals` array could be empty. After the timed run, this was corrected: machine check/audit output and expanded operation views retain all goals; normal status still hides completed goals unless `--all` is requested. A regression test covers completed, audited, reviewed goals. This correction did not change normal concise text, proof acceptance, or the frozen trial binary, and its effect is not included in the timed measurements.

Thirty ordinary Rust tests and Clippy with warnings denied passed after the correction. The new trial and fresh investigator audit supply native five-API evidence; the ordinary suite's opt-in Kani/Creusot/native tests remain separate.

The saved-output reduction is demonstrated for the measured state. Overall proof-authoring token/time improvement is **not demonstrated**. One new trajectory cannot isolate interface effects from mathematics, model variance, documentation changes or runtime differences. No interruption/recovery benefit is measured. Raw sessions, SDK runtime state and inherited workspace archives remain local; public data are curated measurements and necessary proof artifacts.
