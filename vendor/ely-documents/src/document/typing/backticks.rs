use std::ops::Range;

use gpui::{Context, EntityId, Focusable as _, Window};

use super::MarkdownDocument;
use crate::{document::block::eol, markdown::code_fence, modes::joint};

/// Only the next key in the same field and selection can extend this pair.
pub(in crate::document) struct Backticks {
    target: EntityId,
    revision: u64,
    selected: Vec<Range<usize>>,
    count: usize,
}

impl Backticks {
    fn next(&self, target: EntityId, revision: u64, selected: &[Range<usize>]) -> usize {
        if self.target == target && self.revision == revision && self.selected == selected {
            self.count + 1
        } else {
            1
        }
    }
}

impl MarkdownDocument {
    pub(super) fn type_backtick(
        &mut self,
        previous: Option<Backticks>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some(open) = &self.open {
            let input = open.input.read(cx);
            let selected = input.selection();
            if selected.is_empty() || !input.focus_handle(cx).is_focused(window) {
                return false;
            }
            let field = open.input.clone();
            let target = field.entity_id();
            let revision = self.typed;
            let count = previous.as_ref().map_or(1, |run| {
                run.next(target, revision, std::slice::from_ref(&selected))
            });
            if open.cell.is_some() {
                let written = format!("`{}`", &input.text()[selected]);
                field.update(cx, |input, cx| input.insert(&written, cx));
                return true;
            }

            let (range, written, picked) = if count == 3 {
                let source = self.editor.read(cx).text();
                let start = open.range.start + if open.append { joint(source).len() } else { 0 };
                let text = self.text(cx);
                let (range, written, picked) = fenced(
                    &text,
                    start + selected.start..start + selected.end,
                    eol(&text),
                );
                (range.start - start..range.end - start, written, picked)
            } else {
                paired(input.text(), selected.clone())
            };
            let backwards = input.cursor() == selected.start;
            let picked = range.start + picked.start..range.start + picked.end;
            let mut text = input.text().to_owned();
            text.replace_range(range, &written);
            field.update(cx, |input, cx| {
                // Inserting through TextInput would normalize the original CRLFs.
                input.set_text(text, cx);
                if backwards {
                    input.select_from(picked.end, picked.start, cx);
                } else {
                    input.select(picked.clone(), cx);
                }
            });
            if count == 3 {
                self.commit(cx);
                window.focus(&self.focus, cx);
            } else {
                self.backticks = Some(Backticks {
                    target,
                    revision: revision + 1,
                    selected: vec![picked],
                    count,
                });
            }
            return true;
        }

        let editor = self.editor.read(cx);
        if editor.is_read_only()
            || !editor.focus_handle(cx).is_focused(window)
            || editor
                .selections()
                .iter()
                .all(|selection| selection.is_empty())
        {
            return false;
        }
        let target = self.editor.entity_id();
        let mut selected: Vec<_> = editor
            .selections()
            .iter()
            .map(|cursor| cursor.range())
            .collect();
        let count = previous
            .as_ref()
            .map_or(1, |run| run.next(target, self.edits, &selected));
        let mut order: Vec<_> = selected.iter().cloned().enumerate().collect();
        order.sort_by_key(|(_, range)| (range.start, range.end));
        let text = editor.text();
        let newline = if count == 3 { eol(text) } else { "" };
        let mut offset = 0;
        let mut edits = Vec::with_capacity(order.len());
        for (index, range) in order {
            let (range, written, picked) = if count == 3 && !range.is_empty() {
                fenced(text, range, newline)
            } else {
                paired(text, range)
            };
            let start = range.start + offset;
            let cursor = editor.selections()[index];
            selected[index] = if cursor.head < cursor.anchor {
                start + picked.end..start + picked.start
            } else {
                start + picked.start..start + picked.end
            };
            offset += written.len() - range.len();
            edits.push((range, written));
        }
        let selected = self.editor.update(cx, |editor, cx| {
            editor.edit(edits, cx);
            editor.select(selected, cx);
            editor
                .selections()
                .iter()
                .map(|cursor| cursor.range())
                .collect()
        });
        if count < 3 {
            self.backticks = Some(Backticks {
                target,
                revision: self.edits + 1,
                selected,
                count,
            });
        }
        true
    }
}

/// The replaced range, its replacement, and the selection inside the replacement.
fn paired(text: &str, range: Range<usize>) -> (Range<usize>, String, Range<usize>) {
    if range.is_empty() {
        return (range, "`".to_string(), 1..1);
    }
    let body = &text[range.clone()];
    (range, format!("`{body}`"), 1..1 + body.len())
}

fn fenced(
    text: &str,
    selected: Range<usize>,
    newline: &str,
) -> (Range<usize>, String, Range<usize>) {
    // The two pairs this run added are replaced by line fences.
    let range = selected.start - 2..selected.end + 2;
    let body = &text[selected];
    let fence = code_fence(body);
    let before = &text[..range.start];
    let after = &text[range.end..];
    let lead = if before.is_empty() || before.ends_with(['\n', '\r']) {
        ""
    } else {
        newline
    };
    let body_end = body
        .strip_suffix("\r\n")
        .or_else(|| body.strip_suffix(['\n', '\r']));
    let close = if body_end.is_some() { "" } else { newline };
    let tail = if (!after.is_empty() && !after.starts_with(['\n', '\r']))
        || (after.is_empty() && body_end.is_some())
    {
        newline
    } else {
        ""
    };
    let caret = lead.len() + fence.len() + newline.len() + body_end.unwrap_or(body).len();
    (
        range,
        format!("{lead}{fence}{newline}{body}{close}{fence}{tail}"),
        caret..caret,
    )
}
