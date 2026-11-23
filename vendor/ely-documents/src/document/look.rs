//! A writing view: how text shows, not what it says.

use gpui::{Context, Focusable as _, Pixels, Rems, SharedString, Window};

use super::MarkdownDocument;
use crate::MarkdownMode;
use crate::editor::LineNumbers;

/// How the document shows its text (MARK-09).
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentLook {
    /// All but the current block or line fade.
    pub focus: bool,
    /// The current block or line stays mid-view.
    pub typewriter: bool,
    /// The view can scroll past the last line and leave space below the caret while typing.
    pub scroll_past_end: bool,
    /// The widest the text column grows.
    pub measure: Option<Rems>,
    /// The prose font; none keeps the theme's.
    pub font: Option<SharedString>,
    /// Text size against the theme's.
    pub scale: f32,
    /// Source mode numbers its lines.
    pub line_numbers: bool,
    /// Strong and emphasized text use distinct theme colors.
    pub color_marks: bool,
}

impl Default for DocumentLook {
    fn default() -> Self {
        Self {
            focus: false,
            typewriter: false,
            scroll_past_end: false,
            measure: None,
            font: None,
            scale: 1.0,
            line_numbers: false,
            color_marks: false,
        }
    }
}

impl MarkdownDocument {
    /// How the document shows its text.
    pub fn look(&self) -> &DocumentLook {
        &self.look
    }

    /// Sets how the document shows its text.
    pub fn set_look(&mut self, look: DocumentLook, cx: &mut Context<Self>) {
        log::info!("markdown document: look {look:?}");
        self.color_marks.set(look.color_marks);
        let source = matches!(self.mode, MarkdownMode::Source | MarkdownMode::Split);
        self.editor.update(cx, |editor, cx| {
            editor.set_scale(look.scale, cx);
            editor.set_focus_line(look.focus, cx);
            editor.set_typewriter(look.typewriter, cx);
            editor.set_scroll_past_end(source && look.scroll_past_end, cx);
            let numbers = match look.line_numbers {
                true => LineNumbers::Absolute,
                false => LineNumbers::Hidden,
            };
            editor.set_line_numbers(numbers, cx);
        });
        self.look = look;
        self.centered = None;
        cx.notify();
    }

    /// The reader's block: the open one, or the caret's.
    pub(super) fn current_block(&self, cx: &gpui::App) -> usize {
        match &self.open {
            Some(open) if open.append => self.parsed.blocks.len(),
            Some(open) => self.block_at(open.range.start),
            None => self.block_at(self.editor.read(cx).primary().head),
        }
    }

    /// Half a view of room below the text, and above it for typewriter.
    pub(super) fn make_room(&mut self) {
        let half = self.list.viewport_bounds().size.height / 2.0;
        let top = if self.look.typewriter && self.mode == MarkdownMode::Visual {
            half
        } else {
            Pixels::ZERO
        };
        let bottom = if self.look.scroll_past_end { half } else { top };
        if top != self.top_room {
            self.top_room = top;
            self.list.remeasure_items(0..1);
        }
        if bottom != self.bottom_room {
            self.bottom_room = bottom;
            let last = self.parsed.blocks.len();
            self.list.remeasure_items(last..last + 1);
        }
    }

    /// After an edit, keep the laid-out caret two rows clear of the lower edge.
    pub(super) fn follow_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !std::mem::take(&mut self.reveal_caret)
            || !self.look.scroll_past_end
            || self.look.typewriter
        {
            return;
        }
        let Some(open) = &self.open else { return };
        let field = open.input.clone();
        let seen = (self.typed, field.read(cx).cursor());
        cx.on_next_frame(window, move |document, window, cx| {
            let input = field.read(cx);
            if document.mode != MarkdownMode::Visual
                || document.look.typewriter
                || !document.look.scroll_past_end
                || document
                    .open
                    .as_ref()
                    .is_none_or(|open| open.input != field)
                || !input.focus_handle(cx).is_focused(window)
                || seen != (document.typed, input.cursor())
                || document.list.is_scrollbar_dragging()
            {
                return;
            }
            let Some(caret) = input.caret_bounds() else {
                return;
            };
            let view = document.list.viewport_bounds();
            let margin = (caret.size.height * 2.0).min(view.size.height / 4.0);
            let distance = if caret.top() < view.top() {
                caret.top() - view.top()
            } else {
                (caret.bottom() + margin - view.bottom()).max(Pixels::ZERO)
            };
            if distance != Pixels::ZERO {
                document.list.scroll_by(distance);
                cx.notify();
            }
        });
    }

    /// The current block mid-view once it or its text moves.
    pub(super) fn center_current(&mut self, cx: &gpui::App) {
        if !self.look.typewriter || self.mode != MarkdownMode::Visual {
            return;
        }
        let ix = self.current_block(cx);
        let Some(mut block) = self.list.bounds_for_item(ix) else {
            return;
        };
        // The ends' room is no block.
        if ix == 0 {
            block.origin.y += self.top_room;
            block.size.height -= self.top_room;
        }
        if ix == self.parsed.blocks.len() {
            block.size.height -= self.bottom_room;
        }
        let view = self.list.viewport_bounds();
        // New layout, resize or end room moves it.
        let heights = [block.size.height, view.size.height, self.top_room]
            .map(|height| f32::from(height).round() as u64);
        let moved = (ix, self.edits, self.typed, heights);
        if self.centered == Some(moved) {
            return;
        }
        self.list.scroll_by(block.center().y - view.center().y);
        self.centered = Some(moved);
    }
}
