# Concise CLI feedback and fresh adler2 authoring

The primary implementation scope is concise interactive output, immediate requested-check outcomes plus remaining work/candidates, explicit detail escape hatches, and skill guidance against redundant reads. Differential status checkpoints are deferred.

Two measurements serve different purposes:

1. `compare_saved.py` reads one identical saved snapshot with the previous and concise executables, comparing normal status/plan and JSON output bytes, characters and lines. It never runs a verifier and checks that the snapshot is unchanged. These units are not model tokens; no tokenizer estimate is presented as exact usage.
2. `prepare.py` and `run.py` conduct one new proof-free adler2 2.0.1 trial with the frozen concise CLI and updated skill. The mathematical targets, optional laws and trust policy match the previous five-API experiment. Operational reports use `.elicit/notes/`. Historical authoring measurements are unmatched references, not a fresh control arm or an isolated causal estimate of this display change.

The worker chooses its own mathematics and native-tool workflow, receives no proof hints or historical solutions, and has a three-hour maximum. No product or copied skill/doc changes occur during the valid trial. Source/model correspondence, exact propositions and kernel/axiom evidence must be reviewed after the timed run; turn completion and self-reports alone do not establish coverage.

Record cumulative provider token counters once at the end, separating cached input, noncached input, output and reasoning subsets. Also retain completed command-tool item counts, failure counts, output bytes and observed usage-event counts. Command items are not native subprocess counts; usage notifications are not an exact model-request count. Commands containing CLI text can include heredocs or mixed native output, so any such classification is heuristic and must not be labeled exact CLI invocation/token counts.

The runner preserves failed setup attempts separately. The installed Codex required a writable runtime directory and explicit workspace-write thread policy in this environment. `CODEX_SQLITE_HOME` redirects only runtime state; authentication files are never inspected or copied. Actual network failures are distinguished from authentication errors. Model/provider/settings are recorded from the server response.

```sh
# Use the installed pinned native verifier environment.
python experiments/concise-cli-authoring/prepare.py --destination /path/to/new-run
CODEX_SQLITE_HOME=/path/to/writable/runtime \
python experiments/concise-cli-authoring/run.py with \
  --root /path/to/new-run/case --results /path/to/new-run/results \
  --codex /path/to/codex --seconds 10800
```

The current runner uses an ephemeral, workspace-write Codex thread; no session resumption is tested. Runtime filesystem/network permissions must permit ordinary Codex operation. Preparation installs the frozen CLI into this environment's toolchain bin (saving the previous executable), so login shells use the same binary as the observer. Restore/change that installation only after the trial.

Publish curated metrics and necessary verification records. Raw model sessions, inherited workspace archives, authentication/runtime databases and server stderr are not public artifacts.
