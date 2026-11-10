# AGENTS.md

Guidance for coding agents working on this project.

## Project Overview

Alfred Claude Workflow is an Alfred workflow, written in Rust, that asks Claude through the Claude Code CLI in print mode (`claude -p`). It ships as one arm64 binary, `alfred-claude <aspell|claude|translate> <query>`, with no dependency on the user environment (Node.js, shell `PATH`). Authentication belongs to Claude Code: the workflow stores no credentials.

## Build & Test Commands

- `make check` - `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (`pedantic` enabled in `Cargo.toml`) and `cargo test`; the CI runs this target
- `make fmt` - Format code (`rustfmt.toml`)
- `make build` - Build the release binary and package `claude.alfredworkflow` (`scripts/build-workflow.sh`)
- `make install` - Same, then open the package in Alfred
- `make clean` - Remove build outputs

CI (`.github/workflows/ci.yml`) runs `make check` on `macos-latest`. Releases are managed by release-please (`.github/workflows/release-please.yml`): conventional commits on `main` feed a release PR that bumps `Cargo.toml`, `Cargo.lock` and `info.plist` and updates `CHANGELOG.md`; merging it tags the release, and the `upload` job attaches `claude.alfredworkflow`. Never bump versions by hand. Commit messages and PR titles follow Conventional Commits.

How the workflow works is documented in `docs/`: `architecture.md`, `claude-cli.md`.

## Architecture

```
src/
├── main.rs   # Composition root: reads args and variables, prints Alfred JSON
├── lib.rs
├── client/   # ClaudeCliClient: runs `claude -p`, parses its JSON result
├── command/  # aspell, claude, translate and the shared CommandRunner
├── helper/   # Alfred output, translation query
└── types/    # ClaudeRequest, EffortLevel (one per file)

tests/
├── fixture/  # FakeClaudeCli: fake `claude` executable recording arguments, stdin, PATH and cwd
└── *.rs      # claude_cli_client.rs (CLI wrapper), commands.rs (commands end to end)
```

The only external system is the Claude Code CLI. Tests point the real `claude_path` variable at the fake executable; production code has no test-only hooks.

## Claude Code invocation

`ClaudeCliClient::ask` runs `claude --print --output-format json --model <m> --effort <e> --system-prompt <p> --tools "" --strict-mcp-config --setting-sources "" --disable-slash-commands --no-session-persistence`, with the prompt on stdin and the system temporary directory as working directory. Keep these options: they disable tools (selected text must never run a command), hooks, MCP servers and session files. Do not switch to `--bare`: it ignores subscription logins.

The CLI needs `USER` to find its keychain login; `ClaudeCliClient` derives it from `LOGNAME` or `HOME` when Alfred does not pass it. It also sets `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`: without it, the telemetry flush on exit adds about 1 second per query.

## Alfred Configuration Variables

- `model` (optional) - `haiku` (default), `sonnet`, `opus`, `fable`, passed to `--model`
- `level` (optional) - `low` (default), `medium`, `high`, `xhigh`, `max`, passed to `--effort`
- `default_lang` (optional) - Default target language for translations
- `claude_path` (optional) - CLI location; default: `~/.local/bin/claude`, `/opt/homebrew/bin/claude`, `/usr/local/bin/claude`, then `claude` from the `PATH`. Listed in `variablesdontexport`

## Alfred Workflow Triggers

Each command has three entry points (Script Filter keyword, Universal Action, Hotkey with "Selection in macOS") feeding the same Script Filter, then Text View, Copy and Paste outputs. Keywords are `claude`, `aspell` and `translate`; the last two are shared with the Bedrock workflow. Each Script Filter runs `./alfred-claude <command> "$1"`.

## Code Style

- Clippy `pedantic` (minus `missing_errors_doc`, `missing_panics_doc`, `must_use_candidate`), `unsafe_code` forbidden
- `rustfmt` with `max_width = 120`
- One struct, enum or trait per file, re-exported by the module's `mod.rs`
- Constants and helpers are associated items of their type, never free items above it

## Documentation

Update `AGENTS.md`, `CONTRIBUTING.md`, `README.md` and `docs/` in the same change when adding files, commands, options passed to the CLI or dependencies.

## Running the Workflow

`make install` builds and imports the workflow into Alfred. The binary can also be run directly, configured through environment variables like Alfred does:

```bash
model=haiku ./target/release/alfred-claude translate ":fr Hello world"
```
