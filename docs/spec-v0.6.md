# cargo-elicit — Specification v0.6

*Agentic specification and verification for Rust crates*

## 1. Purpose and scope of the first release

`cargo-elicit` is a Cargo subcommand that lets an AI agent continuously develop formal specifications and proofs for Rust crates. It selects work targets, persists progress, tracks the effects of changes, and reuses verification results.

The first release targets existing local Rust projects, the Codex app-server, a single worker, and one verifier: either Creusot or Kani. Contracts, type invariants, and proof code are written in the original source tree.

Useful specifications are a priority, but the tool does not automatically guarantee that AI-written specifications correctly express the user's intent.

## 2. Configuration and completion criteria

`elicit.toml` contains three settings:

```toml
[agent]
provider = "codex"

[verification]
tool = "creusot" # "creusot" or "kani"
required = false
```

The model, reasoning effort, and authentication use Codex's own configuration. The verifier and toolchain are detected, and supported versions and required capabilities are checked before execution. Changing the verifier partway through is unsupported in the first release and produces an error without deleting existing state.

### Specification mode: `required = false`

Develop contracts and invariants for the targets, and check their syntax, types, and references with the selected verifier. Proof search is not required. With Creusot, all definitions may be trusted.

However, distinguish between a specification being written, the verifier being able to process it, and the implementation being proved to satisfy it. Properties that could not be expressed, or specifications that could not be processed, must not be hidden as completed work.

### Verification-required mode: `required = true`

In addition to specification work, completion requires **verification of the entire written contract, with dependencies on unproved assumptions within the target project resolved**.

Successful results limited to particular types or input ranges, or results that still depend on unproved helpers or lemmas, are retained but do not count as verification completion. The tool must not automatically narrow the verification scope or move unproved parts out of scope to declare completion.

The trusted base in use, including the verifier, logical foundations, and standard models, must be explicitly recorded. Project-specific trusted declarations and custom external specifications must not automatically be promoted into the trusted base. Legitimate preconditions and trait requirements stated in contracts must be distinguished from unresolved assumptions about the implementation.

If some targets cannot be proved, the tool may preserve the work and exit as incomplete. Changing `false` to `true` reuses existing specifications.

## 3. Initialization and analysis targets

If no configuration exists, initialize it as follows:

```bash
cargo elicit init --tool creusot
```

The initial setting is `required = false`. Existing configuration and the ledger are not overwritten or deleted. Running `init` again reconciles and updates the ledger against the current sources.

The scope is **one package and one build configuration**. Package, features, and target can be specified at initialization; resolved values for omitted options are also saved and displayed. If multiple packages are candidates, explicit selection is required. The tool must not claim coverage of all features or all targets. Because rust-analyzer's semantic analysis also represents a crate with cfg and features applied, analysis and verification must use the same scope. [1]

| Target | Work |
|---|---|
| Struct / Enum | Write type invariants |
| Trait | Write method contracts and laws that implementations must satisfy |
| Function / Method | Write contracts and verify them as configured |
| Trait Impl | Associate implementation methods with trait requirements and check conformance |

Specifications of types and traits are not independent proof tasks. However, invariant preservation and laws spanning multiple methods must be connected to the necessary implementation proof obligations and auxiliary harnesses. Do not assume that Creusot and Kani support identical capabilities. [2]

Public APIs are the primary goal, with necessary internal functions, types, and lemmas also managed. Distinguish public APIs, internal targets, and added auxiliary proof definitions. Auxiliary definitions must not be included in the progress denominator.

Use rust-analyzer for analysis. Account for re-exports, treating public names for the same definition as aliases. IDs must not depend solely on line numbers or display names; they must distinguish impls and definition origins. If correspondence is unclear because of renaming, deletion, macro generation, or similar changes, do not silently inherit earlier completion state. Display incomplete analysis and a need for review.

Cache the call graph and analysis results. Reuse analysis state during a run, updating it when sources or analysis settings change. Unresolved calls must not be treated as having no dependencies.

## 4. Artifacts and verifier operation

**Edit contracts and proofs directly in the original sources being verified.** Duplicating functions into a separate project and verifying only those copies is prohibited. Harnesses that call the original functions, ghost code, lemmas, and model definitions may be added.

Do not change the runtime behavior of the original implementation. If an implementation change is needed, report it as an issue. Implementation bodies added or changed by the agent are subject to diff checks, and a state containing unapproved changes must not count as goal completion.

Weakening a contract solely to make verification pass is prohibited. Incorrect specifications may be corrected with a recorded reason. If a verification task discovers a specification problem, return it to a review task.

### Verifier adapters

Adapters handle notation, readiness checks, target enumeration, syntax and type checking of specifications, verification execution, result parsing, and input fingerprints. Unsupported capabilities must not be silently omitted.

Creusot trusted declarations must not produce successful body-proof results. For Kani, record harnesses, type instantiations, and additional input restrictions; success for one selected harness must not be promoted to success for an entire generic API. The name `stub_verified` alone is not evidence that a dependency is proved. [3][4]

### Recording official verification results

Accept official results **only from executions performed and collected by cargo-elicit's verification wrapper**. Agents may run experiments directly, but a self-reported success alone cannot update the ledger.

The wrapper fixes the targets and input state, stops agent editing during execution, and checks the inputs afterward. Runs whose inputs changed during execution are not accepted as current results.

Distinguish success, counterexamples or unresolved obligations, timeouts, interruptions, build or environment errors, and targets not executed or skipped. An exit code of 0 or zero proof obligations alone must not count as success. Results whose required obligation coverage, assumptions, or input restrictions cannot be established are retained with unknown scope.

This mechanism prevents incorrect recording. It is not a security boundary against an agent that can modify the same working tree or ledger.

## 5. Ledger and verification results

Keep API state separate from verifier execution results. Do not introduce numerous progress states or a general-purpose task management system.

| Record | Main contents |
|---|---|
| Target | ID, kind, public/internal classification, source location, specification completion, specification fingerprint, known specification dependencies, notes |
| Verification result | Target ID, actual harnesses and proof obligations, verification scope, assumptions, input fingerprint, tool/configuration, outcome |
| Run state | Current task, starting inputs, Codex thread, last outcome, retry suppression information |

An API may have multiple verification results. Track required verification units and their coverage. Derive `verified` from **whether current results satisfy the completion criteria in Section 2**, rather than exposing it as a manually set boolean.

Specification completion means that specification work on the current contents has reached a stable point and the written specification can be processed. It does not mechanically guarantee correspondence to intent. Types and traits do not display an independent `verified` status.

### Stale rules

Stale means that review is required; it does not mean deleting artifacts. Track specifications and verification results separately, preserving reasons.

| Change or event | Action |
|---|---|
| A target's contract, invariant, or logical definition used in specifications changes | Review that specification and mark corresponding proofs stale |
| A known specification dependency changes | Review directly dependent specifications |
| An implementation changes | Check consistency with its specification and mark affected verification results stale |
| A verifier, configuration, or dependency input changes | Mark affected verification results stale |
| Verification fails or times out | Record the failure without immediately concluding that the specification is wrong |
| A concrete specification problem is reported | Return the target to review even before a fix, and exclude its proof from current completion |
| Only notes or logs change | Do not invalidate specifications or proofs |

**If a specification is stale, its corresponding proofs are stale too.** Preserve existing contracts, proof code, and historical results.

### Stopping change propagation

If B uses A's specification and C uses B's specification, review B first after A changes.

If B's specification does not need to change, update only the recorded fingerprint of A and stop specification-change propagation through B. Review C if B changes. If C also directly references A, handle that dependency separately.

While B is awaiting review, proofs that assume B have unresolved assumptions, but do not preemptively regenerate C's specification. Once review restores the assumptions, saved results can be reused if verification inputs also match.

Agents may declare specification dependencies. However, **the absence of a declaration must not justify verification cache reuse**. Successful verification also does not automatically waive specification review that is semantically necessary.

### Verification cache

Use dependency-input fingerprints from the verifier if it provides sufficient information. Otherwise, include the package, local dependencies, manifests and lockfiles, build settings, tools, models, and other inputs broadly. Restrict reuse of results with external inputs that cannot be tracked.

Exclude the ledger, logs, caches, and execution timestamps from verification inputs. Include source changes made for proofs. Mutually dependent proofs must not be considered complete solely through circular unproved assumptions.

A change to a broad fingerprint must not immediately trigger AI regeneration of every existing API. Batch the required verification work and rerun it once sources have stabilized. Overall completion requires results corresponding to the same current state.

## 6. Work control

The scheduler selects tasks and tells the agent what to do. The agent decides how to write specifications and proofs.

There are three task kinds: specification, review, and verification. Prioritize repairing specifications awaiting review. Normally prepare type and trait specifications first, then follow the call graph in callee-first order. Group cycles into strongly connected components (SCCs), and select equal-priority targets in stable ID order.

However, unfinished type or trait work must not be an absolute barrier for the whole crate. Necessary model functions and similar definitions may be prepared first.

Verification-required mode generally follows **specify one target group → verify it → move to the next group**. A large backlog of unspecified targets must not prevent verification from starting. If a proof is difficult, save a handoff and continue with other targets.

Do not unconditionally reselect the same failed task on the same inputs. Return to it only after related inputs change, an explicit retry with a different proof strategy, or a user retry request. Apply finite total-attempt and no-progress limits to each task, and a work or time limit to the entire run. Changing auxiliary files alone must not allow limits to reset indefinitely. Define defaults in the implementation, display them in help, and expose necessary overrides as CLI arguments.

Specification mode does not require new proofs as mandatory tasks. Verification-required mode finishes by checking targets that lack valid results in a batch and reporting completion or unresolved work.

## 7. Codex app-server integration

The first release uses stdio, a single thread, and at most one active turn. Encapsulate connection initialization, thread creation and resumption, and turn start, completion, and interruption in the adapter. A `turn/completed` notification alone is not success; check both the final status and actual accepted work. [5]

Shared instructions include verifier notation, the mode, editing original sources, specification-change rules, and ledger tool usage. Each turn adds only the task kind, targets, related specifications, completion criteria, and input revision. Do not resend a long specification document for every task.

Reuse a thread across tasks. Resumption from the ledger must work after compaction, reconnection, or thread loss. Session reuse does not guarantee Prompt Cache hits. Record available input, cached-input, and output token counts, along with time, for evaluation. Do not retain unnecessary history solely to increase cache hit rates. [6]

The agent CLI provides target queries, specification and review submissions, dependency and issue reporting, and execution of the verification wrapper. Writes are accepted only after checking the input revision. Agent reports alone cannot promote a target to verified.

Approval requests follow existing Codex permission settings. Present them in an interactive terminal. During unattended execution, stop with a recorded reason if required approval cannot be obtained. Do not approve automatically or expand permissions. The first release does not allow parallel subagents that perform writes.

## 8. CLI, persistence, and interruption

| Command | Purpose |
|---|---|
| `cargo elicit init` | Generate configuration, analyze targets, and reconcile with an existing ledger |
| `cargo elicit run` | Execute or resume work from saved state |
| `cargo elicit status` | Display scope, specification and verification counts, unresolved work, and the current task |
| `cargo elicit list` | List public/internal targets, specification status, proof scope, assumptions, and stale reasons |

Explicit retries use `run --retry`. Provide machine-readable output for `status` and `list`, separating type/trait specification counts from function/method verification counts. Partial proofs must not be included in completed-verification counts. Reconcile current sources when displaying state and when resuming, so externally edited code does not retain an outdated completion display.

```text
my-crate/
├── Cargo.toml
├── elicit.toml
├── src/                    # Original implementation, contracts, and proof code
└── .elicit/
    ├── state.toml           # Targets, progress, result summaries, and fingerprints
    ├── runtime/            # Run locks, current task, thread information, and events
    └── cache/              # Analysis data, detailed logs, and regenerable verification data
```

Do not create a separate state file per API. `state.toml` has a schema version and can be tracked in Git. Keep successful-result summaries separate from the cache so deleting detailed logs alone does not lose progress. However, results must be rechecked if information required to decide reuse has been lost.

Update the ledger through the dedicated CLI, using locks and atomic replacement. Separately prevent multiple `run` processes in the same working tree. Do not persist credentials in logs or conversation history.

On interruption, stop child processes where possible and save the current task and work. After a crash, resume by reconciling sources; started but unfinalized verification must not be treated as successful. Preserve the user's uncommitted changes without automatically committing, resetting, or cleaning them.

The `run` exit codes are 0 for goal completion, 1 for unresolved work, 2 for configuration or environment errors, and 130 for interruption. `status` and `list` return 0 on a successful read, regardless of unresolved work.

## 9. Implementation acceptance criteria and future features

Acceptance of the first release requires checking both modes and interruption/resumption with a small fixture for each of Creusot and Kani. In particular, the following regression tests are required:

- Unexecuted targets, obligations omitted through `trusted`, and success restricted to particular types or input ranges must not be promoted to verification completion.
- Results with unproved assumptions about internal helpers are retained but do not complete verification-required mode.
- If B does not need to change after A changes, specification regeneration for C must not be requested through B. Proof validity is checked separately.
- Changes to undeclared dependencies must prevent invalid cache reuse, while note updates must not invalidate the cache.
- Stale specifications or proofs must not delete code. Reinitialization, crashes, and interruptions must not lose work.
- Prevent infinite retries of the same failure, verification starvation due to unspecified tasks, and conflicting ledger updates.

Parallel workers, isolation and sequential integration through Git worktrees, the `cargo elicit top` TUI, proofs.rs integration, cloud execution, and support for other agents are future features. Do not implement them in the first release. Keep the scheduler separate from agent connections to support later extension.

## References

These sources describe notation and protocols for external tools. cargo-elicit's own behavior is defined by this specification.

[1] rust-analyzer HIR: https://rust-lang.github.io/rust-analyzer/hir/index.html

[2] Creusot — Type invariants: https://guide.creusot.rs/type_invariants.html

[3] Creusot contracts — macros / trusted: https://docs.rs/creusot-contracts/latest/creusot_contracts/macros/index.html

[4] Kani — Function contracts: https://model-checking.github.io/kani/crates/doc/kani/contracts/index.html

[5] OpenAI — Codex app-server: https://developers.openai.com/codex/app-server/

[6] OpenAI — Prompt caching: https://developers.openai.com/api/docs/guides/prompt-caching

## Extension: Aeneas / Lean 4

The implementation additionally supports `verification.tool = "aeneas"` with Lean 4 as its initial proof backend. The original v0.6 Creusot/Kani requirements above remain applicable; this extension uses fresh Charon extraction, Aeneas translation, typed contracts, kernel-checked Lean proofs, and a transitive axiom audit rather than treating translation as proof. Persistent proof files stay in the selected package. See [Aeneas setup and contract format](aeneas.md) for exact toolchain pins, configuration, proof obligations, invalidation, and initial coverage limits. Other Aeneas backends remain outside this extension.
