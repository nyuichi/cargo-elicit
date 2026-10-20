#!/usr/bin/env python3
"""One external Codex authoring turn, observed without managing proof work."""
import argparse, hashlib, json, os, pathlib, queue, resource, signal, subprocess, threading, time

p = argparse.ArgumentParser()
p.add_argument('arm', choices=['with', 'without'])
p.add_argument('--root', required=True, type=pathlib.Path)
p.add_argument('--results', required=True, type=pathlib.Path)
p.add_argument('--codex', required=True)
p.add_argument('--seconds', type=int, default=10800)
a = p.parse_args()
root, out = a.root.resolve(), a.results.resolve()
if a.seconds <= 0 or out.exists():
    raise SystemExit('Positive budget and a new results directory are required')
if any(root.rglob('*.lean')) or (root / '.elicit/snapshot.json').exists():
    raise SystemExit('Refusing a non-fresh proof workspace')
out.mkdir(parents=True)
def write(name, value):
    (out / name).write_text(json.dumps(value, indent=2) + '\n')
def hashes():
    files = [root / 'Cargo.toml'] + sorted((root / 'src').rglob('*'))
    return {str(f.relative_to(root)): hashlib.sha256(f.read_bytes()).hexdigest() for f in files if f.is_file()}
before = hashes()
started = time.monotonic()
start_unix = time.time()
seq = 0
observations, seen, settings, commands, counts = [], set(), {}, [], {}
usage = None
model_usage_events = 0
status = 'starting'
final_messages = []
events = (out / 'telemetry.jsonl').open('w')
stderr = (out / 'server.stderr').open('w')
child = subprocess.Popen([a.codex, 'app-server'], cwd=root, stdin=subprocess.PIPE,
                         stdout=subprocess.PIPE, stderr=stderr, start_new_session=True)
messages = queue.Queue()
def receive():
    for line in child.stdout:
        try:
            messages.put(json.loads(line))
        except ValueError:
            messages.put({'method': 'observer/unparseable_server_line'})
    messages.put(None)
threading.Thread(target=receive, daemon=True).start()
def send(value):
    child.stdin.write((json.dumps(value) + '\n').encode())
    child.stdin.flush()
def observe(reason):
    tick = time.monotonic()
    record = {'reason': reason, 'at_seconds': tick - started, 'commands': []}
    if a.arm == 'with' and (root / '.elicit/snapshot.json').is_file():
        for suffix, argv in [('text', ['cargo', 'elicit', 'status']), ('json', ['cargo', 'elicit', 'status', '--json'])]:
            t = time.monotonic()
            r = subprocess.run(argv, cwd=root, capture_output=True, timeout=30)
            name = f'status-{len(observations):02}-{suffix}'
            (out / (name + '.stdout')).write_bytes(r.stdout)
            (out / (name + '.stderr')).write_bytes(r.stderr)
            record['commands'].append({'argv': argv, 'exit': r.returncode, 'seconds': time.monotonic() - t})
    else:
        record['note'] = 'Artifact observation only; no ledger coverage inferred'
        record['proof_sources'] = [str(f.relative_to(root)) for f in root.rglob('*.lean') if not any(v in f.parts for v in ['.elicit', '.lake', 'target'])]
    record['seconds'] = time.monotonic() - tick
    observations.append(record)
    write('observations.json', observations)
def checkpoints():
    elapsed = time.monotonic() - started
    triggers = []
    if a.arm == 'with':
        try:
            s = json.loads((root / '.elicit/snapshot.json').read_text())
            accepted = []
            for goal in s['goals']:
                if not goal.get('target'):
                    continue
                current = [e for e in s['evidence'].get(goal['id'], []) if e['input'] == s['input'] and e['specification'] == goal['specification']]
                # The latest record is observed evidence, never an independent semantic verdict.
                if current and current[-1]['accepted']:
                    accepted.append(goal['id'])
            if len(accepted) >= 1:
                triggers.append('first_api_evidence')
            if len(accepted) >= 3:
                triggers.append('three_api_evidence')
        except (OSError, KeyError, ValueError):
            pass
    if elapsed >= 600:
        triggers.append('ten_minutes')
    if elapsed >= 1800:
        triggers.append(f'checkpoint_{int(elapsed // 1800) * 30}_minutes')
    for reason in triggers:
        if reason not in seen:
            seen.add(reason)
            observe(reason)
def next_message():
    global usage, model_usage_events
    while True:
        checkpoints()
        remaining = a.seconds - (time.monotonic() - started)
        if remaining <= 0:
            raise TimeoutError('outer authoring budget exhausted')
        try:
            value = messages.get(timeout=min(2, remaining))
        except queue.Empty:
            continue
        if value is None:
            raise RuntimeError('Codex app-server disconnected')
        method = value.get('method', '')
        record = {'seconds': time.monotonic() - started, 'method': method, 'response_id': value.get('id')}
        if method:
            counts[method] = counts.get(method, 0) + 1
        params = value.get('params', {})
        if method == 'thread/tokenUsage/updated':
            model_usage_events += 1
            usage = params.get('tokenUsage')
            record['usage'] = usage
        if method in ['item/started', 'item/completed']:
            item = params.get('item', {})
            record['item_type'] = item.get('type')
            if item.get('type') == 'commandExecution':
                record['command'] = {k: item[k] for k in ['id', 'command', 'cwd', 'status', 'exitCode', 'durationMs', 'aggregatedOutput'] if k in item}
                if method == 'item/completed':
                    commands.append(record)
            if method == 'item/completed' and item.get('type') == 'agentMessage':
                text = item.get('text', '')
                final_messages.append({'seconds': record['seconds'], 'text': text})
        if method in ['error', 'turn/completed']:
            record['detail'] = params
        events.write(json.dumps(record) + '\n')
        events.flush()
        if 'method' in value and 'id' in value:
            if method in ['item/commandExecution/requestApproval', 'item/fileChange/requestApproval']:
                send({'id': value['id'], 'result': {'decision': 'decline'}})
            else:
                send({'id': value['id'], 'error': {'code': -32601, 'message': 'No human intervention during benchmark'}})
            raise RuntimeError('Approval or interactive input requested; not auto-approved')
        return value
def request(method, params):
    global seq
    seq += 1
    send({'id': seq, 'method': method, 'params': params})
    while True:
        value = next_message()
        if value.get('id') == seq:
            if 'error' in value:
                raise RuntimeError(f'{method} failed: {value["error"]}')
            return value['result']
def shutdown():
    # App-server owns its commands; signal the process group and retain artifacts.
    if child.poll() is None:
        os.killpg(child.pid, signal.SIGINT)
        try:
            child.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
try:
    request('initialize', {'clientInfo': {'name': 'independent-cli-authoring-observer', 'version': '1'}, 'capabilities': {}})
    send({'method': 'initialized', 'params': {}})
    treatment = ('Read .agents/skills/cargo-elicit/SKILL.md and tool-docs. Use the frozen cargo-elicit CLI as a normal utility, not an AI orchestrator.' if a.arm == 'with' else 'Do not invoke cargo-elicit or read its sources, docs, skill or any prior experiment. Use native proof tools only.')
    result = request('thread/start', {'cwd': str(root), 'ephemeral': True, 'sandbox': 'workspace-write', 'developerInstructions': 'You are the single verification author. Read BENCHMARK_TASK.md. Do not start subagents. ' + treatment})
    settings = {k: result[k] for k in ['model', 'modelProvider', 'reasoningEffort', 'serviceTier', 'approvalPolicy', 'sandbox'] if k in result}
    write('resolved-settings.json', settings)
    request('turn/start', {'threadId': result['thread']['id'], 'input': [{'type': 'text', 'text': 'Complete BENCHMARK_TASK.md from scratch, including final validation and .elicit/notes/FINAL_REPORT.md. You have up to three hours. Choose your own proof order and supporting mathematics. Preserve original runtime bytes and accurately report any incomplete work.'}]})
    status = 'running'
    while True:
        value = next_message()
        if value.get('method') == 'turn/completed':
            status = value['params']['turn']['status']
            break
except TimeoutError as error:
    status = 'timeout'
    write('termination.json', {'status': status, 'reason': str(error)})
except Exception as error:
    status = 'environment_or_protocol_error'
    write('termination.json', {'status': status, 'reason': str(error)})
finally:
    shutdown()
    work_seconds = time.monotonic() - started
    observe('termination')
    events.close()
    stderr.close()
    write('author-messages.json', final_messages)
    write('commands.json', commands)
    write('metrics.json', {'arm': a.arm, 'start_unix': start_unix, 'budget_seconds': a.seconds,
          'status': status, 'live_response_observed': bool(final_messages or commands or usage), 'timed_work_seconds': work_seconds, 'end_to_end_seconds': time.monotonic() - started,
          'observation_seconds': sum(v['seconds'] for v in observations), 'settings': settings,
          'tokens_cumulative_last': usage, 'commands_completed': len(commands), 'token_usage_event_count': model_usage_events,
          'command_duration_seconds_sum': sum((v['command'].get('durationMs') or 0) / 1000 for v in commands),
          'failed_commands': sum(v['command'].get('exitCode') not in [0, None] for v in commands),
          'event_counts': counts, 'human_proof_hints': 0,
          'source_hashes_before': before, 'source_hashes_after': hashes(), 'original_sources_unchanged': hashes() == before,
          'children_resource_usage': {'user_seconds': resource.getrusage(resource.RUSAGE_CHILDREN).ru_utime, 'system_seconds': resource.getrusage(resource.RUSAGE_CHILDREN).ru_stime},
          'coverage': 'Requires investigator semantic review and independent kernel audit; do not infer from turn status'})
    for name in ['snapshot.json', 'events.jsonl']:
        source = root / '.elicit' / name
        if source.is_file():
            (out / ('terminal-' + name)).write_bytes(source.read_bytes())
print(json.dumps({'arm': a.arm, 'status': status, 'seconds': time.monotonic() - started}))
