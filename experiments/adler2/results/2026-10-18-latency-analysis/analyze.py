"""Recompute disjoint timing categories and de-duplicate resumed thread usage.

Read whitelisted original telemetry from the retained local result directories.
No conversations, command strings, outputs or credentials are emitted.
"""
import argparse,json,pathlib
p=argparse.ArgumentParser();p.add_argument('--results',type=pathlib.Path,required=True);p.add_argument('--out',type=pathlib.Path,required=True);args=p.parse_args()
names=['round5-plain-1','round9-elicit-1','round9-continuation-1'];segments=[];thread_totals={};thread_sets=[];raw_usage=[]
for name in names:
 root=args.results/name;metrics=json.loads((root/'metrics.json').read_text());rows=[json.loads(l) for l in (root/'telemetry.jsonl').read_text().splitlines()];commands=[];active=0;begin=None;command_count=0
 for v in rows:
  if v.get('usage') and v.get('thread_id'):
   total=v['usage']['total'];old=thread_totals.setdefault(v['thread_id'],{})
   for k in ['inputTokens','cachedInputTokens','outputTokens','reasoningOutputTokens']:old[k]=max(old.get(k,0),total.get(k,0))
  if v.get('item_type')=='commandExecution':
   if v.get('method')=='item/started':
    if active==0:begin=v['at']
    active+=1;command_count+=1
   elif v.get('method')=='item/completed':
    active-=1;assert active>=0
    if active==0:commands.append((begin,v['at']))
 assert active==0
 ids={v['thread_id'] for v in rows if v.get('usage') and v.get('thread_id')};thread_sets.append(ids);usage=[v['usage']['total'] for v in rows if v.get('usage')];raw_usage.append((usage[0],usage[-1]))
 if name==names[0]:
  workers=[(next(v['at'] for v in rows if v.get('method')=='turn/started'),max(v['at'] for v in rows if v.get('method')=='turn/completed'))];deadlines=0;plans=0
 else:
  timeline=json.loads((root/'workflow-analysis.json').read_text())['timeline'];workers=[];st=None
  for v in timeline:
   if v['event']=='worker_started':st=metrics['start_unix']+v['at_seconds']
   if v['event']=='worker_finished':
    assert st is not None;workers.append((st,metrics['start_unix']+v['at_seconds']));st=None
  assert st is None
  deadlines=sum(v['event']=='task_finished' and v['data'].get('worker_deadline',False) for v in timeline);plans=sum(v['event']=='task_started' and v['data']['task']['kind']=='plan' for v in timeline)
 worker=sum(b-a for a,b in workers);command=sum(max(0,min(b,d)-max(a,c)) for a,b in workers for c,d in commands)
 record={'segment':name,'elapsed_seconds':metrics['end_to_end_seconds'],'worker_command_active_seconds':command,'worker_without_active_command_seconds':worker-command,'outside_worker_seconds':metrics['end_to_end_seconds']-worker,'worker_turns':len(workers),'started_command_items':command_count,'worker_deadlines':deadlines,'plan_tasks':plans};assert abs(sum(record[k] for k in ['worker_command_active_seconds','worker_without_active_command_seconds','outside_worker_seconds'])-record['elapsed_seconds'])<1e-6;segments.append(record)
keys=[k for k in segments[0] if k!='segment'];product={k:sum(s[k] for s in segments[1:]) for k in keys};control={k:segments[0][k] for k in keys};delta={k:product[k]-control[k] for k in keys}
def totals(ids):return {k:sum(thread_totals[id][k] for id in ids) for k in ['inputTokens','cachedInputTokens','outputTokens','reasoningOutputTokens']}
assert thread_sets[1]==thread_sets[2]
assert raw_usage[1][1]==raw_usage[2][0]
result={'segments':segments,'ordinary':control,'product_two_segments':product,'product_minus_ordinary':delta,'ordinary_tokens':totals(thread_sets[0]),'product_tokens_unique_threads':totals(thread_sets[1]|thread_sets[2]),'continuation_token_increment':{k:raw_usage[2][1][k]-raw_usage[1][1][k] for k in ['inputTokens','cachedInputTokens','outputTokens','reasoningOutputTokens']},'resumed_same_thread':True,'continuation_first_counter_equals_initial_last':True,'notes':['Timing categories are disjoint observed wall intervals, not CPU time or a causal attribution of model computation.','Command intervals use the union of overlapping commandExecution start/completion events, clipped to worker windows.','No-command worker time includes generation, reasoning, request latency, other tool types and handoff/compaction effects; do not call all of it reasoning time.','Initial and continuation usage counters belong to the same resumed thread; use the final maximum, never sum the two cumulative summaries.','Resumption preparation138.34seconds and independent audits are outside the elapsed two-segment comparison.','This is a comparison of recorded separate trajectories, not a randomized causal ablation.']}
args.out.write_text(json.dumps(result,indent=2)+'\n')
