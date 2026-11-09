use ely_gpui_component::{
    buttons::SegmentedControl,
    primitives::{FocusRing, IconName, Tooltip},
    theme::{ActiveTheme as _, ControlSize, Radius, TextSize},
    typography::Ellipsis,
};
use ely_ui::i18n::text;
use gpui::{
    AnyElement, Bounds, Context, Font, FontWeight, Hsla, InteractiveElement as _, IntoElement as _,
    MouseButton, ParentElement as _, Pixels, Role, StatefulInteractiveElement as _, Styled as _,
    TextRun, Window, canvas, div, prelude::FluentBuilder as _, px, rems,
};

use super::{Pane, Side};
use crate::reimu::glyphs;

impl Pane {
    const ALL: [Self; 3] = [Self::Files, Self::Outline, Self::Search];

    fn key(self) -> &'static str {
        match self {
            Self::Files => "files",
            Self::Outline => "outline",
            Self::Search => "search",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Files => "mark.pane.files",
            Self::Outline => "mark.pane.outline",
            Self::Search => "mark.pane.search",
        }
    }
}

impl Side {
    pub(super) fn panes(
        &self,
        reimu: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if reimu {
            return self.themed_panes(window, cx);
        }
        let this = cx.entity();
        let initial = SegmentedControl::new("mark-pane", self.pane.key()).size(ControlSize::Sm);
        Pane::ALL
            .into_iter()
            .fold(initial, |tabs, pane| {
                let icon = match pane {
                    Pane::Files => IconName::Folder,
                    Pane::Outline => IconName::List,
                    Pane::Search => IconName::Search,
                };
                tabs.segment(pane.key(), text(cx, pane.label(), &[]), Some(icon))
            })
            .on_change(move |key, _, cx| {
                if let Some(pane) = Pane::ALL
                    .into_iter()
                    .find(|pane| pane.key() == key.as_ref())
                {
                    this.update(cx, |side, cx| side.choose(pane, cx));
                }
            })
            .into_any_element()
    }

    fn choose(&mut self, pane: Pane, cx: &mut Context<Self>) {
        self.pane = pane;
        cx.notify();
    }

    fn themed_panes(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let width = window.use_keyed_state("mark-pane-width", cx, |_, _| Pixels::ZERO);
        let theme = cx.theme();
        let colors = &theme.colors;
        let panes = [Pane::Files, Pane::Search];
        let labels = panes.map(|pane| text(cx, pane.label(), &[]));
        let font_size = theme.text_size(TextSize::Sm).to_pixels(window.rem_size());
        let font = Font {
            weight: FontWeight::SEMIBOLD,
            ..window.text_style().font()
        };
        let widest = labels
            .iter()
            .map(|label| {
                let run = TextRun {
                    len: label.len(),
                    font: font.clone(),
                    color: Hsla::default(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                window
                    .text_system()
                    .shape_line(label.clone(), font_size, &[run], None)
                    .width
            })
            .fold(Pixels::ZERO, Pixels::max);
        let spacing = rems(0.75).to_pixels(window.rem_size());
        let compact = *width.read(cx) < (widest + px(16.0) + spacing) * 2.0 + spacing;
        let measure = canvas(
            move |bounds: Bounds<Pixels>, window, cx| {
                if *width.read(cx) != bounds.size.width {
                    width.update(cx, |width, cx| {
                        *width = bounds.size.width;
                        cx.notify();
                        window.request_animation_frame();
                    });
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full();
        div()
            .id("mark-pane")
            .role(Role::RadioGroup)
            .relative()
            .flex()
            .w_full()
            .flex_none()
            .gap_1()
            .p_0p5()
            .rounded(theme.radius(Radius::Md))
            .bg(colors.sunken)
            .child(measure)
            .children(panes.into_iter().zip(labels).map(|(pane, label)| {
                let glyph = match pane {
                    Pane::Files => glyphs::FOLDER,
                    Pane::Outline => glyphs::OUTLINE,
                    Pane::Search => glyphs::SEARCH,
                };
                let this = cx.entity();
                let key = pane.key();
                let on = self.pane == pane;
                let fg = if on { colors.accent } else { colors.fg_muted };
                div()
                    .id(key)
                    .debug_selector(move || format!("segment {key}"))
                    .role(Role::RadioButton)
                    .aria_toggled(on.into())
                    .aria_label(label.clone())
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .px_1()
                    .h(theme.control_height(ControlSize::Sm))
                    .border_1()
                    .border_color(gpui::transparent_black())
                    .rounded(theme.radius(Radius::Sm))
                    .text_size(theme.text_size(TextSize::Sm))
                    .text_color(fg)
                    .when(on, |tab| {
                        tab.bg(colors.surface).font_weight(FontWeight::SEMIBOLD)
                    })
                    .when(!on, |tab| tab.hover(|style| style.bg(colors.hover)))
                    .child(glyph.draw(theme.is_dark(), px(14.0), fg))
                    .children((!compact).then(|| {
                        div()
                            .min_w_0()
                            .overflow_hidden()
                            .debug_selector(move || format!("segment-label {key}"))
                            .child(Ellipsis::new(label.clone()))
                    }))
                    .tooltip(Tooltip::text(label))
                    .cursor_pointer()
                    .tab_index(0)
                    .focus_ring(cx)
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                    .on_click(move |_, _, cx| {
                        if !on {
                            this.update(cx, |side, cx| side.choose(pane, cx));
                        }
                    })
            }))
            .into_any_element()
    }
}
