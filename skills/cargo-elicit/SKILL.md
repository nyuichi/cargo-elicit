---
name: cargo-elicit
description: Use cargo-elicit to inspect Rust verification obligations, check work, and preserve evidence while authoring with native tools.
---

Use this skill when the user wants systematic verification of a Rust crate or its selected APIs. cargo-elicit is a normal CLI; it does not launch or coordinate AI agents.

1. Inspect `elicit.toml` and the user's mathematical requirements. If a project exists, use `cargo elicit status` / `list` / `show ID` to inspect its saved snapshot. These reads do not establish workspace freshness; use `refresh` when an updated inventory is needed. For a new project, initialize the intended verifier, package/features/target and API selection.
2. Preserve the requested arbitrary-input properties and original runtime implementation. Do not weaken contracts, remove requested goals, or substitute copied/simplified runtime functions to obtain a green result. Ask about ambiguous intended semantics when judgment is needed.
3. Author freely with the native compiler/verifier and your own proof order, supporting lemmas and module organization. No per-lemma plan, submission, revision token or forced return is required. Save useful work normally. Use `check [ID ...]` when its feedback will help, rather than repeatedly collecting status or running clean audits after every edit.
4. After a meaningful amount of work or when resuming, inspect status/receipts as useful. Distinguish checked, audited, stale/unobserved, unsupported and unreviewed states. A compilation/mapping problem may require fixing a declaration or wrapper rather than redoing mathematics.
5. Run `audit [ID ...]` for final fresh evidence. Inspect failed phases, scope, assumptions and uncovered inventory. A theorem about an arbitrary proposition is not automatically a proof of the crate API. Report exactly which requested obligations passed and what remains outside coverage.

Use `review --note ...` only to record an actual judgment that the formal specification matches the intended property; it is independent of proof acceptance. Neither a review nor an author's statement establishes a proof. Use the user's requested completion criterion; do not invent additional bookkeeping work merely to populate the ledger.
