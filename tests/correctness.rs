// Tests that the hook produces the right output on valid C# files.

mod common;
use common::{assert_file_after_run, cs};
use indoc::indoc;

// ---------- core correctness ----------

#[test]
fn fix_blank_lines_in_class_body() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            int x;
        ····
            int y;

            int z;
        }
    "}),
        &cs(indoc! {"
        class A
        {
            int x;
        ····
            int y;
        ····
            int z;
        }
    "}),
    )
    .assert_silent();
}

#[test]
fn fix_nested_block_indent() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            void F()
            {
                int x;

                int y;
            }
        }
    "}),
        &cs(indoc! {"
        class A
        {
            void F()
            {
                int x;
        ········
                int y;
            }
        }
    "}),
    );
}

#[test]
fn blank_before_closing_brace_keeps_scope_indent() {
    // The blank belongs with the class member at indent 4, not the closing
    // brace at indent 0.
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            int x;

        }
    "}),
        &cs(indoc! {"
        class A
        {
            int x;
        ····
        }
    "}),
    );
}

#[test]
fn blank_before_nested_closing_brace_keeps_method_body_indent() {
    // Blank should be 8-space indented (method body), not 4-space (closing brace).
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            void F()
            {
                int x;

            }
        }
    "}),
        &cs(indoc! {"
        class A
        {
            void F()
            {
                int x;
        ········
            }
        }
    "}),
    );
}

#[test]
fn tab_indent_copied() {
    assert_file_after_run(
        "a.cs",
        b"class A\n{\n\tint x;\n\n\tint y;\n}\n",
        b"class A\n{\n\tint x;\n\t\n\tint y;\n}\n",
    );
}

#[test]
fn crlf_preserved() {
    assert_file_after_run(
        "a.cs",
        b"class A\r\n{\r\n    int x;\r\n\r\n    int y;\r\n}\r\n",
        b"class A\r\n{\r\n    int x;\r\n    \r\n    int y;\r\n}\r\n",
    );
}

// ---------- boundary file shapes ----------

#[test]
fn no_trailing_newline_preserved_on_fix() {
    assert_file_after_run(
        "a.cs",
        b"class A\n{\n    int x;\n\n    int y;\n}",
        b"class A\n{\n    int x;\n    \n    int y;\n}",
    );
}

#[test]
fn trailing_blank_line_untouched_when_no_next_nonblank() {
    assert_file_after_run(
        "a.cs",
        b"class A\n{\n    int x;\n\n    int y;\n}\n\n",
        b"class A\n{\n    int x;\n    \n    int y;\n}\n\n",
    );
}

// ---------- Scope indentation ----------

// Blank between a fluent-chain continuation (`.H();`, 16 spaces) and a new
// statement (`int y`, 8 spaces). Answer is 8 — method body indent — not 16.
#[test]
fn blank_after_fluent_chain_continuation() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// A binary-expression continuation must not become the next blank's indent.
#[test]
fn blank_after_binary_op_continuation() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            void F()
            {
                int a = 1
                    + 2;

                int b = 3;
            }
        }
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// A multi-line argument list must not become the following blank's indent.
#[test]
fn blank_after_arg_list_continuation() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// The blank takes the case-body indent (16), not the case-label indent (12).
#[test]
fn blank_inside_switch_case_body() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// The blank takes the object initializer member indent (12).
#[test]
fn blank_inside_object_initializer() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// The blank between accessors takes the accessor indent (8).
#[test]
fn blank_inside_accessor_list() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            int _x;
            int X
            {
                get { return _x; }

                set { _x = value; }
            }
        }
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// The blank between enum members takes the member indent.
#[test]
fn blank_inside_enum_body() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        enum E
        {
            A,

            B,
        }
    "}),
        &cs(indoc! {"
        enum E
        {
            A,
        ····
            B,
        }
    "}),
    );
}

// The blank between declarations takes the namespace member indent.
#[test]
fn blank_inside_namespace() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        namespace N
        {
            class A { }

            class B { }
        }
    "}),
        &cs(indoc! {"
        namespace N
        {
            class A { }
        ····
            class B { }
        }
    "}),
    );
}

// The blank inside a lambda takes the lambda body indent.
#[test]
fn blank_inside_lambda_body() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// Blank after a nested block's closing brace. The enclosing scope is the
// outer method body (indent 8), not the inner block.
#[test]
fn blank_after_nested_block_close() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// An empty block has no member indent to copy, so the blank follows the braces
// rather than assuming a fixed indentation width.
#[test]
fn blank_inside_empty_block() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            void F()
            {

            }
        }
    "}),
        &cs(indoc! {"
        class A
        {
            void F()
            {
        ····
            }
        }
    "}),
    );
}

// A blank between switch cases takes the case-label indent.
#[test]
fn blank_between_switch_cases() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// A blank inside a multi-line argument list takes the argument indent (12),
// not the surrounding method body indent (8).
#[test]
fn blank_inside_arg_list() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// A blank between a case label and its first statement takes the case-body
// indent (16), not the label indent (12).
#[test]
fn blank_inside_switch_section_before_any_statement() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// A column-zero preprocessor directive must not collapse the following class
// blank to column zero.
#[test]
fn blank_with_preproc_if_first_member() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
        #if DEBUG
            int x;
        #endif

            int y;
        }
    "}),
        &cs(indoc! {"
        class A
        {
        #if DEBUG
            int x;
        #endif
        ····
            int y;
        }
    "}),
    );
}

// A blank inside an anonymous object takes the initializer member indent (12).
#[test]
fn blank_inside_anonymous_object() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// Consecutive blank lines both receive the surrounding member indent.
#[test]
fn consecutive_blank_lines() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            int x;


            int y;
        }
    "}),
        &cs(indoc! {"
        class A
        {
            int x;
        ····
        ····
            int y;
        }
    "}),
    );
}

// A labeled statement's body is deeper than its label; the blank follows the
// body indent.
#[test]
fn blank_inside_labeled_statement_block() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            void F()
            {
                outer:

                    inner:
                        int x = 1;

                        int y = 2;
            }
        }
    "}),
        &cs(indoc! {"
        class A
        {
            void F()
            {
                outer:
        ············
                    inner:
                        int x = 1;
        ················
                        int y = 2;
            }
        }
    "}),
    );
}

// A zero-column method body keeps its blank at column zero even after a
// deeper continuation line.
#[test]
fn blank_inside_zero_column_block() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
        class B
        {
            int x;

            int y;
        }
    "}),
        &cs(indoc! {"
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
        class B
        {
            int x;
        ····
            int y;
        }
    "}),
    );
}

// A blank inside multi-line indexer arguments takes the argument indent.
#[test]
fn blank_inside_bracketed_arg_list() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// A blank inside a property pattern takes the subpattern indent.
#[test]
fn blank_inside_property_pattern() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            bool F(object o) => o is {
                Length: > 0,

                Hash: 0
            };
        }
    "}),
        &cs(indoc! {"
        class A
        {
            bool F(object o) => o is {
                Length: > 0,
        ········
                Hash: 0
            };
        }
    "}),
    );
}

// A blank inside a switch expression takes the arm indent.
#[test]
fn blank_inside_switch_expression() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
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
    "}),
        &cs(indoc! {"
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
    "}),
    );
}

// A blank inside a collection expression takes the element indent.
#[test]
fn blank_inside_collection_expression() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            int[] X = [
                1,

                2
            ];
        }
    "}),
        &cs(indoc! {"
        class A
        {
            int[] X = [
                1,
        ········
                2
            ];
        }
    "}),
    );
}

// A blank inside a multi-line attribute list takes the attribute indent.
#[test]
fn blank_inside_attribute_list() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        [Obsolete,

         Serializable]
        class A { }
    "}),
        &cs(indoc! {"
        [Obsolete,
        ·
         Serializable]
        class A { }
    "}),
    );
}

#[test]
fn blank_lines_in_delimited_csharp_constructs_use_member_indent() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        record R(int X, int Y);

        class A<
            T,

            U>
        {
            Dictionary<
                string,

                int> values;
            int this[
                int first,

                int second] => first + second;
            void F(
                int first,

                int second)
            {
                var tuple = (
                    first,

                    second);
                if (first > 0
                    && second > 0

                    && first != second)
                {
                }
                var copy = new R(1, 2) with
                {
                    X = 3,

                    Y = 4
                };
                if (new[] { 1, 2 } is [
                    1,

                    2])
                {
                }
            }
        }
    "}),
        &cs(indoc! {"
        record R(int X, int Y);

        class A<
            T,
        ····
            U>
        {
            Dictionary<
                string,
        ········
                int> values;
            int this[
                int first,
        ········
                int second] => first + second;
            void F(
                int first,
        ········
                int second)
            {
                var tuple = (
                    first,
        ············
                    second);
                if (first > 0
                    && second > 0
        ············
                    && first != second)
                {
                }
                var copy = new R(1, 2) with
                {
                    X = 3,
        ············
                    Y = 4
                };
                if (new[] { 1, 2 } is [
                    1,
        ············
                    2])
                {
                }
            }
        }
    "}),
    );
}

#[test]
fn blank_lines_in_delimiterless_csharp_constructs_use_local_indent() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        unsafe class A<T> :
            Base,

            IThing
            where T :
                Base,

                IThing
        {
            delegate* unmanaged[Cdecl]<
                int,

                void> pointer;
            void F()
            {
                int first = 1,
                    second = 2,

                    third = 3;
                var sum = first +
                    second +

                    third;
                var chain = Get()
                    .First()

                    .Second();
                var query =
                    from x in values
                    orderby x.Key,

                        x.Value
                    where x.Value > 0

                    select x;
                outer:

                    inner:
                        Use(first);
            }
        }
    "}),
        &cs(indoc! {"
        unsafe class A<T> :
            Base,
        ····
            IThing
            where T :
                Base,
        ········
                IThing
        {
            delegate* unmanaged[Cdecl]<
                int,
        ········
                void> pointer;
            void F()
            {
                int first = 1,
                    second = 2,
        ············
                    third = 3;
                var sum = first +
                    second +
        ············
                    third;
                var chain = Get()
                    .First()
        ············
                    .Second();
                var query =
                    from x in values
                    orderby x.Key,
        ················
                        x.Value
                    where x.Value > 0
        ············
                    select x;
                outer:
        ············
                    inner:
                        Use(first);
            }
        }
    "}),
    );
}

#[test]
fn blank_between_control_flow_clauses_uses_clause_indent() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            void F(bool condition)
            {
                if (condition)
                    Use();

                else
                    Use();
                do
                    Use();

                while (condition);
            }
        }
    "}),
        &cs(indoc! {"
        class A
        {
            void F(bool condition)
            {
                if (condition)
                    Use();
        ········
                else
                    Use();
                do
                    Use();
        ········
                while (condition);
            }
        }
    "}),
    );
}

#[test]
fn top_level_blank_after_continuation_stays_unindented() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        F()
            .G();

        class A
        {
            int x;

            int y;
        }
    "}),
        &cs(indoc! {"
        F()
            .G();

        class A
        {
            int x;
        ····
            int y;
        }
    "}),
    );
}

#[test]
fn comments_do_not_determine_blank_line_indent() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            void BeforeMember()
            {
                    // Intentionally deeper than the method body.

                int value = 1;
            }

            void OnlyComment()
            {

                    // Intentionally deeper than the braces.
            }
        }
    "}),
        &cs(indoc! {"
        class A
        {
            void BeforeMember()
            {
                    // Intentionally deeper than the method body.
        ········
                int value = 1;
            }
        ····
            void OnlyComment()
            {
        ····
                    // Intentionally deeper than the braces.
            }
        }
    "}),
    );
}

#[test]
fn preprocessor_wrapped_statements_set_scope_indent() {
    assert_file_after_run(
        "a.cs",
        &cs(indoc! {"
        class A
        {
            void F()
            {
        #if DEBUG
                G()
                    .H();

                int value = 1;
        #endif
            }
            void OnlyDirectives()
            {

        #if DEBUG
        #endif
            }
        }
    "}),
        &cs(indoc! {"
        class A
        {
            void F()
            {
        #if DEBUG
                G()
                    .H();
        ········
                int value = 1;
        #endif
            }
            void OnlyDirectives()
            {
        ····
        #if DEBUG
        #endif
            }
        }
    "}),
    );
}
