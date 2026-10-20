# CLI observation-cache latency smoke

A disposable disk cache replaces repeated Cargo discovery and Rust inventory
reconstruction when observation inputs match. No daemon is used. Sources needing
no definition/alias queries also skip rust-analyzer on a cache miss.

Release builds were compared on two generated dependency-free crates: one pure
function, and one public function calling a local helper. The native validation
command was `true`, isolating CLI overhead. Before/after binary SHA-256 identities,
source hashes, fixture source, individual samples and limitations are in the
[curated results](../../experiments/cli-productivity/results/2026-10-20-observation-cache-smoke.json).
The before build is commit `88a17e2`. This is not an AI-authoring experiment or
full adler2 verification. Runs were sequential on a warm local filesystem.

| Operation | Before median | After median |
| --- | ---: | ---: |
| Observing status, no-call fixture | 2,450 ms | 208 ms |
| Observing status, local-call fixture | 2,371 ms | 209 ms |
| Verify reusing command results, no-call fixture | 2,486 ms | 300 ms |
| Status after a Lean edit, local-call fixture | Not separately measured | 210 ms |
| Status after a Rust body edit, no-call fixture | Not separately measured | 470 ms |
| Status after a Rust body edit, local-call fixture | Not separately measured | 2,550 ms |

Repeated cases have five samples; Rust-body-edit cases have three. Saved-only
status stayed approximately 1–1.5 ms. Lean edits retained the analysis cache while
invalidating command results. Rust edits invalidated the analysis cache. New
Rust call targets were separately checked in integration tests.

The warm status reduction is about 11–12 times and reused verify about 8 times.
This does not establish similar performance on large crates. Cold semantic
analysis still costs seconds. Build scripts, macro invocations and custom module
paths keep conservative invalidation, so Lean edits can require Rust reanalysis
in those cases. Those fallbacks are tested but not represented by these fixtures.

Thirty normal tests and one real Lean smoke test passed, along with fmt/clippy.
Tests cover source/call-graph changes, features, local dependencies, workspace
member globs, conservative file-read cases, corrupt/deleted caches, native input
races, receipt reuse and missing native commands. Tool outcomes remain native
command results; no theorem, axiom or Rust correspondence policy was reintroduced.
