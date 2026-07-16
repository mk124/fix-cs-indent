// Tree-sitter walk: collect "danger zones" — byte ranges where blank lines
// must not be touched (string literals, comments, ERROR subtrees).

use tree_sitter::{Node, Tree};

#[derive(Clone, Debug)]
pub struct Interval {
    pub start: usize,
    pub end: usize,
}

pub fn collect(tree: &Tree) -> Vec<Interval> {
    let mut out: Vec<Interval> = Vec::new();
    let mut cursor = tree.walk();
    walk(&mut out, &mut cursor);
    out.sort_by_key(|iv| iv.start);
    let mut merged: Vec<Interval> = Vec::with_capacity(out.len());
    for iv in out {
        if let Some(last) = merged.last_mut()
            && iv.start <= last.end
        {
            last.end = last.end.max(iv.end);
            continue;
        }
        merged.push(iv);
    }
    merged
}

// Iterative DFS using tree-sitter's cursor. Recursive walk overflows the
// stack on pathologically nested input (e.g. machine-generated expressions).
fn walk(out: &mut Vec<Interval>, cursor: &mut tree_sitter::TreeCursor) {
    loop {
        let node = cursor.node();
        let descended = if is_danger(&node) {
            out.push(Interval {
                start: node.start_byte(),
                end: node.end_byte(),
            });
            false
        } else {
            cursor.goto_first_child()
        };
        if descended {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return;
            }
        }
    }
}

fn is_danger(node: &Node) -> bool {
    if node.is_error() {
        return true;
    }
    matches!(
        node.kind(),
        "verbatim_string_literal"
            | "raw_string_literal"
            | "string_literal"
            | "interpolated_string_expression"
            | "comment"
    )
}

pub fn byte_in_any(intervals: &[Interval], byte: usize) -> bool {
    let index = intervals.partition_point(|iv| iv.end <= byte);
    intervals.get(index).is_some_and(|iv| iv.start <= byte)
}

pub fn range_overlaps_any(intervals: &[Interval], start: usize, end: usize) -> bool {
    if start >= end {
        return byte_in_any(intervals, start);
    }
    let index = intervals.partition_point(|iv| iv.end <= start);
    intervals.get(index).is_some_and(|iv| iv.start < end)
}
