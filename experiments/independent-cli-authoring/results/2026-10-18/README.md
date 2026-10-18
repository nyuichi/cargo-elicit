# Fresh adler2 authoring comparison, 2026-10-18

See the [investigator report](../../../../docs/experiments/adler2-independent-cli.md), [comparison.json](comparison.json), and [Codex productivity review](PRODUCTIVITY_REVIEW.md).

Both valid arms started from the original release without proofs and completed 5/5 selected APIs. CLI+skill: 1776.600 seconds; native Codex: 1878.678 seconds. This one pair supports near parity, not general or causal speed improvement. Original runtime and manifest bytes were unchanged. See exact properties, supplementary scopes, standard-model trust and excluded APIs in each investigator/review.json.

`with/` and `without/` preserve author reports, proof sources, settings, cumulative usage, observations and raw compressed telemetry. `original/` preserves the original Rust and manifests. `product-evidence.tar.gz` contains durable receipts, generated sources, LLBC and phase logs; its paths are relative to the original `.elicit/evidence/`. The source paths recorded inside receipts retain the validation machine's original paths. `native-authoring-logs.tar.gz` preserves control-native logs/LLBC. `investigator-audit.tar.gz` preserves each separate fresh audit; compiled objects and build caches are omitted. Diagnostic scratch sources are not themselves proof evidence.

Gzip/tar preserves raw file contents, including terminal control characters and trailing whitespace. Rebuild proofs from sources with pinned tools; do not trust imported old .olean files. Authentication files, raw server stderr and model session databases are not included. The initial invalid binary-path attempt is explicitly separate. SHA256SUMS.json binds the published artifacts; the original unedited Codex review is also preserved compressed. Only relative archive links in the readable review were adapted.

The observer's raw reported command-duration sum can be zero because the server supplied zero durationMs values. It is not zero native execution time. Numeric summaries use independently observed item intervals with explicit overlap/coverage limits. Text categories are heuristic mentions, not native-process counts. Cached input and reasoning output are subsets, not extra token categories to add.

A later separate Codex session reviewed the completed evidence. Its usage is in analysis-usage.json and excluded from authoring counters. No production code or skill was tuned during these trials.
