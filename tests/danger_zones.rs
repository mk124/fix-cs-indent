// Tests that strings, comments, and parse-error regions are preserved untouched.

mod common;
use common::{assert_file_after_run, cs};
use indoc::indoc;

// ---------- string and comment danger zones ----------

#[test]
fn raw_string_blank_untouched() {
    assert_file_after_run(
        "b.cs",
        &cs(indoc! {r#"
        class B
        {
            string s = """
        line1

        line3
        """;

            int x;
        }
    "#}),
        &cs(indoc! {r#"
        class B
        {
            string s = """
        line1

        line3
        """;
        ····
            int x;
        }
    "#}),
    );
}

#[test]
fn interpolated_verbatim_blank_untouched() {
    assert_file_after_run(
        "b.cs",
        &cs(indoc! {r#"
        class B
        {
            void F(int p)
            {
                var first = $@"x{p}

        end";
                var second = @$"x{p}

        end";

                Use(p);
            }
            void Use(int value) { }
        }
    "#}),
        &cs(indoc! {r#"
        class B
        {
            void F(int p)
            {
                var first = $@"x{p}

        end";
                var second = @$"x{p}

        end";
        ········
                Use(p);
            }
            void Use(int value) { }
        }
    "#}),
    );
}

#[test]
fn raw_interpolated_string_blank_untouched() {
    // C# 11 raw interpolated string literal $""" ... """.
    assert_file_after_run(
        "b.cs",
        &cs(indoc! {r#"
        class B
        {
            void F(int p)
            {
                var s = $"""x{p}

        end""";

                Use(p);
            }
            void Use(int value) { }
        }
    "#}),
        &cs(indoc! {r#"
        class B
        {
            void F(int p)
            {
                var s = $"""x{p}

        end""";
        ········
                Use(p);
            }
            void Use(int value) { }
        }
    "#}),
    );
}

#[test]
fn block_comment_blank_untouched() {
    assert_file_after_run(
        "b.cs",
        &cs(indoc! {"
        class B
        {
            int x;

            int y;
            /* line1

            line3 */
        }
    "}),
        &cs(indoc! {"
        class B
        {
            int x;
        ····
            int y;
            /* line1

            line3 */
        }
    "}),
    );
}

#[test]
fn blank_before_doc_comment_uses_method_body_indent() {
    // Documentation comments do not determine the surrounding blank's indent.
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            int x;

            /// <summary>Doc</summary>
            void F() {}
        }
    "}),
        &cs(indoc! {"
        class A
        {
            int x;
        ····
            /// <summary>Doc</summary>
            void F() {}
        }
    "}),
    );
}

#[test]
fn blank_after_block_comment_is_fixed() {
    assert_file_after_run(
        "b.cs",
        &cs(indoc! {"
        class B
        {
            /* c */

            int y;
        }
    "}),
        &cs(indoc! {"
        class B
        {
            /* c */
        ····
            int y;
        }
    "}),
    );
}

// ---------- parse-error regions ----------

#[test]
fn unclosed_verbatim_string_exits_zero_no_change() {
    assert_file_after_run(
        "c.cs",
        &cs(indoc! {r#"
        class Valid
        {
            int x;

            int y;
        }
        class C
        {
            string s = @"oops

            int x;
        }
    "#}),
        &cs(indoc! {r#"
        class Valid
        {
            int x;
        ····
            int y;
        }
        class C
        {
            string s = @"oops

            int x;
        }
    "#}),
    );
}

#[test]
fn unclosed_block_comment_exits_zero_no_change() {
    assert_file_after_run(
        "c.cs",
        &cs(indoc! {"
        class Valid
        {
            int x;

            int y;
        }
        class C
        {
            /* oops

            int x;
        }
    "}),
        &cs(indoc! {"
        class Valid
        {
            int x;
        ····
            int y;
        }
        class C
        {
            /* oops

            int x;
        }
    "}),
    );
}

#[test]
fn missing_semicolon_exits_zero_no_change() {
    assert_file_after_run(
        "c.cs",
        &cs(indoc! {"
        class Valid
        {
            int x;

            int y;
        }
        class C
        {
            int x

            int y;
        }
    "}),
        &cs(indoc! {"
        class Valid
        {
            int x;
        ····
            int y;
        }
        class C
        {
            int x

            int y;
        }
    "}),
    );
}

// Tree-sitter cannot parse `#if/#else/#endif` spanning this array initializer.
// Blanks in the affected class region stay untouched, while a blank inside the
// unrelated well-formed method is still repaired.
#[test]
fn preproc_in_array_init_does_not_block_unrelated_fix() {
    assert_file_after_run(
        "c.cs",
        &cs(indoc! {r#"
        public static class C
        {
            static readonly string[] Paths =
            {
                #if UNITY_EDITOR_WIN
                @"C:\p4.exe",
                #else
                "/usr/bin/p4",
                #endif
            };

            static string Resolve()
            {
                string name = "p4";

                return name;
            }
        }
    "#}),
        &cs(indoc! {r#"
        public static class C
        {
            static readonly string[] Paths =
            {
                #if UNITY_EDITOR_WIN
                @"C:\p4.exe",
                #else
                "/usr/bin/p4",
                #endif
            };

            static string Resolve()
            {
                string name = "p4";
        ········
                return name;
            }
        }
    "#}),
    );
}
