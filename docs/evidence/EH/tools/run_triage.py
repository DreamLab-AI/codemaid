#!/usr/bin/env python3
"""EH part 2 runner: one fresh `claude -p` per (model, pair); prompt on stdin, empty cwd, no tools,
`env -i`. Never sees labels or detectors.json. Z.AI key read fresh from agentbox/.env, never printed
or stored. usage: run_triage.py <model-key> [ids...]"""
import sys, os, json, time, subprocess, concurrent.futures as cf
HERE = os.path.dirname(os.path.abspath(__file__))
INP, RUNS, EMPTY = (os.path.join(HERE, x) for x in ("inputs", "runs", "empty"))
ENVFILE = "/home/devuser/workspace/project/agentbox/.env"
MODELS = {
    "haiku-4.5":  {"id": "claude-haiku-4-5", "zai": False},
    "sonnet-5.5": {"id": "claude-sonnet-5-5", "zai": False},
    "glm-5.3-flash": {"id": "glm-5.3-flash", "zai": True},
}
FLAGS = ["--tools", "", "--setting-sources", "", "--strict-mcp-config", "--no-session-persistence",
         "--disable-slash-commands", "--output-format", "json"]

def zai_key():
    vals = {}
    for line in open(ENVFILE, encoding="utf-8"):
        line = line.strip()
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            vals[k.strip().removeprefix("export ").strip()] = v.strip().strip('"').strip("'")
    k = vals.get("ZAI_ANTHROPIC_API_KEY") or vals.get("ZAI_API_KEY")
    if not k: sys.exit("no Z.AI key in .env")
    return k

def base_env(zai):
    e = {"PATH": os.environ["PATH"], "HOME": os.environ["HOME"], "LC_CTYPE": "C.UTF-8", "LANG": "C.UTF-8"}
    if zai:
        e.update(ANTHROPIC_BASE_URL="https://api.z.ai/api/anthropic", ANTHROPIC_AUTH_TOKEN=zai_key(),
                 ANTHROPIC_API_KEY="")
    return e

def one(mkey, pid, env):
    m = MODELS[mkey]; out = os.path.join(RUNS, mkey, pid + ".json")
    if os.path.exists(out): return pid, "cached"
    prompt = open(os.path.join(INP, pid + ".triage.md"), "rb").read()
    attempts = []
    for attempt in (1, 2, 3):
        t0 = time.time()
        p = subprocess.run(["claude", "-p", "--model", m["id"], *FLAGS], input=prompt, cwd=EMPTY, env=env,
                           capture_output=True, timeout=900)
        wall = time.time() - t0
        try: j = json.loads(p.stdout)
        except Exception: j = None
        ok = p.returncode == 0 and isinstance(j, dict) and not j.get("is_error") and isinstance(j.get("result"), str)
        rec = {"pair": pid, "model": mkey, "model_id": m["id"], "attempt": attempt, "wall_s": round(wall, 2),
               "returncode": p.returncode, "infra_ok": ok,
               "result": j.get("result") if isinstance(j, dict) else None,
               "usage": j.get("usage") if isinstance(j, dict) else None,
               "modelUsage": j.get("modelUsage") if isinstance(j, dict) else None,
               "total_cost_usd": j.get("total_cost_usd") if isinstance(j, dict) else None,
               "duration_api_ms": j.get("duration_api_ms") if isinstance(j, dict) else None,
               "stderr_tail": p.stderr.decode(errors="replace")[-300:] if not ok else "",
               "stdout_head": p.stdout.decode(errors="replace")[:300] if not ok else ""}
        if ok:
            rec["failed_infra_attempts"] = attempts
            json.dump(rec, open(out, "w"), indent=1); return pid, "ok"
        attempts.append(rec); time.sleep(10 * attempt)
    json.dump({"pair": pid, "model": mkey, "infra_failed": True, "attempts": attempts}, open(out + ".fail", "w"), indent=1)
    return pid, "FAIL"

if __name__ == "__main__":
    mkey = sys.argv[1]; os.makedirs(os.path.join(RUNS, mkey), exist_ok=True)
    ids = sys.argv[2:] or sorted(f[:-10] for f in os.listdir(INP) if f.endswith(".triage.md"))
    env = base_env(MODELS[mkey]["zai"])
    with cf.ThreadPoolExecutor(6) as ex:
        for pid, st in ex.map(lambda i: one(mkey, i, env), ids):
            print(mkey, pid, st, flush=True)
