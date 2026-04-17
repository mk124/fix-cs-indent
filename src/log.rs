// Opt-in logging via FIX_CS_INDENT_LOG env var. Unix timestamp + action + path.

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn event(action: &str, path: Option<&Path>) {
    let Ok(log_path) = env::var("FIX_CS_INDENT_LOG") else { return };
    if log_path.is_empty() { return; }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let path_str = path.and_then(|p| p.to_str()).unwrap_or("<none>");
    let line = format!("{ts} {action} {path_str}\n");
    if let Some(parent) = Path::new(&log_path).parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&log_path) {
        let _ = f.write_all(line.as_bytes());
    }
}
