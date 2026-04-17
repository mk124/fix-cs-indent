// Blank-line indentation repair.

use tree_sitter::{Node, Tree};

use crate::danger::{Interval, byte_in_any, range_overlaps_any};

pub enum FixOutcome {
    NoChange,
    Changed(Vec<u8>),
}

struct LineInfo<'a> {
    content_start: usize,
    full_end: usize,
    content: &'a [u8],
    newline: &'a [u8],
}

// tree-sitter-c-sharp node kinds that can directly contain statements or
// members and whose "first member row" determines blank-line indentation.
const CONTAINER_KINDS: &[&str] = &[
    "block",
    "declaration_list",
    "enum_member_declaration_list",
    "accessor_list",
    "switch_body",
    "switch_section",
    "initializer_expression",
    "anonymous_object_creation_expression",
    "argument_list",
    "bracketed_argument_list",
    "collection_expression",
    "property_pattern_clause",
    "switch_expression",
    "attribute_list",
];

pub fn fix_blank_lines(body: &[u8], tree: &Tree, danger: &[Interval]) -> FixOutcome {
    let lines = split_lines(body);
    let mut new_indent: Vec<Option<Vec<u8>>> = vec![None; lines.len()];
    let root = tree.root_node();
    for (idx, line) in lines.iter().enumerate() {
        if !is_blank(line.content) {
            continue;
        }
        if range_overlaps_any(danger, line.content_start, line.full_end) {
            continue;
        }
        let chosen: Vec<u8> = match ast_scope_indent(root, body, line.content_start) {
            Some(ws) => ws.to_vec(),
            None => match heuristic_indent(&lines, idx, danger) {
                Some(ws) => ws.to_vec(),
                None => continue,
            },
        };
        if chosen.is_empty() || chosen == line.content {
            continue;
        }
        new_indent[idx] = Some(chosen);
    }
    if new_indent.iter().all(Option::is_none) {
        return FixOutcome::NoChange;
    }
    let extra: usize = new_indent.iter().flatten().map(Vec::len).sum();
    let mut out = Vec::with_capacity(body.len() + extra);
    for (idx, line) in lines.iter().enumerate() {
        if let Some(ws) = &new_indent[idx] {
            out.extend_from_slice(ws);
            out.extend_from_slice(line.newline);
        } else {
            out.extend_from_slice(&body[line.content_start..line.full_end]);
        }
    }
    FixOutcome::Changed(out)
}

fn ast_scope_indent<'a>(root: Node<'a>, body: &'a [u8], pos: usize) -> Option<&'a [u8]> {
    let container = enclosing_container(root, pos)?;
    first_member_indent(container, body)
}

fn enclosing_container<'a>(root: Node<'a>, pos: usize) -> Option<Node<'a>> {
    let mut node = root.named_descendant_for_byte_range(pos, pos)?;
    loop {
        if CONTAINER_KINDS.contains(&node.kind()) {
            return Some(node);
        }
        node = node.parent()?;
    }
}

fn first_member_indent<'a>(container: Node<'_>, body: &'a [u8]) -> Option<&'a [u8]> {
    let container_row = container.start_position().row;
    let mut cursor = container.walk();
    for child in container.named_children(&mut cursor) {
        if child.kind().starts_with("preproc_") {
            continue;
        }
        if child.start_position().row == container_row {
            continue;
        }
        // labeled_statement grammar is [identifier, statement]; the label's
        // column is shallower than the body's, so unwrap to the inner
        // statement (named_child(1)) to get the body indent.
        let representative = if child.kind() == "labeled_statement" {
            child.named_child(1).unwrap_or(child)
        } else {
            child
        };
        return Some(leading_ws_of_line(body, representative.start_byte()));
    }
    None
}

fn leading_ws_of_line(body: &[u8], pos: usize) -> &[u8] {
    let mut line_start = pos;
    while line_start > 0 && body[line_start - 1] != b'\n' {
        line_start -= 1;
    }
    let mut end = line_start;
    while end < body.len() && (body[end] == b' ' || body[end] == b'\t') {
        end += 1;
    }
    &body[line_start..end]
}

fn heuristic_indent<'a>(
    lines: &'a [LineInfo<'a>],
    idx: usize,
    danger: &[Interval],
) -> Option<&'a [u8]> {
    let next_ws = find_next_nonblank_indent(lines, idx, danger);
    let prev_ws = find_prev_nonblank_indent(lines, idx, danger);
    match (prev_ws, next_ws) {
        (Some(p), Some(n)) if p.len() > n.len() => Some(p),
        (_, Some(n)) => Some(n),
        (_, None) => None,
    }
}

fn find_next_nonblank_indent<'a>(
    lines: &'a [LineInfo<'a>],
    idx: usize,
    danger: &[Interval],
) -> Option<&'a [u8]> {
    let mut j = idx + 1;
    while j < lines.len() {
        let cand = &lines[j];
        if is_blank(cand.content) {
            j += 1;
            continue;
        }
        if byte_in_any(danger, cand.content_start) {
            j += 1;
            continue;
        }
        return Some(leading_ws(cand.content));
    }
    None
}

fn find_prev_nonblank_indent<'a>(
    lines: &'a [LineInfo<'a>],
    idx: usize,
    danger: &[Interval],
) -> Option<&'a [u8]> {
    if idx == 0 {
        return None;
    }
    let mut j = idx - 1;
    loop {
        let cand = &lines[j];
        if !is_blank(cand.content) && !byte_in_any(danger, cand.content_start) {
            return Some(leading_ws(cand.content));
        }
        if j == 0 {
            return None;
        }
        j -= 1;
    }
}

fn split_lines(body: &[u8]) -> Vec<LineInfo<'_>> {
    let mut lines = Vec::new();
    let mut i = 0;
    while i < body.len() {
        let line_start = i;
        let mut j = i;
        while j < body.len() && body[j] != b'\n' {
            j += 1;
        }
        let newline: &[u8];
        let content_end;
        let full_end;
        if j < body.len() {
            if j > line_start && body[j - 1] == b'\r' {
                newline = &body[j - 1..=j];
                content_end = j - 1;
            } else {
                newline = &body[j..=j];
                content_end = j;
            }
            full_end = j + 1;
        } else {
            newline = &[];
            content_end = j;
            full_end = j;
        }
        lines.push(LineInfo {
            content_start: line_start,
            full_end,
            content: &body[line_start..content_end],
            newline,
        });
        i = full_end;
        if i == line_start {
            break;
        }
    }
    lines
}

fn is_blank(content: &[u8]) -> bool {
    content.iter().all(|&b| b == b' ' || b == b'\t')
}

fn leading_ws(content: &[u8]) -> &[u8] {
    let end = content
        .iter()
        .position(|&b| b != b' ' && b != b'\t')
        .unwrap_or(content.len());
    &content[..end]
}
