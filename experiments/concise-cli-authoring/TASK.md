# Verification task

Verify the computation core of the original adler2 2.0.1 Rust crate using Charon, Aeneas and Lean 4. All report text, contracts, comments and proof files must be in English. Preserve every byte of src/ and the upstream Cargo manifest. Do not copy runtime functions into another Rust crate or replace the optimized implementation with a scalar rewrite. Work as one worker without subagents.

Prove successful execution and functional correctness for these five APIs:

1. Adler32::new: returns state (1, 0).
2. Adler32::checksum: returns 65536*b+a for every representable state.
3. Adler32::from_checksum: decodes every u32 into its low/high 16-bit halves.
4. Adler32::write_slice: for every representable initial state and byte slice, returns exactly the scalar Adler recurrence modulo 65521, normalizing state halves even for an empty update.
5. adler32_slice: equals that recurrence from (1, 0).

The original optimized compute body, all its loops, machine arithmetic, indexing, and helper calls must be covered. No normalized-seed assumption or artificial input-length bound. If the claim is false, preserve a reproducible counterexample and explain rather than silently weakening it.

Separately attempt codec roundtrips, split-update equivalence, and checksum save/restore/resume equivalence. Explicitly require concatenated slice representability for split laws. Generic BufRead I/O, Hasher laws, derived traits, no_std and alternative targets are outside the common goal; report them as unverified. A selected-API result is not whole-crate completion.

Use the installed pinned tools and their standard models. AENEAS_HOME is the installed Aeneas checkout; its prepared Lean backend is backends/lean. Charon extraction should use --preset=aeneas --sysroot=default with the original Cargo manifest and x86_64-unknown-linux-gnu target. Generated models must be fresh and unedited. Lean axioms may only be propext, Classical.choice and Quot.sound. No sorry/admit, project axioms, or native-decision axioms. Syntax checking or a successful translation is not a proof.

Read the provided SKILL.md and tool-docs, author freely with native tools, and persist contracts/proofs under elicit-proofs/aeneas. Initialize exactly the five requested API targets. Use the concise check summary to inspect progress after useful milestones; do not repeat status or plan when the returned summary is sufficient. Use show ID, --verbose or plan --details for a concrete unresolved question. On resumption inspect progress as useful. Audit for final evidence. Semantic review is independent; no review is required to establish machine evidence. Write concise English progress, handoff, final report and verification-inventory.json under .elicit/notes/, which must never be read from build/proof code. Save all semantic artifacts and reproduction scripts before final audit; operational-note updates alone do not require a repeated audit. The inventory identifies theorem roots, mapped Rust API, contract/preconditions, proof modules, actual checker commands, remaining obligations and trusted boundaries. The investigator will independently check the claims.

Do not access published adler/adler2 proof solutions, proofs.rs report source repositories, sibling experiment directories, cargo-elicit test fixture proofs, authentication files or model session databases. You may read this workspace, the installed tools' source/library documentation and Rust source dependencies. Do not commit/reset/clean or change cargo-elicit. No human proof hints will be supplied during the timed trial. Preserve partial work and accurate limitations at the time limit.


Write .elicit/notes/FINAL_REPORT.md in English with: exact independently checkable theorem names and module order; which requested APIs/laws are proved unconditionally, conditional, disproved or unfinished; actual commands; specification assumptions; remaining risks; and a candid self-assessment of productivity. Explain which CLI/skill operations helped or caused overhead, with concrete examples from your own work. Do not invent wall time or token counts: the external observer measures those. Finish requested APIs before optional laws where that is useful, but choose your own proof order. Work autonomously for up to three hours, preserving partial work if unable to finish. Do not intentionally stop after a short partial result while useful independent work remains.
