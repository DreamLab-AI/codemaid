#!/bin/bash
# ES2: the 3 arm-R2 reviews, concurrent, ES's run.cjs unchanged; a failed run is retried once — over
# node:https with identical bytes if the failure is a fetch/headers timeout (ES2-4), else run.cjs again.
cd "$(dirname "$0")"
export GEMINI_API_KEY=$(grep -E '^GEMINI_API_KEY=' /home/devuser/workspace/project/agentbox/.env | tail -1 | cut -d= -f2- | tr -d '"'"'"' \r')
one() { id=$1; pack=$2
  if node run.cjs "$pack" "runs/$id" > "logs/$id.out" 2> "logs/$id.err"; then echo "$(date -u +%FT%TZ) $id ok" >> logs/status.log
  else echo "$(date -u +%FT%TZ) $id FAILED attempt1: $(cat logs/$id.err)" >> logs/status.log
    r=run.cjs; grep -q -i -E 'fetch failed|timeout|terminated|UND_ERR' "logs/$id.err" && r=run-https.cjs
    if node $r "$pack" "runs/$id" > "logs/$id.out" 2> "logs/$id.err2"; then echo "$(date -u +%FT%TZ) $id ok on retry via $r" >> logs/status.log
    else echo "$(date -u +%FT%TZ) $id FAILED retry via $r: $(cat logs/$id.err2)" >> logs/status.log; fi; fi; }
export -f one
mkdir -p logs runs
date -u +"%FT%TZ start" >> logs/status.log
node run.cjs --count R2/base/pack.txt > tokens.jsonl 2> logs/count.err
cut -f1,2 queue.tsv | xargs -P 3 -L 1 bash -c 'one "$0" "$1"'
date -u +"%FT%TZ end" >> logs/status.log
