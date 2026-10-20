# Aeneas and Lean

Use the ordinary Aeneas/Lean project and commands. cargo-elicit does not translate
Rust, generate contract propositions, pin Aeneas/Charon versions, require a fixed
proof layout, compare theorem types or inspect transitive axioms.

```sh
cargo elicit init --api identity --directory verification --command lake build
cargo elicit verify identity
```

If your normal validation also performs Charon extraction and Aeneas translation,
put those steps and Lean compilation in a fail-fast script and configure that
script instead. Write arbitrary native specifications, lemmas and proofs without
elicit-specific comments, wrappers or registration. A command that exits zero
with `sorry` warnings is a successful command result. A Lean compilation error
still fails. For stricter policy use a native validation command that enforces it.

The API/command association is user-declared. elicit does not establish that the
proof refers to the original implementation, covers all inputs or uses acceptable
assumptions. It tracks command outcomes and freshness. Add relevant external
libraries, scripts and toolchain inputs to `verification.inputs`. Native `.lake`
products/dependencies are excluded by default; native Lake owns its cache policy.
`verify --fresh` reruns the command but does not clean Lake or audit proof objects.

Lean import observations, where parsable, are planning hints only. They do not
control which modules compile and are not theorem-use dependencies. Native
project configuration remains the authority for build roots and dependency rules.
