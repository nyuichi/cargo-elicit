# Fresh adler2 authoring with the native-command CLI

The current cargo-elicit plus its simplified skill completed a fresh adler2 2.0.1
computation-core verification in **31 min 34.410 sec**. The historical no-CLI
trial took **31 min 18.678 sec**: the new run was **15.762 sec (0.84%) slower**.
Both covered the same five required APIs and four common behavior categories,
with independent source/model, statement and fresh kernel review. This is an
observed near tie, not evidence of a speed improvement or statistical equivalence.

## Measured comparison

| Metric | Historical no CLI | Current CLI + skill | Change |
|---|---:|---:|---:|
| End-to-end authoring | 1,878.678 s | 1,894.440 s | +0.84% |
| Completed command tool items | 109 | 113 | +3.7% |
| Nonzero-exit command items | 54 | 48 | -11.1% |
| Noncached input tokens | 272,186 | 243,940 | -10.4% |
| Cached input tokens | 15,347,456 | 18,987,008 | +23.7% |
| Total input tokens | 15,619,642 | 19,230,948 | +23.1% |
| Output tokens | 54,142 | 62,043 | +14.6% |
| Reasoning output, included above | 18,341 | 23,493 | +28.1% |
| Required APIs independently accepted | 5/5 | 5/5 | Same |
| Common behavior categories accepted | 4/4 | 4/4 | Same |

Usage is the provider's final cumulative `total`, counted once. Cached input is a
subset of total input; reasoning output is a subset of output. Command items can
contain several shell commands. Failure counts include normal exploratory Lean
compilation failures. Neither token totals nor the failure count establish a
causal productivity effect. No pricing or quota assumptions are applied.

## What was actually verified

The five APIs are `Adler32::new`, `Adler32::checksum`,
`Adler32::from_checksum`, `Adler32::write_slice`, and `adler32_slice`.
The update theorem accepts every representable raw state and slice, includes
normalization of an empty update, and proves the usual scalar Adler recurrence
modulo 65,521. There is no normalized-seed hypothesis or artificial length limit.
The common categories are both codec roundtrips, split updates, and save/restore
and resume. Concatenation explicitly requires its length to fit the slice model.

This is the full historical **computation-core benchmark**, not all public APIs
of the crate. Generic `BufRead` I/O, Hasher laws, derived traits, `no_std` and other
targets remain outside scope.

The original manifest and Rust source hashes match the checksummed upstream
release. An investigator independently regenerated the Charon/Aeneas model using
a fresh Cargo target directory, obtained a byte-identical Lean model, rebuilt
all 15 reviewed proof modules plus the generated module in a separate directory,
and kernel/axiom-checked 14 roots. All roots use only `propext`,
`Classical.choice` and `Quot.sound`. Root statements, recurrence definitions,
full seed/slice domains and the connection to the original optimized computation
were separately reviewed. The audit took **62.726 s**, outside authoring time,
as investigator audits were for the historical control. Native command success
alone was not treated as mathematical proof acceptance.

## Actual use and timeline

The author used the CLI seven times: one `init`, three `status`, and three
`verify` invocations. The investigator's additional ten saved status reads
were passive observations, not author operations, and totaled about 0.116 s.
The author edited and compiled Lean directly between milestones.

| Author checkpoint, from turn start | Observation |
|---|---|
| 0:29 | Initialized the five goals and shared native command. |
| 5:45 | Successfully validated three codec goals with a scoped native command. |
| 6:00 | Inspected focused dependency planning with `status --plan --details`. |
| 20:19 | Inspected status while proving the optimized loops. |
| 27:22 | Shared full validation succeeded for the five API goals. |
| 28:10 | Status detected added law/build inputs and marked earlier receipts stale. |
| 30:40 | Final validation, including clean local proof build and 14-root audit, succeeded. |
| 31:34 | Author returned after writing the report outside verification inputs. |

These are command-completion checkpoints, not exact times at which a theorem
first existed. In particular, a shared command is associated with all configured
goals; its success cannot by itself establish coverage of each API.

The three recorded verification durations were 5.470 s, 16.173 s and 41.786 s.
They include extraction, native compilation and the author's own axiom audits;
**their sum is not CLI overhead**. The final script deliberately cleaned its
local Lean package. The three author status command items took 2.957 s, 3.022 s
and 3.119 s, including shell setup. Init took 2.743 s including an `ls`.
Thus current status on this real project is still roughly three seconds, unlike
the much smaller latency fixture. No per-process trace was collected, so this
trial cannot apportion those seconds among source analysis, input hashing and
process startup. Nor can it assign the overall 16-second difference to the CLI.

The final report was outside the crate tree. It caused no additional verification
run. Adding actual Lean laws and changing their build configuration did make old
receipts stale, as expected. There was no input-change race, forced author return,
human proof hint, or tool-managed proof restart.

## Interpretation

The current workflow reached the historical no-CLI speed range while retaining
saved command receipts and change detection. This trial does **not** demonstrate
faster proof authoring. Lower noncached input was offset by more cached input and
output; token efficiency is mixed rather than an established improvement.

The author reported that the dependency view helped locate `compute` and the
vector loader, but trait operations did not reveal the full proof structure.
Most mathematical work remained native Lean work. The ledger correctly surfaced
new proof inputs and the author used it at milestones rather than after every
failed compile. Interruption recovery was not exercised, so robustness under
interruption remains unmeasured.

Remaining concrete observations are that real-project status still takes about
three seconds, and project-wide input changes invalidate otherwise successful
codec receipts. Those are usability/caching limits, not evidence that the author
had to redo the codec mathematics. No product or skill changes were made during
the trial, and no adler-specific tuning is proposed from this single run.

## Conditions and limitations

The author started from the same checksummed adler2 2.0.1 archive, without proofs,
a generated model, old ledger or prior solution. One fresh continuous Codex turn
used `gpt-6.1-sol`, provider `openai`, inherited reasoning/service-tier defaults,
no subagents or human proof hints, and a three-hour cap. Setup, authentication,
backend preparation and later investigator auditing were outside the clock;
initialization, native validation, status and final reporting were inside it.

The frozen CLI binary SHA-256 was
`5a05047a1fef0d2027423ec08b092d7c7855b1ac6c81d46cdd1aaee63558ff65`;
the skill SHA-256 was
`b147c9d50490f8b2f217ed6a194afb7e2c4f21733e20c70a8c25e3a2058dca06`.
Product source identities and the upstream archive hash are recorded with results.

This is one new treatment against the October 18 historical control, **not a new
matched pair**. The installed Codex 0.159.2 failed inner workspace-write execution
before authoring, so the timed turn used its supported external-sandbox mode with
restricted networking, delegating filesystem restrictions to the managed outer
environment. The current approval policy was `never`; the historical control used
`on-request`, inner workspace-write and enabled networking. Initial thread settings
and the effective turn override are both recorded; the initial settings must not
be mistaken for the effective execution policy. Preparation failures are not
included in the valid authoring time. The current task required an outside-tree
final report; the historical task also required progress and inventory files.
These differences, normal model variation and different proof decompositions
prevent causal attribution of a small observed timing difference.

## Records

- [Curated results and reproduction notes](../../experiments/native-command-authoring/results/2026-10-20/README.md)
- [Machine-readable comparison](../../experiments/native-command-authoring/results/2026-10-20/comparison.json)
- [Independent semantic review](../../experiments/native-command-authoring/results/2026-10-20/investigator/semantic-review.json)
- [Author's assessment](../../experiments/native-command-authoring/results/2026-10-20/author-report.md)
- [Historical no-CLI comparison](adler2-independent-cli.md)

Only reviewed proofs, bounded metrics, command metadata, selected CLI output and
investigator logs are published. Raw model sessions, reasoning, runtime databases,
server diagnostics and authentication material remain private.
