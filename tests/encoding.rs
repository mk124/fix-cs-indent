// Tests for UTF-8 / UTF-8-BOM handling and the .cs extension gate.

mod common;
use common::{Env, cs, read};
use indoc::indoc;

#[test]
fn utf8_bom_roundtrip() {
    let e = Env::new();
    let mut content = b"\xEF\xBB\xBF".to_vec();
    content.extend_from_slice(&cs(indoc! {"
        class A
        {
            int x;

            int y;
        }
    "}));
    let path = e.write("bom.cs", &content);
    e.run(&path).assert_exit_0();

    let mut expected = b"\xEF\xBB\xBF".to_vec();
    expected.extend_from_slice(&cs(indoc! {"
        class A
        {
            int x;
        ····
            int y;
        }
    "}));
    assert_eq!(read(&path), expected);
}

#[test]
fn utf8_bom_with_verbatim_string_roundtrip() {
    // BOM offset chain stress: tree-sitter parses without BOM, but byte ranges
    // for danger zones must align when concatenated back with the BOM.
    let e = Env::new();
    let mut src = b"\xEF\xBB\xBF".to_vec();
    src.extend_from_slice(&cs(indoc! {r#"
        class B
        {
            string s = @"line1

        line3";

            int y;
        }
    "#}));
    let path = e.write("bom.cs", &src);
    e.run(&path).assert_exit_0();

    let mut expected = b"\xEF\xBB\xBF".to_vec();
    expected.extend_from_slice(&cs(indoc! {r#"
        class B
        {
            string s = @"line1

        line3";
        ····
            int y;
        }
    "#}));
    assert_eq!(read(&path), expected);
}

#[test]
fn invalid_utf8_skip() {
    let e = Env::new();
    let mut src: Vec<u8> = cs(indoc! {"
        class A
        {
            int x;

            int y;
        }
    "});
    src[5] = 0xFF;
    let path = e.write("a.cs", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}

#[test]
fn non_cs_extension_skip() {
    let e = Env::new();
    let src = cs(indoc! {"
        class A
        {
            int x;

            int y;
        }
    "});
    let path = e.write("a.txt", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}
