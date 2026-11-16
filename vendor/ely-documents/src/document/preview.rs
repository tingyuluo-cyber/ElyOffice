use gpui::{App, AppContext as _, Context, Div, Entity, Subscription};

use super::MarkdownDocument;
use crate::{
    MarkdownMode,
    render::select::{self, Gather, Selection},
};

pub(super) struct Preview {
    pub state: Entity<Selection>,
    offsets: Vec<usize>,
    _subscription: Subscription,
}

impl MarkdownDocument {
    pub(super) fn prepare_preview(&mut self, cx: &mut Context<Self>) {
        if self.mode != MarkdownMode::Preview {
            return;
        }
        if !self.read_now(cx) {
            self.preview = None;
            return;
        }
        if self.preview.is_none() {
            let mut texts = Vec::new();
            let mut offsets = Vec::new();
            for block in &self.parsed.blocks {
                offsets.push(texts.len());
                match &block.nodes {
                    Some(nodes) => select::content(nodes, &mut texts),
                    None => select::raw(&self.parsed.text[block.range.clone()], &mut texts),
                }
            }
            let state = cx.new(|cx| {
                let mut state = Selection::new(cx);
                state.reset(texts, self.focus.clone());
                state
            });
            let subscription = cx.observe(&state, |_, _, cx| cx.notify());
            self.preview = Some(Preview {
                state,
                offsets,
                _subscription: subscription,
            });
        }
        self.preview
            .as_ref()
            .expect("prepared")
            .state
            .update(cx, |state, _| state.begin_frame());
    }

    pub(super) fn preview_gather(&self, ix: usize, cx: &App) -> Option<Gather> {
        let preview = self
            .preview
            .as_ref()
            .filter(|_| self.mode == MarkdownMode::Preview)?;
        Some(Gather::block(
            preview.state.clone(),
            *preview.offsets.get(ix)?,
            cx,
        ))
    }

    pub(super) fn preview_body(&self, body: Div, cx: &App) -> Div {
        match &self.preview {
            Some(preview) => select::preview(body, preview.state.clone(), cx),
            None => body,
        }
    }
}
