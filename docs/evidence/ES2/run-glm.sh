#!/bin/bash
# ES2 GLM session (author or repair). Usage: run-glm.sh <name> <brief> <workdir>
# env -i: only HOME, PATH, TERM, TMPDIR, Z.AI URL/key, output cap. No thinking budget (default max effort).
E=$(cd "$(dirname "$0")" && pwd); name=$1; brief=$2; wd=$3
mkdir -p "$wd/.scratch" "$E/homes/$name"
cd "$wd"
date -u +%FT%T.%3NZ > "$E/logs/$name.start"
env -i HOME="$E/homes/$name" PATH="$PATH" TERM=dumb TMPDIR="$wd/.scratch" ANTHROPIC_BASE_URL="$ZAI_URL" ANTHROPIC_API_KEY="$(grep -E "^ZAI_ANTHROPIC_API_KEY=" /home/devuser/workspace/project/agentbox/.env | tail -1 | cut -d= -f2- | tr -d "\"' \r")" CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000 \
  claude -p --bare --strict-mcp-config --dangerously-skip-permissions --tools "Read,Edit,Write,Glob,Grep,Bash" --model glm-5.3-flash --output-format json \
  < "$brief" > "$E/logs/$name.json" 2> "$E/logs/$name.err"
echo "rc=$?" > "$E/logs/$name.rc"
date -u +%FT%T.%3NZ > "$E/logs/$name.end"
