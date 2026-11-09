use std::time::Instant;

use ely_gpui_component::{
    layout::AppShell,
    menus::MenuBar,
    motion,
    navigation::EditorTabs,
    primitives::FocusScope,
    shell::{ResizeBorder, StatusBar, TitleBar, Toolbar},
    theme::{ActiveTheme, TextSize},
};
use gpui::{
    AnyElement, AnyView, App, ElementId, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, Styled, Window, div, px,
};

mod page;
mod panel;
mod root;
#[cfg(test)]
mod tests;

use page::Fade;
use panel::{Side, SidePanel};
pub use root::{Root, blank, document_root, root, set_document_menus, tool_root};

/// The window frame; empty panels take no room.
#[derive(IntoElement)]
pub struct Shell {
    focus: FocusHandle,
    title: SharedString,
    tabs: Option<EditorTabs>,
    rail: Option<(Pixels, AnyElement)>,
    left: Option<AnyView>,
    right: Option<AnyView>,
    status: Option<StatusBar>,
    toolbar: Option<Toolbar>,
    page: Option<ElementId>,
    main: bool,
    body: Vec<AnyElement>,
    workspace_decoration: Option<AnyElement>,
}

impl Shell {
    pub fn new(title: impl Into<SharedString>, focus: &FocusHandle) -> Self {
        Self {
            focus: focus.clone(),
            title: title.into(),
            tabs: None,
            rail: None,
            left: None,
            right: None,
            status: None,
            toolbar: None,
            page: None,
            main: true,
            body: Vec::new(),
            workspace_decoration: None,
        }
    }

    /// Beside the main window: no menus, no update banner.
    pub fn secondary(mut self) -> Self {
        self.main = false;
        self
    }

    pub fn tabs(mut self, tabs: EditorTabs) -> Self {
        self.tabs = Some(tabs);
        self
    }

    /// Replaces the in-window menu bar with a leading rail.
    pub fn rail(mut self, width: Pixels, rail: impl IntoElement) -> Self {
        self.rail = Some((width, rail.into_any_element()));
        self
    }

    /// A passive layer over the workspace beside the rail.
    pub fn workspace_decoration(mut self, layer: impl IntoElement) -> Self {
        self.workspace_decoration = Some(layer.into_any_element());
        self
    }

    /// Navigation: outline, pages, folders. None leaves no strip.
    pub fn left(mut self, view: Option<AnyView>) -> Self {
        self.left = view;
        self
    }

    /// The selection's properties. None leaves no strip.
    pub fn right(mut self, view: Option<AnyView>) -> Self {
        self.right = view;
        self
    }

    /// Document tools beneath the tabs, above both side panels.
    pub fn toolbar(mut self, bar: Option<Toolbar>) -> Self {
        self.toolbar = bar;
        self
    }

    pub fn status(mut self, bar: StatusBar) -> Self {
        self.status = Some(bar);
        self
    }

    /// The content fades in each time this key changes.
    pub fn page(mut self, key: impl Into<ElementId>) -> Self {
        self.page = Some(key.into());
        self
    }
}

#[cfg(target_os = "macos")]
fn menu_bar(_: &App) -> Option<MenuBar> {
    None
}

#[cfg(not(target_os = "macos"))]
fn menu_bar(cx: &App) -> Option<MenuBar> {
    cx.get_menus().map(|menus| MenuBar::new("menu-bar", menus))
}

impl ParentElement for Shell {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(elements);
    }
}

impl RenderOnce for Shell {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let length = motion::duration(motion::BASE, cx);
        let content = div()
            .debug_selector(|| "shell-body".into())
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .children(self.body);
        let content = match self.page {
            Some(page) => {
                let now = Instant::now();
                let state =
                    window.use_keyed_state("page-fade", cx, |_, _| Fade::new(page.clone(), now));
                let (opacity, running) = state.update(cx, |fade, _| {
                    fade.show(&page, now);
                    (fade.opacity(now, length), fade.running(now, length))
                });
                if running {
                    window.request_animation_frame();
                }
                content.opacity(opacity).into_any_element()
            }
            None => content.into_any_element(),
        };
        let header = div()
            .flex()
            .flex_col()
            .child(
                div()
                    .debug_selector(|| "shell-title".into())
                    .child(TitleBar::new("titlebar").title(self.title)),
            )
            .children(self.main.then(|| crate::updates::updater(cx)).flatten())
            .children((self.main && self.rail.is_none()).then(|| menu_bar(cx)).flatten())
            .children(self.tabs)
            .children(self.toolbar);
        let chosen = (crate::settings::panels::shown(true, cx), crate::settings::panels::shown(false, cx));
        let (left_view, right_view) = (self.left.filter(|_| chosen.0), self.right.filter(|_| chosen.1));
        // Panels give way so the content keeps its room.
        let (left, right) = (panel::fitted(Side::Left, window, cx), panel::fitted(Side::Right, window, cx));
        let rail_width = self.rail.as_ref().map_or(px(0.0), |(width, _)| *width);
        let available = window.viewport_size().width - rail_width;
        let (shown_left, shown_right) = panel::room(
            available,
            left_view.as_ref().map(|_| left),
            right_view.as_ref().map(|_| right),
        );
        let (squeezed_left, squeezed_right) = (left_view.is_some() && shown_left.is_none(), right_view.is_some() && shown_right.is_none());
        let mut drawn_left = panel::slide(Side::Left, left_view.filter(|_| shown_left.is_some()), shown_left.unwrap_or(left), squeezed_left, chosen.0, window, cx);
        let mut drawn_right = panel::slide(Side::Right, right_view.filter(|_| shown_right.is_some()), shown_right.unwrap_or(right), squeezed_right, chosen.1, window, cx);
        panel::budget(&mut drawn_left, &mut drawn_right, available, window, cx);
        let row = div().flex_1().min_h_0().flex().child(content).child(SidePanel::new(Side::Right, drawn_right));
        let panel = SidePanel::new(Side::Left, drawn_left);
        let frame = AppShell::new().title_bar(header);
        let frame = match self.workspace_decoration {
            Some(decoration) => {
                let frame = match self.rail {
                    Some((width, rail)) => frame.sidebar(div().w(width).h_full().flex_none().child(rail)),
                    None => frame,
                };
                frame.child(div().relative().flex().flex_1().min_w_0().min_h_0()
                    .child(panel).child(row).child(decoration))
            }
            None => {
                let sidebar = match self.rail {
                    Some((width, rail)) => div().flex().h_full()
                        .child(div().w(width).h_full().flex_none().child(rail))
                        .child(panel).into_any_element(),
                    None => panel.into_any_element(),
                };
                frame.sidebar(sidebar).child(row)
            }
        };
        let frame = match self.status {
            Some(bar) => frame.status_bar(bar),
            None => frame,
        };
        let theme = cx.theme();
        ResizeBorder::new().child(
            FocusScope::new(&self.focus)
                .root()
                .size_full()
                .font_family(theme.font_family.clone())
                .text_size(theme.text_size(TextSize::Base))
                .child(frame),
        )
    }
}
