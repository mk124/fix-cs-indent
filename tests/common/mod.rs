// Shared helpers for integration tests. `tests/common/mod.rs` is the special
// path that Cargo doesn't treat as a separate test crate.

#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fix-cs-indent"))
}

pub struct Env {
    pub dir: tempfile::TempDir,
}

impl Env {
    pub fn new() -> Self {
        Self {
            dir: tempfile::tempdir().expect("create tempdir"),
        }
    }

    pub fn write(&self, name: &str, content: &[u8]) -> PathBuf {
        let p = self.dir.path().join(name);
        fs::write(&p, content).expect("write fixture");
        p
    }

    pub fn run(&self, file: &Path) -> Outcome {
        self.run_with(file, &[])
    }

    pub fn run_with(&self, file: &Path, extra_env: &[(&str, &str)]) -> Outcome {
        let json = format!(
            r#"{{"tool_name":"Edit","tool_input":{{"file_path":"{}"}}}}"#,
            file.display()
        );
        let mut cmd = Command::new(bin_path());
        cmd.env_clear();
        cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
        for (k, v) in extra_env {
            cmd.env(k, v);
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().expect("spawn");
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(json.as_bytes())
            .unwrap();
        drop(child.stdin.take());
        let output = child.wait_with_output().expect("wait");
        Outcome {
            status: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        }
    }
}

pub struct Outcome {
    pub status: i32,
    #[allow(dead_code)]
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Outcome {
    pub fn assert_exit_0(&self) {
        assert_eq!(
            self.status,
            0,
            "expected exit 0, got {}; stderr={:?}",
            self.status,
            String::from_utf8_lossy(&self.stderr)
        );
    }
    pub fn assert_no_stderr(&self) {
        assert!(
            self.stderr.is_empty(),
            "expected no stderr, got: {:?}",
            String::from_utf8_lossy(&self.stderr)
        );
    }
}

pub fn read(path: &Path) -> Vec<u8> {
    fs::read(path).expect("read file")
}

// In fixture strings, '·' (U+00B7) represents a literal ASCII space. Use it
// where trailing whitespace matters (e.g. blank lines with indent) so IDE
// auto-trim can't break the test.
pub fn cs(s: &str) -> Vec<u8> {
    s.replace('·', " ").into_bytes()
}
