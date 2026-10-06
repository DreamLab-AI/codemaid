#!/usr/bin/env python3
"""EH scoring: triage answers vs E0d gold (stale = l1 yes and l2 yes); part 3 projection."""
import json, os, re, sys, math, statistics as st
HERE = os.path.dirname(os.path.abspath(__file__))
LAB = sys.argv[1]
MODELS = ["glm-5.3-flash", "haiku-4.5", "sonnet-5.5"]
def wilson(k, n, z=1.96):
    if n == 0: return None
    p = k / n; d = 1 + z*z/n; c = p + z*z/(2*n); h = z*math.sqrt(p*(1-p)/n + z*z/(4*n*n))
    return [round((c - h)/d, 4), round((c + h)/d, 4)]
def verdict(l):
    try: return json.load(open(os.path.join(LAB, l)))["verdict"]
    except FileNotFoundError: return None
ids = sorted(f[:-10] for f in os.listdir(LAB) if f.endswith(".prompt.md"))
gold = {i: verdict(i + ".l1.json") == "yes" and verdict(i + ".l2.json") == "yes" for i in ids}
assert sum(gold.values()) == 29 and len(ids) == 60
def parse(t):
    s = re.sub(r"[*_`#>]", "", (t or "").lower())
    m = re.search(r'stale\s*[:=]\s*"?(yes|no)\b', s)
    if m: return m.group(1), True
    w = s.strip().split()
    if w and w[0].strip(".,:;") in ("yes", "no"): return w[0].strip(".,:;"), True
    return "yes", False
batch = json.load(open(os.path.join(HERE, "batching.json")))
res = {"models": {}, "gold": {"stale": 29, "pairs": 60}}
for m in MODELS:
    rows = []; fails = []
    for i in ids:
        p = os.path.join(HERE, "runs", m, i + ".json")
        if not os.path.exists(p): fails.append(i); continue
        r = json.load(open(p)); v, ok = parse(r["result"])
        u = r.get("usage") or {}
        tok = {k: u.get(k, 0) or 0 for k in ("input_tokens", "cache_read_input_tokens", "cache_creation_input_tokens", "output_tokens")}
        rows.append({"id": i, "stratum": "agentbox" if i.startswith("ab") else "visionclaw", "gold_stale": gold[i],
                     "triage": v, "parsed": ok, "raw": r["result"].strip()[:400], "wall_s": r["wall_s"],
                     "tokens": tok, "total_tokens": sum(tok.values()), "cost_usd": r.get("total_cost_usd"),
                     "infra_retries": len(r.get("failed_infra_attempts", []))})
    def stats(rs):
        st_ = [r for r in rs if r["gold_stale"]]; ys = [r for r in rs if r["triage"] == "yes"]
        k = sum(r["triage"] == "yes" for r in st_); tp = sum(r["gold_stale"] for r in ys); no = len(rs) - len(ys)
        return {"n": len(rs), "stale": len(st_), "recall": round(k/len(st_), 4) if st_ else None, "recall_k": k,
                "recall_wilson95": wilson(k, len(st_)), "rejection_rate": round(no/len(rs), 4) if rs else None, "rejected": no,
                "rejection_wilson95": wilson(no, len(rs)), "precision": round(tp/len(ys), 4) if ys else None,
                "precision_wilson95": wilson(tp, len(ys)), "flagged": len(ys),
                "missed_stale": [r["id"] for r in st_ if r["triage"] == "no"]}
    S = stats(rows)
    S["by_stratum"] = {s: stats([r for r in rows if r["stratum"] == s]) for s in ("visionclaw", "agentbox")}
    S["unparsable"] = [{"id": r["id"], "raw": r["raw"]} for r in rows if not r["parsed"]]
    S["infra_failed"] = fails
    S["infra_retries"] = sum(r["infra_retries"] for r in rows)
    tt = [r["total_tokens"] for r in rows]; ws = [r["wall_s"] for r in rows]; cs = [r["cost_usd"] or 0 for r in rows]
    S["cost"] = {"mean_total_tokens": round(st.mean(tt), 1), "median_total_tokens": st.median(tt),
                 "mean_input_side_tokens": round(st.mean([r["total_tokens"] - r["tokens"]["output_tokens"] for r in rows]), 1),
                 "mean_output_tokens": round(st.mean([r["tokens"]["output_tokens"] for r in rows]), 1),
                 "mean_wall_s": round(st.mean(ws), 2), "median_wall_s": round(st.median(ws), 2), "max_wall_s": round(max(ws), 2),
                 "sum_cost_usd_claude_code_reported": round(sum(cs), 4), "mean_cost_usd_claude_code_reported": round(st.mean(cs), 5)}
    S["qualifies"] = S["recall"] is not None and S["recall"] >= 0.90 and not fails
    S["rejection_per_1k_tokens"] = round(S["rejection_rate"] / (S["cost"]["mean_total_tokens"]/1000), 5)
    proj = {}
    for repo in ("visionclaw", "agentbox"):
        wk = batch[repo]["week"]["rechecks"]; base = batch[repo]["commit"]["rechecks"]
        proj[repo] = {"per_commit_baseline": base, "weekly_rechecks": wk,
                      "expected_reauthors": round(wk*(1 - S["rejection_rate"]), 1),
                      "sensitivity_stratum_rate": round(wk*(1 - S["by_stratum"][repo]["rejection_rate"]), 1)}
    proj["pooled"] = {k: round(proj["visionclaw"][k] + proj["agentbox"][k], 1) for k in proj["visionclaw"]}
    proj["pooled"]["saving_vs_per_commit"] = round(1 - proj["pooled"]["expected_reauthors"]/proj["pooled"]["per_commit_baseline"], 4)
    proj["pooled"]["saving_vs_weekly_batching"] = round(1 - proj["pooled"]["expected_reauthors"]/proj["pooled"]["weekly_rechecks"], 4)
    S["projection"] = proj
    S["pairs"] = rows
    res["models"][m] = S
q = [m for m in MODELS if res["models"][m]["qualifies"]]
res["qualifying"] = q
res["recommended"] = max(q, key=lambda m: res["models"][m]["rejection_per_1k_tokens"]) if q else None
res["decision"] = (f"{res['recommended']} is the triage layer" if q else "no model qualifies: the hybrid is batching alone")
json.dump(res, open(os.path.join(HERE, "score.json"), "w"), indent=1)
for m in MODELS:
    s = res["models"][m]
    print(m, "recall", s["recall"], s["recall_wilson95"], "rej", s["rejection_rate"], "prec", s["precision"], "tok", s["cost"]["mean_total_tokens"],
          "wall", s["cost"]["mean_wall_s"], "usd", s["cost"]["sum_cost_usd_claude_code_reported"], "unparsable", len(s["unparsable"]), "missed", s["missed_stale"], "fails", s["infra_failed"])
print("decision:", res["decision"])
