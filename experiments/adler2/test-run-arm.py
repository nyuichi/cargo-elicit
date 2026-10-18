#!/usr/bin/env python3
"""Exercise real harness timing with deterministic local peers and no AI calls."""
import json, os, pathlib, subprocess, tempfile, unittest

SCRIPT = pathlib.Path(__file__).with_name('run-arm.py')

class BudgetTests(unittest.TestCase):
    def run_case(self, seconds):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        base = pathlib.Path(temp.name)
        case = base / 'case'
        (case / 'src').mkdir(parents=True)
        (case / 'bin').mkdir()
        (case / 'src/lib.rs').write_text('pub fn identity(x:u32)->u32 { x }\n')
        (case / 'Cargo.toml').write_text('[package]\nname="clock-fixture"\nversion="0.0.0"\n')
        codex = case / 'bin/codex'
        codex.write_text('#!/bin/sh\nexit 0\n')
        codex.chmod(0o755)
        cargo = case / 'bin/cargo'
        cargo.write_text('''#!/usr/bin/env python3
import json,pathlib,sys,time
args=sys.argv[1:]
if args[1]=='init':
 time.sleep(1.2)
 pathlib.Path('.elicit').mkdir(exist_ok=True)
 pathlib.Path('.elicit/state.toml').write_text('input="input"\\nresults=[]\\n[targets]\\n')
elif args[1]=='run':pathlib.Path('called-run.json').write_text(json.dumps(args))
else:print('{}')
''')
        cargo.chmod(0o755)
        env = os.environ.copy()
        env['ELICIT_BENCH_REAL_CODEX'] = str(codex)
        env['PATH'] = str(case / 'bin') + ':' + env['PATH']
        result = subprocess.run(['python3', str(SCRIPT), 'with', '--root', str(case), '--results', str(base / 'result'), '--seconds', str(seconds)], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        return case, base / 'result', result

    def test_initialization_consumes_the_same_outer_budget(self):
        case, results, result = self.run_case(5)
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        args = json.loads((case / 'called-run.json').read_text())
        budget = int(args[args.index('--max-seconds') + 1])
        init = json.loads((results / 'initialization.json').read_text())
        self.assertGreaterEqual(init['elapsed_seconds'], 1.2)
        self.assertLess(budget, 5 - init['elapsed_seconds'])
        self.assertEqual(budget, 1)
        metrics = json.loads((results / 'metrics.json').read_text())
        self.assertEqual(metrics['inner_cli_budget_seconds'], budget)
        self.assertEqual(metrics['outer_shutdown_margin_seconds'], 2)
        self.assertTrue(metrics['original_sources_unchanged'])
        self.assertEqual(len(json.loads((results / 'observations.json').read_text())[0]['commands']), 3)

    def test_initialization_timeout_never_starts_authoring(self):
        case, results, result = self.run_case(1)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((case / 'called-run.json').exists())
        self.assertFalse((results / 'metrics.json').exists())

if __name__ == '__main__':
    unittest.main()
