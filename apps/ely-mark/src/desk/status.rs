use ely_gpui_component::{
    buttons::SegmentedControl,
    shell::{StatusBar, StatusBarItem},
    theme::{ActiveTheme as _, ControlSize},
};
use ely_office::{Chrome as _, Tabs as _};
use ely_ui::i18n::text;
use gpui::{App, Context, SharedString, px};

use super::Desk;
use crate::{keys, reimu::glyphs, view::Mode};

impl Desk {
    pub(crate) fn caret_said(&self, cx: &App) -> Option<SharedString> {
        let view = self.active_view()?;
        let (line, column) = view.read(cx).document().read(cx).caret(cx);
        let (line, column) = ((line + 1).to_string(), (column + 1).to_string());
        Some(text(
            cx,
            "mark.caret",
            &[("line", &line), ("column", &column)],
        ))
    }

    pub(super) fn status(&self, cx: &mut Context<Self>) -> StatusBar {
        let mut state = StatusBarItem::new("mark-state").label(self.save_state(cx));
        if let Some(tab) = self.active_tab().filter(|_| self.reimu) {
            let theme = cx.theme();
            let colors = &theme.colors;
            let color = if tab.state_key(cx) == "office.state.saved" {
                colors.fg_muted
            } else {
                colors.accent
            };
            state = state.leading(glyphs::BOW.draw(theme.is_dark(), px(13.0), color));
        }
        let mut bar = StatusBar::new().left(state);
        let Some(view) = self.active_view() else {
            return bar;
        };
        if let Some(git) = self.git_item(cx) {
            bar = bar.left(git);
        }
        if let Some(count) = self.count_item(cx) {
            bar = bar.right(count);
        }
        let caret = self.caret_said(cx).expect("a tab");
        let bar = bar.right(StatusBarItem::new("mark-caret").label(caret));
        if self.reimu {
            return bar;
        }

        let modes = Mode::ALL.into_iter().fold(
            SegmentedControl::new("mark-mode", view.read(cx).mode.name()).size(ControlSize::Sm),
            |control, mode| control.segment(mode.name(), keys::label(mode, cx), Some(mode.icon())),
        );
        let this = cx.entity();
        bar.right(modes.on_change(move |key, window, cx| {
            if let Some(mode) = Mode::parse(key) {
                this.update(cx, |desk, cx| desk.switch(mode, window, cx));
            }
        }))
    }
}
