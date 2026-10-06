#!/usr/bin/env python3
"""EH part 2: build each pair's triage input deterministically from its E0d labeller prompt.

The labeller prompt is: header, a context paragraph (what TOPIC and DIFF are), then an
instruction section running from "Answer exactly one question:" up to the "## TOPIC" heading,
then the TOPIC and DIFF. The instruction section is replaced, byte for byte everywhere else
unchanged, by the registered triage instruction and a one-line reason. Reads only
<id>.prompt.md; never reads labels or detectors.json.
usage: build_triage.py <E0d/labels dir> <out dir>
"""
import sys, os, re, hashlib
src, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
TRIAGE = (
    "Answer stale: yes|no: would this change make any statement or diagram in this topic wrong? "
    "Answer yes when unsure.\n\n"
    "Reply with exactly two lines and nothing else:\n\n"
    "stale: yes|no\n"
    "reason: <one line>\n\n"
)
START = "Answer exactly one question:\n"
END = "## TOPIC ("
ids = sorted(f[:-len(".prompt.md")] for f in os.listdir(src) if f.endswith(".prompt.md"))
h = hashlib.sha256()
for i in ids:
    t = open(os.path.join(src, i + ".prompt.md"), encoding="utf-8").read()
    a = t.index(START); b = t.index(END)
    assert t.count(START) >= 1 and a < b, i
    new = t[:a] + TRIAGE + t[b:]
    open(os.path.join(out, i + ".triage.md"), "w", encoding="utf-8").write(new)
    h.update(new.encode())
print(len(ids), "inputs; sha256", h.hexdigest()[:16])
