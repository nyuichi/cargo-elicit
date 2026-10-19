#!/usr/bin/env python3
"""Compare root-level reports with reserved operational notes on saved proofs.

No AI/proof authoring. Uses one fixed binary, identical environment, sequential
arms, fresh per-arm workspaces, and proof/source hash assertions.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time

REPORTS = ('PROGRESS.md', 'FINAL_REPORT.md', 'verification-inventory.json')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def semantic_files(case):
    files = list((case / 'src').rglob('*')) + list((case / 'elicit-proofs').rglob('*'))
    files += [case / 'Cargo.toml', case / 'Cargo.lock', case / 'elicit.toml']
    return {str(p.relative_to(case)): digest(p) for p in files if p.is_file()}


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def run_arm(args, name, notes):
    case = args.output / 'cases' / name
    logs = args.output / 'results' / name
    logs.mkdir(parents=True)
    shutil.copytree(args.source, case, ignore=shutil.ignore_patterns('.elicit', 'target', '.git'))
    before = semantic_files(case)
    calls = []

    def invoke(label, *cmd):
        start = time.monotonic()
        proc = subprocess.run([str(args.binary), *cmd], cwd=case, capture_output=True, timeout=900)
        elapsed = time.monotonic() - start
        (logs / (label + '.stdout')).write_bytes(proc.stdout)
        (logs / (label + '.stderr')).write_bytes(proc.stderr)
        row = dict(label=label, argv=list(cmd), wall_seconds=elapsed, exit_code=proc.returncode)
        calls.append(row)
        write_json(logs / 'commands.json', calls)
        print(json.dumps(dict(arm=name, **row)), flush=True)
        if proc.returncode:
            raise RuntimeError(f'{name}/{label} failed: {proc.stderr.decode()[-1000:]}')
        return json.loads(proc.stdout) if '--json' in cmd else None

    def evidence_metrics(label):
        files = list((case / '.elicit/evidence').glob('*.json'))
        receipt = max(files, key=lambda p: p.stat().st_mtime_ns)
        data = json.loads(receipt.read_text())
        assert data['report']['stable']
        results = data['report']['results']
        assert len(results) == 5 and all(r['outcome'] == 'success' and r['scope'] == 'full_contract' for r in results)
        events = []
        for line in (case / '.elicit/events.jsonl').read_text().splitlines():
            event = json.loads(line)
            if event['event'] == 'pipeline_stage_finished':
                events.append(event['data'])
        prior = getattr(evidence_metrics, 'count', 0)
        stages = events[prior:]
        evidence_metrics.count = len(events)
        row = dict(receipt=str(receipt.relative_to(case)), accepted_apis=5, stages=len(stages),
                   cache_hits=sum(e.get('cache_hit') is True for e in stages),
                   cached_modules=[e['stage'] for e in stages if e.get('cache_hit') is True],
                   verifier_stage_ms=sum(e.get('elapsed_ms', 0) for e in stages),
                   input=data['report']['input'])
        write_json(logs / (label + '-evidence.json'), row)
        return row

    invoke('notes-directory', 'notes', '--json')
    location = case / '.elicit/notes' if notes else case
    if notes:
        for report in REPORTS:
            shutil.move(str(case / report), str(location / report))
    invoke('initial-audit', 'audit', '--seconds', '600', '--json')
    initial_audit = evidence_metrics('initial-audit')
    invoke('warm-check', 'check', '--seconds', '600', '--json')
    warm = evidence_metrics('warm-check')
    original_note = location / 'PROGRESS.md'
    original_note.write_text(original_note.read_text() + '\nOperational experiment progress update.\n')
    invoke('progress-refresh', 'refresh', '--json')
    progress = invoke('progress-status', 'status', '--all', '--json')
    expected = 'valid_at_snapshot' if notes else 'stale'
    assert all(g['audit_validity']['state'] == expected for g in progress['goals'])
    invoke('progress-check', 'check', '--seconds', '600', '--json')
    progress_check = evidence_metrics('progress-check')
    if notes:
        assert progress_check['cache_hits'] > 0
        assert progress_check['input'] == warm['input'] == initial_audit['input']
    report = location / 'FINAL_REPORT.md'
    report.write_text(report.read_text() + '\nOperational experiment final report update.\n')
    invoke('final-refresh', 'refresh', '--json')
    final = invoke('final-status', 'status', '--all', '--json')
    assert all(g['audit_validity']['state'] == expected for g in final['goals'])
    final_audit = None
    if not notes:
        invoke('final-audit', 'audit', '--seconds', '600', '--json')
        final_audit = evidence_metrics('final-audit')
    else:
        assert all(not g['audit_validity']['fresh_audit_needed_at_snapshot'] for g in final['goals'])
        assert all(g['audit_validity']['receipt'] == initial_audit['receipt'] for g in final['goals'])
    assert semantic_files(case) == before
    write_json(logs / 'semantic-hashes.json', before)
    shutil.copytree(case / '.elicit/evidence', logs / 'evidence')
    shutil.copy2(case / '.elicit/events.jsonl', logs / 'events.jsonl')
    shutil.copy2(case / '.elicit/snapshot.json', logs / 'snapshot.json')
    measured = [c for c in calls if c['label'].startswith(('progress-', 'final-'))]
    summary = dict(arm=name, initial_audit=initial_audit, warm_check=warm,
                   progress_check=progress_check, final_audit=final_audit,
                   measured_wall_seconds=sum(c['wall_seconds'] for c in measured),
                   final_audit_command_executed=not notes, source_and_proofs_unchanged=True,
                   note_policy='Operational notes are never read by build/proof code; explicit trust boundary.')
    write_json(logs / 'summary.json', summary)
    return summary


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    for key in ('source', 'binary', 'output'):
        setattr(args, key, getattr(args, key).resolve())
    args.output.mkdir(parents=True, exist_ok=True)
    manifest = dict(source=str(args.source), binary=str(args.binary), binary_sha256=digest(args.binary),
                    protocol='Sequential root-report baseline then reserved-notes arm; completed five-API proofs; no authoring.',
                    source_semantic_hashes=semantic_files(args.source))
    write_json(args.output / 'protocol.json', manifest)
    summaries = [run_arm(args, 'root-reports', False), run_arm(args, 'reserved-notes', True)]
    assert digest(args.binary) == manifest['binary_sha256']
    assert semantic_files(args.source) == manifest['source_semantic_hashes']
    write_json(args.output / 'comparison.json', summaries)


if __name__ == '__main__':
    main()
