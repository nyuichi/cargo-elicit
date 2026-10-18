# Productivity review: fresh adler2 comparison

This pair does **not establish improved productivity** for completing the five requested original APIs. Both arms completed all five, with independently accepted total-correctness results. The cargo-elicit arm finished in 29.61 minutes versus 31.31 minutes for native tooling: an observed saving of 102.08 seconds (5.43%, with/without ratio 0.9457). That is a modest time advantage in one pair, accompanied by higher total token consumption and more command items and failures. The defensible judgment is near parity in completion time, with useful evidence-management capabilities and measurable checking friction, rather than a demonstrated general productivity gain.

This reviews the frozen independent CLI and optional skill, not the removed agent-orchestration workflow. Sources are [comparison.json](comparison.json), [numeric-summary.json](numeric-summary.json), both investigator reviews and both author reports; targeted command-timeline inspection supports the workflow observations below. Preparation records are cited separately.

## Comparable completion and scope

The [with investigator review](with/investigator/review.json) and [without investigator review](without/investigator/review.json) each accept `Adler32::new`, `checksum`, `from_checksum`, `write_slice`, and `adler32_slice` (5/5). The claims include successful execution, arithmetic checksum encoding/decoding, and the scalar recurrence modulo 65521 for arbitrary representable states and slices, including empty-update normalization. Neither imposes a normalized-seed precondition or artificial total-input length cap. Both cover the original optimized compute loops. Original source and manifest hashes agree and remain unchanged.

These are full selected-API outcomes, not partial loop developments or merely successful translation. Intermediate acceptance of the three codec APIs in the product arm was partial progress; supporting loop lemmas alone did not complete the two recurrence APIs. Final independent semantic review and fresh kernel audits, rather than worker self-assessment or status counts, establish completion.

Both also prove codec state/checksum round trips, split-update behavior, and split/save/restore/resume behavior. Split laws explicitly require representability of the concatenated slice. The product arm additionally exposes an unconditional single-state save/restore-before-update law and normalization bridge roots; the control has an explicit empty-update corollary. Theorem counts and packaging differ, so these supplementary results are not extra completed requested APIs or evidence of a proportional productivity increase.

Neither verifies the whole crate: generic BufRead I/O, Hasher laws, derived traits, no_std, and other targets/features remain excluded. Compiler/extractor/translator correctness, standard models and Lean's kernel remain trusted. The reviews check permitted standard axioms and original-body correspondence; specification adequacy still involves investigator judgment. Both authors report six unit tests and four doctests passing, which supplement rather than replace universal proofs.

## Quantitative comparison

All ratios below compare the same completed five-API objective. Timings include author initialization, native verification, product checks where applicable, documentation and final author reporting; independent investigator audits are outside them.

| Metric | With product | Without product | With / without |
| --- | ---: | ---: | ---: |
| Authoring wall time | 1,776.60 s | 1,878.68 s | 0.9457 |
| Total tokens | 20,913,889 | 15,673,784 | 1.3343 |
| Input tokens | 20,858,726 | 15,619,642 | 1.3354 |
| Cached input tokens (within input) | 20,608,384 | 15,347,456 | 1.3428 |
| Noncached input tokens | 250,342 | 272,186 | 0.9197 |
| Cache-write input tokens | 0 | 0 | Undefined (0/0) |
| Output tokens | 55,163 | 54,142 | 1.0189 |
| Reported reasoning output tokens | 15,652 | 18,341 | 0.8534 |
| Completed command items, including failures | 143 | 109 | — |
| Failed command items | 65 | 54 | — |
| Extra occurrences of exact repeated failed commands | 0 | 0 | — |
| Sum of observed command-item intervals | 668.31 s | 315.63 s | — |
| Context compaction events | 0 | 0 | — |

The product used 33.43% more total tokens, 8.03% fewer noncached input tokens, and 1.89% more output tokens. Cached input is a subset of input, not an additional chargeable category to add to total. Reported reasoning tokens are shown separately and not added again to output. These counters do not establish money spent or ChatGPT quota consumption; verified pricing is unavailable, so no monetary estimate is warranted.

Failed-item proportions are 45.5% and 49.5%, respectively. They are shell-item observations, not exact tactic-failure, native-process or wasted-effort rates. Commands can bundle edits, reads and multiple processes. Categories overlap and arise from textual mentions: the control even has three items categorized as “elicit,” despite its native-only workflow. Consequently these are unsuitable as exact product invocation counts. Observer intervals can overlap and cannot be subtracted from wall time to derive thinking time or interpreted as CPU time. App-server durationMs is not a reliable substitute where it reports zero.

Independent investigator audit time was 79.93 s plus 1.66 s additional root inspection for the product, versus 54.49 s for control. Audit layouts and checked-root sets differ; these costs are reported separately and do not constitute a matched checker-speed experiment. The control investigator checked 22 roots, whereas its author report described 21: another reason to use the independent audit and avoid equating root counts.

## Checking, repeated work and visibility

The six listed product check/audit receipts cost 11.086, 15.643, 18.089, 72.038, 74.482 and 73.065 seconds, totaling 264.403 seconds. They show unstable zero acceptance, stable zero acceptance, stable three-API acceptance, then three stable five-API results. This is the listed receipt cost, not an exhaustive accounting of every failed setup/check command. The two final audits alone cost 147.547 seconds. Native-only validation costs are included in control wall time and command intervals, but a comparable isolated receipt-level validation total is unavailable.

The recorded product pipeline has 84 stages, zero cache hits and 246.346 summed stage seconds. There is no measured module-cache benefit here. The comparison attributes changed verification context to progress-report edits and execution-environment changes. Stage times and receipt times overlap in scope and must not be added as independent overhead.

External observer status reads cost only 0.088674 s in the product arm; control observation cost was 0.003857 s. Four product observation rounds each ran text and JSON status. This supports cheap saved-snapshot visibility. It does not establish that intermediate snapshots reflected every current native proof edit. In contrast, six author shell items text-classified as status total 80.339 s, almost entirely one 80.329 s compound item that also ran validation. Calling that 80 seconds of status overhead would be incorrect. Targeted evidence is in [with/command-analysis.json.gz](with/command-analysis.json.gz) and `with/observations.json`.

The product author's report credits inventory IDs, generated contracts, diagnostic sources and phase logs with concrete API mappings and exact signatures/axiom sets. The timeline and acceptance receipts support staged visibility from three accepted APIs to five. That is an observed workflow benefit, although its causal effect on completion time is unknown. The control also maintained progress documentation and independently reproducible validation, so useful progress reporting was possible without the product.

Both arms freely authored native Lean modules and reusable helper lemmas. The product imposed no per-lemma submission protocol, and the investigator accepted its Lean-only suffix factoring through checked correspondence with the actual generated suffix. Thus this trial supports freedom of authoring under the CLI. Both workers still faced native elaboration, casts/coercions, tuple naming and loop reasoning; product bookkeeping did not eliminate those tasks.

The product author reports a check invalidated by a concurrent progress-file edit and waiting on whole-graph checks when local feedback was sufficient. The unstable receipt and repeated full-graph runs support a concrete invalidation/rechecking concern. Its expensive failed suffix-matching approach is a proof-strategy episode, not evidence for adler-specific product tuning. Control friction included extraction root syntax and executable/search-path setup. Zero exact repeated failed commands does not mean zero repeated conceptual work; conversely repeated checking can be necessary validation rather than waste.

## Preparation and limits

[Preparation](preparation.json) and [binary-path repair](binary-path-repair.json) record a separate invalid first product attempt lasting 50.748 s, followed by environment repair, with unchanged runtime sources and no reused proof terms. It is excluded from the valid matched pair and should not be attributed to proof orchestration. If charged separately to the product's practical elapsed cost, valid authoring plus that attempt is 1,827.35 s (30.46 minutes); this excludes other preparation and is not a replacement matched estimate. Preparation upstream tests took 0.249 s and 0.230 s. Overall preparation/repair duration is unavailable.

[Readiness status](readiness-summary.json) records an initial rejected Codex token refresh requiring user reauthentication, then readiness before timed authoring. Authentication duration and token cost are unavailable, not zero. The protocol README retains an earlier blocked-readiness statement; the later ready status and completed records resolve it. No further authentication or live AI call was needed for this review.

Settings matched, budgets were 10,800 seconds per arm, and no human proof hints were supplied. Nonetheless there is only one trial per arm, treatment first, shared warm backend libraries, separate native author caches, and different proof decompositions/tactic choices. Temporary treatment helpers were archived before control (`temporary-isolation.json`), which reduces direct artifact contamination but does not randomize order or equalize warmth. No variance estimate, statistical significance, causal speedup, general robustness or future-crate performance can be inferred. The old 28.40-minute baseline and 89.36-minute product trajectory are historical context, not the fresh matched control or a denominator for new ratios.

## General improvement candidates

- Make native-tool discovery and extraction setup reproducible at entry, with actionable path/root diagnostics. Both authors encountered setup friction.
- Separate progress-document changes from proof-relevant invalidation where sound, while preserving clear freshness indicators and a final complete audit. The unstable check and zero cache hits motivate measuring this boundary.
- Offer a clear local-feedback path alongside explicit full/fresh verification, and explain which inputs triggered rebuilding. This may reduce unnecessary whole-graph waiting without weakening final acceptance.
- Preserve cheap snapshot status, exact API mappings and phase logs, while showing saved evidence time and currentness limits prominently.
- Improve telemetry to distinguish native invocations, compound-shell work, local checks, full checks and audits, with disjoint timing where feasible. Repeat and counterbalance trials before claiming productivity gains.

These are candidates supported by observed friction, not proven optimizations. The present evidence supports successful independent authoring and useful assurance/progress infrastructure; it does not establish a net productivity improvement over native tooling.

Archive note from the investigator: only relative record links were adapted for this repository layout. The original Codex text is preserved in codex-productivity-review-original.md.gz, identified by analysis-usage.json. The blocked-readiness README mentioned above is the initial execution-protocol snapshot; later readiness and this repository protocol document record the successful login.
