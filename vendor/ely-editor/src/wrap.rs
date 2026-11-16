use std::ops::Range;

use gpui::{Context, Pixels, Point, SharedString, TextRun, Window, font};
use unicode_segmentation::UnicodeSegmentation as _;

use super::{
    buffer::Buffer,
    cursor::{Motion, Selection},
    layout::{Row, Shown},
    state::CodeEditor,
};

#[derive(Default)]
pub(crate) struct Wrap {
    key: Option<(u64, Pixels, Pixels, SharedString)>,
    lines: Vec<Vec<Range<usize>>>,
}

impl Wrap {
    pub fn update(
        &mut self,
        buffer: &Buffer,
        width: Pixels,
        size: Pixels,
        family: SharedString,
        window: &Window,
    ) {
        let key = (buffer.stamp(), width, size, family.clone());
        if self.key.as_ref() == Some(&key) {
            return;
        }
        self.lines = (0..buffer.lines())
            .map(|line| {
                let text = buffer.line(line);
                let run = TextRun {
                    len: text.len(),
                    font: font(family.clone()),
                    color: Default::default(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let layout = window.text_system().layout_line(text, size, &[run], None);
                let stops: Vec<_> = text
                    .grapheme_indices(true)
                    .map(|(at, _)| at)
                    .chain([text.len()])
                    .collect();
                let mut glyphs: Vec<_> = layout
                    .runs
                    .iter()
                    .flat_map(|run| {
                        run.glyphs
                            .iter()
                            .map(|glyph| (glyph.index, glyph.position.x))
                    })
                    .collect();
                if !glyphs.is_sorted_by_key(|(index, _)| *index) {
                    glyphs.sort_by_key(|(index, _)| *index);
                }
                let mut glyph = 0;
                let positions: Vec<_> = stops
                    .iter()
                    .map(|at| {
                        while glyph < glyphs.len() && glyphs[glyph].0 < *at {
                            glyph += 1;
                        }
                        glyphs.get(glyph).map_or(layout.width, |(_, x)| *x)
                    })
                    .collect();
                let mut ranges = Vec::new();
                let mut first = 0;
                while first + 1 < stops.len() {
                    let left = positions[first];
                    let rest = &positions[first + 1..];
                    let count = rest.partition_point(|x| *x - left <= width).max(1);
                    let mut last = first + count;
                    if last + 1 < stops.len()
                        && let Some(space) = (first + 1..=last).rev().find(|at| {
                            text[stops[*at - 1]..stops[*at]]
                                .chars()
                                .all(char::is_whitespace)
                        })
                    {
                        last = space;
                    }
                    ranges.push(stops[first]..stops[last]);
                    first = last;
                }
                if ranges.is_empty() {
                    ranges.push(0..0);
                }
                ranges
            })
            .collect();
        self.key = Some(key);
    }

    pub fn rows(&self, rows: Vec<Row>, buffer: &Buffer) -> (Vec<Row>, Vec<Range<usize>>) {
        let (mut out, mut segments) = (Vec::new(), Vec::new());
        for row in rows {
            match row {
                Row::Line(line) => {
                    let start = buffer.line_range(line).start;
                    for range in &self.lines[line] {
                        out.push(row);
                        segments.push(start + range.start..start + range.end);
                    }
                }
                _ => {
                    out.push(row);
                    segments.push(0..0);
                }
            }
        }
        (out, segments)
    }
}

impl CodeEditor {
    pub(crate) fn wrapped_motion(
        &self,
        selection: Selection,
        motion: Motion,
        extend: bool,
    ) -> Option<(Selection, bool)> {
        if !self.options.soft_wrap || self.frame.stamp != self.buffer.stamp() {
            return None;
        }
        let by = match motion {
            Motion::Up => -1,
            Motion::Down => 1,
            Motion::Page(by) => by,
            _ => return None,
        };
        let row = self.caret_row(selection.head)?;
        let line = self.buffer.line_of(selection.head);
        let range = self.segment(row, line);
        let goal = selection.goal.unwrap_or_else(|| {
            self.buffer.text()[range.start..selection.head]
                .chars()
                .count()
        });
        let lines: Vec<_> = self
            .frame
            .rows
            .iter()
            .enumerate()
            .filter_map(|(ix, row)| matches!(row, Row::Line(_)).then_some(ix))
            .collect();
        let current = lines.iter().position(|ix| *ix == row)?;
        let next = (current as isize + by).clamp(0, lines.len() as isize - 1) as usize;
        let range = self.frame.segments[lines[next]].clone();
        let text = &self.buffer.text()[range.clone()];
        let head = range.start
            + text
                .char_indices()
                .nth(goal)
                .map_or(text.len(), |(at, _)| at);
        let upstream =
            head == range.end && head < self.buffer.line_range(self.buffer.line_of(head)).end;
        Some((
            Selection {
                head,
                anchor: if extend { selection.anchor } else { head },
                goal: Some(goal),
            },
            upstream,
        ))
    }

    pub(crate) fn segment_shown(
        &self,
        row: usize,
        line: usize,
        cx: &Context<Self>,
    ) -> (Range<usize>, Shown) {
        let range = self.segment(row, line);
        let full = self.row_range(line);
        let local = range.start - full.start..range.end - full.start;
        let notes = self
            .notes(line, cx)
            .into_iter()
            .filter_map(|(at, text, style)| {
                (local.start <= at && at <= local.end).then(|| (at - local.start, text, style))
            })
            .collect();
        let styles = self
            .styles(line, cx)
            .into_iter()
            .filter_map(|(span, style)| {
                let start = span.start.max(local.start);
                let end = span.end.min(local.end);
                (start < end).then_some((
                    start.saturating_sub(local.start)..end.saturating_sub(local.start),
                    style,
                ))
            })
            .collect();
        let shown = Shown::new(&self.buffer.text()[range.clone()], notes, styles);
        (range, shown)
    }

    pub(crate) fn segment(&self, row: usize, line: usize) -> Range<usize> {
        if self.options.soft_wrap
            && self.frame.stamp == self.buffer.stamp()
            && let Some(range) = self.frame.segments.get(row)
        {
            return range.clone();
        }
        self.row_range(line)
    }

    pub(crate) fn caret_row(&self, at: usize) -> Option<usize> {
        let line = self.buffer.line_of(at);
        if self.upstream.contains(&(self.buffer.stamp(), at))
            && let Some(row) = self.frame.rows.iter().enumerate().position(|(row, shown)| {
                *shown == Row::Line(line) && self.segment(row, line).end == at
            })
        {
            return Some(row);
        }
        self.frame
            .rows
            .iter()
            .enumerate()
            .rfind(|(row, shown)| {
                **shown == Row::Line(line) && self.segment(*row, line).start <= at
            })
            .map(|(row, _)| row)
    }

    pub(crate) fn upstream_at(&self, at: usize, position: Point<Pixels>) -> Option<(u64, usize)> {
        if !self.options.soft_wrap || self.frame.stamp != self.buffer.stamp() {
            return None;
        }
        let scrolled = -self.scroll.0.borrow().base_handle.offset().y;
        let row = ((position.y - self.metrics.top + scrolled) / self.metrics.line)
            .floor()
            .max(0.) as usize;
        let Row::Line(line) = *self.frame.rows.get(row)? else {
            return None;
        };
        let range = self.segment(row, line);
        (at == range.end && at < self.row_range(line).end).then_some((self.buffer.stamp(), at))
    }

    pub(crate) fn set_upstream_at(&mut self, at: usize, position: Point<Pixels>) {
        self.upstream.remove(&(self.buffer.stamp(), at));
        if let Some(upstream) = self.upstream_at(at, position) {
            self.upstream.insert(upstream);
        }
    }
}
