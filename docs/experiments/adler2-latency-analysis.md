# Why the recorded cargo-elicit trajectory took longer

This analyzes the retained ordinary run (28.40 minutes) and the final frozen product run plus same-case continuation (89.36 minutes, or 91.66 including resumption preparation). It changes no production code or saved proof. The mathematical endpoint is the same five selected APIs; this is a comparison of different recorded proof trajectories, not a causal ablation.

## Disjoint measured wall-time accounting

The [analysis program and output](../../experiments/adler2/results/2026-10-18-latency-analysis/) recompute intervals from actual worker events and whitelisted commandExecution start/completion timestamps. Overlapping command items are unioned, then clipped to worker windows. Categories sum to each elapsed segment exactly; they are not CPU measurements.

| Observed interval | Ordinary minutes | Product minutes | Increase minutes |
| --- | ---: | ---: | ---: |
| Inside a worker, no active command item | 17.43 | 45.45 | 28.02 |
| Inside a worker, command items active | 10.94 | 30.21 | 19.27 |
| Outside worker windows | 0.03 | 13.70 | 13.67 |
| Total | 28.40 | 89.36 | 60.96 |

No-active-command time includes model output/reasoning, request latency, other tool kinds, handoffs and compaction. It is not all reasoning time. Command-active time includes proof/compiler experiments, file operations and product CLI use, rather than being exclusively Lean time. Outside-worker time includes initialization, official stopped-worker checking, status observations, reconciliation and shutdown. Some command events can lie outside worker windows; those seconds belong to the final row's category, not twice to the command row.

The largest observed increase is inside the model's worker activity: **47.29 additional minutes**, versus **13.67 additional minutes outside workers**. Even removing every second outside product workers would leave roughly 75.66 minutes, far above the retained ordinary 28.40. Removing official checking alone is therefore insufficient to explain or close this recorded difference.

Stage sums are a different, overlapping diagnostic. The product recorded 15 proof pipelines and 25 processing pipelines. Instrumented stages total 724.73 seconds of proof work and 266.31 seconds of processing. Some processing runs inside workers, and observations overlap authoring; do not add these numbers to the disjoint categories.

## What changed the actual work

### More model work, not a three-times-larger final proof

Correct unique-thread usage is:

| Counter | Ordinary | Product initial plus continuation |
| --- | ---: | ---: |
| Input | 28,327,415 | 66,515,350 |
| Cached input subset | 28,067,584 | 65,408,384 |
| Output, including reported reasoning subset | 76,504 | 166,626 |
| Reasoning output subset | 31,477 | 72,101 |
| Started command items | 204 | 362 |
| Worker turns | 1 | 15 |

Product output is approximately 2.18 times the ordinary count; reasoning output is 2.29 times, input 2.35 times. The final persistent proof/model source text is nearly equal in size: ordinary ten modules 49,304 bytes, product three persistent modules 49,500 bytes (generated model, audit modules and lakefile excluded). Consequently, a three-times-larger final deliverable is not the explanation. Extra counted work includes the intermediate exploration, evolving checkpoints, tool calls, plan/protocol handling and model output. The available aggregate telemetry cannot assign an exact output-token count to each of those purposes.

The harness changes the model's workflow: task-specific contracts and approvals, explicit plan and checkpoint obligations, and requests to finish after useful proof checkpoints, followed by official checks and another assigned task. The ordinary worker receives the whole goal and manages its own proof modules in one uninterrupted turn. That is materially different authoring behavior, not the same model proof with a constant wrapper delay added afterward. These workflow differences are plausible contributors; one pair of different generated proofs cannot identify the exact causal cost of each instruction.

### The generated proof also costs more to compile

Independent fresh rebuild timings show the actual artifact difference:

- Ordinary: all eleven generated/proof modules total approximately **53.92 seconds**; its compute-containing Core module costs **25.50 seconds**.
- Product: all six modules total approximately **162.07 seconds**; Proofs.lean alone costs **149.39 seconds**.

The timed product run confirms that this is not only an outside-clock audit effect: the two final Proofs.lean builds take **149.84 and 149.70 seconds**. Adding the decoder changes that same module and rebuilds the already completed optimized-update proof too. Those two builds alone consume approximately five minutes. Earlier Proofs builds, before the completed update theorem existed, cost roughly 2.3–2.4 seconds each.

Both proofs are valid but take different mathematical routes and use different module layouts. The ordinary proof carries a scalar/integer projection invariant through its loops; the product first establishes lane-run/chunk facts and later recombines them with the scalar recurrence in its final caller. The final product caller contains substantial tactic automation. The observed compile cost is established; identifying which tactic or invariant choice causes which seconds would require profiling, not guessing. Broad product extraction versus ordinary selected-root extraction is another structural difference, not evidence of a weaker five-API ordinary contract.

### Many boundaries, but no forced-turn failure in this result

The product has 15 worker turns: 11 write_slice Integrate turns, plus initial convenience-root work, checksum work, one explicit lemma turn and decoder work. Write_slice gets nine checked-progress credits and eventually completes. The ordinary case uses one model turn.

All 15 product workers complete normally: **zero worker-deadline cutoffs and zero Plan tasks**. The continuation resumes the same thread, so blaming lost conversation context or repeated hard 600-second interruptions would be incorrect. The extra boundaries are real, but they are model-completed checkpoints under the task protocol, not eleven forced restarts from scratch.

The resumed interval has no same-input repeated official checks and no report-only invalidation. The initial interval has two checks sharing a source input but different proof plans. The old report invalidation and blocked-retry defects should not be reused as explanations for all of this latest run's cost. Product proof progress is cumulative; returning to write_slice does not itself mean redoing the same mathematics.

## Token-count correction

The earlier iteration 9 report and continuation evidence README incorrectly added two cumulative usage summaries. The first usage notification after resuming exactly equals the initial segment's last total, and the thread identity is the same. The final continuation total already includes the first segment.

Correct cumulative totals are **66,515,350 input / 65,408,384 cached subset / 166,626 output**. The continuation increment alone is **46,339,812 input / 45,459,200 cached / 105,714 output**. Earlier totals **86,690,888 / 85,357,568 / 227,538** double-counted the initial segment and are superseded. The living iteration report and evidence README are annotated accordingly. Raw telemetry, timed metrics, state, contracts and proof/audit artifacts remain unchanged. Elapsed times and five-API proof acceptance are unaffected.

## What can actually be concluded

The recorded gap is explained at the workload level: about fourteen extra minutes outside workers, nineteen extra minutes with commands active inside workers, and twenty-eight extra minutes in other worker activity. There is direct evidence of more model output, more command work, more task boundaries and a slower-to-rebuild generated proof. Calling it solely checker overhead, useless helpers, forced restarts or old invalidation bugs would be inaccurate.

The evidence does not establish how much of the extra exploration is caused by the harness's checkpoint/task instructions versus stochastic proof strategy, context-dependent latency and other trajectory differences. That causal question requires a controlled workflow/prompt ablation or repeated trials. The final fixes demonstrated better preservation and continuation; they did not demonstrate that the harness helps Codex find a proof more efficiently. No new optimization or authoring experiment was performed for this analysis.
