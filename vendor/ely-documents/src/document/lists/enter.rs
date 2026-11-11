use std::ops::Range;

use super::{
    changes::apply,
    index::{Index, end, padding},
    shift::{renumber, shift},
};

pub(super) fn enter(
    text: &str,
    selection: Range<usize>,
    soft: bool,
) -> Option<(String, Range<usize>)> {
    let index = Index::new(text);
    let at = selection.start;
    let id = index.at(at)?;
    let item = &index.items[id];
    if at < item.body || selection.end > item.position.end {
        return None;
    }
    let line_end = end(text, at);
    let own_end = end(text, item.body);
    let raw = &text[item.body..own_end];
    let task = ["[ ]", "[x]", "[X]"]
        .iter()
        .any(|mark| raw.starts_with(mark));
    let empty = text[item.body + if task { 3 } else { 0 }..item.position.end]
        .trim()
        .is_empty();
    if !soft && empty && selection.is_empty() && at == line_end {
        if let Some(parent) = index.parent(text, id) {
            if index.items[parent].row == item.row {
                let start = item.marker.start;
                let mut edits = vec![(start..line_end, String::new())];
                let out = apply(text, &mut edits, &mut []);
                return Some((out, start..start));
            }
            let (text, ranges) = shift(text, vec![selection], false);
            return Some((text, ranges[0].clone()));
        }
        let start = padding(text, item.row, item.marker.start);
        let mut edits = vec![(start..line_end, String::new())];
        let mut ranges = std::iter::once(start..start).collect::<Vec<_>>();
        let out = apply(text, &mut edits, &mut ranges);
        let out = renumber(&index, &out, &edits, &[id], &[], &mut ranges);
        return Some((out, ranges.remove(0)));
    }
    let newline = if text.as_bytes().get(line_end) == Some(&b'\r') || text[..at].contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let prefix: String = text[item.row..item.marker.start]
        .chars()
        .map(|ch| {
            if ch == '>' || ch.is_whitespace() {
                ch
            } else {
                ' '
            }
        })
        .collect();
    let written = if soft {
        let indent = " ".repeat(item.body - item.marker.start);
        format!("  {newline}{prefix}{indent}")
    } else {
        let marker = &text[item.marker.clone()];
        let marker = if index.groups[item.group].ordered {
            let number: u64 = marker[..marker.len() - 1].parse().ok()?;
            if number == 999_999_999 {
                return None;
            }
            format!("{}{}", number + 1, &marker[marker.len() - 1..])
        } else {
            marker.to_owned()
        };
        let gap = &text[item.marker.end..item.body];
        let gap = if gap.is_empty() { " " } else { gap };
        let task = if task { "[ ] " } else { "" };
        format!("{newline}{prefix}{marker}{gap}{task}")
    };
    let caret = at + written.len();
    let mut edits = vec![(selection, written)];
    let mut ranges = std::iter::once(caret..caret).collect::<Vec<_>>();
    let out = apply(text, &mut edits, &mut []);
    let out = if soft {
        out
    } else {
        renumber(&index, &out, &edits, &[id], &[], &mut ranges)
    };
    Some((out, ranges.remove(0)))
}
