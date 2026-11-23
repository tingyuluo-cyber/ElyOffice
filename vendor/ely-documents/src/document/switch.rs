use gpui::{Context, Focusable as _, Window};

use super::MarkdownDocument;
use crate::MarkdownMode;

impl MarkdownDocument {
    /// Shows the selected text in the new mode; otherwise keeps the top line.
    pub fn set_mode(&mut self, mode: MarkdownMode, window: &mut Window, cx: &mut Context<Self>) {
        if mode == self.mode {
            return;
        }
        self.backticks = None;
        let preview = (self.mode == MarkdownMode::Preview)
            .then(|| self.preview_ends(cx))
            .flatten();
        self.commit(cx);
        if let Some((anchor, head)) = preview {
            self.editor.update(cx, |editor, cx| {
                editor.select(std::iter::once(anchor..head), cx)
            });
        }
        log::info!("markdown document: {:?} to {mode:?}", self.mode);
        if !self.read_now(cx) {
            self.reparse(true, cx);
        }
        let picked = self.editor.read(cx).primary();
        let listed = |mode| matches!(mode, MarkdownMode::Visual | MarkdownMode::Preview);
        let selected = !picked.is_empty();
        let top = if selected || preview.is_some() {
            Some(self.editor.read(cx).buffer().line_of(picked.range().start))
        } else {
            (!(listed(self.mode) && listed(mode))).then(|| self.top_line(cx))
        };
        self.mode = mode;
        let scroll_past_end =
            self.look.scroll_past_end && matches!(mode, MarkdownMode::Source | MarkdownMode::Split);
        self.editor.update(cx, |editor, cx| {
            editor.set_scroll_past_end(scroll_past_end, cx)
        });
        self.preview = None;
        self.preview_selection = (mode == MarkdownMode::Preview).then_some((
            (picked.anchor, picked.head),
            self.edits + self.writes as u64,
        ));
        if let Some(top) = top {
            self.set_top_line(top, window, cx);
        }
        let focus = self.focus_handle(cx);
        window.focus(&focus, cx);
        if mode == MarkdownMode::Visual && selected {
            self.show_match(picked.range(), window, cx);
        }
        if matches!(mode, MarkdownMode::Source | MarkdownMode::Split)
            && (selected || preview.is_some())
        {
            let ask = self.asks;
            // A returning editor may need another frame to update its wrap width.
            cx.on_next_frame(window, move |_, window, cx| {
                cx.on_next_frame(window, move |document, _, cx| {
                    if document.mode == mode
                        && document.asks == ask
                        && document.editor.read(cx).primary() == picked
                    {
                        document.editor.read(cx).reveal_offset(picked.range().start);
                        cx.notify();
                    }
                });
            });
        }
        // Preview to visual keeps one handle: no focus event.
        self.focused(window, cx);
        cx.notify();
    }
}
