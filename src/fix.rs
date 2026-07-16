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

#[derive(Clone, Copy)]
struct Container<'a> {
    node: Node<'a>,
    content_start: usize,
    content_end: usize,
    prefer_next: bool,
}

pub fn fix_blank_lines(body: &[u8], tree: &Tree, danger: &[Interval]) -> FixOutcome {
    let lines = split_lines(body);
    let mut new_indent: Vec<Option<&[u8]>> = vec![None; lines.len()];
    let root = tree.root_node();
    for (idx, line) in lines.iter().enumerate() {
        if !is_blank(line.content) {
            continue;
        }
        if range_overlaps_any(danger, line.content_start, line.full_end) {
            continue;
        }
        // Skip blank lines whose enclosing container contains a parse error or
        // missing node. A parser limitation in one region must not prevent
        // repairs in unrelated, well-formed regions.
        let container = enclosing_container(root, line.content_start);
        if container.is_some_and(|c| c.node.has_error()) {
            continue;
        }
        let next_indent = || {
            find_next_nonblank_indent(
                &lines,
                idx,
                danger,
                container.map_or(body.len(), |c| c.content_end),
            )
        };
        let Some(chosen) = (match container {
            Some(container) if container.prefer_next => {
                next_indent().or_else(|| first_member_indent(container, body))
            }
            Some(container) => first_member_indent(container, body).or_else(next_indent),
            None => next_indent(),
        }) else {
            continue;
        };
        if chosen.is_empty() || chosen == line.content {
            continue;
        }
        new_indent[idx] = Some(chosen);
    }
    if new_indent.iter().all(Option::is_none) {
        return FixOutcome::NoChange;
    }
    let extra: usize = new_indent.iter().flatten().map(|indent| indent.len()).sum();
    let mut out = Vec::with_capacity(body.len() + extra);
    for (idx, line) in lines.iter().enumerate() {
        if let Some(ws) = new_indent[idx] {
            out.extend_from_slice(ws);
            out.extend_from_slice(line.newline);
        } else {
            out.extend_from_slice(&body[line.content_start..line.full_end]);
        }
    }
    FixOutcome::Changed(out)
}

fn enclosing_container<'a>(root: Node<'a>, pos: usize) -> Option<Container<'a>> {
    let mut node = root.named_descendant_for_byte_range(pos, pos)?;
    // Delimiterless syntax still provides a useful local indentation scope.
    // Keep its smallest node, but climb to the nearest structural container
    // so a parse error in that enclosing scope still blocks the repair.
    let mut local_container = None;
    loop {
        if let Some(container) = as_container(node, pos) {
            return if container.node.has_error() {
                Some(container)
            } else {
                local_container.or(Some(container))
            };
        }
        if local_container.is_none() && node.start_byte() < pos && pos < node.end_byte() {
            local_container = Some(Container {
                node,
                content_start: node.start_byte(),
                content_end: node.end_byte(),
                prefer_next: true,
            });
        }
        node = node.parent()?;
    }
}

fn as_container(node: Node<'_>, pos: usize) -> Option<Container<'_>> {
    let bounds = match node.kind() {
        "compilation_unit" | "switch_section" => Some((node.start_byte(), node.end_byte())),
        "type_argument_list" | "type_parameter_list" => delimiter_bounds(node, pos, "<", ">"),
        _ => [("{", "}"), ("(", ")"), ("[", "]")]
            .into_iter()
            .find_map(|(open, close)| delimiter_bounds(node, pos, open, close)),
    }?;

    Some(Container {
        node,
        content_start: bounds.0,
        content_end: bounds.1,
        prefer_next: false,
    })
}

fn delimiter_bounds(node: Node<'_>, pos: usize, open: &str, close: &str) -> Option<(usize, usize)> {
    let children = 0..node.child_count() as u32;
    let opening = children
        .clone()
        .filter_map(|index| node.child(index))
        .find(|child| !child.is_named() && child.kind() == open);
    let closing = children
        .rev()
        .filter_map(|index| node.child(index))
        .find(|child| !child.is_named() && child.kind() == close);

    opening
        .zip(closing)
        .map(|(opening, closing)| (opening.end_byte(), closing.start_byte()))
        .filter(|&(start, end)| start <= pos && pos <= end)
}

fn first_member_indent<'a>(container: Container<'_>, body: &'a [u8]) -> Option<&'a [u8]> {
    let node = container.node;
    let container_row = node.start_position().row;
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.start_byte() < container.content_start {
            continue;
        }
        if child.start_byte() >= container.content_end {
            break;
        }
        if child.kind() == "comment" {
            continue;
        }
        if child.kind().starts_with("preproc_") {
            if let Some(indent) = first_member_indent(
                Container {
                    node: child,
                    ..container
                },
                body,
            ) {
                return Some(indent);
            }
            continue;
        }
        if node.kind() != "compilation_unit" && child.start_position().row == container_row {
            continue;
        }
        // A label is shallower than its body. When a surrounding scope uses a
        // labeled statement as its first member, follow nested labels to the
        // statement they introduce. Within a label, retain the next label's
        // own indentation.
        let mut representative = child;
        if node.kind() != "labeled_statement" {
            while representative.kind() == "labeled_statement" {
                let Some(body) = representative.named_child(1) else {
                    break;
                };
                representative = body;
            }
        }
        return Some(leading_ws_of_line(body, representative.start_byte()));
    }
    None
}

fn leading_ws_of_line(body: &[u8], pos: usize) -> &[u8] {
    let line_start = body[..pos]
        .iter()
        .rposition(|&byte| byte == b'\n')
        .map_or(0, |newline| newline + 1);
    leading_ws(&body[line_start..])
}

fn find_next_nonblank_indent<'a>(
    lines: &'a [LineInfo<'a>],
    idx: usize,
    danger: &[Interval],
    content_end: usize,
) -> Option<&'a [u8]> {
    let mut j = idx + 1;
    while j < lines.len() {
        let cand = &lines[j];
        if is_blank(cand.content) {
            j += 1;
            continue;
        }
        let indent = leading_ws(cand.content);
        let content_start = cand.content_start + indent.len();
        if content_start > content_end {
            return None;
        }
        if cand.content.get(indent.len()) == Some(&b'#') || byte_in_any(danger, content_start) {
            j += 1;
            continue;
        }
        return Some(indent);
    }
    None
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
