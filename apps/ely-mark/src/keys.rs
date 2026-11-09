//! Modes, menus and keys, as Typora's (MARK-01, MARK-02, MARK-15).

use ely_gpui_component::secondary;
use ely_ui::{commands, i18n::text};
use gpui::{Action, App, Context, InteractiveElement as _, KeyBinding, Menu, MenuItem, actions};

use crate::{desk::Desk, view::Mode};

actions!(
    mark,
    [
        VisualMode,
        SourceMode,
        PreviewMode,
        ToggleSource,
        ToggleReading,
        NextTab,
        PreviousTab,
        CheckDisk
    ]
);

pub fn bind(reimu: bool, cx: &mut App) {
    let at = Some(commands::CONTEXT);
    let source = if reimu {
        secondary!("shift-e")
    } else {
        secondary!("/")
    };
    cx.bind_keys([
        KeyBinding::new(commands::OPEN_FOLDER.keys, commands::OpenFolder, at),
        KeyBinding::new(source, ToggleSource, at),
        KeyBinding::new(source, ToggleSource, Some("ElyEditor")),
        KeyBinding::new("f8", crate::look::FocusMode, at),
        KeyBinding::new("f9", crate::look::Typewriter, at),
        KeyBinding::new(secondary!("+"), crate::look::Larger, at),
        KeyBinding::new(secondary!("_"), crate::look::Smaller, at),
        KeyBinding::new(secondary!(")"), crate::look::ActualSize, at),
        KeyBinding::new(secondary!("f"), ely_office::FindReplace, at),
    ]);
    if reimu {
        cx.bind_keys([
            KeyBinding::new(secondary!("e"), ToggleReading, at),
            KeyBinding::new(commands::SIDE_EXPORT_KEYS, commands::Export, at),
            KeyBinding::new("ctrl-tab", NextTab, at),
            KeyBinding::new("ctrl-shift-tab", PreviousTab, at),
        ]);
    }
    // Windows' Chinese input keeps Ctrl+Shift+0 for itself.
    #[cfg(target_os = "windows")]
    cx.bind_keys([KeyBinding::new("ctrl-(", crate::look::ActualSize, at)]);
    crate::blocks::bind(cx);
    crate::tools::bind(cx);
    crate::clip::bind(cx);
}

pub fn file(reimu: bool) -> &'static [&'static commands::Command] {
    if reimu {
        &commands::MARKDOWN_SIDE
    } else {
        &commands::MARKDOWN
    }
}

/// The label of `mode`.
pub fn label(mode: Mode, cx: &App) -> gpui::SharedString {
    text(cx, &format!("mark.mode.{}", mode.name()), &[])
}

/// A menu row named by `key`, ticked when `checked`.
pub(crate) fn checked(key: &str, action: Box<dyn Action>, checked: bool, cx: &App) -> MenuItem {
    MenuItem::Action {
        name: text(cx, key, &[]),
        action,
        os_action: None,
        checked,
        disabled: false,
    }
}

pub(crate) fn item(key: &str, action: Box<dyn Action>, cx: &App) -> MenuItem {
    checked(key, action, false, cx)
}

pub fn menus(look: &crate::look::Look, reimu: bool, cx: &App) -> Vec<Menu> {
    use crate::look::{self, Font, Width};
    let checked = |key: &str, action: Box<dyn Action>, on: bool| checked(key, action, on, cx);
    let item = |key: &str, action: Box<dyn Action>| item(key, action, cx);
    let reading = reimu
        .then(|| {
            [
                item("mark.view.reading", Box::new(ToggleReading)),
                checked("mark.look.wide_edit", Box::new(look::WideEdit), look.wide_edit),
                checked(
                    "mark.look.wide_preview",
                    Box::new(look::WidePreview),
                    look.wide_preview,
                ),
                checked("mark.look.minimap", Box::new(look::ToggleMinimap), look.minimap),
            ]
        })
        .into_iter()
        .flatten();
    vec![
        Menu::new(text(cx, "menu.edit", &[])).items([
            item("mark.copy.plain", Box::new(crate::export::CopyPlain)),
            item("mark.copy.markdown", Box::new(crate::clip::CopyMarkdown)),
            item("mark.copy.html", Box::new(crate::export::CopyHtml)),
            item("mark.copy.rich", Box::new(crate::export::CopyRich)),
            item("mark.paste.plain", Box::new(crate::clip::PastePlain)),
            MenuItem::separator(),
            item("office.find.open", Box::new(ely_office::FindReplace)),
        ]),
        crate::blocks::paragraph_menu(cx),
        crate::blocks::format_menu(cx),
        Menu::new(text(cx, "menu.view", &[])).items(reading.chain([
            item("mark.view.source", Box::new(ToggleSource)),
            MenuItem::action(label(Mode::Visual, cx), VisualMode),
            MenuItem::action(label(Mode::Source, cx), SourceMode),
            MenuItem::action(label(Mode::Preview, cx), PreviewMode),
            MenuItem::separator(),
            ely_ui::commands::left_panel_item(cx),
            checked("mark.look.focus", Box::new(look::FocusMode), look.focus),
            checked(
                "mark.look.typewriter",
                Box::new(look::Typewriter),
                look.typewriter,
            ),
            checked(
                "mark.look.line_numbers",
                Box::new(look::LineNumbers),
                look.line_numbers,
            ),
            MenuItem::separator(),
            checked(
                "mark.look.narrow",
                Box::new(look::Narrow),
                look.width == Width::Narrow,
            ),
            checked(
                "mark.look.medium",
                Box::new(look::Medium),
                look.width == Width::Medium,
            ),
            checked(
                "mark.look.wide",
                Box::new(look::Wide),
                look.width == Width::Wide,
            ),
            MenuItem::separator(),
            checked(
                "mark.look.sans",
                Box::new(look::Sans),
                look.font == Font::Sans,
            ),
            checked(
                "mark.look.serif",
                Box::new(look::Serif),
                look.font == Font::Serif,
            ),
            checked(
                "mark.look.mono",
                Box::new(look::Mono),
                look.font == Font::Mono,
            ),
            checked(
                "mark.look.other",
                Box::new(look::OtherFont),
                matches!(look.font, Font::Named(_)),
            ),
            MenuItem::separator(),
            item("mark.look.actual", Box::new(look::ActualSize)),
            item("mark.look.larger", Box::new(look::Larger)),
            item("mark.look.smaller", Box::new(look::Smaller)),
            MenuItem::separator(),
            item("mark.outside.check", Box::new(CheckDisk)),
        ])),
    ]
}

impl Desk {
    fn step_tab(&mut self, step: isize, window: &mut gpui::Window, cx: &mut Context<Self>) {
        use ely_office::Chrome as _;
        let tabs: Vec<_> = self
            .core
            .tabs
            .iter()
            .map(|tab| tab.id)
            .chain(self.core.loading.iter().map(|(id, ..)| *id))
            .collect();
        if let Some(active) = tabs.iter().position(|id| Some(*id) == self.core.active()) {
            let next = (active as isize + step).rem_euclid(tabs.len() as isize) as usize;
            self.show_tab(tabs[next], window, cx);
        }
    }

    fn toggle_reading(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) {
        let Some(view) = self.active_view() else {
            return self.no_document(cx);
        };
        let view = view.read(cx);
        let mode = if view.mode == Mode::Preview {
            view.editing_mode
        } else {
            Mode::Preview
        };
        self.switch(mode, window, cx);
    }

    /// Source and visual, as Typora's Source Code Mode.
    fn toggle_source(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) {
        let Some(view) = self.active_view() else {
            return self.no_document(cx);
        };
        let mode = match view.read(cx).mode {
            Mode::Source => Mode::Visual,
            Mode::Visual | Mode::Preview => Mode::Source,
        };
        self.switch(mode, window, cx);
    }

    pub fn mode_actions(root: gpui::Div, cx: &mut Context<Self>) -> gpui::Div {
        root.on_action(cx.listener(|desk, _: &CheckDisk, _, cx| desk.look_all(cx)))
            .on_action(cx.listener(|desk, _: &NextTab, window, cx| desk.step_tab(1, window, cx)))
            .on_action(
                cx.listener(|desk, _: &PreviousTab, window, cx| desk.step_tab(-1, window, cx)),
            )
            .on_action(
                cx.listener(|desk, _: &ToggleReading, window, cx| desk.toggle_reading(window, cx)),
            )
            .on_action(
                cx.listener(|desk, _: &ToggleSource, window, cx| desk.toggle_source(window, cx)),
            )
            .on_action(
                cx.listener(|desk, _: &VisualMode, window, cx| {
                    desk.switch(Mode::Visual, window, cx)
                }),
            )
            .on_action(
                cx.listener(|desk, _: &SourceMode, window, cx| {
                    desk.switch(Mode::Source, window, cx)
                }),
            )
            .on_action(cx.listener(|desk, _: &PreviewMode, window, cx| {
                desk.switch(Mode::Preview, window, cx)
            }))
    }
}
