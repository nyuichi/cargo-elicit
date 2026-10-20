# Native-command authoring trial, 2026-10-20

A fresh Codex author completed the five-API adler2 2.0.1 computation-core benchmark
and common codec/split/save-restore laws. Authoring took 1,894.440 s versus the
historical no-CLI 1,878.678 s. Independent statement/source review and a fresh
kernel audit accepted both scopes. Read the [analysis](../../../../docs/experiments/adler2-native-command.md)
for comparison limits and interpretation.

Files:

- `metrics.json`: final provider cumulative usage, clocks, command counters,
  protected source hashes, initial settings and effective turn sandbox override.
- `numeric-summary.json`, `comparison.json`: derived counters and historical deltas.
- `preparation.json`: frozen product/skill/archive and source identities.
- `cli-usage.json`: seven manually classified real author CLI invocations, selected
  outputs and verification history events. Command durations include native work;
  these are not isolated CLI overhead measurements.
- `observations.json`: passive investigator saved-status observations.
- `command-metadata.json`: timestamps/durations/exits/output byte counts for all
  113 completed command tool items; no raw model prompts or reasoning.
- `author-report.md`: the author's final self-report; acceptance depends on the
  separate investigator record, not this report alone.
- `verification/`, `elicit.toml`: original final reviewed proof artifacts and
  validation configuration. Unused scratch modules are omitted.
- `review.json`, `investigator/`: hash-pinned proof inventory, fresh model/build
  audit, exact root/definition output and semantic coverage judgment.

For reproduction, provision Charon 0.1.293 commit
`de8736f7c995ac54dc490345cef9ea5143c06c62`, Aeneas
`e62dd83d133d9043e7d3628096838b7430f36565`, Lean 4.35.0 and the installed backend's
pinned libraries. These paths are environment-specific; the archived files retain
the paths actually used rather than silently rewriting author inputs. Recreate
`/tmp/elicit-implementation-env` to select those prepared toolchains, or adapt
paths in a separate reproduction copy.

Extract the checksummed upstream archive at
[the original release archive](../../../independent-cli-authoring/results/2026-10-18/original/adler2-2.0.1.crate), copy `verification/` and
`elicit.toml` into its crate root, and run `bash verification/validate.sh`.
The archive SHA-256 is
`320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa`.
Alternatively run the independent audit helper from the repository root:

```sh
source /tmp/elicit-implementation-env
python3 experiments/independent-cli-authoring/audit-native.py \
  --root /path/to/reconstructed/crate \
  --review experiments/native-command-authoring/results/2026-10-20/review.json \
  --out /path/to/new-independent-audit
```

The independent helper uses reviewed proof modules, not the author's Lake cache.
Do not treat successful native commands as theorem/API coverage checks. The
published semantic review explains the separate judgment.

Raw model sessions, reasoning, server stderr, runtime SQLite and authentication
files are deliberately not in this archive. This is a curated results package,
not a complete private runtime backup. The runner protocol is in the parent
experiment directory. No product changes were made during authoring.
