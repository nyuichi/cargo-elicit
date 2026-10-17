#!/usr/bin/env python3
"""Summarize measured telemetry only; this does not accept proof claims."""
import argparse,json,pathlib
p=argparse.ArgumentParser();p.add_argument('results',type=pathlib.Path);args=p.parse_args()
r=args.results
usage={};settings=[];items={}
for line in (r/'telemetry.jsonl').read_text().splitlines():
 v=json.loads(line)
 if v.get('settings') and v['settings'] not in settings:settings.append(v['settings'])
 if v.get('method')=='item/started':
  kind=v.get('item_type','unknown');items[kind]=items.get(kind,0)+1
 if v.get('usage') and v.get('thread_id'):
  total=v['usage'].get('total',{})
  if all(isinstance(total.get(k),int) for k in ['inputTokens','cachedInputTokens','outputTokens']):
   old=usage.setdefault(v['thread_id'],{})
   for key in ['inputTokens','cachedInputTokens','outputTokens']:old[key]=max(old.get(key,0),total[key])
summary={'resolved_settings':settings,'thread_count_with_usage':len(usage),'tokens':{k:sum(u[k] for u in usage.values()) for k in ['inputTokens','cachedInputTokens','outputTokens']} if usage else None,'started_items':items,'accepted_api_coverage':None,'note':'Cached input is a subset of input, not an extra token category. Coverage requires independent source, theorem and axiom checks.'}
(r/'telemetry-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
