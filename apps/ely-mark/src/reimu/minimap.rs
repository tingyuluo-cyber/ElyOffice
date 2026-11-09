use std::{cell::Cell, rc::Rc};

use ely_gpui_component::{
    documents::MarkdownDocument,
    primitives::{FocusRing, Tooltip},
    theme::ActiveTheme as _,
};
use ely_ui::i18n::text;
use gpui::{
    App, AppContext as _, Bounds, Div, DragMoveEvent, EmptyView, Entity, EntityId,
    InteractiveElement as _, MouseButton, ParentElement as _, Pixels, Role, ScrollHandle,
    StatefulInteractiveElement as _, Styled as _, Window, bounds, canvas, div, fill, point, px,
    size,
};

use crate::view::{MarkView, Mode};

const ROWS: usize = 280;
pub(super) const WIDTH: Pixels = px(92.0);

#[derive(Clone, Copy)]
struct MapDrag(EntityId);

struct Scroll {
    document: Entity<MarkdownDocument>,
    source: Option<ScrollHandle>,
    lines: usize,
    plot: Cell<Bounds<Pixels>>,
}

impl Scroll {
    fn position(&self, cx: &App) -> (f32, Option<f32>) {
        if let Some(handle) = &self.source {
            let reach = handle.max_offset().y;
            let view = handle.bounds().size.height;
            return if reach > px(0.0) {
                (
                    (-handle.offset().y / reach).clamp(0.0, 1.0),
                    Some(view / (view + reach)),
                )
            } else {
                (0.0, Some(1.0))
            };
        }
        let top = self.document.read(cx).top_line(cx) as f32;
        let last = self.lines.saturating_sub(1).max(1) as f32;
        ((top / last).clamp(0.0, 1.0), None)
    }

    fn go(&self, share: f32, window: &mut Window, cx: &mut App) {
        let share = share.clamp(0.0, 1.0);
        self.document.update(cx, |doc, cx| {
            if let Some(handle) = &self.source {
                let offset = handle.offset();
                handle.set_offset(point(offset.x, -(handle.max_offset().y * share)));
                cx.notify();
                return;
            }
            let top = (self.lines.saturating_sub(1) as f32 * share).round() as usize;
            if doc.look().typewriter {
                doc.set_caret(top, 0, cx);
            }
            doc.set_top_line(top, window, cx);
        });
    }

    fn pointer(&self, y: Pixels, grip: Option<Pixels>, window: &mut Window, cx: &mut App) {
        let plot = self.plot.get();
        let mut travel = plot.size.height;
        let mut offset = y - plot.origin.y;
        if let (Some(grip), Some(shown)) = (grip, self.position(cx).1) {
            travel -= thumb_height(travel, Some(shown));
            offset -= grip;
        }
        if travel > px(0.0) {
            self.go(offset / travel, window, cx);
        }
    }
}

fn thumb_height(height: Pixels, shown: Option<f32>) -> Pixels {
    (height * shown.unwrap_or(0.0)).max(px(3.0)).min(height)
}

pub(crate) fn fits(window: &Window, cx: &App) -> bool {
    let min = if ely_ui::settings::panels::shown(true, cx) {
        940.0
    } else {
        680.0
    };
    window.viewport_size().width >= px(min)
}

pub(crate) fn panel(view: Entity<MarkView>, window: &mut Window, cx: &mut App) -> Div {
    let view = view.read(cx);
    let doc = view.document().read(cx);
    let editor = doc.editor().read(cx);
    let buffer = editor.buffer();
    let lines = buffer.lines();
    let rows = lines.min(ROWS);
    let strokes: Vec<_> = (0..rows)
        .map(|row| {
            let line = &buffer.text()[buffer.line_range(row * lines / rows)];
            let line = line
                .char_indices()
                .nth(130)
                .map_or(line, |(at, _)| &line[..at]);
            let text = line.trim();
            let indent = line.len().saturating_sub(line.trim_start().len()).min(20);
            (
                indent,
                text.chars().take(110).count(),
                text.starts_with('#'),
            )
        })
        .collect();
    let theme = cx.theme();
    let colors = &theme.colors;
    let (surface, border) = (colors.surface, colors.border);
    let scroll = Rc::new(Scroll {
        document: view.document().clone(),
        source: (view.mode == Mode::Source).then(|| editor.scroll_handle()),
        lines,
        plot: Cell::new(Bounds::default()),
    });
    let (ink, accent, mark) = (
        colors.fg_muted.alpha(0.64),
        colors.accent,
        colors.accent.alpha(0.16),
    );
    let measure = scroll.clone();
    let drawing = canvas(
        move |area: Bounds<Pixels>, _, cx| {
            let plot = bounds(
                area.origin + point(px(7.0), px(7.0)),
                size(
                    (area.size.width - px(14.0)).max(px(0.0)),
                    (area.size.height - px(14.0)).max(px(0.0)),
                ),
            );
            measure.plot.set(plot);
            (plot, measure.position(cx))
        },
        move |_, (plot, (passed, shown)), window, _| {
            if rows == 0 || plot.size.height <= px(0.0) {
                return;
            }
            let step = plot.size.height / rows as f32;
            let thumb = thumb_height(plot.size.height, shown);
            let y = (plot.size.height - thumb) * passed;
            window.paint_quad(fill(
                bounds(
                    plot.origin + point(px(0.0), y),
                    size(plot.size.width, thumb),
                ),
                mark,
            ));
            for (row, (indent, length, heading)) in strokes.iter().copied().enumerate() {
                if length == 0 {
                    continue;
                }
                let x = px(indent as f32 * 0.42);
                let w = px(length as f32 * 0.62).min(plot.size.width - x);
                let line = bounds(
                    plot.origin + point(x, step * row as f32),
                    size(w, step.min(px(if heading { 2.4 } else { 1.4 }))),
                );
                window.paint_quad(fill(line, if heading { accent } else { ink }));
            }
        },
    )
    .size_full();
    let label = text(cx, "mark.look.minimap", &[]);
    let id = scroll.document.entity_id();
    let progress = scroll.position(cx).0;
    let grip = window
        .use_keyed_state(("reimu-minimap-grip", id), cx, |_, _| {
            Rc::new(Cell::new(None::<Pixels>))
        })
        .read(cx)
        .clone();
    let held = grip.clone();
    let press = scroll.clone();
    let drag = scroll.clone();
    let map = div()
        .id("reimu-minimap")
        .debug_selector(|| "reimu-minimap".into())
        .role(Role::Slider)
        .aria_label(label.clone())
        .aria_min_numeric_value(0.0)
        .aria_max_numeric_value(100.0)
        .aria_numeric_value((progress as f64 * 100.0).round())
        .aria_numeric_value_step(1.0)
        .w(WIDTH)
        .h_full()
        .flex_none()
        .bg(surface)
        .border_l_1()
        .border_color(border)
        .tab_index(0)
        .focus_ring(cx)
        .cursor_pointer()
        .tooltip(Tooltip::text(label))
        .child(drawing)
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            window.prevent_default();
            let plot = press.plot.get();
            let (passed, shown) = press.position(cx);
            let thumb = thumb_height(plot.size.height, shown);
            let top = plot.top() + (plot.size.height - thumb) * passed;
            let offset = event.position.y - top;
            let grabbed =
                (shown.is_some() && offset >= px(0.0) && offset <= thumb).then_some(offset);
            held.set(grabbed);
            if grabbed.is_none() {
                press.pointer(event.position.y, None, window, cx);
            }
        })
        .on_drag(MapDrag(id), |_, _, _, cx| cx.new(|_| EmptyView))
        .on_drag_move(move |event: &DragMoveEvent<MapDrag>, window, cx| {
            if event.drag(cx).0 == id {
                drag.pointer(event.event.position.y, grip.get(), window, cx);
            }
        })
        .on_key_down(move |event, window, cx| {
            if event.keystroke.modifiers.modified() {
                return;
            }
            let (at, shown) = scroll.position(cx);
            let step = if shown.is_some() {
                0.01
            } else {
                1.0 / lines.saturating_sub(1).max(1) as f32
            };
            let page = shown.map_or(20.0 * step, |shown| shown / (1.0 - shown).max(0.01));
            let to = match event.keystroke.key.as_str() {
                "up" => at - step,
                "down" => at + step,
                "pageup" => at - page,
                "pagedown" => at + page,
                "home" => 0.0,
                "end" => 1.0,
                _ => return,
            };
            cx.stop_propagation();
            scroll.go(to, window, cx);
        });
    div().h_full().flex_none().child(map)
}
