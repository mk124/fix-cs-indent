// Tests for UTF-8 BOM and multibyte text preservation.

mod common;
use common::{assert_file_after_run, cs};
use indoc::indoc;

#[test]
fn utf8_bom_roundtrip() {
    let mut content = b"\xEF\xBB\xBF".to_vec();
    content.extend_from_slice(&cs(indoc! {"
        class A
        {
            int x;

            int y;
        }
    "}));
    let mut expected = b"\xEF\xBB\xBF".to_vec();
    expected.extend_from_slice(&cs(indoc! {"
        class A
        {
            int x;
        ····
            int y;
        }
    "}));
    assert_file_after_run("bom.cs", &content, &expected);
}

#[test]
fn utf8_bom_with_verbatim_string_roundtrip() {
    // The multiline string stays byte-for-byte intact while the later code
    // blank is indented and the BOM is preserved.
    let mut src = b"\xEF\xBB\xBF".to_vec();
    src.extend_from_slice(&cs(indoc! {r#"
        class B
        {
            string s = @"line1

        line3";

            int y;
        }
    "#}));
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
    assert_file_after_run("bom.cs", &src, &expected);
}

#[test]
fn valid_multibyte_utf8_is_preserved_while_fixing_indent() {
    assert_file_after_run(
        "utf8.cs",
        &cs(indoc! {r#"
            class 文档
            {
                string 问候 = "你好，🌏";

                int 数量 = 1;
            }
        "#}),
        &cs(indoc! {r#"
            class 文档
            {
                string 问候 = "你好，🌏";
            ····
                int 数量 = 1;
            }
        "#}),
    );
}
