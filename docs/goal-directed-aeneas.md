# Goal-directed Aeneas verification

Select API roots independently of the Cargo build configuration:

```sh
cargo elicit init --tool aeneas --api 'Adler32::write_slice' --api adler32_slice
```

Selectors accept an exact ledger target ID, an inherent/trait method alias such as `Adler32::write_slice`, or a unique function name. Ambiguous or missing selectors fail. Reinitialization without goal flags preserves roots; `init --whole-package` explicitly restores the complete inventory goal. Build scope changes still require a separate working directory. Set `[verification] required = true` in `elicit.toml` for proof authoring. Selected completion does not claim whole-package completion; the full inventory remains visible.

With Aeneas proof verification enabled, an unfinished root starts with an `Integrate` turn: a real attempt at the original body using the normal worker budget. The scheduler continues that caller and its concrete prerequisites before inspecting unrelated roots; a selected API used as a prerequisite follows its own plan. Short `Plan` turns repair blocked work and have a separate `--plan-seconds` limit (180 seconds by default), bounded by the overall and worker limits. An explicit proof plan guides completion instead of draining every leaf in the Rust call graph. The worker records actual missing conditions, explains their use at the caller, and returns to the root after checked prerequisites. Call edges are discovery information, not proof evidence or a complete account of trait/operator dispatch.

Install a plan with `cargo elicit agent plan --file plan.json --revision N`, after fetching the current revision with `agent show`. JSON format:

```json
{"steps":[
  {"id":"bound","goal":"ROOT_ID","kind":"lemma","target":null,
   "reason":"Needed to discharge the caller's multiplication precondition",
   "needs":[],"statement":"EXACT_LEAN_PROPOSITION","theorem":"Checkpoint.bound"},
  {"id":"caller","goal":"ROOT_ID","kind":"integrate","target":"ROOT_ID",
   "reason":"Connect the checked bound to the original loop",
   "needs":["bound"],"statement":null,"theorem":null}
]}
```

Use `kind: "contract"` with an original target ID for helper contracts. Plans reject duplicate IDs, cycles, unknown prerequisites and foreign roots. A contract step targeting another selected root waits for that root's own plan rather than bypassing its prerequisites; root-delegation cycles are rejected too. Explicit step dependencies currently stay within their owning root. Updating a plan replaces its steps, so retain other roots' work. Plans and dependencies express intentions; installing one accepts no proof. They cannot change the selected roots, required contract units, or API verification results.

Put independently compiling checkpoints in `elicit-proofs/aeneas/Lemmas.lean`, importing `ElicitContracts`. `Proofs.lean` can import `Lemmas`. Keep unfinished attempts in `.elicit/cache`, not in these modules. On a stopped worker turn, the official wrapper freshly extracts the Rust model and checks each declared lemma against its exact proposition, including the transitive axiom allowlist. Checked lemmas count as intermediate progress, never API coverage. Source/tool/proof-input changes make their evidence non-current until checked again; changing a step's statement or theorem also invalidates its evidence. Interrupted checks accept no new evidence. Independent roots are checked in a batch, with synchronous kernel registration and a separate exact-type/axiom decision for each root; missing, malformed or invalid roots do not discard valid peers. An unexpected batch-process failure discards that batch. Lemmas are checked before `Proofs.lean`, so a later caller build failure does not discard an already checked lemma batch. A worker task deadline preserves partial files and allows other work; environment and approval failures still stop the run.

## Reusing helper contracts in callers

The pinned Aeneas uses `@[step]` for the `step*` theorem database. `@[spec]` belongs to a separate automation interface. Generate a caller-facing WP theorem after proving an existential execution contract:

```lean
elicit_spec helper_rule from helper_contract
```

The command derives the WP statement using the kernel-checked `ElicitAdapter.contract_to_spec` and registers it with `[step]`. `ElicitAdapter.spec_to_contract` exports WP results in the opposite direction. Put the generated rule before callers in the same module, or place helpers in `Lemmas.lean` and import it from `Proofs.lean`, avoiding circular imports. This does not discharge the helper's preconditions at call sites: their bounds/invariants remain proof obligations.

`status --json` exposes the selected goal, plan, evidence and current checked-lemma count. API results remain in the existing ledger. Pipeline events separately record extraction, translation and Lean module/audit wall times; worker wall time includes both model work and local commands.

This workflow does not guarantee that a proposed decomposition is sufficient or that a hard loop proof completes within a budget. Whole-package verification remains blocked by unsupported trait implementation obligations; selecting APIs changes the declared goal rather than hiding those obligations.

Specification approval records its accepted semantic fingerprint. If transient metadata collection or an edited specification later returns to exactly that approved identity, reconciliation clears only that specification-change flag; changed reference semantics, runtime changes and blockers still require review. A restored approval is not proof evidence for a different global input. Legacy states acquire an approval anchor only while already ready; stale legacy approvals cannot be reconstructed automatically. Checker-only task completion requires the post-reconciliation readiness and verification state, not merely a successful compiler result.

## Attempts and observations

Lifetime task attempts remain visible in the ledger. A turn that adds a new kernel-checked proposition for its own goal earns at most one attempt credit; all propositions observed in the turn are remembered. The attempt cap applies to lifetime attempts minus these credits. The existing no-progress and same-input failure guards, overall task count and time limit still apply. Credits use whitespace-normalized recorded proposition text, excluding plan IDs, needs and theorem selectors; this is conservative bookkeeping, not a semantic-equivalence checker or a proof certificate. Rechecking the same recorded proposition, changing reports, accepting a specification or recording negative audit results earns no credit. Legacy attempts start with zero credits.

The tracked local `Lemmas` module is supported without making operational reports proof inputs. Its contents are still fingerprinted and scanned conservatively; IO or unknown imports retain broad invalidation. A completed normal batch also retains actual unresolved per-root verdicts for ready roots on the exact current input, preventing repeated whole-pipeline checks solely to populate missing results. Missing mappings, build failures and timeouts do not create such observations. Changed proof or model inputs require new checks.

## Finalizing a timed run

In required-verification mode, `--finalize-seconds` defaults to 60 and is capped at one quarter of `--max-seconds`. This time stays inside the overall budget. The run clock includes initial reconciliation and tool probing. Authoring turns use the earlier deadline; after a normal worker deadline, the stopped worker is joined before the latest files are reconciled and checked. No new worker starts once the authoring budget is gone. Finalization also checks pending planned lemmas when no API is ready. A zero reservation disables this allowance. A user interrupt remains an interrupt and accepts no new evidence; saved files remain available on resume. Invalid or uncompleted final checks remain unresolved.
