#!/usr/bin/env python3
import os,sys,json,subprocess
args=sys.argv[1:]
if args and args[0] in ('kani','creusot'):
 tool=args[0]
 if '--version' in args or 'version' in args:
  print('kani 0.68.0' if tool=='kani' else 'cargo-creusot 0.14.0 nightly-2026-08-03');sys.exit(0)
 if os.getenv('MOCK_CHECK_FAIL')=='1':sys.exit(1)
 # Bare exit 0 intentionally supplies no obligations. It must never become verified.
 sys.exit(0)
os.execv(os.environ['ELICIT_REAL_CARGO'],[os.environ['ELICIT_REAL_CARGO']]+args)
