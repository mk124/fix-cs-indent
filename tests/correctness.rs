// Tests that the hook produces the right output on valid C# files.

mod common;
use common::{Env, cs, read};
use indoc::indoc;

// ---------- core correctness ----------

#[test]
fn fix_blank_line_in_method_body() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            int x;

            int y;
        }
    "}));
    let out = e.run(&path);
    out.assert_exit_0();
    out.assert_no_stderr();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            int x;
        ····
            int y;
        }
    "}));
}

#[test]
fn fix_nested_block_indent() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                int x;

                int y;
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                int x;
        ········
                int y;
            }
        }
    "}));
}

#[test]
fn blank_before_closing_brace_keeps_scope_indent() {
    // Next non-blank is `}` at indent 0; without the dedent rule the blank
    // would copy `}`'s indent (empty). With the rule it copies the deeper
    // previous-line indent (4 spaces).
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            int x;

        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            int x;
        ····
        }
    "}));
}

#[test]
fn blank_before_nested_closing_brace_keeps_method_body_indent() {
    // Blank should be 8-space indented (method body), not 4-space (closing brace).
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                int x;

            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                int x;
        ········
            }
        }
    "}));
}

#[test]
fn top_level_blank_stays_empty() {
    let e = Env::new();
    let src = cs(indoc! {"
        using A;

        using B;
    "});
    let path = e.write("a.cs", &src);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), src);
}

#[test]
fn tab_indent_copied() {
    let e = Env::new();
    let path = e.write("a.cs", b"class A\n{\n\tint x;\n\n\tint y;\n}\n");
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), b"class A\n{\n\tint x;\n\t\n\tint y;\n}\n");
}

#[test]
fn crlf_preserved() {
    let e = Env::new();
    let path = e.write(
        "a.cs",
        b"class A\r\n{\r\n    int x;\r\n\r\n    int y;\r\n}\r\n",
    );
    e.run(&path).assert_exit_0();
    assert_eq!(
        read(&path),
        b"class A\r\n{\r\n    int x;\r\n    \r\n    int y;\r\n}\r\n"
    );
}

#[test]
fn already_correct_zero_diff() {
    let e = Env::new();
    let original = cs(indoc! {"
        class A
        {
            int x;
        ····
            int y;
        }
    "});
    let path = e.write("a.cs", &original);
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), original);
}

// ---------- boundary file shapes ----------

#[test]
fn zero_byte_file() {
    let e = Env::new();
    let path = e.write("a.cs", b"");
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), b"");
}

#[test]
fn single_line_no_trailing_newline() {
    let e = Env::new();
    let path = e.write("a.cs", b"class A {}");
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), b"class A {}");
}

#[test]
fn no_trailing_newline_preserved_on_fix() {
    let e = Env::new();
    let path = e.write("a.cs", b"class A\n{\n    int x;\n\n    int y;\n}");
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), b"class A\n{\n    int x;\n    \n    int y;\n}");
}

#[test]
fn trailing_blank_line_untouched_when_no_next_nonblank() {
    let e = Env::new();
    let path = e.write("a.cs", b"class A {}\n\n");
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), b"class A {}\n\n");
}

// ---------- idempotency ----------

fn assert_idempotent(name: &str, src: &[u8]) {
    let e = Env::new();
    let path = e.write(name, src);
    e.run(&path).assert_exit_0();
    let after_first = read(&path);
    e.run(&path).assert_exit_0();
    let after_second = read(&path);
    assert_eq!(
        after_first, after_second,
        "fixture {name} drifted on second run"
    );
}

#[test]
fn idempotent_blank_with_indent() {
    assert_idempotent("a.cs", &cs(indoc! {"
        class A
        {
            int x;

            int y;
        }
    "}));
}

#[test]
fn idempotent_already_correct() {
    assert_idempotent("a.cs", &cs(indoc! {"
        class A
        {
            int x;
        ····
            int y;
        }
    "}));
}

#[test]
fn idempotent_crlf() {
    assert_idempotent(
        "a.cs",
        b"class A\r\n{\r\n    int x;\r\n\r\n    int y;\r\n}\r\n",
    );
}

#[test]
fn idempotent_tab_indent() {
    assert_idempotent("a.cs", b"class A\n{\n\tint x;\n\n\tint y;\n}\n");
}

#[test]
fn idempotent_with_utf8_bom() {
    let mut src = b"\xEF\xBB\xBF".to_vec();
    src.extend_from_slice(&cs(indoc! {"
        class A
        {
            int x;

            int y;
        }
    "}));
    assert_idempotent("a.cs", &src);
}

#[test]
fn idempotent_dedent_before_brace() {
    assert_idempotent("a.cs", &cs(indoc! {"
        class A
        {
            int x;

        }
    "}));
}

// ---------- AST-driven scope indent ----------
//
// Blank-line indent must come from the AST scope the blank line lives in,
// not from the byte-level indent of the previous non-blank line. The prev-line
// heuristic confuses multi-line statement continuations with "deeper scope".
//
// A few of these tests currently fail — they pin the expected behavior for
// the AST rewrite. Others are regression guards: the current heuristic happens
// to produce the right answer for them, and the AST rewrite must not regress.

// Blank between a fluent-chain continuation (`.H();`, 16 spaces) and a new
// statement (`int y`, 8 spaces). Answer is 8 — method body indent — not 16.
// Currently fails (produces 16 due to prev-deeper rule).
#[test]
fn blank_after_fluent_chain_continuation() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                var x = G()
                    .H();

                int y = 1;
            }
            int G() { return 0; }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                var x = G()
                    .H();
        ········
                int y = 1;
            }
            int G() { return 0; }
        }
    "}));
}

// Same shape but the continuation is a binary-op wrap, not a method chain.
// Currently fails.
#[test]
fn blank_after_binary_op_continuation() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                int a = 1
                    + 2;

                int b = 3;
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                int a = 1
                    + 2;
        ········
                int b = 3;
            }
        }
    "}));
}

// Continuation is a multi-line argument list. Currently fails.
#[test]
fn blank_after_arg_list_continuation() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                G(
                    1,
                    2);

                H();
            }
            void G(int a, int b) { }
            void H() { }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                G(
                    1,
                    2);
        ········
                H();
            }
            void G(int a, int b) { }
            void H() { }
        }
    "}));
}

// switch-case body. Blank should take case-body indent (16), not case-label
// indent (12). Regression guard under current heuristic; AST rewrite must
// treat `switch_section` as a scope container.
#[test]
fn blank_inside_switch_case_body() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                switch (1)
                {
                    case 1:
                        int a = 1;

                        int b = 2;
                        break;
                }
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                switch (1)
                {
                    case 1:
                        int a = 1;
        ················
                        int b = 2;
                        break;
                }
            }
        }
    "}));
}

// Object initializer block. Blank should take initializer-member indent (12).
// Regression guard; AST rewrite must treat `initializer_expression` as a scope.
#[test]
fn blank_inside_object_initializer() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                var o = new B
                {
                    X = 1,

                    Y = 2
                };
            }
        }
        class B { public int X; public int Y; }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                var o = new B
                {
                    X = 1,
        ············
                    Y = 2
                };
            }
        }
        class B { public int X; public int Y; }
    "}));
}

// Accessor list (`get`/`set`). Blank takes accessor indent (8).
// Regression guard; AST rewrite must treat `accessor_list` as a scope.
#[test]
fn blank_inside_accessor_list() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            int _x;
            int X
            {
                get { return _x; }

                set { _x = value; }
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            int _x;
            int X
            {
                get { return _x; }
        ········
                set { _x = value; }
            }
        }
    "}));
}

// Enum body. Regression guard; AST rewrite must treat
// `enum_member_declaration_list` as a scope.
#[test]
fn blank_inside_enum_body() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        enum E
        {
            A,

            B,
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        enum E
        {
            A,
        ····
            B,
        }
    "}));
}

// Class body between two methods. Regression guard; AST rewrite must treat
// `declaration_list` as a scope.
#[test]
fn blank_between_methods_in_class() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F() { }

            void G() { }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F() { }
        ····
            void G() { }
        }
    "}));
}

// Namespace body. Regression guard; namespace body is also `declaration_list`.
#[test]
fn blank_inside_namespace() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        namespace N
        {
            class A { }

            class B { }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        namespace N
        {
            class A { }
        ····
            class B { }
        }
    "}));
}

// Lambda body is a `block` node. Regression guard.
#[test]
fn blank_inside_lambda_body() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                System.Action g = () =>
                {
                    int a = 1;

                    int b = 2;
                };
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                System.Action g = () =>
                {
                    int a = 1;
        ············
                    int b = 2;
                };
            }
        }
    "}));
}

// Blank after a nested block's closing brace. The enclosing scope is the
// outer method body (indent 8), not the inner block. Regression guard.
#[test]
fn blank_after_nested_block_close() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                if (true)
                {
                    int y = 1;
                }

                int z = 2;
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                if (true)
                {
                    int y = 1;
                }
        ········
                int z = 2;
            }
        }
    "}));
}

// Empty block — the AST scope has no member rows. Fall back to the brace
// indent (4 spaces, the current heuristic's answer). Regression guard for
// backward compatibility. Deepening this to the would-be member indent would
// require assuming a fixed indent step, which this project refuses to do.
#[test]
fn blank_inside_empty_block() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {

            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
        ····
            }
        }
    "}));
}

// Blank between two multi-line statements. Both surrounding statements have
// continuation lines, so the prev non-blank line (12 spaces) is the
// continuation of the prior statement, not a scope indent. Currently fails.
#[test]
fn blank_between_two_multi_line_statements() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                var a = P()
                    .Q();

                var b = R()
                    .S();
            }
            int P() { return 0; }
            int Q() { return 0; }
            int R() { return 0; }
            int S() { return 0; }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                var a = P()
                    .Q();
        ········
                var b = R()
                    .S();
            }
            int P() { return 0; }
            int Q() { return 0; }
            int R() { return 0; }
            int S() { return 0; }
        }
    "}));
}

// Blank between two switch cases — lives in `switch_body` itself, not in any
// `switch_section`. Regression guard; AST rewrite must treat `switch_body` as
// a scope and walk into its first `switch_section` for the member row.
#[test]
fn blank_between_switch_cases() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                switch (1)
                {
                    case 1: break;

                    case 2: break;
                }
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                switch (1)
                {
                    case 1: break;
        ············
                    case 2: break;
                }
            }
        }
    "}));
}

// ---------- review follow-ups ----------
//
// Added after an independent review called out container-node gaps and
// "first member" pitfalls. Most are regression guards where the current
// heuristic happens to produce the right answer; the AST rewrite must not
// regress them. Passing these pins forces the implementation to treat
// several more node kinds as scopes and to filter the "first member" choice.

// Blank inside a multi-line argument list. The innermost container is
// `argument_list`, which is NOT in the original plan's 7-node list. If the
// AST rewrite skips `argument_list` and walks up to the method body, it
// would give 8 spaces; correct answer is 12 (argument indent). Regression
// guard: the old heuristic gives 12 because prev == next == 12.
#[test]
fn blank_inside_arg_list() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                G(
                    1,

                    2);
            }
            void G(int a, int b) { }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                G(
                    1,
        ············
                    2);
            }
            void G(int a, int b) { }
        }
    "}));
}

// Blank between a case label and the first statement of the case body. The
// innermost container is `switch_section`, whose first named child is a
// `pattern` (the `1` in `case 1:`), NOT a statement. If the AST rewrite
// naively takes the first named child, it would give 12 (case-label indent);
// correct answer is 16 (case-body indent). The rewrite must filter out
// pattern / when_clause / expression children and pick the first statement.
#[test]
fn blank_inside_switch_section_before_any_statement() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                switch (1)
                {
                    case 1:

                        int a = 1;
                        break;
                }
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                switch (1)
                {
                    case 1:
        ················
                        int a = 1;
                        break;
                }
            }
        }
    "}));
}

// Blank after a `#endif` whose enclosing `preproc_if` is the first member of
// the class's declaration_list. If the AST rewrite takes the first named
// child verbatim, it gets `preproc_if` starting at column 0 and would
// collapse the blank to 0 spaces. The rewrite must skip start_col == 0
// members (or preproc_* kinds) and pick the next real declaration.
#[test]
fn blank_with_preproc_if_first_member() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
        #if DEBUG
            int x;
        #endif

            int y;
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
        #if DEBUG
            int x;
        #endif
        ····
            int y;
        }
    "}));
}

// Blank inside an anonymous object initializer. The innermost container is
// `anonymous_object_creation_expression` (not `initializer_expression`). If
// the AST rewrite doesn't list it as a scope, it would walk up to the method
// body and give 8; correct is 12 (member indent).
#[test]
fn blank_inside_anonymous_object() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                var o = new
                {
                    X = 1,

                    Y = 2
                };
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                var o = new
                {
                    X = 1,
        ············
                    Y = 2
                };
            }
        }
    "}));
}

// Two consecutive blank lines must both receive the scope indent. The AST
// rewrite must not, e.g., process only one and leave the other at 0 spaces.
#[test]
fn consecutive_blank_lines() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            int x;


            int y;
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            int x;
        ····
        ····
            int y;
        }
    "}));
}

// Idempotency pins for the two currently-failing cases: after the AST
// rewrite makes them pass, they must not drift on re-run.
#[test]
fn idempotent_fluent_chain_continuation() {
    assert_idempotent("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                var x = G()
                    .H();

                int y = 1;
            }
            int G() { return 0; }
        }
    "}));
}

#[test]
fn idempotent_arg_list_continuation() {
    assert_idempotent("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                G(
                    1,
                    2);

                H();
            }
            void G(int a, int b) { }
            void H() { }
        }
    "}));
}

// ---------- review round 2 follow-ups ----------
//
// Added after a second independent review flagged:
//   1. `labeled_statement` unwrap (a `block` whose first member is `label:`
//      has a shallower start column than its body, so the AST rewrite must
//      unwrap into the inner statement — otherwise the blank gets the label's
//      column instead of the body's).
//   2. Removing the `column > 0` filter. The filter was meant to skip
//      top-of-file `#region`/preproc, but `preproc_*` kinds already handle
//      that; meanwhile `column > 0` breaks legitimate zero-indent code.
//   3. Five container kinds listed in the plan but never pinned:
//      `bracketed_argument_list`, `property_pattern_clause`,
//      `switch_expression`, `collection_expression`, `attribute_list`.

// labeled_statement: block's first named child is the label, which starts at a
// shallower column than the label's body. AST rewrite must unwrap
// `labeled_statement` to its inner statement (grammar: named_child(1)).
#[test]
fn blank_inside_labeled_statement_block() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                label:
                    int x = 1;

                    int y = 2;
                    goto label;
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                label:
                    int x = 1;
        ············
                    int y = 2;
                    goto label;
            }
        }
    "}));
}

// Zero-column block body. The previous plan had a `column > 0` filter that
// would cause the AST path to skip all members and fall back to the byte
// heuristic — which then picks the deeper continuation indent (4) instead of
// the correct 0. Currently fails under the heuristic alone; a correct AST
// path (no column filter) must give 0.
#[test]
fn blank_inside_zero_column_block() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
        void F()
        {
        var x = G()
            .H();

        int y = 1;
        }
        int G() { return 0; }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
        void F()
        {
        var x = G()
            .H();

        int y = 1;
        }
        int G() { return 0; }
        }
    "}));
}

// `bracketed_argument_list` — indexer call with multi-line args. The blank's
// innermost container is the `[...]` list; AST rewrite must list this kind.
#[test]
fn blank_inside_bracketed_arg_list() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            void F()
            {
                int[,] a = new int[10, 10];
                int x = a[
                    1,

                    2];
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            void F()
            {
                int[,] a = new int[10, 10];
                int x = a[
                    1,
        ············
                    2];
            }
        }
    "}));
}

// `property_pattern_clause` — C# 8 pattern matching `{ A: B, C: D }`.
#[test]
fn blank_inside_property_pattern() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            bool F(object o) => o is {
                Length: > 0,

                Hash: 0
            };
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            bool F(object o) => o is {
                Length: > 0,
        ········
                Hash: 0
            };
        }
    "}));
}

// `switch_expression` — C# 8 switch expression with multi-line arms. AST
// rewrite must list `switch_expression` as a container; its first named child
// is the input expression (same row as the container) so it must be skipped
// by the "same row" rule, falling to the first arm.
#[test]
fn blank_inside_switch_expression() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            int F(int y)
            {
                return y switch
                {
                    1 => 10,

                    _ => 20,
                };
            }
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            int F(int y)
            {
                return y switch
                {
                    1 => 10,
        ············
                    _ => 20,
                };
            }
        }
    "}));
}

// `collection_expression` — C# 12 `[1, 2]` literal. tree-sitter-c-sharp
// 0.23.5's node-types.json defines `collection_expression`, but whether the
// compiled parser actually produces one without ERROR for this fixture is
// worth pinning. If this test fails by producing the input unchanged, that
// means `has_error` tripped and main.rs skipped — indicating the parser
// rejects C# 12 syntax and we should drop `collection_expression` from the
// plan's container list. Otherwise, AST rewrite must list it.
#[test]
fn blank_inside_collection_expression() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        class A
        {
            int[] X = [
                1,

                2
            ];
        }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        class A
        {
            int[] X = [
                1,
        ········
                2
            ];
        }
    "}));
}

// `attribute_list` — multi-line attribute list `[A,\n B]`. Rare in practice
// but the plan lists `attribute_list` as a container, so pin the behavior.
#[test]
fn blank_inside_attribute_list() {
    let e = Env::new();
    let path = e.write("a.cs", &cs(indoc! {"
        [Obsolete,

         Serializable]
        class A { }
    "}));
    e.run(&path).assert_exit_0();
    assert_eq!(read(&path), cs(indoc! {"
        [Obsolete,
        ·
         Serializable]
        class A { }
    "}));
}
