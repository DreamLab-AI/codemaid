#!/bin/bash
# Runs the 24 EK reviews, 4 at a time; a failed run is retried once and logged.
cd "$(dirname "$0")"
export GEMINI_API_KEY=$(grep -E '^GEMINI_API_KEY=' /home/devuser/workspace/project/agentbox/.env | tail -1 | cut -d= -f2- | tr -d '"'"'"' \r')
one() { id=$1; pack=$2
  if node run.cjs "$pack" "runs/$id" > "logs/$id.out" 2> "logs/$id.err"; then echo "$(date -u +%FT%TZ) $id ok" >> logs/status.log
  else echo "$(date -u +%FT%TZ) $id FAILED attempt1: $(cat logs/$id.err)" >> logs/status.log
    if node run.cjs "$pack" "runs/$id" > "logs/$id.out" 2> "logs/$id.err2"; then echo "$(date -u +%FT%TZ) $id ok on retry" >> logs/status.log
    else echo "$(date -u +%FT%TZ) $id FAILED retry: $(cat logs/$id.err2)" >> logs/status.log; fi; fi; }
export -f one
mkdir -p logs runs
date -u +"%FT%TZ start" >> logs/status.log
cut -f1,2 queue.tsv | xargs -P 4 -L 1 bash -c 'one "$0" "$1"'
date -u +"%FT%TZ end" >> logs/status.log
