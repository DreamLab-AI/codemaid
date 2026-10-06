#!/bin/bash
# GLM-5.3-Flash ER reviewer: identical prompt.txt (pack + lens) as Luna, on stdin, no tools, empty cwd.
c=$1; o=/home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/erg/reviews/$c-glm; mkdir -p $o
cp /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/47520655-f105-4c26-b318-3f1103e004ae/scratchpad/er/reviews/$c-luna/prompt.txt $o/prompt.txt
cd /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/erg/empty
s=$(date +%s.%N); date -u +%FT%TZ > $o/start
env -i HOME=/home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/erg/zai-home PATH="$PATH" TERM=dumb ANTHROPIC_BASE_URL="$ZAI_URL" ANTHROPIC_API_KEY="$ZAI_ANTHROPIC_API_KEY" CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000 \
  claude -p --bare --strict-mcp-config --tools "" --model glm-5.3-flash --output-format json < $o/prompt.txt > $o/out.json 2> $o/stderr.log
echo "exit=$? wall=$(echo "$(date +%s.%N) - $s" | bc)" > $o/run.log
