#!/usr/bin/env python3
"""Time one arm and preserve actual status observations; never invent metrics."""
import argparse,hashlib,json,math,os,pathlib,shutil,signal,subprocess,time,tomllib
p=argparse.ArgumentParser();p.add_argument('arm',choices=['with','without']);p.add_argument('--root',required=True);p.add_argument('--results',required=True);p.add_argument('--seconds',type=int,default=1800);p.add_argument("--api",action="append",default=[]);p.add_argument("--continue-control",action="store_true");p.add_argument("--retry",action="store_true");p.add_argument("--prior-results",action="append",required=True);args=p.parse_args()
if args.seconds<=0:p.error('--seconds must be positive')
root=pathlib.Path(args.root).resolve();out=pathlib.Path(args.results).resolve()
if out.exists() and any(out.iterdir()):raise SystemExit('Refusing to overwrite prior experiment results')
if not (root/'.elicit/state.toml').exists():raise SystemExit('Continuation requires the preserved existing case')
out.mkdir(parents=True,exist_ok=True)
real=os.environ.get('ELICIT_BENCH_REAL_CODEX') or shutil.which('codex')
if not real:raise SystemExit('Codex CLI is unavailable')
os.environ['ELICIT_BENCH_REAL_CODEX']=str(pathlib.Path(real).resolve())
# The readiness call is setup evidence, not a timed authoring attempt.
with (out/'auth-smoke.jsonl').open('wb') as stdout,(out/'auth-smoke.stderr').open('wb') as stderr:
 smoke=subprocess.run([real,'exec','--ephemeral','--skip-git-repo-check','--json','-C',str(root),'Reply only OK. Do not run tools or inspect files.'],stdin=subprocess.DEVNULL,stdout=stdout,stderr=stderr,timeout=45)
if smoke.returncode:
 (out/'readiness.json').write_text(json.dumps({'ready':False,'reason':'Codex live request failed; no timed trial started'})+'\n')
 raise SystemExit('Codex readiness failed; inspect auth-smoke output. This is not a benchmark result.')
(out/'readiness.json').write_text(json.dumps({'ready':True})+'\n')
def source_hashes():
 return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((root/'src').rglob('*')) if p.is_file()} | {'Cargo.toml':hashlib.sha256((root/'Cargo.toml').read_bytes()).hexdigest()}
before=source_hashes()
os.environ['ELICIT_BENCH_TELEMETRY']=str(out/'telemetry.jsonl')
os.environ['PATH']=str(root/'bin')+':'+os.environ['PATH']
start=time.monotonic();start_unix=time.time();observations=[];done=set();milestones=set();any_proof=False;last_scan=0
common={'new','checksum','from_checksum','write_slice','adler32_slice'}
def snapshot(reason):
 observed_start=time.monotonic()
 n=len(observations);record={'reason':reason,'at_seconds':time.monotonic()-start,'commands':[]}
 if args.arm=='with':
  for suffix,cmd in [('text',['cargo','elicit','status']),('json',['cargo','elicit','status','--json']),('targets',['cargo','elicit','list','--json'])]:
   tick=time.monotonic()
   with (out/f'status-{n}-{suffix}.stdout').open('wb') as stdout,(out/f'status-{n}-{suffix}.stderr').open('wb') as stderr:
    observation=subprocess.Popen(cmd,cwd=root,stdout=stdout,stderr=stderr,start_new_session=True)
    try:
     code=observation.wait(timeout=90 if reason=='termination' else min(90,max(0.1,args.seconds-(time.monotonic()-start))))
    except subprocess.TimeoutExpired:
     os.killpg(observation.pid,signal.SIGINT)
     try:observation.wait(timeout=5)
     except subprocess.TimeoutExpired:os.killpg(observation.pid,signal.SIGKILL);observation.wait()
     code='timeout'
   record['commands'].append({'argv':cmd,'exit':code,'elapsed_seconds':time.monotonic()-tick})
 else:
  # Observation of available artifacts only, with no hints sent to the agent.
  record['lean_files']=[]
  for directory,dirs,files in os.walk(root):
   dirs[:]=[d for d in dirs if d not in ['.lake','target','.git','.elicit']]
   record['lean_files'].extend(str((pathlib.Path(directory)/f).relative_to(root)) for f in files if f.endswith('.lean'))
  record['note']='No cargo-elicit ledger in this arm; theorem declarations alone are not verified coverage.'
 record['elapsed_seconds']=time.monotonic()-observed_start
 observations.append(record);(out/'observations.json').write_text(json.dumps(observations,indent=2)+'\n')
# Count the initial trial, preceding continuation segments and measured setup.
prior=[json.loads(pathlib.Path(p).read_text()) for p in args.prior_results]
initial=prior[0]['end_to_end_seconds']
prep_start=json.loads(pathlib.Path('/workspace/elicit-comparison/round9-resume-preparation-start.json').read_text())['unix']
# Wall time since preparation began includes any earlier resumed segment.
preparation_and_previous=time.time()-prep_start
terminal_reserve=300
args.seconds=max(0,math.floor(10800-initial-preparation_and_previous-terminal_reserve))
if args.seconds<1:raise SystemExit('Cumulative three-hour budget exhausted')
protocol={'mode':'same-case interrupted/resumed practical evaluation; not a fresh uninterrupted trial or matched comparison','cumulative_limit_seconds':10800,'initial_end_to_end_seconds':initial,'preparation_and_previous_wall_seconds':preparation_and_previous,'continuation_work_cap_seconds':args.seconds,'terminal_observation_reserve_seconds':terminal_reserve,'retry':args.retry,'prior_metrics':args.prior_results,'math_hints':0,'binary_unchanged':True}
(out/'continuation-budget.json').write_text(json.dumps(protocol,indent=2)+'\n')
(out/'initialization.json').write_text(json.dumps({'elapsed_seconds':0,'note':'No init: preserved existing ledger and proof artifacts; readiness/setup counted in continuation budget'})+'\n')
# Keep a small predeclared process-shutdown margin within the same outer cap.
remaining_cli_seconds=max(0,math.floor(args.seconds-(time.monotonic()-start))-2)
if args.arm=='with' and remaining_cli_seconds<1:raise SystemExit('Initialization exhausted the work budget; no authoring worker started')
cmd=['cargo','elicit','run','--max-seconds',str(remaining_cli_seconds),'--max-tasks','32'] if args.arm=='with'  else ['python3',str(pathlib.Path(__file__).with_name('baseline.py')),str(root)]+([str(args.seconds)] if args.continue_control else [])
if args.retry:cmd.append('--retry')
with (out/'run.stdout').open('wb') as stdout,(out/'run.stderr').open('wb') as stderr:
 proc=subprocess.Popen(cmd,cwd=root,stdout=stdout,stderr=stderr,start_new_session=True)
 try:
  while proc.poll() is None:
   elapsed=time.monotonic()-start
   if elapsed>=args.seconds:
    os.killpg(proc.pid,signal.SIGINT)
    try:proc.wait(timeout=15)
    except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
    break
   if args.arm=='with':
    try:
     s=tomllib.loads((root/'.elicit/state.toml').read_text())
     if any(e.get('outcome')=='success' and e.get('input')==s.get('input') for e in s.get('proof_plan',{}).get('evidence',{}).values()):any_proof=True
     for r in s.get('results',[]):
      name=r['target'].split('::')[-2]
      if r['outcome']=='success' and r['scope']=='full_contract':any_proof=True
      if r['outcome']=='success' and r['scope']=='full_contract' and name in common:milestones.add(name)
    except (FileNotFoundError,KeyError,tomllib.TOMLDecodeError):pass
   triggers=[]
   if any_proof:triggers.append('first_verified_record')
   if len(milestones)>=1:triggers.append('first_api_proof')
   if len(milestones)>=3:triggers.append('halfway_coverage')
   if elapsed>=600:triggers.append('ten_minutes')
   if elapsed>=1200:triggers.append(f'periodic_{int(elapsed//600)*10}_minutes')
   for trigger in triggers:
    if trigger not in done:done.add(trigger);snapshot(trigger)
   time.sleep(2)
 finally:
  if proc.poll() is None:os.killpg(proc.pid,signal.SIGTERM);proc.wait()
stop=time.monotonic();snapshot('termination');finished=time.monotonic()
if args.arm=='with':
 for name in ['events.jsonl','state.toml']:
  source=root/'.elicit'/name
  if source.exists():shutil.copyfile(source,out/('terminal-'+name))
metrics={'start_unix':start_unix,'arm':args.arm,'command':cmd,'exit':proc.returncode,'end_to_end_seconds':finished-start,'timed_work_seconds':stop-start,'terminal_observation_seconds':finished-stop,'observation_seconds_total':sum(o['elapsed_seconds'] for o in observations),'observation_seconds_during_run':sum(o['elapsed_seconds'] for o in observations if o['reason']!='termination'),'historical_api_proof_milestones':sorted(milestones),'tokens':None,'human_proof_hints':0,'api_selectors':args.api,'continue_control':args.continue_control,'inner_cli_budget_seconds':remaining_cli_seconds if args.arm=='with' else None,'outer_shutdown_margin_seconds':2 if args.arm=='with' else 0,'original_sources_unchanged':source_hashes()==before,'source_hashes_before':before,'source_hashes_after':source_hashes(),'continuation':protocol,'cumulative_end_to_end_seconds':initial+time.time()-prep_start,'note':'Token usage and final independently audited coverage must be populated from actual telemetry/artifacts, not inferred from exit status.'}
(out/'metrics.json').write_text(json.dumps(metrics,indent=2)+'\n')
