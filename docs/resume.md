# Status and resuming verification

`cargo elicit status` observes inputs and shows current/stale command results,
source locations, command configuration, durable logs, changed paths and recent
milestones. It does not execute the verification command. `status --saved` is a
fast pure saved read with unknown workspace freshness; `status API` focuses one
goal, `--inventory` shows APIs and `--plan` shows dependency planning hints.

Goals with current successful native command results are hidden by default.
`--all` includes them. Counts retain the whole selected scope. New failures or
changed inputs restore goals to the unfinished view. Input changes are compared
to the last saved result, not merely the preceding status call, so repeated
observations do not erase their explanation. Status completion is command-result
completion, not a claim of proof soundness or API coverage.

Continue ordinary native proof work after inspecting saved results. Use verify
at meaningful milestones, not after every edit or native compiler failure.
`verify --fresh` reruns the configured command; native clean-build policy remains
inside that command. Automatic `.elicit/history.jsonl` records observations,
command outcomes, receipt reuse and errors. Status reads a bounded tail. There is
no handoff-file reader or notes directory. Use ordinary comments/Markdown as
needed; keep memos outside the project tree when their edits should not stale
results. Archive `.elicit` explicitly when transferring workspaces.

Rust/Cargo observations are reused from a disposable disk cache when their inputs
match. A Lean proof edit can stale command results without reloading Rust's
semantic workspace; uncertain build/macro dependencies retain conservative
invalidation. JSON `analysis_reused` reports whether the last observation reused
that cache. `--saved` leaves it unchanged and is not a fresh cache lookup.
