#!/usr/bin/env python3
"""EH part 1: re-checks when each topic is re-checked once per window (from E0 per-commit T_file flags)."""
import json, sys, datetime
E0 = sys.argv[1]
d = json.load(open(E0))
out = {}
for repo in ("visionclaw", "agentbox"):
    cs = d["repos"][repo]["commits"]
    def key(c, w):
        t = datetime.datetime.fromisoformat(c["date"].replace("Z", "+00:00")).astimezone(datetime.timezone.utc)
        if w == "commit": return c["sha"]
        if w == "day": return t.date().isoformat()
        if w == "week": y, wk, _ = t.isocalendar(); return f"{y}-W{wk:02d}"
        return "all"
    r = {"commits": len(cs), "date_field": "E0 commits[].date, converted to UTC"}
    for w in ("commit", "day", "week", "window"):
        buckets = {}
        for c in cs:
            buckets.setdefault(key(c, w), set()).update(c["file"])
        r[w] = {"windows": len(buckets), "rechecks": sum(len(s) for s in buckets.values()),
                "windows_with_flags": sum(1 for s in buckets.values() if s)}
    base = r["commit"]["rechecks"]
    for w in ("day", "week", "window"):
        r[w]["ratio_vs_commit"] = round(base / r[w]["rechecks"], 3) if r[w]["rechecks"] else None
    out[repo] = r
pooled = {w: sum(out[x][w]["rechecks"] for x in out) for w in ("commit", "day", "week", "window")}
out["pooled"] = pooled
json.dump(out, sys.stdout, indent=2, sort_keys=True); print()
