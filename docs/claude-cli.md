# Claude Code CLI

Each query is one run of `claude -p` (print mode): a single turn, then the process exits.

## Invocation

```bash
claude --print --output-format json \
  --model <model> --effort <level> \
  --system-prompt <prompt of the command> \
  --tools "" --strict-mcp-config --setting-sources "" \
  --disable-slash-commands --no-session-persistence
```

The prompt is written to stdin, never passed as an argument: a query starting with `-` cannot be read as an option.

| Option | Why |
| --- | --- |
| `--output-format json` | One JSON object with `result`, `is_error`, usage and cost, instead of free text |
| `--system-prompt` | Replaces the Claude Code system prompt, written for coding sessions, with the prompt of the command |
| `--tools ""` | No tool at all: text sent from a selection can never run a command or edit a file |
| `--strict-mcp-config` | Skips the configured MCP servers (start-up time, tokens) |
| `--setting-sources ""` | Ignores user, project and local settings, and the hooks they define |
| `--disable-slash-commands` | Skips skills and custom commands |
| `--no-session-persistence` | Nothing is written to the session history |

The process runs in the system temporary directory, so no project `CLAUDE.md` is picked up from the working directory.

`--bare` would skip even more start-up work, but it only accepts `ANTHROPIC_API_KEY` and never reads a subscription login, so the workflow does not use it.

## Environment

Alfred runs scripts with a minimal environment. The CLI gets:

- `PATH`: the directory of the CLI, then `/opt/homebrew/bin` and `/usr/local/bin`, then the inherited `PATH`
- `USER`: taken from `LOGNAME` or the home directory when missing; without it, the CLI does not find its login in the keychain and answers `Not logged in`
- `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`: turns off telemetry, error reporting and update checks; their flush when the CLI exits adds about 1 second to every query

Other variables are inherited, so `ANTHROPIC_API_KEY` or a Bedrock or Vertex configuration of Claude Code applies when set in the environment Alfred passes.

## Authentication, usage and cost

Authentication is entirely Claude Code's: the workflow stores no credentials.

- **Claude subscription login** (`claude` then `/login`): queries count toward the subscription usage limits; nothing is billed per query. The `total_cost_usd` field of the JSON result is the list price equivalent, not a charge.
- **`ANTHROPIC_API_KEY`**: queries are billed per token by the API.

A short query to Claude Haiku at `low` level uses roughly 1,000 input tokens (system prompt plus the context Claude Code adds) and the length of the answer in output tokens.

## Latency

Each query starts a CLI process, which adds about 0.2 seconds to the model answer. Medians over 5 runs, Claude Haiku 5.5 at `low` level, compared with the Bedrock workflow (same model, cached key):

| Query | Bedrock workflow | This workflow | Without `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` |
| --- | --- | --- | --- |
| Spell check of a sentence | 1.06 s | 1.18 s | 2.18 s |
| Translation of a sentence | 0.93 s | 1.26 s | 2.07 s |
| Three-sentence answer | 1.97 s | 2.05 s | 3.01 s |

The workflow kills the CLI after 120 seconds.

## Errors

| Situation | Alfred shows |
| --- | --- |
| CLI not found | `Claude Code not found at <path>: install it or set claude_path` |
| Not logged in | ``Claude Code is not logged in: run `claude` in a terminal, then /login`` |
| JSON result with `is_error` | `claude failed: <result>` (API error, overload, usage limit…) |
| Output that is not JSON | `claude failed: <stderr, else stdout>` |
| No answer after 120 s | `claude timed out after 120s` |
