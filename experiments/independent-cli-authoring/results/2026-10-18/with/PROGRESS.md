# Progress

Completed all five requested API proofs, including unconditional successful
execution and functional correctness of the original optimized compute body.
Exactly five API targets are configured. Authored contracts and proofs are under
`elicit-proofs/aeneas/`; generated models remain unedited.

The full check accepted all five APIs (`check-1792331223299.json`). The fresh
audit accepted all five with stable inputs and full-contract scope
(`audit-1792331422261.json`, under `.elicit/evidence/`). Its fresh compilation also
checked codec roundtrips, split-update and save/restore/resume laws, and scalar
specification bridge lemmas. Every inspected transitive axiom set contains only
allowed axioms. Split API laws explicitly require concatenated representability.

All original hashes still match `elicit-proofs/runtime.sha256`. Six upstream
unit tests and four doctests pass. No counterexample or incomplete common-goal
obligation remains. Generic I/O, Hasher laws, derived traits, no_std and other
targets remain unverified. This is a selected-API result, not whole-crate completion.

`FINAL_REPORT.md` and `verification-inventory.json` document roots, specifications,
commands, evidence and trust boundaries. `bash scripts/validate.sh` reproduces
hash checks, upstream tests and a fresh audit. A final audit after these report
updates preserves evidence for the completed files; inspect the latest audit
receipt and saved snapshot for that result.
