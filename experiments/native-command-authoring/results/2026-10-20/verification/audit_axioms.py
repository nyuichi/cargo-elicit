"""Reject theorem roots with axioms beyond Lean's three permitted foundations."""
import re
import sys
from pathlib import Path

roots = {
    "Verification.new_correct", "Verification.checksum_correct",
    "Verification.from_checksum_correct", "Verification.write_slice_correct",
    "Verification.adler32_slice_correct", "Verification.codec_state", "Verification.codec_word",
    "Verification.write_slice_returns", "Verification.adler32_slice_returns",
    "Verification.split_update", "Verification.codec_state_execution", "Verification.codec_word_execution",
    "Verification.save_restore_resume", "Verification.split_save_restore_resume",
}
allowed = {"propext", "Classical.choice", "Quot.sound"}
output = Path(sys.argv[1]).read_text()
found = {}
for name, axioms in re.findall(r"'([^']+)' depends on axioms: \[([^\]]*)\]", output, re.S):
    found[name] = {a.strip() for a in axioms.split(',') if a.strip()}
for name in re.findall(r"'([^']+)' does not depend on any axioms", output):
    found[name] = set()
if set(found) != roots:
    raise SystemExit(f"Unexpected audit inventory: {set(found) ^ roots}")
for name, axioms in found.items():
    if axioms - allowed:
        raise SystemExit(f"Forbidden axioms in {name}: {axioms - allowed}")
print(f"Axiom audit passed for {len(roots)} theorem roots.")
