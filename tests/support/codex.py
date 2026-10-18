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
  pathlib.Path('.elicit/runtime/last-task.json').write_text(json.dumps(task))
  if case=='stage':
   assert task['task']=='author' and 'proof_plan' not in task
   assert task['turn_seconds']>1
   subprocess.run([os.environ['ELICIT_BIN'],'agent','stage','--all','--note','Reviewed all original arbitrary-input contracts'],check=True,stdout=subprocess.DEVNULL)
  if case.startswith('deadline_'):
   name='Lemmas.lean' if case=='deadline_lemma' else 'Proofs.lean'
   pathlib.Path('elicit-proofs/aeneas',name).write_text(pathlib.Path('.elicit/cache/deadline-edit.txt').read_text())
   if case=='deadline_lemma':
    exe=os.environ['ELICIT_BIN'];data=json.loads(subprocess.check_output([exe,'agent','show']))
    step={'id':'final_lemma','goal':task['targets'][0]['id'],'kind':'lemma','target':None,'reason':'Retain a checked checkpoint at the worker deadline','needs':[],'statement':'True','theorem':'FinalCheckpoint.saved'}
    plan=pathlib.Path('.elicit/cache/deadline-plan.json');plan.write_text(json.dumps({'steps':[step]}))
    subprocess.run([exe,'agent','plan','--file',str(plan),'--revision',str(data['revision'])],check=True,stdout=subprocess.DEVNULL)
   pathlib.Path('.elicit/runtime/deadline-edit-saved').write_text('saved')
   continue
  if case=='interrupt':continue
  if case=='approval':send({'id':777,'method':'item/commandExecution/requestApproval','params':{'threadId':thread,'turnId':turn,'command':'needs-human','cwd':os.getcwd()}});continue
  if case=='body':
   file=pathlib.Path(task['targets'][0]['file']);file.write_text(file.read_text().replace('{ x }','{ x.wrapping_add(1) }'))
  if case in ('submit','body','resubmit') and (task['task']!='verify' or case=='resubmit'):
   for target in task['targets']:
    exe=os.environ['ELICIT_BIN'];data=json.loads(subprocess.check_output([exe,'agent','show',target['id']]))
    subprocess.run([exe,'agent','submit',target['id'],'--revision',str(data['revision']),'--note','fixture: reviewed original contract'],stdout=subprocess.DEVNULL)
  if case=='submit' and task['task']=='plan':
   exe=os.environ['ELICIT_BIN'];data=json.loads(subprocess.check_output([exe,'agent','show']))
   steps=data.get('proof_plan',{}).get('steps',[])
   for target in task['targets']:
    steps.append({'id':'integrate_'+str(len(steps)),'goal':target['id'],'kind':'integrate','target':target['id'],'reason':'Return to original goal after body inspection','needs':[]})
   file=pathlib.Path('.elicit/cache/mock-plan.json');file.parent.mkdir(parents=True,exist_ok=True);file.write_text(json.dumps({'steps':steps}))
   subprocess.run([exe,'agent','plan','--file',str(file),'--revision',str(data['revision'])],check=True,stdout=subprocess.DEVNULL)
  send({'method':'thread/tokenUsage/updated','params':{'threadId':thread,'tokenUsage':{'total':{'inputTokens':100,'cachedInputTokens':40,'outputTokens':20}}}})
  send({'method':'turn/completed','params':{'threadId':thread,'turn':{'id':turn,'status':'failed' if case=='failed' else 'completed'}}})
 elif m=='turn/interrupt':
  send({'id':id,'result':{}});send({'method':'turn/completed','params':{'threadId':thread,'turn':{'id':turn,'status':'interrupted'}}})
