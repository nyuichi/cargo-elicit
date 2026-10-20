# Fresh adler2 authoring with native-command cargo-elicit

One fresh Codex author starts from the checksummed upstream adler2 2.0.1 release,
without proof artifacts or historical solutions. The frozen current CLI and skill
are ordinary utilities. Goals match the historical no-CLI trial: the five
computation-core APIs with arbitrary representable seeds/slices and the common
optional codec/split/save-restore laws. The cap is three hours; one continuous
session chooses its own mathematics and receives no proof hints or subagents.

`prepare.py` copies the installed CLI, current skill/docs and protocol, runs
upstream library tests, and records source/binary identities. `run.py` observes
provider cumulative token usage and completed command items. Investigator status
reads use `--saved` and do not refresh inputs, lock the workspace or run proofs.
The author is instructed to inspect progress at meaningful checkpoints. Reports
are outside the crate tree; no product handoff/notes feature is used.

Native command success is not proof acceptance. The investigator must separately
check the final source/model correspondence, root statements, fresh compilation
and transitive assumptions against the common historical benchmark criterion.
Author self-reports and elicit completion counters alone are insufficient.

Comparison uses the previous no-CLI result, not a new matched control arm.
Record model/provider/reasoning/service settings and effective sandbox policy.
The installed Codex 0.159.2 currently fails inner workspace-write execution with
an app-server socket-directory permission error; its legacy backend also fails.
An external-sandbox trial delegates filesystem isolation to this managed execution
environment. That differs from the historical inner workspace-write setting and
must be disclosed rather than described as a controlled causal comparison.
Preflight/runtime preparation costs are outside the timed authoring turn, as
installation/authentication preparation were in the historical comparison.

Use a writable private Codex runtime home outside the trial. Existing credentials
may be referenced by the native CLI; never inspect/copy/publish their contents.
Runtime/session databases, raw model sessions, server stderr and inherited
workspace archives are private. Publish only reviewed proof artifacts, bounded
curated metrics, source hashes and investigator verification records.

```sh
source /tmp/elicit-implementation-env
python experiments/native-command-authoring/prepare.py --destination /path/to/fresh-run
# Set the private writable runtime paths used by the local Codex installation.
python experiments/native-command-authoring/run.py with \
  --root /path/to/fresh-run/case --results /path/to/fresh-run/results \
  --codex /opt/codex/bin/codex --model gpt-6.1-sol --seconds 10800 \
  --external-sandbox
```
