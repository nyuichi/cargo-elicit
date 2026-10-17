# Adler-32 comparison after goal-directed planning

Status: implementation validation in progress; authoring trials have not started.

This experiment tests selected-API roots, caller-driven proof plans, checked intermediate lemma checkpoints and standard Aeneas caller-facing WP rules. It follows the [workflow repair experiment](adler2-follow-up.md); the earlier ordinary control is historical evidence, not this experiment's control.

## Predeclared protocol

Prepare two fresh adler2 2.0.1 cases from the same pinned release and [common five-API task](../../experiments/adler2/TASK.md). Preserve all Rust and manifest bytes, universal representable inputs and original optimized loops. Do not import prior solutions or give human proof hints. Set the product's selected roots to all five common APIs rather than the whole package inventory. Unsupported and out-of-goal obligations remain visible but do not define selected completion.

Run ordinary Codex first, then cargo elicit, with the same resolved model/reasoning/permissions and 1,800-second end-to-end authoring budget, including product initialization and in-run observations. The ordinary runner now automatically continues completed partial turns within that budget, using a predeclared generic reminder and the same thread; it does not supply a proof strategy. It stops early on a five-API completion marker, whose claims must pass the independent audit; the marker itself is not proof evidence. Keep this changed stop policy explicit when comparing historical results. The product receives remaining time and plan context each turn. No authoring or compiler/audit workloads overlap the measured arms.

Observe actual product status/text/JSON/list after first verified helper/lemma, first selected API, halfway coverage, ten minutes, and termination. Include observation costs. Freeze the terminal ledger and events before independent post-trial checks. Keep saved artifacts separate from timed accepted results.

Measure selected full contracts and eventual goal completion; intermediate checked lemmas are not API coverage. Report when a root planning turn starts separately from when checked proof material about the optimized core appears: merely opening a Plan task does not demonstrate loop-proof progress. Describe whether helper proofs are actually used in a checked caller. Break out extraction/translation/Lean pipeline wall times, worker turns, status costs and observed token counters. One pair is exploratory, not a general performance claim.

Independently re-extract the unchanged Rust, inspect original transparent bodies/overflow checks, rebuild saved proof modules and exact contracts/lemma statements, and audit transitive axioms against propext/Classical.choice/Quot.sound. A translated model, theorem count, declared plan dependency or zero worker exit is not acceptance. Record failures and incomplete runs without excluding them based on performance.
