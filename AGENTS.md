# Agent Workspace Rules

## CLI Tool Preferences
- **Content Search**: ALWAYS use `rg` (ripgrep) instead of `grep`. Never invoke `grep`.
- **File Finding**: ALWAYS use `fd` instead of `find`. Never invoke `find`.
  - *Why*: `find` traverses `target/`, `node_modules/`, and `.git/`, causing hangs, timeouts, and truncated output. `fd` respects `.gitignore` and finishes in milliseconds.

## Automation and Scripting
- The agent is permitted and encouraged to use `curl` and `python3` commands to fetch web content, automate tasks, run data processing scripts, or perform programmatic checks (bypassing SSL verification if needed).

## Audio Notifications
- Whenever a long-running process, background task, build, or wait completes, execute the macOS `say` command (e.g. `say "Build complete"`) to audibly notify the user.
