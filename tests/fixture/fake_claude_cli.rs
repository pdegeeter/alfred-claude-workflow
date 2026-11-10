use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use tempfile::TempDir;

/// Fake `claude` executable recording its arguments (one per line), stdin, environment and working directory,
/// then printing the configured stdout and stderr and exiting with the configured code.
pub struct FakeClaudeCli {
    dir: TempDir,
}

impl FakeClaudeCli {
    const SCRIPT: &str = r#"#!/bin/sh
DIR="__DIR__"
: > "$DIR/args.log"
for arg in "$@"; do
  printf '%s\n' "$arg" >> "$DIR/args.log"
done
cat > "$DIR/stdin.log"
printf '%s' "$PATH" > "$DIR/path.log"
pwd -P > "$DIR/cwd.log"
env > "$DIR/env.log"
cat "$DIR/stdout"
cat "$DIR/stderr" >&2
exit "$(cat "$DIR/exit-code")"
"#;

    /// Answers `answer` successfully, as `claude -p --output-format json` does.
    pub fn answering(answer: &str) -> Self {
        let dir = TempDir::new().unwrap();
        let script = dir.path().join("claude");
        fs::write(
            &script,
            Self::SCRIPT.replace("__DIR__", &dir.path().display().to_string()),
        )
        .unwrap();
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();

        let cli = Self { dir };
        cli.respond(
            &serde_json::json!({ "is_error": false, "result": answer }).to_string(),
            "",
            0,
        );

        cli
    }

    pub fn respond(&self, stdout: &str, stderr: &str, exit_code: i32) {
        fs::write(self.dir.path().join("stdout"), stdout).unwrap();
        fs::write(self.dir.path().join("stderr"), stderr).unwrap();
        fs::write(self.dir.path().join("exit-code"), exit_code.to_string()).unwrap();
    }

    /// Value for the `claude_path` variable.
    pub fn path(&self) -> String {
        self.dir.path().join("claude").display().to_string()
    }

    pub fn dir(&self) -> PathBuf {
        self.dir.path().to_path_buf()
    }

    pub fn args(&self) -> Vec<String> {
        self.read("args.log").lines().map(str::to_owned).collect()
    }

    /// Value following `option` in the last invocation.
    pub fn option(&self, option: &str) -> Option<String> {
        let args = self.args();

        args.iter()
            .position(|arg| arg == option)
            .and_then(|index| args.get(index + 1).cloned())
    }

    pub fn stdin(&self) -> String {
        self.read("stdin.log")
    }

    pub fn last_path(&self) -> String {
        self.read("path.log")
    }

    /// Value of an environment variable in the last invocation.
    pub fn env(&self, name: &str) -> Option<String> {
        let prefix = format!("{name}=");

        self.read("env.log")
            .lines()
            .find_map(|line| line.strip_prefix(&prefix).map(str::to_owned))
    }

    pub fn last_cwd(&self) -> String {
        self.read("cwd.log").trim().to_owned()
    }

    fn read(&self, name: &str) -> String {
        fs::read_to_string(self.dir.path().join(name)).unwrap_or_default()
    }
}
