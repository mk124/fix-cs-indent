#![cfg(unix)]

mod common;

use std::fs;
use std::io::Write;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::process::{Command, Stdio};

use common::{Env, bin_path, cs, read};
use indoc::indoc;

#[test]
fn failed_write_reports_error_without_truncating_source() {
    let e = Env::new();
    let padding = "x".repeat(5_000);
    let source = format!(
        "class A\n{{\n    string Padding = \"{padding}\";\n    int x;\n\n    int y;\n}}\n// preserved-tail\n"
    );
    let path = e.write("limited.cs", source.as_bytes());
    let json = format!(
        r#"{{"tool_name":"Edit","tool_input":{{"file_path":"{}"}}}}"#,
        path.display()
    );

    let mut child = Command::new("/bin/sh")
        .args([
            "-c",
            "trap '' XFSZ; ulimit -f 1; exec \"$1\"",
            "fix-cs-indent-test",
        ])
        .arg(bin_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn with file-size limit");
    child
        .stdin
        .as_mut()
        .expect("child stdin")
        .write_all(json.as_bytes())
        .expect("write hook input");
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("wait for hook");

    assert!(!output.status.success(), "write failure must exit non-zero");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("failed to update"),
        "write failure must explain the error: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        read(&path),
        source.as_bytes(),
        "a failed update must leave every source byte unchanged"
    );
}

#[test]
fn update_through_symlink_preserves_the_link() {
    let e = Env::new();
    let target = e.write(
        "target.txt",
        &cs(indoc! {"
            class A
            {
                int x;

                int y;
            }
        "}),
    );
    let link = e.dir.path().join("linked.cs");
    symlink(&target, &link).expect("create symlink");

    e.run(&link).assert_exit_0();

    assert!(
        fs::symlink_metadata(&link)
            .expect("read symlink metadata")
            .file_type()
            .is_symlink(),
        "the hook must update the target without replacing the symlink"
    );
    assert_eq!(
        read(&target),
        cs(indoc! {"
            class A
            {
                int x;
            ····
                int y;
            }
        "})
    );
}

#[test]
fn update_preserves_unix_permissions() {
    let e = Env::new();
    let path = e.write(
        "mode.cs",
        &cs(indoc! {"
            class A
            {
                int x;

                int y;
            }
        "}),
    );
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).expect("set fixture permissions");

    e.run(&path).assert_exit_0();

    assert_eq!(
        read(&path),
        cs(indoc! {"
            class A
            {
                int x;
            ····
                int y;
            }
        "})
    );
    assert_eq!(
        fs::metadata(&path)
            .expect("read updated metadata")
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
}
