# Dependency-first verification planning

Start from the APIs and properties you actually want. Read callers to understand
which properties they need, then use the dependency view to plan native proof
work. A call edge does not make a separate callee proof mandatory.

```sh
cargo elicit status
cargo elicit status --plan
cargo elicit status --plan MyType::method other
cargo elicit status --plan --saved --json
```

Planning observes current inputs without executing verification commands.
`--saved` reads only saved facts and cannot assert workspace freshness.
`--details` expands the graph. Planning never creates proof obligations or
certifies contracts. Saved success means a configured native command exited zero,
not that elicit proved API coverage or mathematical correctness.

The graph contains local callable targets reachable from the selected roots,
including private helpers. Selectors accept obligation IDs, then unique inventory
IDs, names or aliases. Standalone goals appear separately without inferred Rust
edges. Ambiguous/missing selectors fail instead of silently changing scope.

Mutually recursive functions are collapsed into SCCs; self-calls also mark a
recursive SCC. Layers run from callees to callers. Independent SCCs are ordered
by how many selected roots reach them, then by ID. Shared helpers can be useful
starting points, but this is structural guidance rather than a proof-time estimate.

Each SCC shows members, recursion, outgoing callee SCCs, incoming callers,
selected roots that reach it and open callee SCCs. A candidate is an unfinished
SCC with no known open callee SCC. Unresolved calls can still hide dependencies.
Each function shows its source, direct callees/callers, declared goals, current
native command status, last result/receipt and structural source facts (loops,
branches, unsafe blocks, macros and indirect calls). Successful goals remain in
planning even though ordinary status hides them by default.

Lean module imports and reverse imports are a separate advisory graph. They do
not establish theorem-use dependencies, determine build roots or gate command
execution. Native tools remain responsible for compilation and dependencies.
The Rust graph combines source indexing and rust-analyzer resolution; dynamic
dispatch, macros, generated code, cfg and external calls can leave gaps.

After a useful batch, use verify; its remaining-work summary often avoids another
status round trip. Use status when new observations are needed, and `--all` to
include current successful goals. An input change makes affected saved command
bindings stale conservatively; elicit does not infer which mathematical proofs
must be redone. Decide specification strength and proof decomposition as normal
native verification work.
