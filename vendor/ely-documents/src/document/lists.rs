//! Shared list keys for source and visual editing.

use gpui::{Context, Div, InteractiveElement as _, Window};

use super::{
    MarkdownDocument,
    cells::{Aim, Move, beside},
    edit::Landing,
};

mod changes;
mod enter;
mod index;
mod shift;
#[cfg(test)]
mod tests;

impl MarkdownDocument {
    pub(super) fn source_actions(root: Div, cx: &mut Context<Self>) -> Div {
        root.key_context("MarkdownDocument")
            .capture_action(cx.listener(|document, _: &crate::editor::Newline, _, cx| {
                if document.source_newline(false, cx) {
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|document, _: &super::visual::Indent, _, cx| {
                document.source_shift(true, cx);
                cx.stop_propagation();
            }))
            .capture_action(cx.listener(|document, _: &super::visual::Outdent, _, cx| {
                document.source_shift(false, cx);
                cx.stop_propagation();
            }))
            .capture_action(
                cx.listener(|document, _: &super::visual::SoftBreak, window, cx| {
                    if !document.source_newline(true, cx) {
                        window.dispatch_action(Box::new(crate::editor::Newline), cx);
                    }
                    cx.stop_propagation();
                }),
            )
    }

    pub(super) fn source_newline(&mut self, soft: bool, cx: &mut Context<Self>) -> bool {
        let editor = self.editor.read(cx);
        if editor.selections().len() != 1 {
            return false;
        }
        let selection = editor.primary().range();
        let text = editor.text();
        if text.len() > super::LARGEST_READ {
            return false;
        }
        let Some((out, range)) = enter::enter(text, selection, soft) else {
            return false;
        };
        let edits = changes::difference(text, &out);
        log::info!("markdown document: list newline, soft={soft}");
        self.editor.update(cx, |editor, cx| {
            editor.edit(edits, cx);
            editor.select([range], cx);
        });
        true
    }

    pub(super) fn source_shift(&mut self, deeper: bool, cx: &mut Context<Self>) {
        let editor = self.editor.read(cx);
        let text = editor.text();
        let selections = editor
            .selections()
            .iter()
            .map(|selection| selection.anchor..selection.head)
            .collect();
        let (edits, ranges) = if text.len() > super::LARGEST_READ {
            log::info!("markdown document: large source uses plain indentation");
            shift::plain(text, selections, deeper)
        } else {
            let (out, ranges) = shift::shift(text, selections, deeper);
            (changes::difference(text, &out), ranges)
        };
        log::info!("markdown document: source indent, deeper={deeper}");
        self.editor.update(cx, |editor, cx| {
            editor.edit(edits, cx);
            editor.select(ranges, cx);
        });
    }

    pub(super) fn block_newline(&mut self, soft: bool, cx: &mut Context<Self>) -> bool {
        let Some(open) = self.open.as_ref().filter(|open| open.cell.is_none()) else {
            return false;
        };
        let input = open.input.clone();
        let (text, selection) = {
            let input = input.read(cx);
            (input.text().to_string(), input.selection())
        };
        let Some((out, range)) = enter::enter(&text, selection, soft) else {
            return false;
        };
        log::info!("markdown document: block list newline, soft={soft}");
        input.update(cx, |input, cx| {
            input.set_text(out, cx);
            input.select(range, cx);
        });
        true
    }

    pub(super) fn block_shift(
        &mut self,
        deeper: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(open) = &self.open else {
            return;
        };
        if let Some(at) = open.cell {
            let here = self.block_at(open.range.start);
            let block = self.parsed.blocks[here].range.clone();
            let by = if deeper { Move::Next } else { Move::Previous };
            let text = self.editor.read(cx).text();
            match beside(text, block, at, by) {
                Some(to) => self.open(Some(here), Landing::Cell(Aim::At(to)), true, window, cx),
                None => log::info!("markdown document: no cell {by:?} of {at:?}"),
            }
            return;
        }
        let input = open.input.clone();
        let (text, selection) = {
            let input = input.read(cx);
            (input.text().to_string(), input.selection())
        };
        let (out, ranges) = shift::shift(&text, vec![selection], deeper);
        log::info!("markdown document: block indent, deeper={deeper}");
        input.update(cx, |input, cx| {
            input.set_text(out, cx);
            input.select(ranges[0].clone(), cx);
        });
    }
}
