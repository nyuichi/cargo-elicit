#!/usr/bin/env python3
"""Summarize recorded counters without counting cumulative usage twice."""
import argparse,json,pathlib,re
p=argparse.ArgumentParser();p.add_argument('--results',required=True,type=pathlib.Path);a=p.parse_args();root=a.results
metrics=json.loads((root/'metrics.json').read_text())
commands=json.loads((root/'commands.json').read_text())
usage=(metrics.get('tokens_cumulative_last') or {}).get('total') or {}
cli=re.compile(r'(?:cargo\s+elicit\b|cargo-elicit\b)')
contains=[c for c in commands if cli.search(c['command'].get('command',''))]
def output_bytes(cs):return sum(len((c['command'].get('aggregatedOutput') or '').encode()) for c in cs)
summary={'status':metrics['status'],'authoring_wall_seconds':metrics['timed_work_seconds'],
 'provider_total_usage':usage,'noncached_input_tokens':usage.get('inputTokens',0)-usage.get('cachedInputTokens',0),
 'completed_command_tool_items':len(commands),'nonzero_exit_command_items':metrics['failed_commands'],
 'failed_status_command_items':sum(c['command'].get('status')=='failed' for c in commands),
 'unknown_exit_command_items':sum(c['command'].get('exitCode') is None for c in commands),
 'command_output_utf8_bytes':output_bytes(commands),'command_items_containing_cli_text':len(contains),
 'output_bytes_of_cli_text_items':output_bytes(contains),'usage_notifications':metrics.get('token_usage_event_count'),
 'classification_limit':'CLI-text matching includes docs/heredocs and mixed output; not exact CLI invocation or token counts. Usage notifications are not exact model-request counts. Command items are not subprocess counts.',
 'original_sources_unchanged':metrics['original_sources_unchanged'],'settings':metrics['settings'],
 'coverage':'Requires separate investigator semantic and fresh kernel review; not inferred from status or self-report.'}
(root/'numeric-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
