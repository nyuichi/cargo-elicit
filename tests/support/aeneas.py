#!/usr/bin/env python3
# Deterministic protocol fixtures; native tests separately exercise the Lean kernel.
import sys, os, pathlib, json, time
name=pathlib.Path(sys.argv[0]).name
args=sys.argv[1:]
if name=='charon':
 if args==['toolchain-path']:sys.exit(1)
 if args==['version']:
  print('0.1.293 (de8736f7c995ac54dc490345cef9ea5143c06c62)');sys.exit(0)
 assert '--preset=aeneas' in args and '--sysroot=default' in args
 assert '--manifest-path' in args and '--package' in args and '--target' in args
 out=pathlib.Path(args[args.index('--dest-file')+1])
 root=pathlib.Path(args[args.index('--manifest-path')+1]).parent
 source=(root/'src/lib.rs').read_text()
 if os.environ.get('MOCK_AENEAS')=='wait':
  (root/'.elicit/runtime/mock-checker-started').touch()
  time.sleep(60)
 data=({'charon_version':'0.1.293','has_errors':False,'translated':{
  'runtime_checks':{'overflow_checks':True},'target_information':[{'key':args[args.index('--target')+1]}],
  'files':[{'id':0,'name':{'Local':'src/lib.rs'},'contents':source}],
  'fun_decls':[{'item_meta':{'name':[{'Ident':['fixture',0]},{'Ident':['identity',0]}],
   'is_local':True,'is_extern':False,'has_errors':False,'opacity':'Transparent',
   'span':{'Untagged':{'data':{'file_id':0,'beg':{'line':1,'col':0},'end':{'line':3,'col':1}}}}},
   'body':{'Structured':{}}}]}})
 case=os.environ.get('MOCK_AENEAS')
 if case=='opaque':data['translated']['fun_decls'][0]['item_meta']['opacity']='Opaque'
 if case=='missing':data['translated']['fun_decls']=[]
 if case=='wrongtarget':data['translated']['target_information'][0]['key']='other-target'
 if case=='errors':data['has_errors']=True
 if case in ['specialized','specialized_mut','disabled_checks']:
  data['translated']['options']={ {'specialized':'monomorphize','specialized_mut':'monomorphize_mut','disabled_checks':'skip_borrowck'}[case]:True }
 out.write_text(json.dumps(data));sys.exit(0)
if name=='aeneas':
 if args==['-version']:
  print('aeneas e62dd83d133d9043e7d3628096838b7430f36565');sys.exit(0)
 out=pathlib.Path(args[args.index('-dest')+1])
 (out/'Fixture.lean').write_text("namespace ElicitGenerated\n/-- [fixture::identity]:\n Source: 'src/lib.rs', lines 1:0-3:1\n-/\ndef identity (x : U32) : Result U32 := .ok x\nend ElicitGenerated\n")
 sys.exit(0)
assert name=='lake' and args[:2]==['env','lean']
if '--version' in args:print('Lean (version 4.35.0)');sys.exit(0)
if '--print-prefix' in args:print(os.environ['AENEAS_HOME']);sys.exit(0)
source=pathlib.Path(args[-1])
if os.environ.get('MOCK_CHECK_FAIL')=='1':sys.exit(1)
if source.name=='Audit.lean':
 if os.environ.get('MOCK_AENEAS')=='mismatch':sys.exit(1)
 axiom=os.environ.get('MOCK_AENEAS','propext')
 print("'ElicitAudit' depends on axioms: ["+axiom+"]")
pathlib.Path(args[args.index('-o')+1]).touch()
