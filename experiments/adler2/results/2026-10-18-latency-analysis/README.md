# Latency accounting and resumed-counter correction

Derived from preserved actual round5 ordinary and round9 initial/resumed events and whitelisted telemetry. No new authoring trial. The script emits aggregate timing/counter data only. Run analyze.py --results /path/to/original/result-directories --out /path/to/new-summary.json. This checks closed command/worker intervals, exact timing partition and the resumed same-thread counter boundary. Public raw conversation/command telemetry remains excluded; the observed inputs are retained in the local experiment workspace.

See docs/experiments/adler2-latency-analysis.md for interpretation and limits. SHA256SUMS covers files except itself.
