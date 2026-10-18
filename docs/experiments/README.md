# Verification experiment records

These records preserve successful and unsuccessful trials, original measurements, independently checked proofs, development changes, limitations and later corrections. They do not establish that cargo-elicit is generally faster than ordinary Codex.

## Start here

- [Why the recorded product trajectory took longer](adler2-latency-analysis.md): disjoint timing analysis of the 28.40-minute ordinary trial versus the 89.36-minute product trajectory, and the corrected resumed-thread token accounting.
- [Complete change and overfitting audit](adler2-change-audit.md): all 26 pre-audit commits and every changed path; identifies benchmark coupling separately from general mechanisms.
- [Final selected-goal result](adler2-iteration-9.md): five selected APIs completed across an initial segment and same-case continuation; exact contracts, audits and the remaining CLI status limitation.

The analysis and correction are preserved in [commit 529441599c628028404f2ea202fdfdca82fcbb7d](https://github.com/nyuichi/cargo-elicit/commit/529441599c628028404f2ea202fdfdca82fcbb7d). Use that immutable snapshot when citing the records that motivated the design review. Later design proposals are not changes retroactively applied to these trials.

- [Astra improvement review](adler2-improvement-review.md): general design candidates, measured evidence, untested hypotheses and what should remain unchanged. This is advice, not an implemented change.

## Important facts and corrections

| Record | What it establishes |
| --- | --- |
| Ordinary Codex, round 5 | 1,704.225159 seconds (28 min 24.225 sec), one authoring turn, five selected API proofs independently accepted; this is **without cargo-elicit** |
| Product, final iteration 9 | 5,361.532036 seconds (89.36 min) across initial and resumed segments; 5,499.868422 seconds (91.66 min) including measured resumption preparation; same frozen binary and case, no attempt reset |
| Final proof endpoint | Five selected APIs, two internal helper contracts, 28 planned lemmas and four additional laws independently checked; original optimized Rust and manifest unchanged |
| CLI limitation | Exit 1 / complete=false remains because the conservative global discovery gate flags excluded derives; selected proofs are current and approved, not whole-crate completion |
| Token correction | Initial and continuation summaries are cumulative counters from the same resumed thread. Correct product total: 66,515,350 input, including 65,408,384 cached subset, and 166,626 output. Earlier 86,690,888 / 85,357,568 / 227,538 totals double-counted the first segment |
| Timing interpretation | Preparation/build validation and separate independent audits are identified separately. Token correction changes no elapsed time or proof verdict |
| Comparison limit | These are separate proof trajectories, not a fresh matched speed study. A user-reported roughly three-hour reference is a different, unmatched reference. No general speedup claim is supported |

## Reports and evidence

| Development stage | Report | Evidence |
| --- | --- | --- |
| Initial live comparison | [Comparison](adler2-comparison.md) | [Experiment directory](../../experiments/adler2/README.md) |
| Workflow repair | [Follow-up](adler2-follow-up.md) | Links in report |
| Selected goals and planning; fresh ordinary 28-minute result | [Goal-planning comparison](adler2-goal-planning.md) | [Initial paired evidence](../../experiments/adler2/results/2026-10-17-goal-planning/README.md) |
| Iteration 6 | [Report](adler2-iteration-6.md) | Links in report |
| Iteration 7 | [Report](adler2-iteration-7.md) | Links in report |
| Iteration 8 | [Report](adler2-iteration-8.md) | [Evidence](../../experiments/adler2/results/2026-10-18-iteration-8/README.md) |
| Iteration 9, initial stopped segment | [Report](adler2-iteration-9.md) | [Initial evidence](../../experiments/adler2/results/2026-10-18-iteration-9/README.md) |
| Iteration 9, same-case completion | [Report](adler2-iteration-9.md#same-binary-continuation-result) | [Final evidence and correction](../../experiments/adler2/results/2026-10-18-iteration-9-continuation/README.md) |
| Change review and ordinary-proof re-audit | [Audit](adler2-change-audit.md) | [Fresh re-audit](../../experiments/adler2/results/2026-10-18-change-audit/README.md), [complete commit/path inventory](adler2-change-audit/change-inventory.json) |
| Latency accounting and counter correction | [Analysis](adler2-latency-analysis.md) | [Calculation program and aggregate data](../../experiments/adler2/results/2026-10-18-latency-analysis/README.md) |

Evidence directories contain measured metrics, actual status observations, frozen state/events, source/binary identities, saved Lean modules, exact-contract/axiom audits and integrity manifests where applicable. Historical acceptance, current timed evidence and outside-clock independent acceptance are distinct; none should be silently substituted for another. Read exact propositions and assumptions before assigning API coverage.

Raw model conversations, credentials and full command telemetry are not public. The detailed aggregate latency calculation reads retained local whitelisted telemetry; its aggregate output and computation are published. A saved proof re-audit verifies correspondence and correctness, not reproducibility of the historical model authoring time.

These directories contain proof solutions. Withhold them from future fresh authoring workers and use unrelated held-out workloads for transfer evaluations. Preserve failed trials and corrections rather than replacing them with a later favorable result.
