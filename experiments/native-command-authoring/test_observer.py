#!/usr/bin/env python3
"""Measurement boundary tests; these never invoke an AI or establish proof results."""
import json, pathlib, subprocess, tempfile, unittest
RUN = pathlib.Path(__file__).with_name('run.py')
FAKE = '''#!/usr/bin/env python3
import json,sys,time
for line in sys.stdin:
 v=json.loads(line)
 if 'id' not in v: continue
 method=v['method']
 result={'thread':{'id':'test-thread'},'model':'test-only','reasoningEffort':None} if method=='thread/start' else {}
 print(json.dumps({'id':v['id'],'result':result}),flush=True)
 if method=='turn/start':
  if HANG: time.sleep(60)
  for total in [10,20]: print(json.dumps({'method':'thread/tokenUsage/updated','params':{'tokenUsage':{'total':{'inputTokens':total,'cachedInputTokens':total//2,'outputTokens':3}}}}),flush=True)
  print(json.dumps({'method':'item/completed','params':{'item':{'type':'commandExecution','id':'test','command':'lean Proofs.lean','exitCode':1,'durationMs':250}}}),flush=True)
  print(json.dumps({'method':'turn/completed','params':{'turn':{'status':'completed'}}}),flush=True)
'''
class ObserverTests(unittest.TestCase):
 def execute(self, hang, external=False):
  with tempfile.TemporaryDirectory() as d:
   base=pathlib.Path(d);root=base/'case';root.mkdir();(root/'src').mkdir();(root/'src/lib.rs').write_text('pub fn identity(x:u8)->u8{x}\n');(root/'Cargo.toml').write_text('[package]\nname="test"\nversion="0.0.0"\n')
   fake=base/'fake';fake.write_text(FAKE.replace('if HANG:',f'if {hang}:'));fake.chmod(0o755)
   output=base/'result'
   r=subprocess.run(['python3',str(RUN),'without','--root',str(root),'--results',str(output),'--codex',str(fake),'--seconds','1' if hang else '10']+(['--external-sandbox'] if external else []),capture_output=True,text=True,timeout=20)
   self.assertEqual(r.returncode,0,r.stderr)
   return json.loads((output/'metrics.json').read_text())
 def test_cumulative_usage_not_summed_and_failures_counted(self):
  m=self.execute(False);self.assertEqual(m['status'],'completed');self.assertEqual(m['tokens_cumulative_last']['total']['inputTokens'],20);self.assertEqual(m['failed_commands'],1);self.assertEqual(m['command_duration_seconds_sum'],0.25);self.assertTrue(m['original_sources_unchanged'])
 def test_effective_external_policy_is_reported_separately(self):
  m=self.execute(False,external=True);self.assertEqual(m['effective_turn_sandbox_policy'],{'type':'externalSandbox','networkAccess':'restricted'})
 def test_timeout_preserves_measurements(self):
  m=self.execute(True);self.assertEqual(m['status'],'timeout');self.assertIsNone(m['tokens_cumulative_last']);self.assertTrue(m['original_sources_unchanged'])
if __name__=='__main__':unittest.main()
