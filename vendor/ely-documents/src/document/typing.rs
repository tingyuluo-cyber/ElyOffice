use gpui::{Context, Focusable as _, KeyDownEvent, Window};

use super::MarkdownDocument;
use crate::MarkdownMode;

impl MarkdownDocument {
    pub(super) fn surround_selection(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.mode == MarkdownMode::Preview
            || event.prefer_character_input
            || event
                .keystroke
                .key_char
                .as_deref()
                .unwrap_or(&event.keystroke.key)
                != "*"
            || event.keystroke.modifiers.control
            || event.keystroke.modifiers.platform
            || event.keystroke.modifiers.alt
        {
            return;
        }
        if let Some(open) = &self.open {
            let input = open.input.read(cx);
            if !input.focus_handle(cx).is_focused(window) {
                return;
            }
            let range = input.selection();
            if range.is_empty() {
                return;
            }
            let written = format!("*{}*", &input.text()[range.clone()]);
            open.input.update(cx, |input, cx| {
                input.insert(&written, cx);
                input.select(range.start + 1..range.end + 1, cx);
            });
        } else {
            let editor = self.editor.read(cx);
            if !editor.focus_handle(cx).is_focused(window)
                || editor
                    .selections()
                    .iter()
                    .all(|selection| selection.is_empty())
            {
                return;
            }
            let selections: Vec<_> = editor.selections().to_vec();
            let edits: Vec<_> = selections
                .iter()
                .map(|selection| {
                    let range = selection.range();
                    let written = if range.is_empty() {
                        "*".to_string()
                    } else {
                        format!("*{}*", &editor.text()[range.clone()])
                    };
                    (range, written)
                })
                .collect();
            let ranges: Vec<_> = selections
                .iter()
                .enumerate()
                .map(|(index, selection)| {
                    let delta = selections
                        .iter()
                        .enumerate()
                        .filter(|(other_index, other)| {
                            *other_index != index && other.range().end <= selection.range().start
                        })
                        .map(|(_, other)| if other.is_empty() { 1 } else { 2 })
                        .sum::<usize>();
                    selection.anchor + delta + 1..selection.head + delta + 1
                })
                .collect();
            self.editor.update(cx, |editor, cx| {
                editor.edit(edits, cx);
                editor.select(ranges, cx);
            });
        }
        window.prevent_default();
        cx.stop_propagation();
    }
}
