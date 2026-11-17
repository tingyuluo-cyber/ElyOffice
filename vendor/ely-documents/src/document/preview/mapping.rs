use gpui::App;

use super::Preview;
use crate::{document::parse::Parsed, render::select::Spot};

impl Preview {
    /// The fixed and moving ends in Markdown bytes, including a plain click.
    pub(super) fn source_ends(&self, parsed: &Parsed, cx: &App) -> Option<(usize, usize)> {
        let (anchor, head) = self.state.read(cx).ends()?;
        if let Some((spots, ends)) = self.original
            && spots == (anchor, head)
        {
            return Some(ends);
        }
        Some((
            self.source_at(parsed, anchor, anchor > head)?,
            self.source_at(parsed, head, head > anchor)?,
        ))
    }

    fn source_at(&self, parsed: &Parsed, spot: Spot, upstream: bool) -> Option<usize> {
        let block = self
            .offsets
            .partition_point(|offset| *offset <= spot.text)
            .checked_sub(1)?;
        let start = parsed.blocks.get(block)?.range.start;
        Some(
            start
                + self
                    .sources
                    .get(spot.text)?
                    .at(&parsed.text[start..], spot.at, upstream)?,
        )
    }

    /// Nearest drawn character boundary, skipping Markdown-only markers.
    pub(super) fn spot_at(&self, parsed: &Parsed, at: usize) -> Option<Spot> {
        let mut nearest = None;
        for (text, source) in self.sources.iter().enumerate() {
            let first = self.source_at(parsed, Spot { text, at: 0 }, false);
            let last = self.source_at(
                parsed,
                Spot {
                    text,
                    at: source.text().len(),
                },
                true,
            );
            let (Some(first), Some(last)) = (first, last) else {
                continue;
            };
            let gap = first.saturating_sub(at).max(at.saturating_sub(last));
            if nearest.is_none_or(|(best, _)| gap < best) {
                nearest = Some((gap, text));
            }
        }
        let (_, text) = nearest?;
        let drawn = self.sources[text].text();
        let stops: Vec<_> = drawn
            .char_indices()
            .map(|(at, _)| at)
            .chain([drawn.len()])
            .collect();
        let boundary = stops.partition_point(|offset| {
            self.source_at(parsed, Spot { text, at: *offset }, true)
                .is_some_and(|source| source <= at)
        });
        Some(Spot {
            text,
            at: stops[boundary.saturating_sub(1)],
        })
    }
}
