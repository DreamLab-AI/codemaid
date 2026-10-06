#!/bin/bash
# usage: run-label.sh <id> <round>
D=/home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/e0d-labels
id=$1; r=$2
cd /home/devuser/workspace/.tmp/lab-empty
for attempt in 1 2; do
  claude -p --model sonnet --tools "" --no-session-persistence --disable-slash-commands --setting-sources "" --strict-mcp-config --system-prompt "You are a careful labeller. Follow the user's instructions exactly." < $D/$id.prompt.md > $D/$id.$r.raw 2>/dev/null
  if python3 - "$D/$id.$r.raw" "$D/$id.$r.json" <<'PY'
import sys,json,re,html
t=open(sys.argv[1]).read().strip()
t=re.sub(r'^```(?:json)?\s*|\s*```$','',t)
try: o=json.loads(t)
except Exception:
    try: o=json.loads(html.unescape(t))
    except Exception: sys.exit(1)
if not isinstance(o,dict) or o.get('verdict') not in('yes','no') or 'reason' not in o: sys.exit(1)
open(sys.argv[2],'w').write(t)
PY
  then rm -f $D/$id.$r.raw; exit 0; fi
  mv $D/$id.$r.raw $D/$id.$r.invalid.txt
done
exit 1
