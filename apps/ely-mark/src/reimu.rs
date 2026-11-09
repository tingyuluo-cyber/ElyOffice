use ely_gpui_component::{
    menus::{Menu, MenuItem, menu_under},
    primitives::{FocusRing, Tooltip},
    theme::{ActiveTheme as _, ControlSize, Platform, Radius},
    typography::keys::os,
};
use ely_ui::{commands, i18n::text};
use gpui::{
    Action, AnyElement, App, Div, ElementId, InteractiveElement as _, IntoElement, MouseButton,
    OwnedMenuItem, ParentElement as _, Role, SharedString, Stateful,
    StatefulInteractiveElement as _, Styled as _, Window, canvas, div, prelude::FluentBuilder as _,
    px, rems, svg,
};

use crate::{keys, view::Mode};

pub(crate) mod appearance;
pub(crate) mod art;
pub(crate) mod glyphs;
pub(crate) mod minimap;
mod welcome;
use art::Art;
use glyphs::Glyph;
pub(crate) use welcome::welcome;

pub(crate) const NAME: &str = "博丽手记";

pub(crate) fn enabled() -> bool {
    cfg!(feature = "standalone") || ely_platform::env::flag("ELY_MARK_REIMU") == Some(true)
}

pub(crate) fn rail(
    active: Option<Mode>,
    side: bool,
    minimap: bool,
    wide: bool,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    let menu = cx
        .get_menus()
        .filter(|menus| !menus.is_empty())
        .map(|menus| {
            let menu = menu_rows(menus.into_iter().map(OwnedMenuItem::Submenu), window);
            let label = ely_gpui_component::i18n::text(cx, "menu.more", &[]);
            let trigger = rail_button("reimu-menu-button", glyphs::MENU, label, None, cx)
                .debug_selector(|| "reimu-menu-button".into());
            menu_under(
                "reimu-menu".into(),
                &menu,
                |press| {
                    trigger
                        .on_click(move |event, window, cx| press(event, window, cx))
                        .into_any_element()
                },
                window,
                cx,
            )
        });
    let theme = cx.theme();
    let dark = theme.is_dark();
    let ink = if dark {
        theme.colors.fg
    } else {
        theme.colors.on_accent
    };
    let compact = window.viewport_size().height < px(500.0);
    let divider = || {
        div()
            .my(rems(if compact { 0.25 } else { 0.5 }))
            .w_7()
            .h(px(1.0))
            .flex()
            .items_center()
            .gap_1()
            .children([false, true, false].map(|dot| {
                if dot {
                    svg()
                        .data(glyphs::SEAL.asset(dark))
                        .size(px(4.0))
                        .text_color(ink.alpha(0.75))
                        .into_any_element()
                } else {
                    div()
                        .flex_1()
                        .h(px(1.0))
                        .bg(ink.alpha(0.28))
                        .into_any_element()
                }
            }))
    };
    let rail = div()
        .flex()
        .flex_col()
        .flex_none()
        .items_center()
        .size_full()
        .py(rems(if compact { 0.25 } else { 0.75 }))
        .gap(px(if compact { 2.0 } else { 4.0 }))
        .border_r_1()
        .border_color(theme.colors.border)
        .bg(if dark {
            theme.colors.bg
        } else {
            theme.colors.accent
        })
        .children(menu)
        .children(side.then(|| {
            let shown = ely_ui::settings::panels::shown(true, cx);
            tool(
                "toggle sidebar",
                glyphs::PANEL,
                text(cx, "menu.view.left", &[]),
                Some(Box::new(commands::ToggleLeftPanel)),
                Some(shown),
                cx,
            )
        }))
        .child(
            rail_button(
                "reimu-appearance",
                glyphs::APPEARANCE,
                text(
                    cx,
                    if dark {
                        "settings.appearance.light"
                    } else {
                        "settings.appearance.dark"
                    },
                    &[],
                ),
                None,
                cx,
            )
            .debug_selector(|| "reimu-appearance".into())
            .size(theme.control_height(ControlSize::Sm))
            .on_click(|_, _, cx| appearance::toggle(cx)),
        );
    let Some(active) = active else { return rail };
    rail.child(divider())
        .child(tool(
            "reimu-new",
            glyphs::NEW,
            text(cx, "command.file.new", &[]),
            Some(Box::new(commands::New)),
            None,
            cx,
        ))
        .child(tool(
            "reimu-open",
            glyphs::OPEN,
            text(cx, "command.file.open", &[]),
            Some(Box::new(commands::Open)),
            None,
            cx,
        ))
        .child(tool(
            "reimu-save",
            glyphs::SAVE,
            text(cx, "command.file.save", &[]),
            Some(Box::new(commands::Save)),
            None,
            cx,
        ))
        .child(divider())
        .children(Mode::ALL.map(|mode| {
            let (id, glyph, command): (&str, Glyph, Box<dyn Action>) = match mode {
                Mode::Visual => ("toggle visual", glyphs::VISUAL, Box::new(keys::VisualMode)),
                Mode::Source => ("toggle source", glyphs::SOURCE, Box::new(keys::SourceMode)),
                Mode::Preview => (
                    "toggle preview",
                    glyphs::PREVIEW,
                    Box::new(keys::PreviewMode),
                ),
            };
            tool(
                id,
                glyph,
                keys::label(mode, cx),
                (mode != active).then_some(command),
                Some(mode == active),
                cx,
            )
        }))
        .child(wide_tool(active, wide, cx))
        .children(minimap::fits(window, cx).then(|| {
            tool(
                "toggle minimap",
                glyphs::MINIMAP,
                text(cx, "mark.look.minimap", &[]),
                Some(Box::new(crate::look::ToggleMinimap)),
                Some(minimap),
                cx,
            )
        }))
        .children((window.viewport_size().height >= px(500.0)).then(|| {
            div()
                .flex_1()
                .min_h_0()
                .w_full()
                .flex()
                .items_center()
                .justify_end()
                .flex_col()
                .pb_2()
                .overflow_hidden()
                .child(
                    svg()
                        .data(if dark {
                            include_bytes!("../assets/reimu/glyphs/parasol.svg")
                        } else {
                            include_bytes!("../assets/reimu/glyphs/gohei.svg")
                        })
                        .debug_selector(|| "reimu-gohei".into())
                        .w(px(34.0))
                        .h(px(92.0))
                        .max_h_full()
                        .flex_none()
                        .text_color(ink.alpha(0.30)),
                )
        }))
}

fn wide_tool(mode: Mode, wide: bool, cx: &App) -> impl IntoElement + use<> {
    let (id, key, action): (&str, &str, Box<dyn Action>) = match mode {
        Mode::Preview => (
            "toggle wide-preview",
            "mark.look.wide_preview",
            Box::new(crate::look::WidePreview),
        ),
        Mode::Visual | Mode::Source => (
            "toggle wide-edit",
            "mark.look.wide_edit",
            Box::new(crate::look::WideEdit),
        ),
    };
    tool(
        id,
        glyphs::WIDE,
        text(cx, key, &[]),
        Some(action),
        Some(wide),
        cx,
    )
}

fn menu_rows(items: impl IntoIterator<Item = OwnedMenuItem>, window: &Window) -> Menu {
    items
        .into_iter()
        .fold(Menu::new(), |menu, item| match item {
            OwnedMenuItem::Separator => menu.separator(),
            OwnedMenuItem::Submenu(sub) => menu.item(
                MenuItem::submenu(sub.name, menu_rows(sub.items, window)).disabled(sub.disabled),
            ),
            OwnedMenuItem::SystemMenu(system) => {
                menu.item(MenuItem::new(system.name).disabled(true))
            }
            OwnedMenuItem::Action {
                name,
                action,
                checked,
                disabled,
                ..
            } => {
                let key = window.highest_precedence_binding_for_action(action.as_ref());
                let row = if checked {
                    MenuItem::check(name, true)
                } else {
                    MenuItem::new(name)
                };
                let row = match key {
                    Some(key) => row.keys(
                        key.keystrokes()
                            .iter()
                            .map(|stroke| stroke.unparse(os(Platform::current())))
                            .collect::<Vec<_>>()
                            .join(" "),
                    ),
                    None => row,
                };
                menu.item(row.disabled(disabled).on_click(move |window, cx| {
                    window.dispatch_action(action.boxed_clone(), cx);
                }))
            }
        })
}

pub(crate) fn paper(body: AnyElement, map: Option<Div>, cx: &App) -> Div {
    let theme = cx.theme();
    div().size_full().p_2().bg(theme.colors.sunken).child(
        div()
            .size_full()
            .flex()
            .rounded(theme.radius(Radius::Lg))
            .border_1()
            .border_color(theme.colors.border)
            .overflow_hidden()
            .child(div().flex_1().min_w_0().h_full().child(body))
            .children(map),
    )
}

pub(crate) fn wallpaper(art: Art, map: bool, cx: &App) -> Option<Div> {
    let theme = cx.theme();
    (!theme.high_contrast()).then(|| {
        div()
            .debug_selector(|| "reimu-wallpaper".into())
            .absolute()
            .top_0()
            .bottom_0()
            .left_0()
            .right(if map {
                minimap::WIDTH + px(9.0)
            } else {
                px(0.0)
            })
            .overflow_hidden()
            .child(
                div()
                    .debug_selector(move || format!("reimu-wallpaper-{}", art.key()))
                    .size_full()
                    .opacity(if theme.is_dark() { 0.07 } else { 0.08 })
                    .child(
                        canvas(
                            move |_, window, cx| art.image().use_render_image(window, cx),
                            move |viewport, image, window, _| {
                                let Some(image) = image else { return };
                                let source_size =
                                    image.size(0).map(|dimension| px(dimension.0 as f32));
                                let bounds = art.wallpaper_bounds(viewport, source_size);
                                if let Err(error) = window.paint_image(
                                    viewport,
                                    bounds,
                                    Default::default(),
                                    image,
                                    0,
                                    false,
                                ) {
                                    log::error!("mark: could not draw wallpaper: {error}");
                                }
                            },
                        )
                        .size_full(),
                    ),
            )
    })
}

fn tool(
    id: &'static str,
    glyph: Glyph,
    label: SharedString,
    command: Option<Box<dyn Action>>,
    on: Option<bool>,
    cx: &App,
) -> Stateful<Div> {
    rail_button(id, glyph, label, on, cx)
        .debug_selector(move || id.into())
        .when_some(command, |button, command| {
            button.on_click(move |_, window, cx| window.dispatch_action(command.boxed_clone(), cx))
        })
}

fn rail_button(
    id: &'static str,
    glyph: Glyph,
    label: SharedString,
    on: Option<bool>,
    cx: &App,
) -> Stateful<Div> {
    let theme = cx.theme();
    let colors = &theme.colors;
    let selected = on == Some(true);
    let dark = theme.is_dark();
    let (ink, seal, on_seal) = if dark {
        (colors.fg, colors.accent, colors.on_accent)
    } else {
        (colors.on_accent, colors.on_accent, colors.accent)
    };
    let fg = if selected { on_seal } else { ink };
    button(id, label, on, ControlSize::default(), cx)
        .focus_visible(|style| style.border_color(ink))
        .when(!selected, |button| {
            button.hover(|style| style.bg(ink.alpha(0.16)))
        })
        .active(|style| style.bg(ink.alpha(0.23)))
        .children(selected.then(|| {
            svg()
                .absolute()
                .size_full()
                .data(glyphs::SEAL.asset(dark))
                .text_color(seal)
        }))
        .child(glyph.draw(dark, px(19.0), fg))
}

pub(crate) fn icon_button(
    id: impl Into<ElementId>,
    glyph: Glyph,
    label: SharedString,
    on: Option<bool>,
    size: ControlSize,
    cx: &App,
) -> Stateful<Div> {
    let theme = cx.theme();
    let colors = &theme.colors;
    let selected = on == Some(true);
    let fg = if selected {
        colors.on_accent
    } else if on.is_some() {
        colors.accent
    } else {
        colors.fg
    };
    button(id, label, on, size, cx)
        .when(!selected, |button| {
            button.hover(|style| style.bg(colors.accent.alpha(0.12)))
        })
        .active(|style| style.bg(colors.accent.alpha(0.18)))
        .children(selected.then(|| {
            svg()
                .absolute()
                .size_full()
                .data(glyphs::SEAL.asset(theme.is_dark()))
                .text_color(colors.accent)
        }))
        .child(glyph.draw(
            theme.is_dark(),
            px(if size == ControlSize::Sm { 14.0 } else { 19.0 }),
            fg,
        ))
}

fn button(
    id: impl Into<ElementId>,
    label: SharedString,
    on: Option<bool>,
    size: ControlSize,
    cx: &App,
) -> Stateful<Div> {
    let theme = cx.theme();
    div()
        .id(id)
        .role(Role::Button)
        .aria_label(label.clone())
        .when_some(on, |button, on| button.aria_toggled(on.into()))
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(theme.control_height(size))
        .border_1()
        .border_color(gpui::transparent_black())
        .rounded(theme.radius(Radius::Md))
        .tooltip(Tooltip::text(label))
        .tab_index(0)
        .focus_ring(cx)
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
}

#[cfg(test)]
mod tests;
