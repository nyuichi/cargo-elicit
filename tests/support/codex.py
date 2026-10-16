#!/usr/bin/env python3
import os,sys,json,subprocess,time,pathlib
if '--version' in sys.argv:print('codex-cli 0.159.2');sys.exit(0)
thread='mock-thread';turn='mock-turn'
def send(v):print(json.dumps(v),flush=True)
for line in sys.stdin:
 req=json.loads(line);m=req.get('method');p=req.get('params',{});id=req.get('id')
 if m=='initialize':send({'id':id,'result':{'userAgent':'fixture'}})
 elif m in ('thread/start','thread/resume'):
  assert 'model' not in p and 'effort' not in p and 'approvalPolicy' not in p and 'sandbox' not in p
  assert 'developerInstructions' in p
  send({'id':id,'result':{'thread':{'id':thread}}})
 elif m=='turn/start':
  task=json.loads(p['input'][0]['text']);send({'id':id,'result':{'turn':{'id':turn,'status':'inProgress'}}})
  pathlib.Path('.elicit/runtime/mock-started').write_text('started')
  case=os.getenv('MOCK_CASE','submit')
  if case=='interrupt':continue
  if case=='approval':send({'id':777,'method':'item/commandExecution/requestApproval','params':{'threadId':thread,'turnId':turn,'command':'needs-human','cwd':os.getcwd()}});continue
  if case=='body':
   file=pathlib.Path(task['targets'][0]['file']);file.write_text(file.read_text().replace('{ x }','{ x.wrapping_add(1) }'))
  if case in ('submit','body') and task['task']!='verify':
   for target in task['targets']:
    exe=os.environ['ELICIT_BIN'];data=json.loads(subprocess.check_output([exe,'agent','show',target['id']]))
    subprocess.run([exe,'agent','submit',target['id'],'--revision',str(data['revision']),'--note','fixture: reviewed original contract'],stdout=subprocess.DEVNULL)
  send({'method':'thread/tokenUsage/updated','params':{'threadId':thread,'tokenUsage':{'total':{'inputTokens':100,'cachedInputTokens':40,'outputTokens':20}}}})
  send({'method':'turn/completed','params':{'threadId':thread,'turn':{'id':turn,'status':'failed' if case=='failed' else 'completed'}}})
 elif m=='turn/interrupt':
  send({'id':id,'result':{}});send({'method':'turn/completed','params':{'threadId':thread,'turn':{'id':turn,'status':'interrupted'}}})
