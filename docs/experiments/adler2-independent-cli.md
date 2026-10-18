# Fresh adler2 authoring with the independent CLI

The incompatible cargo-elicit 0.2 CLI plus its skill finished a fresh adler2 computation-core proof in **29 min 36.600 sec**. Ordinary Codex finished in **31 min 18.678 sec**. Both results passed a separate fresh extraction/kernel audit and semantic review for the same five APIs and the common codec/split/save-restore laws. Original Rust and manifest bytes were preserved.

This pair demonstrates **near parity**, not a general productivity improvement. The treatment was 102.078 seconds (5.43%) shorter in this pair, but there was only one valid trial per arm, with different proof decompositions. cargo-elicit's module cache had **zero observed hits**. Caching cannot explain the observed time difference.

## Conditions and completion criterion

The subject was the checksummed crates.io adler2 2.0.1 release, with no initial proofs, generated model, old ledger or prior solution provided. Each arm used one fresh Codex session and one continuous authoring turn, sequentially: CLI plus skill, then ordinary Codex. Actual settings matched: `gpt-6.1-sol`, provider `openai`, reasoning/service-tier fields null (inherited defaults), identical workspace-write and approval policies. Each arm had a 10,800-second cap and returned naturally without timeout, human proof hints or forced worker returns. Installed backend libraries were already prepared; author caches and source workspaces were separate.

The frozen product was [acc40d2](https://github.com/nyuichi/cargo-elicit/commit/acc40d2a994bc9c944560c2aeb1b31243be1e278), tree `50da528e6f6dd338093e10d7632f37983eec780e`, release executable SHA-256 `068f4f66a57cabd106bdf40b904c38acaa0a3881af27d55d46960a181c030fe8`. No product code or skill was tuned during the valid trials.

The required properties were successful execution and functional correctness for `new`, `checksum`, `from_checksum`, `write_slice`, and `adler32_slice`, covering the original optimized compute body, loops, checked arithmetic, indexing and helper calls. Seeds were arbitrary representable u16 pairs; slices had no artificial length cap. Empty updates normalize both halves. Common additional behaviors were both codec roundtrips, split-update equivalence and split checksum-save/restore/resume equivalence, with actual concatenated-slice representability explicitly required.

The treatment received the short skill and product documentation and used native tools freely. The control used Charon/Aeneas/Lean directly, without cargo-elicit. Both wrote an English progress/inventory/final report. The investigator supplied no proof advice during the valid authoring turns. Authentication, installation/preparation and the later investigator audits were outside authoring time; initialization, final author validation, reports and progress observations were inside it.

## Observed results

| Metric | CLI + skill | Ordinary Codex |
| --- | ---: | ---: |
| Authoring through final validation/report | 1,776.600 s | 1,878.678 s |
| Independently accepted required APIs | 5/5 | 5/5 |
| Common additional behavior categories | 4/4 | 4/4 |
| Original source/manifest preserved | Yes | Yes |
| Noncached input tokens | 250,342 | 272,186 |
| Cached input tokens | 20,608,384 | 15,347,456 |
| Output tokens | 55,163 | 54,142 |
| Completed command items | 143 | 109 |
| Failed command items | 65 | 54 |
| Repeated failed exact command strings, extra occurrences | 0 | 0 |
| Controlled progress-observer overhead | 0.088674 s | 0.003857 s |

Cached input is a subset of total input. Inclusive input totals were 20,858,726 and 15,619,642 respectively; reasoning tokens are a subset of output, not another amount to add. The treatment used 8.03% less noncached input, 34.28% more cached input and 1.89% more output. This is not a monetary or quota-cost estimate; the ChatGPT-backed CLI's billing/quota weights were not measured.

Command items are app-server observations, not exact native subprocess counts. Shell text categories are heuristic mentions and can overlap. Reported `durationMs` values were zero even for nonzero observed intervals and are not interpreted as zero execution time. Observer item-interval sums are retained as diagnostics, not a disjoint CPU/model-time decomposition. Repeated logical work can exist even when exact command strings differ.

## What happened in the treatment

The first three API checks were accepted at 212.907 seconds. All five were accepted at 1,455.956 seconds (24 min 15.956 sec). The author then completed/report-checked the optional laws and performed final fresh audits. These milestones describe the treatment's evidence collection; the control had no equivalent ledger, so they are not paired intermediate speed comparisons.

The six receipt-reported verifier runs were:

| Run | Reported duration | Result |
| --- | ---: | --- |
| Check 1 | 11.086 s | Input changed during checking; no acceptance |
| Check 2 | 15.643 s | Proof audit failed |
| Check 3 | 18.089 s | Three APIs accepted |
| Check 4 | 72.038 s | Five APIs accepted |
| Fresh audit 1 | 74.482 s | Five APIs accepted |
| Fresh audit 2 | 73.065 s | Five APIs accepted after report updates |

Total receipt-reported verifier duration was 264.403 seconds, about 14.9% of treatment wall time. This includes necessary extraction, translation and proof checking; it is **not all incremental product overhead**, and excludes some outer refresh/startup work. Eight controlled status calls took approximately 0.089 seconds in total. Pure status viewing was not a material latency source.

There were 84 recorded pipeline stages, with 246.346 seconds of summed stage time and **zero cargo-elicit cache hits**. Comparing receipt input-file hashes shows a progress-report change between the first checks, an execution-toolchain environment change during setup, then another progress-report change before the full proof check. Reports/scripts also changed before final audits. The cache key includes the verification context, so unchanged proof modules did not automatically imply reusable keys.

The early input race was a concurrent progress-file edit, acknowledged in the author's report. The later second audit followed updates to `PROGRESS.md`, `FINAL_REPORT.md` and `verification-inventory.json`. This preserved a defensible input binding but caused another 73-second clean rebuild. The proof work itself remained saved; there was no product-controlled task restart, approval sequence or forced model return.

The author reported concrete benefits from inventory IDs, exact generated contract types, preserved phase diagnostics/axiom sets and freedom to author native lemmas without a submission protocol. Reported friction included environment setup, the progress-file race and full-graph checks when local Lean feedback was sufficient. The control author reported root-pattern setup, dependent tuple/coercion elaboration and bit-vector normalization as native-tool friction. Both used `ZMod 65521` invariants with different helper decompositions; their mathematical strategy is an uncontrolled contributor to elapsed time.

## Independent acceptance

The investigator reviewed the exact API roots and scalar definitions, not merely the authors' claimed counts. Both specify normalization followed by the bytewise Adler recurrence and arithmetic `65536*b+a` encoding. Both quantify over the complete requested argument domains and prove successful execution. Split laws require exactly the representability of the concatenated slice.

The treatment was re-extracted and rebuilt in another fresh isolated audit workspace: 79.932 seconds, plus 1.660 seconds for nine supplementary root inspections. The control's fresh native audit took 54.491 seconds, checked a byte-identical regenerated model, source contents, transparent selected bodies and overflow checks, rebuilt its 13 proof modules, and inspected 22 roots. All accepted roots used only `propext`, `Classical.choice` and `Quot.sound`. These investigator audit layouts and root counts differ; their durations are reported separately and do not establish a paired checker speed comparison.

The treatment also names a standalone arbitrary-state save/restore/resume theorem and specification-characterization results. The control explicitly names empty-update normalization. The common behavioral denominator, rather than raw theorem counts, is the comparison basis.

These are selected-API proofs relative to the pinned Rust/Charon/Aeneas/standard-model/Lean trust boundary. Generic BufRead I/O, Hasher laws, derived traits, no_std and other targets remain outside the result. No whole-crate correctness claim is made.

## Interpretation and general follow-up

The previous design's large controller penalty did not recur: a single external author completed the full requested proof, with the CLI acting as an evidence/progress utility. This is useful evidence that the pivot can operate near ordinary Codex speed on this task. It does not establish improved general completion robustness or a causal 5.43% speed gain. The treatment made more command items and failures, not fewer, despite its slightly shorter wall time.

The strongest concrete remaining issue is the boundary between proof inputs and report artifacts. A general documented output location outside verified inputs, and guidance to finalize reports before the one final audit, could avoid this kind of rebuild without benchmark-specific filename exceptions. Consistent effective toolchain identity during setup is another general cache concern. Any such change must preserve actual build-input/provenance guards. No adler-specific mathematical tuning is warranted by these results.

The earlier 28.40-minute ordinary trial and 89.36-minute legacy product trajectory motivated the redesign; they are historical context, not this pair's control. Do not divide the old and new product times to claim a causal threefold speedup.

## Records and reproduction

[Protocol and scripts](../../experiments/independent-cli-authoring/README.md) and [archived results](../../experiments/independent-cli-authoring/results/2026-10-18/README.md) contain settings, source hashes, frozen identity, complete cumulative usage, command telemetry, authored proofs/reports, native validation and investigator judgments. Large raw records are compressed with byte-preserving gzip/tar; hashes identify the archives. Authentication files and server diagnostic stderr are excluded.

Authentication initially failed and was repaired by user-authorized device login. A 50.748-second initial treatment attempt resolved an obsolete installed 0.1 executable and was interrupted as an invalid environment attempt. It was retained separately, then the frozen 0.2 executable was installed and a new proof-free case/session was used. No mathematical work was carried over. This practical preparation failure is not silently hidden or counted as a valid product proof trial.

A separate Codex posthoc analysis is archived as `PRODUCTIVITY_REVIEW.md`. Its time and tokens are not part of either authoring result.
