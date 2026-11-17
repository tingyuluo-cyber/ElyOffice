//! Read-only text the pointer selects across blocks.

use std::{cell::RefCell, ops::Range, rc::Rc};

use gpui::{
    App, ClipboardItem, CursorStyle, Div, Entity, FocusHandle, HighlightStyle, InteractiveElement,
    MouseButton, Pixels, Point, Styled, TextLayout, Window, point,
};

use crate::theme::ActiveTheme;

mod content;
pub(crate) use content::{Source, content, raw};

/// What a right press on selectable text found.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RightPress {
    /// The selected text, if any.
    pub selected: Option<String>,
    /// The address of the link pressed.
    pub link: Option<String>,
    /// The address of the picture pressed.
    pub image: Option<String>,
    /// Where the press fell, in window pixels.
    pub position: Point<Pixels>,
}

/// Takes a right press on selectable text.
pub(crate) type OnMenu = Rc<dyn Fn(RightPress, &mut Window, &mut App)>;

/// A byte of one drawn text, texts counted in drawing order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Spot {
    pub(crate) text: usize,
    pub(crate) at: usize,
}

/// One drawn text, as the last frame laid it out.
#[derive(Clone)]
pub(crate) struct Drawn {
    /// A list item's marker, copied before its first text.
    lead: String,
    text: String,
    layout: Option<TextLayout>,
    links: Vec<(Range<usize>, String)>,
    mapped: Option<Rc<RefCell<Option<Mapped>>>>,
}

#[derive(Default)]
pub(crate) struct Mapped {
    pub layout: TextLayout,
    pub swaps: Vec<(Range<usize>, Range<usize>)>,
}

impl Drawn {
    fn layout(&self) -> Option<TextLayout> {
        match &self.mapped {
            Some(hit) => hit.borrow().as_ref().map(|hit| hit.layout.clone()),
            None => self.layout.clone(),
        }
    }
}

/// A body's selection, kept across frames.
pub(crate) struct Selection {
    focus: FocusHandle,
    anchor: Spot,
    head: Spot,
    dragging: bool,
    positioned: bool,
    texts: Vec<Drawn>,
    visible: Vec<Range<usize>>,
}

impl Selection {
    pub(crate) fn new(cx: &mut App) -> Self {
        Self {
            focus: cx.focus_handle(),
            anchor: Spot::default(),
            head: Spot::default(),
            dragging: false,
            positioned: false,
            texts: Vec::new(),
            visible: Vec::new(),
        }
    }

    fn span(&self) -> (Spot, Spot) {
        (self.anchor.min(self.head), self.anchor.max(self.head))
    }

    /// The selected text, blocks joined by line ends.
    pub(crate) fn selected(&self) -> Option<String> {
        copied(&self.texts, self.span())
    }

    pub(crate) fn ends(&self) -> Option<(Spot, Spot)> {
        self.positioned.then_some((self.anchor, self.head))
    }

    pub(crate) fn select(&mut self, anchor: Spot, head: Spot) {
        self.anchor = anchor;
        self.head = head;
        self.positioned = anchor != head;
    }

    /// A selected byte's position in the current drawn layout.
    pub(crate) fn position(&self, spot: Spot) -> Option<Point<Pixels>> {
        let drawn = self.texts.get(spot.text)?;
        let at = match &drawn.mapped {
            Some(hit) => super::math_text::forward(&hit.borrow().as_ref()?.swaps, spot.at),
            None => spot.at,
        };
        drawn.layout()?.position_for_index(at)
    }

    pub(crate) fn reset(&mut self, texts: Vec<Drawn>, focus: FocusHandle) {
        self.texts = texts;
        self.focus = focus;
        self.anchor = Spot::default();
        self.head = Spot::default();
        self.dragging = false;
        self.positioned = false;
        self.visible.clear();
    }

    pub(crate) fn begin_frame(&mut self) {
        for range in self.visible.drain(..) {
            for text in &mut self.texts[range] {
                text.layout = None;
                text.mapped = None;
            }
        }
    }

    /// The spot under `position`: a text beside it, or one past it.
    fn spot(&self, position: Point<Pixels>) -> Option<Spot> {
        let mut nearest = None;
        for (k, drawn) in self.texts.iter().enumerate() {
            let Some(layout) = drawn.layout() else {
                continue;
            };
            let bounds = layout.bounds();
            if bounds.size.height <= Pixels::ZERO {
                continue;
            }
            // A cell can hold several paragraphs; traversal is not sorted by y.
            let vertical = (bounds.top() - position.y)
                .max(position.y - bounds.bottom())
                .max(Pixels::ZERO);
            let horizontal = (bounds.left() - position.x)
                .max(position.x - bounds.right())
                .max(Pixels::ZERO);
            let distance = (vertical, horizontal);
            if nearest.is_none_or(|(best, _)| distance < best) {
                nearest = Some((distance, k));
            }
        }
        let (_, k) = nearest?;
        let drawn = &self.texts[k];
        let layout = drawn.layout().expect("a drawn text");
        let bounds = layout.bounds();
        if position.y < bounds.top() {
            return Some(Spot { text: k, at: 0 });
        }
        if position.y >= bounds.bottom() {
            return Some(Spot {
                text: k,
                at: drawn.text.len(),
            });
        }
        let across = point(position.x.clamp(bounds.left(), bounds.right()), position.y);
        let (Ok(at) | Err(at)) = layout.index_for_position(across);
        let at = drawn.mapped.as_ref().map_or(at, |hit| {
            hit.borrow()
                .as_ref()
                .map_or(at, |hit| super::math_text::back(&hit.swaps, at))
        });
        Some(Spot { text: k, at })
    }

    fn link(&self, spot: Spot) -> Option<String> {
        let links = &self.texts.get(spot.text)?.links;
        let (_, url) = links.iter().find(|(range, _)| range.contains(&spot.at))?;
        Some(url.clone())
    }
}

/// `texts` inside the span; a marker leads text taken from its start.
fn copied(texts: &[Drawn], (from, to): (Spot, Spot)) -> Option<String> {
    if from == to {
        return None;
    }
    let mut parts = Vec::new();
    for (k, drawn) in texts.iter().enumerate().take(to.text + 1).skip(from.text) {
        let range = part(k, drawn.text.len(), (from, to));
        let led = range.start == 0 && !range.is_empty();
        let Some(text) = drawn.text.get(range) else {
            log::error!("markdown: a selection fell inside a character");
            return None;
        };
        parts.push(match led {
            true => format!("{}{text}", drawn.lead),
            false => text.to_owned(),
        });
    }
    Some(parts.join("\n"))
}

/// Text `k`'s bytes inside the span.
fn part(k: usize, len: usize, (from, to): (Spot, Spot)) -> Range<usize> {
    let start = if k == from.text { from.at } else { 0 };
    let end = if k == to.text { to.at } else { len };
    start.min(end)..end
}

/// What one frame's drawing reads and gathers.
pub(crate) struct Gather {
    pub(crate) state: Entity<Selection>,
    pub(crate) on_menu: OnMenu,
    pub(crate) picture_menu: bool,
    span: (Spot, Spot),
    texts: RefCell<Vec<Drawn>>,
    lead: RefCell<String>,
    offset: usize,
}

impl Gather {
    pub(crate) fn new(state: Entity<Selection>, on_menu: OnMenu, cx: &App) -> Self {
        let span = state.read(cx).span();
        Self {
            state,
            on_menu,
            picture_menu: true,
            span,
            texts: RefCell::default(),
            lead: RefCell::default(),
            offset: 0,
        }
    }

    /// The selection's highlight on `text`, the next drawn.
    pub(crate) fn over(&self, text: &str, cx: &App) -> Vec<(Range<usize>, HighlightStyle)> {
        let k = self.offset + self.texts.borrow().len();
        let (from, to) = self.span;
        let range = part(k, text.len(), self.span);
        if from == to || k < from.text || k > to.text || range.is_empty() {
            return Vec::new();
        }
        // Last frame's span may not fit new text.
        let fits = range.end <= text.len()
            && text.is_char_boundary(range.start)
            && text.is_char_boundary(range.end);
        if !fits {
            return Vec::new();
        }
        let style = HighlightStyle {
            background_color: Some(cx.theme().colors.selection),
            ..Default::default()
        };
        vec![(range, style)]
    }

    /// Notes a drawn text, in order.
    pub(crate) fn push(&self, text: &str, layout: TextLayout, links: &[(Range<usize>, String)]) {
        self.texts.borrow_mut().push(Drawn {
            lead: self.lead.take(),
            text: text.to_owned(),
            layout: Some(layout),
            links: links.to_vec(),
            mapped: None,
        });
    }

    pub(crate) fn push_mapped(
        &self,
        text: &str,
        links: &[(Range<usize>, String)],
        hit: Rc<RefCell<Option<Mapped>>>,
    ) {
        self.push(text, TextLayout::default(), links);
        self.texts
            .borrow_mut()
            .last_mut()
            .expect("just pushed")
            .mapped = Some(hit);
    }

    pub(crate) fn block(state: Entity<Selection>, offset: usize, cx: &App) -> Self {
        let mut gather = Self::new(state, Rc::new(|_, _, _| {}), cx);
        gather.offset = offset;
        gather.picture_menu = false;
        gather
    }

    pub(crate) fn finish_block(self, cx: &mut App) {
        let texts = self.texts.into_inner();
        let range = self.offset..self.offset + texts.len();
        self.state.update(cx, |selection, _| {
            assert!(range.end <= selection.texts.len());
            selection.texts[range.clone()].clone_from_slice(&texts);
            selection.visible.push(range);
        });
    }

    /// Sets the marker the next text copies with; empty drops it.
    pub(crate) fn lead(&self, lead: String) {
        *self.lead.borrow_mut() = lead;
    }

    /// Keeps this frame's texts; other texts drop the selection.
    pub(crate) fn finish(self, cx: &mut App) -> Entity<Selection> {
        let texts = self.texts.into_inner();
        self.state.update(cx, |selection, cx| {
            let same = selection.texts.len() == texts.len()
                && (selection.texts.iter().zip(&texts)).all(|(old, new)| old.text == new.text);
            if !same {
                selection.anchor = Spot::default();
                selection.head = Spot::default();
                selection.dragging = false;
                selection.positioned = false;
                cx.notify();
            }
            selection.texts = texts;
        });
        self.state
    }
}

/// Dragging a link selects its label; a click still follows it.
pub(crate) fn link(
    gather: Option<&Gather>,
    on_link: &Option<super::OnLink>,
) -> Option<super::OnLink> {
    let Some(gather) = gather else {
        return on_link.clone();
    };
    let (state, on_link) = (gather.state.clone(), on_link.clone());
    Some(Rc::new(move |url, window, cx| {
        if state.read(cx).selected().is_none() {
            super::open_link(&on_link, url, window, cx);
        }
    }))
}

/// The document owns its focus and right-click menu.
pub(crate) fn preview(body: Div, state: Entity<Selection>, cx: &App) -> Div {
    selectable(body, state, Rc::new(|_, _, _| {}), cx)
}

/// A right press on a picture, its selection kept.
pub(crate) fn picture_press(
    state: &Entity<Selection>,
    on_menu: &OnMenu,
    (image, link): (String, Option<String>),
    position: Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let selected = state.read(cx).selected();
    let press = RightPress {
        selected,
        link,
        image: Some(image),
        position,
    };
    on_menu(press, window, cx);
}

/// `body` selects by drag, copies on the secondary C and selects all on A.
pub(crate) fn selectable(body: Div, state: Entity<Selection>, on_menu: OnMenu, cx: &App) -> Div {
    let focus = state.read(cx).focus.clone();
    let (down, moved, up, out, right) = (
        state.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
        state.clone(),
    );
    body.track_focus(&focus)
        .cursor(CursorStyle::IBeam)
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            down.update(cx, |selection, cx| {
                window.focus(&selection.focus, cx);
                let Some(spot) = selection.spot(event.position) else {
                    return;
                };
                if !event.modifiers.shift {
                    selection.anchor = spot;
                }
                selection.head = spot;
                selection.positioned = true;
                selection.dragging = true;
                cx.notify();
            })
        })
        .on_mouse_move(move |event, _, cx| {
            moved.update(cx, |selection, cx| {
                if !selection.dragging || !event.dragging() {
                    return;
                }
                if let Some(spot) = selection.spot(event.position)
                    && spot != selection.head
                {
                    selection.head = spot;
                    cx.notify();
                }
            })
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            up.update(cx, |selection, _| selection.dragging = false)
        })
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
            out.update(cx, |selection, _| selection.dragging = false)
        })
        .on_mouse_down(MouseButton::Right, move |event, window, cx| {
            let selection = right.read(cx);
            let link = selection
                .spot(event.position)
                .and_then(|spot| selection.link(spot));
            let press = RightPress {
                selected: selection.selected(),
                link,
                image: None,
                position: event.position,
            };
            on_menu(press, window, cx);
        })
        .on_key_down(move |event, _, cx| {
            let modifiers = event.keystroke.modifiers;
            if !modifiers.secondary(cx) || modifiers.shift || modifiers.alt {
                return;
            }
            match event.keystroke.key.as_str() {
                "c" => {
                    let Some(text) = state.read(cx).selected() else {
                        return;
                    };
                    log::info!("markdown: a selection copied");
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                    cx.stop_propagation();
                }
                "a" => {
                    state.update(cx, |selection, cx| {
                        let Some(last) = selection.texts.last() else {
                            return;
                        };
                        selection.anchor = Spot::default();
                        selection.head = Spot {
                            text: selection.texts.len() - 1,
                            at: last.text.len(),
                        };
                        selection.positioned = true;
                        cx.notify();
                    });
                    cx.stop_propagation();
                }
                _ => {}
            }
        })
}

#[cfg(test)]
mod tests;
