# Exploratory Adler-32 comparison

Read the [protocol and current status](../../docs/experiments/adler2-comparison.md) first. No speedup or token result is available until actual authoring trials and independent proof checks finish. The scripts require Python 3.12+, Git, curl, Cargo, authenticated Codex CLI 0.159.x, and the [prepared Aeneas toolchain](../../docs/aeneas.md). They use the inherited model, reasoning and permissions. Live trials consume Codex usage.

```sh
python3 experiments/adler2/prepare.py /tmp/elicit-adler-benchmark
python3 experiments/adler2/run-arm.py without \
  --root /tmp/elicit-adler-benchmark/adler2-without \
  --results /tmp/elicit-adler-benchmark/results/without
python3 experiments/adler2/run-arm.py with \
  --root /tmp/elicit-adler-benchmark/adler2-with \
  --results /tmp/elicit-adler-benchmark/results/with
```

Prepare/warm the shared libraries, Cargo metadata and upstream tests before starting authoring. Do not copy proof files between arms or read the published proof solution. Use fresh cases and result directories for repetitions. Initialization remains inside the with-arm timer. Each arm first makes a tiny live readiness request outside the timer; if that fails, it starts no authoring trial. `ELICIT_BENCH_REAL_CODEX` can explicitly identify the actual Codex executable if PATH already contains a proxy.

The with arm records real text and JSON status plus a target listing after the first proof, around halfway through coverage, at ten minutes when unfinished, and at termination. Those commands use the same environment and Cargo entry point as the run. Their time is included in the ongoing trial and is also reported separately. The without arm records available Lean artifacts without using cargo-elicit; it does not infer proof success from filenames. Observations provide no hints to either worker.

Telemetry records resolved settings, item kinds, turn status and actual token-usage events without storing conversation text. Confirm the resolved model/reasoning match before comparing runs. `summarize.py RESULTS_DIR` groups actual cumulative token usage by thread (including resumed peers) and leaves missing telemetry unavailable. Cached input is a subset of input. `metrics.json` initially leaves tokens and accepted coverage unpopulated; populate them from actual events and independent artifact checks. A completed agent turn, a declaration count, or a zero exit status is not acceptance. Check fresh extraction, original-source identity, exact theorem statements/API mappings, proof rebuilds and transitive axioms for both arms.

Raw run diagnostics and readiness errors are private experiment artifacts: review before publishing them. Publish sanitized evidence and explicit missing metrics. A single pair is exploratory, and overall cargo-elicit completion must remain separate from the five selected core APIs.
