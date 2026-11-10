use std::env;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, bail};
use serde_json::Value;

use crate::types::ClaudeRequest;

/// Wrapper around `claude -p`, the non-interactive mode of the Claude Code CLI.
/// The CLI owns authentication: a Claude subscription login from the macOS keychain, or `ANTHROPIC_API_KEY`.
pub struct ClaudeCliClient {
    path: PathBuf,
}

impl ClaudeCliClient {
    const EXECUTABLE: &str = "claude";
    /// Native installer location, relative to the home directory.
    const HOME_INSTALL_DIR: &str = ".local/bin";
    /// Install locations of Homebrew on Apple Silicon and Intel, missing from the PATH Alfred runs scripts with.
    const INSTALL_DIRS: [&str; 2] = ["/opt/homebrew/bin", "/usr/local/bin"];
    const NOT_LOGGED_IN: &str = "Not logged in";
    /// Turns off telemetry, error reporting and update checks, whose flush on exit adds about 1 s per query.
    const NONESSENTIAL_TRAFFIC: (&str, &str) = ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1");
    const POLL_INTERVAL: Duration = Duration::from_millis(20);
    const SYSTEM_PATH: &str = "/usr/bin:/bin:/usr/sbin:/sbin";
    const TIMEOUT: Duration = Duration::from_secs(120);

    /// `configured` when set, else the first install found (native installer, Homebrew), else `claude` from the PATH.
    pub fn locate(configured: Option<&str>, home: Option<&str>) -> Self {
        let path = configured.map_or_else(
            || {
                home.map(|home| Path::new(home).join(Self::HOME_INSTALL_DIR))
                    .into_iter()
                    .chain(Self::INSTALL_DIRS.iter().map(PathBuf::from))
                    .map(|dir| dir.join(Self::EXECUTABLE))
                    .find(|path| path.is_file())
                    .unwrap_or_else(|| PathBuf::from(Self::EXECUTABLE))
            },
            PathBuf::from,
        );

        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Sends `message` as a single turn, without tools, settings, MCP servers nor saved session.
    ///
    /// # Errors
    /// When the CLI is missing, not logged in, times out or reports an error.
    pub fn ask(&self, request: &ClaudeRequest, message: &str) -> anyhow::Result<String> {
        let args = [
            "--print",
            "--output-format",
            "json",
            "--model",
            &request.model,
            "--effort",
            request.effort.as_str(),
            "--system-prompt",
            &request.system_prompt,
            "--tools",
            "",
            "--strict-mcp-config",
            "--setting-sources",
            "",
            "--disable-slash-commands",
            "--no-session-persistence",
        ];
        let (success, stdout, stderr) = self.run(&args, message)?;

        Self::parse(success, &stdout, &stderr)
    }

    /// The prompt goes through stdin, so text starting with `-` is never read as an option.
    fn run(&self, args: &[&str], input: &str) -> anyhow::Result<(bool, String, String)> {
        let mut command = Command::new(&self.path);
        command
            .args(args)
            .current_dir(env::temp_dir())
            .env("PATH", self.child_path())
            .env(Self::NONESSENTIAL_TRAFFIC.0, Self::NONESSENTIAL_TRAFFIC.1)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // The CLI finds its keychain login through USER, which a bare environment may lack.
        if env::var_os("USER").is_none()
            && let Some(user) = Self::fallback_user()
        {
            command.env("USER", user);
        }

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                bail!(
                    "Claude Code not found at {}: install it or set claude_path",
                    self.path.display()
                )
            }
            Err(error) => return Err(error).context("Cannot run claude"),
        };
        let stdin = child.stdin.take();
        let input = input.to_owned();
        let writer = thread::spawn(move || {
            if let Some(mut stdin) = stdin {
                stdin.write_all(input.as_bytes()).ok();
            }
        });
        let stdout = Self::drain(child.stdout.take());
        let stderr = Self::drain(child.stderr.take());
        let deadline = Instant::now() + Self::TIMEOUT;

        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                child.kill().ok();
                child.wait().ok();
                bail!("claude timed out after {}s", Self::TIMEOUT.as_secs());
            }
            thread::sleep(Self::POLL_INTERVAL);
        };
        writer.join().ok();

        Ok((
            status.success(),
            stdout.join().unwrap_or_default(),
            stderr.join().unwrap_or_default(),
        ))
    }

    /// Answer of a `--output-format json` result, or the most specific error available.
    fn parse(success: bool, stdout: &str, stderr: &str) -> anyhow::Result<String> {
        let Ok(data) = serde_json::from_str::<Value>(stdout.trim()) else {
            let message = [stderr.trim(), stdout.trim()]
                .into_iter()
                .find(|text| !text.is_empty())
                .unwrap_or("no output");
            bail!("claude failed: {message}");
        };
        let result = data.get("result").and_then(Value::as_str).unwrap_or_default();
        let is_error = data.get("is_error").and_then(Value::as_bool).unwrap_or(!success);

        if result.starts_with(Self::NOT_LOGGED_IN) {
            bail!("Claude Code is not logged in: run `claude` in a terminal, then /login");
        }
        if is_error || !success {
            let message = if result.is_empty() {
                data.get("subtype").and_then(Value::as_str).unwrap_or("unknown error")
            } else {
                result
            };
            bail!("claude failed: {message}");
        }

        Ok(result.to_owned())
    }

    /// PATH of the CLI process: its own directory and Homebrew first, for the tools it may spawn.
    fn child_path(&self) -> String {
        let cli_dir = self
            .path
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .map(|dir| dir.display().to_string());
        let inherited = env::var("PATH").unwrap_or_else(|_| Self::SYSTEM_PATH.to_owned());

        cli_dir
            .into_iter()
            .chain(Self::INSTALL_DIRS.iter().map(|dir| (*dir).to_owned()))
            .chain([inherited])
            .collect::<Vec<_>>()
            .join(":")
    }

    fn fallback_user() -> Option<String> {
        env::var("LOGNAME").ok().filter(|name| !name.is_empty()).or_else(|| {
            env::var("HOME").ok().and_then(|home| {
                Path::new(&home)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
        })
    }

    fn drain(pipe: Option<impl Read + Send + 'static>) -> JoinHandle<String> {
        thread::spawn(move || {
            let mut output = String::new();
            if let Some(mut pipe) = pipe {
                pipe.read_to_string(&mut output).ok();
            }

            output
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ClaudeCliClient;

    #[test]
    fn returns_the_result_of_a_successful_answer() {
        let answer = ClaudeCliClient::parse(true, r#"{"is_error":false,"result":"Hello"}"#, "").unwrap();

        assert_eq!(answer, "Hello");
    }

    #[test]
    fn reports_a_missing_login_with_the_fix() {
        let error = ClaudeCliClient::parse(
            false,
            r#"{"is_error":true,"result":"Not logged in · Please run /login"}"#,
            "",
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "Claude Code is not logged in: run `claude` in a terminal, then /login"
        );
    }

    #[test]
    fn reports_the_error_result() {
        let error =
            ClaudeCliClient::parse(false, r#"{"is_error":true,"result":"API Error: 529 Overloaded"}"#, "").unwrap_err();

        assert_eq!(error.to_string(), "claude failed: API Error: 529 Overloaded");
    }

    #[test]
    fn falls_back_on_the_subtype_without_result() {
        let error = ClaudeCliClient::parse(false, r#"{"is_error":true,"subtype":"error_max_turns"}"#, "").unwrap_err();

        assert_eq!(error.to_string(), "claude failed: error_max_turns");
    }

    #[test]
    fn reports_stderr_when_the_output_is_not_json() {
        let error = ClaudeCliClient::parse(false, "", "error: unknown option '--effort'\n").unwrap_err();

        assert_eq!(error.to_string(), "claude failed: error: unknown option '--effort'");
    }

    #[test]
    fn reports_stdout_when_stderr_is_empty() {
        let error = ClaudeCliClient::parse(false, "Invalid API key\n", "").unwrap_err();

        assert_eq!(error.to_string(), "claude failed: Invalid API key");
    }
}
