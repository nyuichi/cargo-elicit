# Completed selected-API verification

All five requested APIs are proved with total successful execution in the native Charon/Aeneas/Lean model. The complete original optimized compute body, six loops, arithmetic, indexing, and vector helpers are covered. Initial state halves are arbitrary representable `u16` values; there is no artificial input-length bound. Empty updates normalize both halves.

Both codec roundtrips are proved unconditionally. Split-update and checksum save/restore/resume equivalence are proved with an explicit concatenated-slice representability precondition. No common-goal obligation remains unfinished and no claim was disproved.

Final `./proofs/validate.sh` passed after fresh extraction and translation, rebuilding all proof modules in dependency order, checking 21 theorem roots for only the permitted axioms, and running 6 upstream unit tests and 4 doctests. Hash checks confirm every original byte of `src/`, `Cargo.toml`, and `Cargo.toml.orig` is preserved. Generated models are unedited. Logs are under `proofs/logs/`.

The theorem inventory, contracts, commands, remaining out-of-scope work, and trusted boundaries are recorded in `verification-inventory.json` and `FINAL_REPORT.md`. Generic BufRead I/O, Hasher laws, derived traits, no_std, alternative targets, and translation-pipeline correctness remain unverified. This is selected-API verification, not whole-crate completion.

Only native proof tools were used, by one worker without subagents. No cargo-elicit tooling or prior experiment material was used.
