# Independent CLI validation, 2026-10-18

See [design and interpretation](../../../docs/experiments/2026-10-18-independent-cli.md) and [summary.json](summary.json).

`aeneas-commands/` contains complete command output and wall time from the final native identity/proposition fixture; `aeneas-events.jsonl` records compilation/cache stages. Regular checks, native attempts (including the refused Kani attempt), successful Kani retry and the virtual-workspace regression are preserved as raw logs. There were zero AI authoring invocations. No end-to-end authoring speedup was measured.

`aeneas-evidence/` preserves the per-run receipts, diagnostic sources and raw phase logs. Receipts retain the original project-relative paths; map `.elicit/evidence/` to this archived directory when reading them.
