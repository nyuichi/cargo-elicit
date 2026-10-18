# Dependency-first verification planning

Start from the API properties you actually want. Read the callers to understand which properties they need, then use the dependency-first view to develop reusable callee contracts and proofs. A call edge does not mean a separate callee proof is mandatory, or that an existing contract is strong enough for its caller.

```sh
cargo elicit refresh
cargo elicit plan                       # All declared Rust API goals
cargo elicit plan MyType::method other  # Selected obligations or inventory targets
cargo elicit plan MyType::method --json # Same saved facts for external tools
```

`plan` is a pure saved-snapshot read. It does not launch a verifier, update files, add obligations, or claim that the workspace is unchanged. Use `refresh` after editing or when resuming; `check` and `audit` also refresh. Existing snapshots remain readable, but refresh is needed to collect newly added source/contract/module information.

## Frequent progress checks

After a useful batch of proof work, run `check` and then `status` for a concise remaining-work view. `status` hides goals whose fresh audit and semantic review are accepted (unless a current check failed), retains counts, and shows up to five candidate SCCs. Use `status --all` for everything, or `plan` for the full dependency context. JSON uses the same filter with explicit totals. Completed dependencies remain in `plan` and `list`. Nothing disappears from evidence storage. An input or review binding that becomes stale on refresh returns to the remaining-work display.

## Reading the plan

The graph contains local callable inventory targets reachable from the selected API roots, including private helpers. Unrelated functions are excluded. Selectors accept obligation IDs first, then unique inventory IDs, names or aliases; ambiguous or missing selectors fail. Proposition-only obligations are listed separately because no Rust call edges can be inferred for them. Missing inventory bindings are reported explicitly.

Mutually recursive functions are collapsed into strongly connected components (SCCs); a self-call also marks a recursive SCC. The SCC's first lexicographic member is its display ID. Layers run from callees to callers: layer zero has no known outgoing local SCC dependency. Independent SCCs in a layer are displayed by the number of selected roots reaching them, then by ID. This is a structural ordering, not a proof-time estimate or a required workflow.

Each SCC includes:

- Members and recursion status; outgoing callee SCCs and incoming caller SCCs.
- The selected roots that reach it, counted once each, including a root itself. This measures sharing within the selected goals, not whole-crate importance.
- Callee SCCs lacking accepted evidence for at least one member.
- A candidate marker when the SCC is not fully covered and has no known open callee SCC. Candidates can still have unresolved/external calls and unsuitable contracts.

Each function includes:

- Source location, direct callees and callers across the entire saved inventory, plus in-scope callers in JSON.
- Declared obligations with separate check, fresh audit and semantic review states. All contract bindings are shown; the structural summary counts a member covered if any binding has current accepted check or audit evidence. This does **not** establish suitability for another contract or caller.
- Requirement descriptions, coverage limitations, the last recorded result/receipt (possibly stale), and full result details in JSON, including timing, assumptions and restrictions when recorded.
- Aeneas native preconditions, postconditions, theorem binding and generated proposition name when available. An absent contract is distinct from absent obligation registration; private helpers are not automatically registered by planning.
- Unexpanded source counts for loops, if/match branches, unsafe blocks, macros and syntactically indirect calls. Counts include nested bodies and are neither verification-cost estimates nor exhaustive safety analysis. Macro expansion, unsafe function declarations and semantic dispatch are not inferred by these counts.
- Unresolved call sites, external call boundaries with call-site/definition locations, and dangling saved target IDs. These remain visible rather than being treated as proved leaves.

The end of the report also shows local Lean module imports and reverse imports, build scope, changed input paths and global analysis limitations. Module imports are a separate graph: they are **not** theorem-use dependencies and do not show whether a caller actually reuses a callee's proof. A full theorem-dependency graph and automatic contract-strength compatibility checking are not implemented.

## How to use it

1. Select the desired APIs, inspect their requirements, and follow the displayed callers/callees to determine what the helpers need to guarantee.
2. Consider candidate SCCs, especially those shared by several selected roots. Read unresolved/external boundaries before assuming a component is a leaf.
3. Inspect existing contracts and the exact proof scope before reusing a result. A green weak contract is not a proof of a stronger property needed by a caller.
4. Write contracts/proofs with native tools; use `check` at useful milestones. Registration of a helper as an obligation, if wanted, is an explicit `elicit.toml` edit.
5. Re-read the plan after refreshed evidence. Use `audit` for final fresh evidence and record semantic review separately.

The call graph is conservative source indexing plus rust-analyzer definition resolution, not a complete monomorphized compiler call graph. Dynamic dispatch, function pointers, macros, generated code, cfg and external models can leave gaps. Older snapshots did not record external boundaries. Consequently, “no known dependency” is not proof that a function is closed, and the plan never grants verification evidence.
