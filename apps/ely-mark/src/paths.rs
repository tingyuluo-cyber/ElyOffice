//! Copying a file's name and path, from tab or row.

use std::path::{Path, PathBuf};

use ely_gpui_component::{
    lists::{ListItem, SelectableList, row_action},
    menus::{ContextMenu, Menu, MenuItem},
    navigation::EditorTabs,
    primitives::{Icon, IconName},
    theme::{ActiveTheme as _, ControlSize},
};
use ely_office::prelude::*;
use ely_ui::{i18n::text, menu::Offer};
use gpui::{
    App, ClipboardItem, Context, Entity, InteractiveElement as _, IntoElement as _, Pixels, Point,
    SharedString, StatefulInteractiveElement as _, Styled as _, px,
};

use crate::desk::Desk;
use crate::reimu::{glyphs, icon_button};

/// Copy File Name, then Copy Path; roots have no name.
pub fn copies(menu: Menu, path: &Path, cx: &App) -> Menu {
    let menu = match path.file_name() {
        Some(name) => {
            let name = name.to_string_lossy().into_owned();
            menu.item(copy(text(cx, "mark.copy.name", &[]), name, "name"))
        }
        None => menu,
    };
    let whole = path.display().to_string();
    menu.item(copy(text(cx, "mark.copy.path", &[]), whole, "path"))
}

fn copy(label: gpui::SharedString, words: String, what: &'static str) -> MenuItem {
    MenuItem::new(label).on_click(move |_, cx| {
        cx.write_to_clipboard(ClipboardItem::new_string(words.clone()));
        log::info!("mark: copied a file's {what}");
    })
}

impl Desk {
    pub(crate) fn side_tabs(desk: &Entity<Self>, cx: &App) -> Option<SelectableList> {
        let desk_view = desk.read(cx);
        let core = &desk_view.core;
        if !desk_view.reimu || (core.tabs.is_empty() && core.loading.is_empty()) {
            return None;
        }
        let close_name = ely_gpui_component::i18n::text(cx, "tabs.close", &[]);
        let row = |id, name: SharedString, dirty, loading| {
            let (close, menu) = (desk.downgrade(), desk.downgrade());
            let color = if dirty {
                cx.theme().colors.accent
            } else {
                cx.theme().colors.fg_muted
            };
            let mark = if loading {
                Icon::new(IconName::LoaderCircle)
                    .color(color)
                    .into_any_element()
            } else {
                let glyph = if dirty { glyphs::DIRTY } else { glyphs::NOTE };
                glyph
                    .draw(cx.theme().is_dark(), px(16.0), color)
                    .into_any_element()
            };
            let close_name = format!("{close_name} · {name}").into();
            ListItem::new(("side-tab", id), name)
                .quiet(loading)
                .leading(mark)
                .trailing(
                    row_action(
                        icon_button(
                            ("side-tab-close", id),
                            glyphs::CLOSE,
                            close_name,
                            None,
                            ControlSize::Sm,
                            cx,
                        )
                        .on_click(move |_, window, cx| {
                            close
                                .update(cx, |desk, cx| {
                                    desk.request_close(id, ely_office::After::Tab, window, cx)
                                })
                                .ok();
                        }),
                    )
                    .debug_selector(move || format!("side-tab-close-{id}")),
                )
                .on_secondary(move |at, _, cx| {
                    menu.update(cx, |desk, cx| desk.ask_tab(id, at, cx)).ok();
                })
        };
        let open = core
            .tabs
            .iter()
            .map(|tab| (tab.id, row(tab.id, tab.name.clone(), tab.dirty(cx), false)));
        let waiting = core
            .loading
            .iter()
            .map(|(id, name, _)| (*id, row(*id, name.clone(), false, true)));
        let list = SelectableList::new("mark-side-tabs")
            .label(text(cx, "mark.tabs.open", &[]))
            .selected(core.active().map(|id| id.to_string()))
            .flex_none()
            .max_h(px(148.0));
        let (select, activate) = (desk.downgrade(), desk.downgrade());
        Some(
            open.chain(waiting)
                .fold(list, |list, (id, row)| list.row(id.to_string(), row))
                .on_change(move |keys, _, cx| {
                    if let Some(key) = keys.first() {
                        let id = key.parse().expect("tab ids are numbers");
                        select
                            .update(cx, |desk, cx| {
                                desk.core.set_active(Some(id));
                                cx.notify();
                            })
                            .ok();
                    }
                })
                .on_activate(move |key, window, cx| {
                    let id = key.parse().expect("tab ids are numbers");
                    activate
                        .update(cx, |desk, cx| desk.show_tab(id, window, cx))
                        .ok();
                }),
        )
    }

    /// The tab strip; a right click offers the tab's file.
    pub(crate) fn tabs(&self, cx: &mut Context<Self>) -> EditorTabs {
        let desk = cx.weak_entity();
        self.tab_strip(cx).on_context(move |id, at, _, cx| {
            let id = id.parse().expect("tab ids are numbers");
            desk.update(cx, |desk, cx| desk.ask_tab(id, at, cx)).ok();
        })
    }

    fn ask_tab(&mut self, id: u64, at: Point<Pixels>, cx: &mut Context<Self>) {
        let path = self
            .core
            .tabs
            .iter()
            .find(|tab| tab.id == id)
            .and_then(|tab| tab.session.path());
        let Some(path) = path.map(PathBuf::from) else {
            return log::info!("mark: tab {id} has no file to copy");
        };
        self.tab_asked = Some(Offer::new(self.tab_asked.as_ref(), id, path, at));
        cx.notify();
    }

    /// The open tab menu's host, if one was asked.
    pub(crate) fn tab_menu(&self, cx: &App) -> Option<ContextMenu> {
        let offer = self.tab_asked.as_ref()?;
        Some(offer.host("mark-tab-menu", copies(Menu::new(), &offer.what, cx)))
    }
}
