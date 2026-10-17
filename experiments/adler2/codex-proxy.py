#!/usr/bin/env python3
"""Forward Codex unchanged; collect only whitelisted experiment telemetry."""
import json,os,pathlib,subprocess,sys,threading,time
real=os.environ['ELICIT_BENCH_REAL_CODEX']
if sys.argv[1:]!=['app-server']:os.execv(real,[real]+sys.argv[1:])
log=pathlib.Path(os.environ['ELICIT_BENCH_TELEMETRY'])
child=subprocess.Popen([real,'app-server'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=sys.stderr.buffer)
def incoming():
 try:
  for line in sys.stdin.buffer:
   child.stdin.write(line);child.stdin.flush()
 except (BrokenPipeError,OSError):pass
threading.Thread(target=incoming,daemon=True).start()
with log.open('a') as f:
 for line in child.stdout:
  try:
   v=json.loads(line);r={'at':time.time(),'method':v.get('method'),'response_id':v.get('id')}
   if isinstance(v.get('result'),dict):
    result=v['result'];r['settings']={k:result[k] for k in ['model','modelProvider','reasoningEffort','serviceTier','approvalPolicy','sandbox'] if k in result}
   if v.get('method')=='thread/tokenUsage/updated':r['usage']=v['params'].get('tokenUsage')
   if v.get('method') in ['item/started','item/completed']:r['item_type']=v['params'].get('item',{}).get('type')
   if v.get('method')=='turn/completed':r['turn_status']=v['params'].get('turn',{}).get('status')
   f.write(json.dumps(r)+'\n');f.flush()
  except (ValueError,KeyError,TypeError):pass
  sys.stdout.buffer.write(line);sys.stdout.buffer.flush()
sys.exit(child.wait())
