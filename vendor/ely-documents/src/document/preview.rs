use gpui::{App, AppContext as _, Context, Div, Entity, Subscription, Window};

use super::MarkdownDocument;
use crate::{
    MarkdownMode,
    render::select::{self, Gather, Selection},
};

mod mapping;

pub(super) struct Preview {
    pub state: Entity<Selection>,
    offsets: Vec<usize>,
    sources: Vec<select::Source>,
    original: Option<((select::Spot, select::Spot), (usize, usize))>,
    reveal: Option<(select::Spot, u64)>,
    _subscription: Subscription,
}

impl MarkdownDocument {
    pub(super) fn prepare_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
            let mut sources = Vec::new();
            for block in &self.parsed.blocks {
                offsets.push(texts.len());
                match &block.nodes {
                    Some(nodes) => select::content(nodes, &mut texts, &mut sources),
                    None => select::raw(
                        &self.parsed.text[block.range.clone()],
                        &mut texts,
                        &mut sources,
                    ),
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
                sources,
                original: None,
                reveal: None,
                _subscription: subscription,
            });
        }
        if let Some(((anchor, head), expected)) = self.preview_selection.take()
            && expected == self.edits + self.writes as u64
        {
            let preview = self.preview.as_mut().expect("prepared");
            let ends = (anchor, head);
            if let (Some(anchor), Some(head)) = (
                preview.spot_at(&self.parsed, anchor),
                preview.spot_at(&self.parsed, head),
            ) {
                preview
                    .state
                    .update(cx, |state, _| state.select(anchor, head));
                preview.original = Some(((anchor, head), ends));
                preview.reveal = (anchor != head).then_some((anchor.min(head), self.asks));
            }
        }
        // Line placement finishes before measuring the selected wrapped row.
        if self.top.is_none()
            && let Some(preview) = self.preview.as_mut()
            && let Some((spot, ask)) = preview.reveal.take()
        {
            let state = preview.state.clone();
            cx.on_next_frame(window, move |document, _, cx| {
                let current = document
                    .preview
                    .as_ref()
                    .is_some_and(|preview| preview.state == state);
                if document.asks != ask || !current {
                    return;
                }
                let held = state
                    .read(cx)
                    .ends()
                    .is_some_and(|(anchor, head)| anchor != head && anchor.min(head) == spot);
                if !held {
                    return;
                }
                let Some(position) = state.read(cx).position(spot) else {
                    return;
                };
                let view = document.list.viewport_bounds();
                if !view.contains(&position) {
                    document
                        .list
                        .scroll_by(position.y - view.top() - gpui::px(8.));
                    cx.notify();
                }
            });
        }
        self.preview
            .as_ref()
            .expect("prepared")
            .state
            .update(cx, |state, _| state.begin_frame());
    }

    pub(super) fn preview_ends(&self, cx: &App) -> Option<(usize, usize)> {
        self.preview.as_ref()?.source_ends(&self.parsed, cx)
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
