# Alfred Claude Workflow

An Alfred workflow to ask Claude, correct spelling and translate text directly from Alfred.

It calls Claude through [Claude Code](https://code.claude.com/docs) in non-interactive mode (`claude -p`): there is no API key to configure, the workflow reuses the Claude Code login. The workflow itself is a single native binary, with no dependency on Node.js, Python or your shell configuration.

## Requirements

- [Alfred 5+](https://www.alfredapp.com/) with Powerpack
- macOS on Apple Silicon (arm64)
- [Claude Code](https://code.claude.com/docs) installed and logged in once in a terminal (`claude`, then `/login`), with a Claude subscription or an API key

## Installation

### From Release

1. Download `claude.alfredworkflow` from the [releases page](https://github.com/pdegeeter/alfred-claude-workflow/releases)
2. Double-click to install in Alfred

The binary is not notarized. If macOS refuses to run it, remove the quarantine flag from the installed workflow folder (Alfred → Workflows → right-click the workflow → Open in Finder):

```bash
xattr -dr com.apple.quarantine "<workflow folder>"
```

### From Source

Requires a [Rust toolchain](https://rustup.rs/) (stable).

```bash
make install
```

## How It Works

Each query runs `claude -p` once, with the prompt on stdin, a system prompt of the workflow and everything a coding session needs turned off: no tools, no settings files (so no hooks), no MCP servers, no slash commands, no saved session, no telemetry. A short answer takes about as long as with the Bedrock workflow (1 to 2 seconds with Claude Haiku). See [`docs/`](docs/):

- [Architecture](docs/architecture.md): request flow, modules, configuration
- [Claude Code CLI](docs/claude-cli.md): invocation, authentication, usage and cost, latency, errors

### Configuration

| Variable       | Required | Description                                                                    |
| -------------- | -------- | ------------------------------------------------------------------------------ |
| `model`        | No       | `haiku` (default), `sonnet`, `opus` or `fable`: latest model of the family      |
| `level`        | No       | Effort level: `low` (default), `medium`, `high`, `xhigh`, `max`                |
| `default_lang` | No       | Default target language for translations (e.g., en, fr, de)                    |
| `claude_path`  | No       | Path of the Claude Code CLI, when not in `~/.local/bin` or Homebrew            |

Alfred runs workflows without your shell `PATH`, so the CLI is looked up in this order: `claude_path`, `~/.local/bin/claude` (native installer), `/opt/homebrew/bin/claude`, `/usr/local/bin/claude`, then `claude` from the `PATH`.

### Usage and cost

With a Claude subscription login, queries count toward the usage limits of the subscription; nothing is billed per query. With `ANTHROPIC_API_KEY` set in the Claude Code environment, queries are billed per token by the API. A short query to Claude Haiku at `low` level uses about 1,000 input tokens, mostly the system prompt and the context Claude Code adds.

## Usage

| Keyword      | Command      | Description                                                    |
| ------------ | ------------ | -------------------------------------------------------------- |
| `claude`     | Ask Claude   | Free-form question                                             |
| `aspell`     | Spell Check  | Correct spelling and grammar, keeping the language and format  |
| `translate` | Translate    | Translate to `default_lang`, or to the language of a `:xx` prefix |

`aspell` and `translate` are also the keywords of the Bedrock workflow: with both installed, Alfred lists both results.

Translation examples:

- `translate Hello world` → translates to default language
- `translate :fr Hello world` → translates to French
- `translate :de Bonjour le monde` → translates to German

Without the colon, only known codes (de, en, es, fr, it, ja, ko, nl, pl, pt, ru, zh) are read as a language, so text starting with a short word such as "Le chat" is translated as a whole.

### Universal Actions and Hotkeys

Select text anywhere in macOS and use Alfred's Universal Actions (default: `⌘⌥\`): **Ask Claude**, **Spell Check with Claude**, **Translate with Claude**. Each command also has a Hotkey trigger with "Selection in macOS": double-click it in the workflow to set a shortcut.

### Keyboard Shortcuts (all commands)

- **Enter**: View response in Text View (markdown)
- **Cmd + Enter**: Copy response to clipboard
- **Opt + Enter**: Paste response directly

Each query is sent on its own, without the previous messages.

## License

MIT
