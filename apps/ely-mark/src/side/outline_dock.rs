use ely_gpui_component::{
    layout::SplitPane,
    primitives::{FocusRing, Icon, IconName},
    theme::{ActiveTheme as _, ControlSize, IconSize, TextSize},
};
use ely_ui::i18n::text;
use gpui::{
    Axis, Context, Div, Entity, FontWeight, InteractiveElement as _, IntoElement as _, MouseButton,
    ParentElement as _, Role, StatefulInteractiveElement as _, Styled as _, div, px,
};

use super::Side;
use crate::{desk::Desk, reimu::glyphs};

impl Side {
    pub(super) fn outline_dock(
        &self,
        explorer: Div,
        desk: &Entity<Desk>,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = cx.theme();
        let this = cx.entity();
        let key = this.clone();
        let label = text(cx, "mark.pane.outline", &[]);
        let header = div()
            .id("mark-outline-switch")
            .debug_selector(|| "mark-outline-switch".into())
            .role(Role::Button)
            .aria_label(label.clone())
            .aria_expanded(self.outline_open)
            .h(theme.control_height(ControlSize::Md))
            .flex_none()
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .border_t_1()
            .border_color(theme.colors.border)
            .text_size(theme.text_size(TextSize::Sm))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.colors.fg)
            .child(
                Icon::new(if self.outline_open {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                })
                .size(IconSize::Xs),
            )
            .child(glyphs::OUTLINE.draw(theme.is_dark(), px(14.0), theme.colors.accent))
            .child(label)
            .hover(|style| style.bg(theme.colors.hover))
            .focus_ring(cx)
            .tab_index(0)
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
            .on_click(move |_, _, cx| {
                this.update(cx, |side, cx| {
                    side.outline_open = !side.outline_open;
                    if side.outline_open {
                        side.outline_share = side.outline_share.max(0.15);
                    }
                    cx.notify();
                });
            })
            .on_key_down(move |event, _, cx| {
                if event.keystroke.modifiers.modified() || !key.read(cx).outline_open {
                    return;
                }
                let share = key.read(cx).outline_share;
                let share = match event.keystroke.key.as_str() {
                    "up" => share + 0.12,
                    "down" => share - 0.12,
                    "home" => 1.0,
                    "end" => 0.15,
                    _ => return,
                };
                cx.stop_propagation();
                key.update(cx, |side, cx| {
                    side.outline_share = share.clamp(0.15, 1.0);
                    cx.notify();
                });
            });
        let body = if self.outline_open {
            let this = cx.entity();
            let share = self.outline_share;
            SplitPane::new("mark-outline-split", Axis::Vertical, px(0.0))
                .sizes(&[1.0 - share, share])
                .pane(if share < 1.0 { explorer } else { div() })
                .pane(
                    div()
                        .debug_selector(|| "mark-outline-dock".into())
                        .p_2()
                        .size_full()
                        .child(self.outline(desk, cx)),
                )
                .on_resize(move |sizes, _, cx| {
                    this.update(cx, |side, cx| {
                        if sizes[1] <= 0.0 {
                            side.outline_open = false;
                        } else {
                            side.outline_share = sizes[1].min(1.0);
                        }
                        cx.notify();
                    });
                })
                .into_any_element()
        } else {
            explorer.into_any_element()
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().colors.surface)
            .child(div().flex_1().min_h_0().overflow_hidden().child(body))
            .child(header)
    }
}
