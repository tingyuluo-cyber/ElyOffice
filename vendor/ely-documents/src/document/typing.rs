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
            || event.keystroke.key != "*"
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
                    .any(|selection| selection.is_empty())
            {
                return;
            }
            let selections: Vec<_> = editor.selections().to_vec();
            let edits: Vec<_> = selections
                .iter()
                .map(|selection| {
                    let range = selection.range();
                    (range.clone(), format!("*{}*", &editor.text()[range]))
                })
                .collect();
            let ranges: Vec<_> = selections
                .iter()
                .map(|selection| {
                    let delta = selections
                        .iter()
                        .filter(|other| other.range().end <= selection.range().start)
                        .count()
                        * 2;
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
