// Tests that strings, comments, and parse-error regions are preserved untouched.

mod common;
use common::{Env, cs, read};
use indoc::indoc;

// ---------- string and comment danger zones ----------

#[test]
fn verbatim_string_blank_untouched() {
    let e = Env::new();
    let src = cs(indoc! {r#"
        class B
        {
            string s = @"line1

        line3";
        }
    "#});
    let path = e.write("b.cs", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}

#[test]
fn raw_string_blank_untouched() {
    let e = Env::new();
    let src = cs(indoc! {r#"
        class B
        {
            string s = """
        line1

        line3
        """;
        }
    "#});
    let path = e.write("b.cs", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}

#[test]
fn interpolated_verbatim_blank_untouched() {
    let e = Env::new();
    let src = cs(indoc! {r#"
        class B
        {
            void F(int p)
            {
                var s = $@"x{p}

        end";
            }
        }
    "#});
    let path = e.write("b.cs", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}

#[test]
fn raw_interpolated_string_blank_untouched() {
    // C# 11 raw interpolated string literal $""" ... """.
    let e = Env::new();
    let src = cs(indoc! {r#"
        class B
        {
            void F(int p)
            {
                var s = $"""x{p}

        end""";
            }
        }
    "#});
    let path = e.write("b.cs", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}

#[test]
fn interpolated_verbatim_at_dollar_blank_untouched() {
    // Alternate ordering: @$"..." instead of $@"..."
    let e = Env::new();
    let src = cs(indoc! {r#"
        class B
        {
            void F(int p)
            {
                var s = @$"x{p}

        end";
            }
        }
    "#});
    let path = e.write("b.cs", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}

#[test]
fn block_comment_blank_untouched() {
    let e = Env::new();
    let src = cs(indoc! {"
        class B
        {
            /* line1

            line3 */
            int x;
        }
    "});
    let path = e.write("b.cs", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}

#[test]
fn blank_before_doc_comment_uses_method_body_indent() {
    // `/// summary` lines are themselves comment nodes, so the blank-line
    // search skips them. Both the prev (`int x;`) and the next non-comment
    // (`void F() {}`) are at indent 4 — blank gets 4 spaces.
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            int x;

            /// <summary>Doc</summary>
            void F() {}
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            int x;
        ····
            /// <summary>Doc</summary>
            void F() {}
        }
    "}));
}

#[test]
fn blank_after_block_comment_is_fixed() {
    let e = Env::new();
    let path = e.write("b.cs", &cs(indoc! {"
        class B
        {
            /* c */

            int y;
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class B
        {
            /* c */
        ····
            int y;
        }
    "}));
}

// ---------- ERROR gate (any syntax error => exit 0, no change) ----------

#[test]
fn unclosed_verbatim_string_exits_zero_no_change() {
    let e = Env::new();
    let src = cs(indoc! {r#"
        class C
        {
            string s = @"oops

            int x;
        }
    "#});
    let path = e.write("c.cs", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}

#[test]
fn unclosed_block_comment_exits_zero_no_change() {
    let e = Env::new();
    let src = cs(indoc! {"
        class C
        {
            /* oops

            int x;
        }
    "});
    let path = e.write("c.cs", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}

#[test]
fn missing_semicolon_exits_zero_no_change() {
    let e = Env::new();
    let src = cs(indoc! {"
        class C
        {
            int x

            int y;
        }
    "});
    let path = e.write("c.cs", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}

// `#if/#else/#endif` spanning an array initializer is valid C# that tree-sitter-c-sharp
// fails to parse. A blank line in an unrelated, well-formed method body must
// still be fixed — we should not abandon the whole file just because one local
// region trips the parser. The class-body blank stays untouched because its
// container (`declaration_list`) has a parse error somewhere in its subtree.
#[test]
fn preproc_in_array_init_does_not_block_unrelated_fix() {
    let e = Env::new();
    let path = e.write("c.cs", &cs(indoc! {r#"
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
    "#}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {r#"
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
    "#}));
}
