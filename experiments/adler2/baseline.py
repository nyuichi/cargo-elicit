#!/usr/bin/env python3
"""One ordinary Codex app-server turn, with inherited model and permissions."""
import json,pathlib,subprocess,sys,time
root=pathlib.Path(sys.argv[1]).resolve()
continue_seconds=int(sys.argv[2]) if len(sys.argv)>2 else None
started=time.monotonic();turns=1
p=subprocess.Popen(['codex','app-server'],cwd=root,stdin=subprocess.PIPE,stdout=subprocess.PIPE)
seq=0
def send(v):p.stdin.write((json.dumps(v)+'\n').encode());p.stdin.flush()
def next_event():
 line=p.stdout.readline()
 if not line:raise RuntimeError('Codex app-server disconnected')
 v=json.loads(line)
 if 'method' in v and 'id' in v:
  # Match cargo-elicit's unattended approval behavior; never auto-approve.
  if v['method'] in ['item/commandExecution/requestApproval','item/fileChange/requestApproval']:
   send({'id':v['id'],'result':{'decision':'decline'}})
  else:send({'id':v['id'],'error':{'code':-32601,'message':'unsupported interactive request'}})
  raise RuntimeError('approval or interactive input required')
 return v
def request(method,params):
 global seq
 seq+=1;send({'id':seq,'method':method,'params':params})
 while True:
  v=next_event()
  if v.get('id')==seq:
   if 'error' in v:raise RuntimeError('app-server request failed')
   return v['result']
try:
 request('initialize',{'clientInfo':{'name':'elicit-comparison-baseline','version':'1'},'capabilities':{}})
 send({'method':'initialized','params':{}})
 r=request('thread/start',{'cwd':str(root),'developerInstructions':'Work on the original sources as the single verification worker. Read BENCHMARK_TASK.md and follow its common constraints. Use the inherited model, reasoning, authentication and permissions. Never start subagents. Do not use cargo-elicit in this arm.'})
 request('turn/start',{'threadId':r['thread']['id'],'input':[{'type':'text','text':'Complete the verification task in BENCHMARK_TASK.md. Work autonomously within the stated limits; preserve partial results and state any unsupported obligations.'+(f' The total authoring budget is {continue_seconds} seconds; use it to continue unresolved proofs rather than stopping after easy partial results. If all five requested API contracts have been fully proved and kernel-checked, write BENCHMARK_COMPLETE.json containing an apis array with exactly new, checksum, from_checksum, write_slice, adler32_slice, then finish. This is only a completion signal: the investigator will independently audit every claim.' if continue_seconds else '')}]})
 while True:
  v=next_event()
  if v.get('method')=='turn/completed':
   status=v['params']['turn']['status'];print(json.dumps({'turn_status':status,'turn':turns}),flush=True)
   remaining=continue_seconds-(time.monotonic()-started) if continue_seconds else 0
   claimed_complete=False
   try:
    marker=json.loads((root/'BENCHMARK_COMPLETE.json').read_text())
    claimed_complete=set(marker.get('apis',[]))=={'new','checksum','from_checksum','write_slice','adler32_slice'}
   except (FileNotFoundError,ValueError,TypeError):pass
   if continue_seconds and claimed_complete:
    print(json.dumps({'completion_claim':True,'note':'Requires independent full-contract audit'}),flush=True);sys.exit(0)
   if status=='completed' and continue_seconds and remaining>15 and turns<32:
    turns+=1
    request('turn/start',{'threadId':r['thread']['id'],'input':[{'type':'text','text':f'Continue the original BENCHMARK_TASK.md verification goal with the saved work. About {int(remaining)} seconds remain. Focus on unresolved obligations and preserve checked partial results; do not restart completed work or weaken the requested contracts. If the entire goal is already proved, independently check it and document the evidence.'}]})
   else:sys.exit(0 if status=='completed' else 1)
finally:
 p.terminate()
 try:p.wait(timeout=5)
 except subprocess.TimeoutExpired:p.kill();p.wait()
