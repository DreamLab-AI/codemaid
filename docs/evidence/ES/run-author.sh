#!/bin/bash
cd /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/author
date -u +%FT%T.%3NZ > /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/author.start
env -i HOME=/home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/zai-home PATH="$PATH" TERM=dumb ANTHROPIC_BASE_URL="$ZAI_URL" ANTHROPIC_API_KEY="$ZAI_ANTHROPIC_API_KEY" CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000 \
  claude -p --bare --strict-mcp-config --dangerously-skip-permissions --tools "Read,Edit,Write,Glob,Grep,Bash" --model glm-5.3-flash --output-format json \
  < /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/author-brief.md > /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/author.json 2> /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/author.err
echo "rc=$?" > /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/author.rc
date -u +%FT%T.%3NZ > /home/devuser/workspace/.tmp/claude-1000/-home-devuser-workspace-project-agentbox/b01c5fc4-3280-499b-877b-822952d5876f/scratchpad/es/author.end
