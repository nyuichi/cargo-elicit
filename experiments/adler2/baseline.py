#!/usr/bin/env python3
"""One ordinary Codex app-server turn, with inherited model and permissions."""
import json,pathlib,subprocess,sys,time
root=pathlib.Path(sys.argv[1]).resolve()
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
 request('turn/start',{'threadId':r['thread']['id'],'input':[{'type':'text','text':'Complete the verification task in BENCHMARK_TASK.md. Work autonomously within the stated limits; preserve partial results and state any unsupported obligations.'}]})
 while True:
  v=next_event()
  if v.get('method')=='turn/completed':
   status=v['params']['turn']['status'];print(json.dumps({'turn_status':status}));sys.exit(0 if status=='completed' else 1)
finally:
 p.terminate()
 try:p.wait(timeout=5)
 except subprocess.TimeoutExpired:p.kill();p.wait()
