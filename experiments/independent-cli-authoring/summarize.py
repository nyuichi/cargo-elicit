#!/usr/bin/env python3
"""Compute observed metrics only; theorem coverage is independently reviewed."""
import argparse, collections, hashlib, json, pathlib, re
p=argparse.ArgumentParser();p.add_argument('results',type=pathlib.Path);a=p.parse_args();base=a.results
summary={}
for arm in ['with','without']:
 root=base/arm
 if not (root/'telemetry.jsonl').exists():continue
 events=[json.loads(x) for x in (root/'telemetry.jsonl').read_text().splitlines()]
 metrics=json.loads((root/'metrics.json').read_text()) if (root/'metrics.json').exists() else {'status':'running'}
 starts={};rows=[];last_usage=None
 for e in events:
  if e.get('usage') is not None:last_usage=e['usage']
  if e.get('item_type')!='commandExecution':continue
  c=e.get('command') or {};key=c.get('id')
  if e['method']=='item/started':starts[key]=e['seconds']
  elif e['method']=='item/completed':
   command=c.get('command') or '';began=starts.get(key);seconds=e['seconds']-began if began is not None else None
   categories=[name for name,pattern in [('elicit',r'(?:cargo[ -]elicit|cargo-elicit)'),('lean',r'\b(?:lean|lake)\b'),('charon',r'\bcharon\b'),('aeneas',r'\baeneas\b'),('status',r'elicit[^\n]*\bstatus\b')] if re.search(pattern,command)]
   rows.append({'at_seconds':e['seconds'],'start_seconds':began,'observer_item_seconds':seconds,'reported_duration_ms':c.get('durationMs'),'exit':c.get('exitCode'),'categories':categories,'command_sha256':hashlib.sha256(command.encode()).hexdigest(),'command':command})
 total=(last_usage or {}).get('total',{});input_tokens=total.get('inputTokens');cached=total.get('cachedInputTokens')
 duplicate=collections.Counter(r['command_sha256'] for r in rows if r['exit'] not in [0,None])
 categories={name:{'command_items':sum(name in r['categories'] for r in rows),'observer_item_seconds_sum':sum(r['observer_item_seconds'] or 0 for r in rows if name in r['categories'])} for name in ['elicit','lean','charon','aeneas','status']}
 summary[arm]={'status':metrics.get('status'),'end_to_end_seconds':metrics.get('end_to_end_seconds'),'token_usage_last_cumulative':total or None,'noncached_input_tokens':input_tokens-cached if input_tokens is not None and cached is not None else None,'completed_command_items':len(rows),'failed_command_items':sum(r['exit'] not in [0,None] for r in rows),'repeated_failed_exact_commands_extra_occurrences':sum(max(0,n-1) for n in duplicate.values()),'observer_command_item_seconds_sum':sum(r['observer_item_seconds'] or 0 for r in rows),'command_categories_overlapping':categories,'observation_seconds':metrics.get('observation_seconds'),'original_sources_unchanged':metrics.get('original_sources_unchanged'),'context_compaction_events':sum('compact' in e.get('method','').lower() for e in events),'coverage':'Not inferred; supply investigator review and kernel audit','measurement_limits':['Command categories overlap and are text-based mentions, not exact native invocation counts; one shell command can spawn many native processes','Observer item intervals are not a disjoint wall-time decomposition; parallel items can overlap','App-server durationMs may be zero even for nonzero observer intervals','Cached input is a subset of input, not additional input','Repeated commands alone do not establish wasted work']}
 (root/'command-analysis.json').write_text(json.dumps(rows,indent=2)+'\n')
(base/'numeric-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
