#!/usr/bin/env python3
import collections,json,pathlib,tomllib
root=pathlib.Path('/workspace/elicit-comparison/results/round9-continuation-1')
events=[json.loads(l) for l in (root/'terminal-events.jsonl').read_text().splitlines()]
start=json.loads((root/'metrics.json').read_text())['start_unix']*1000
# The preserved case event log includes the first segment; analyze only this resumed interval.
events=[v for v in events if v['timestamp_ms']>=start]
stage=collections.defaultdict(lambda:{'calls':0,'elapsed_seconds':0,'failed':0})
rows=[];inputs=[]
for v in events:
 d=v['data'];kind=v['event'];at=round((v['timestamp_ms']-start)/1000,3)
 if kind=='pipeline_stage_finished':
  k=d['stage']+('/proof' if d['prove'] else '/processing');s=stage[k];s['calls']+=1;s['elapsed_seconds']+=d['elapsed_ms']/1000;s['failed']+=not d['success']
 if kind in ['task_started','task_finished','worker_started','worker_finished','verification_finished']:
  rows.append({'at_seconds':at,'event':kind,'data':d})
 if kind=='input_changed':inputs.append({'at_seconds':at,'changed_files':[str(pathlib.Path(x).name) for x in d.get('changed_files',[])],'changed_context_keys':d.get('changed_context_keys',[]),'previous':d.get('previous'),'current':d.get('current')})
summary={'timeline':rows,'input_changes':inputs,'pipeline_stages':dict(stage),'notes':['Stage sums are accumulated call durations, not disjoint end-to-end phases. Metadata collection has a separate Lean invocation not represented by these stage events.','Historical success records can be stale after a subsequent input change; only the frozen terminal state defines timed current acceptance.','Plan installation and work names are declared intent; exact checked propositions and caller dependencies determine proof extent.']}
runs=collections.defaultdict(list)
for row in rows:
 if row['event']=='verification_finished':runs[row['data']['input']].append({'at_seconds':row['at_seconds'],'requested':row['data']['requested']})
state=tomllib.loads((root/'terminal-state.toml').read_text())
summary['same_input_verifications']={k:v for k,v in runs.items() if len(v)>1}
summary['attempt_accounting']={k:{'total':v['total'],'progress_credits':v.get('progress_credits',0),'no_progress':v['no_progress'],'failed':v['failed'],'seen_checked_facts':len(v.get('checked_work',[]))} for k,v in state['run']['attempts'].items()}
summary['report_only_input_changes']=[v for v in inputs if v['changed_files'] and set(v['changed_files']) <= {'PROGRESS.md','verification-inventory.json'} and not v['changed_context_keys']]
(root/'workflow-analysis.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({'events':len(events),'timeline_records':len(rows),'input_changes':len(inputs),'pipeline_stages':dict(stage)},indent=2))
