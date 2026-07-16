// fix-cs-indent: PostToolUse hook that restores blank-line indentation in .cs
// files after Claude Code's Write/Edit tools strip it.

mod danger;
mod fix;
mod log;

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::Value;
use tree_sitter::Parser;

use crate::fix::FixOutcome;

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("fix-cs-indent: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let Some(file_path) = read_file_path() else {
        log::event("skip-no-file-path", None);
        return Ok(());
    };

    let cs = file_path
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|e| e.eq_ignore_ascii_case("cs"));
    if !cs {
        log::event("skip-non-cs", Some(&file_path));
        return Ok(());
    }

    let Ok(target_path) = file_path.canonicalize() else {
        log::event("skip-read-failed", Some(&file_path));
        return Ok(());
    };
    let Ok(bytes) = fs::read(&target_path) else {
        log::event("skip-read-failed", Some(&file_path));
        return Ok(());
    };

    let body_offset = if bytes.starts_with(UTF8_BOM) {
        UTF8_BOM.len()
    } else {
        0
    };
    let body = &bytes[body_offset..];

    if std::str::from_utf8(body).is_err() {
        log::event("skip-not-utf8", Some(&file_path));
        return Ok(());
    }

    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
        .is_err()
    {
        log::event("skip-parser-init", Some(&file_path));
        return Ok(());
    }
    let Some(tree) = parser.parse(body, None) else {
        log::event("skip-parse-failed", Some(&file_path));
        return Ok(());
    };

    let danger = danger::collect(&tree);

    let new_body = match fix::fix_blank_lines(body, &tree, &danger) {
        FixOutcome::NoChange => {
            log::event("skip-no-change", Some(&file_path));
            return Ok(());
        }
        FixOutcome::Changed(b) => b,
    };

    let mut final_bytes = Vec::with_capacity(body_offset + new_body.len());
    final_bytes.extend_from_slice(&bytes[..body_offset]);
    final_bytes.extend_from_slice(&new_body);

    match replace_if_unchanged(&target_path, &bytes, &final_bytes) {
        Ok(true) => log::event("fix", Some(&file_path)),
        Ok(false) => log::event("skip-concurrent-change", Some(&file_path)),
        Err(error) => {
            log::event("update-failed", Some(&file_path));
            return Err(io::Error::new(
                error.kind(),
                format!("failed to update {}: {error}", file_path.display()),
            ));
        }
    }
    Ok(())
}

fn replace_if_unchanged(path: &Path, original: &[u8], replacement: &[u8]) -> io::Result<bool> {
    let parent = path.parent().expect("canonicalized file path has a parent");
    let permissions = fs::metadata(path)?.permissions();
    let mut temp = tempfile::Builder::new()
        .prefix(".fix-cs-indent-")
        .tempfile_in(parent)?;

    temp.write_all(replacement)?;
    temp.as_file().set_permissions(permissions.clone())?;
    temp.as_file().sync_all()?;

    if fs::read(path)? != original {
        return Ok(false);
    }

    let _persisted = temp.persist(path)?;
    #[cfg(windows)]
    {
        // tempfile clears Windows file attributes before replacing a file.
        _persisted.set_permissions(permissions)?;
        _persisted.sync_all()?;
    }
    Ok(true)
}

fn read_file_path() -> Option<PathBuf> {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).ok()?;
    let value: Value = serde_json::from_str(&raw).ok()?;
    let s = value.get("tool_input")?.get("file_path")?.as_str()?;
    Some(PathBuf::from(s))
}
