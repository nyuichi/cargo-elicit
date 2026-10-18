# Architecture and evidence guarantees

Version 0.2 separates obligations, machine evidence, semantic review and observation freshness. The CLI is an author-independent tool. It has no AI client, scheduler, token accounting, task limits or approval/submission protocol.

## Observation

`status`, `list` and `show` deserialize the last saved snapshot. They spawn no processes, inspect no tools, acquire no write locks and make no attempt to silently update evidence. Every output identifies its saved-snapshot nature and observation time. A saved accepted audit is not a claim about an unobserved later workspace.

`refresh` explicitly resolves Cargo scope and uses rust-analyzer plus structural source indexing to discover stable IDs and locations. It reads contract/model declarations without compiling them. Input/configuration fingerprints surround discovery, and Cargo scope is resolved again afterward; raced observations are rejected. Missing verifier installations can be recorded as inspection limitations rather than turning progress viewing into a setup workflow.

Discovery is conservative, not complete rustc HIR extraction. Unknown cfg, macros/attributes, unresolved calls, generated definitions and feature mismatches remain visible. Public types and trait laws do not automatically acquire executable function-contract obligations. Complete declared obligations and whole-crate correctness are different claims.

## Machine evidence

`check` and `audit` refresh inputs automatically. They require no semantic review. Before/after fingerprints cover original sources, local dependencies, scope, formal declarations, environment, installed tools/models and the CLI executable. An input race, interruption, unsupported mapping, unexecuted obligation, extra assumption or unknown scope cannot become successful API evidence.

Formal config includes tool/scope, target bindings, statement and theorem declarations. Editorial descriptions are excluded from machine fingerprints but included in review bindings. User review notes, snapshots and receipts live under `.elicit`, outside compiler inputs. Other project files remain conservatively tracked, including files a build might read.

The trusted acceptance logic is retained from the previous verifier adapters:

- Aeneas maps fresh Charon bodies and generated definitions to the original source and exact definition span. Generated propositions quantify the actual translated arguments, require successful execution and state the supplied pre/postconditions. Lean checks exact theorem types and transitive axioms. Unrelated rejected audit roots do not discard accepted peers; a shared compilation failure can still prevent their audit.
- Kani accepts only native unrestricted, explicitly typed arbitrary-input harnesses calling the original independent primitive free function, with actual contract/safety checks executed. Limited/generic/method/helper cases remain partial or unsupported.
- Creusot requires fresh Coma correspondence and a complete Why3 proof tree for the supported primitive free-function scope, using standard models/drivers. General type-invariant and trait-law generation is not implemented.

Proposition obligations check the exact declared Lean proposition and axiom policy. They do not establish a Rust API contract merely by being associated with the same project. Semantic adequacy of any specification still requires judgment.

## Incremental checking versus fresh audit

For Aeneas, `check` has a persistent checker-owned workspace. Rust extraction and translation still run. A Lean module can reuse its own previously produced objects only when source, imported dependency provenance, tracked context/toolchain and object hashes match. A changed supporting module invalidates its dependents; unchanged independent modules can be reused. Missing local sources, unsupported import modifiers, cycles, source symlinks and shadowing generated/toolchain modules are rejected. Stale local products are pruned. Arbitrary objects from the author's proof tree are never copied in.

Local metaprogramming/I/O indicators use a more conservative whole-input cache key. This does not claim complete dependency inference for arbitrary external I/O. Keep semantic inputs tracked; unrestricted external metaprogram inputs are outside the reproducibility guarantee. Import names currently use ASCII identifiers and ordinary single-line headers (`import` / `public import`, optionally `all`).

The exact-type/transitive-axiom audit module runs on every check. Check evidence is labeled `check`, not `audit`. `audit` uses a new isolated build directory and recompiles all required modules independently, ignoring incremental project products. Kani/Creusot keep their native conservative checking paths rather than claiming the new Lean module cache applies to them.

This cache is a correctness mechanism for ordinary development, not a security boundary against a process deliberately rewriting the checker cache, manifest and ledger. An independently fresh audit avoids relying on those incremental products.

## Persistence and diagnostics

One project lock serializes mutation/check operations. Read commands remain lock-free against atomic snapshot replacement. Original source files are never modified by the CLI. Durable run receipts contain scope, declarations, input-file hashes, result coverage, trusted base and backend decisions. Per-run logs and generated diagnostic sources are copied out of mutable build caches; deleting the cache does not erase them. Evidence is tied to inputs, not to an AI session identity.

`show` provides the target location, generated Aeneas proposition name, saved results, phase and receipt. Historical failed results remain inspectable; newer failures are not hidden by an older success for the same operation/input. Review has its own accepted/needs-work/stale state and never erases a kernel result.
