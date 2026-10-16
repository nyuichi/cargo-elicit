#!/usr/bin/env python3
import os,sys,json,subprocess
args=sys.argv[1:]
if len(args)>1 and args[0].startswith('+') and args[1]=='clean':
 assert '.elicit/cache' in os.environ.get('CARGO_TARGET_DIR','')
 sys.exit(0)
if args and args[0] in ('kani','creusot'):
 tool=args[0]
 if '--version' in args or 'version' in args:
  print('kani 0.68.0' if tool=='kani' else 'cargo-creusot 0.14.0 nightly-2026-08-03');sys.exit(0)
 if os.getenv('MOCK_CHECK_FAIL')=='1' or os.getenv('MOCK_PROOF_FAIL')=='1' and '--export-json' in args:sys.exit(1)
 # Bare exit 0 intentionally supplies no obligations. It must never become verified.
 sys.exit(0)
os.execv(os.environ['ELICIT_REAL_CARGO'],[os.environ['ELICIT_REAL_CARGO']]+args)
