use ely_gpui_component::secondary;
use ely_gpui_component::theme::{ActiveTheme as _, Mode as ColorMode, Theme};
use ely_office::prelude::*;
use gpui::{
    Bounds, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px, rgb, size,
};

use super::art::Art;
use crate::{desk, view::Mode};

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx.debug_bounds(selector).expect(selector);
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
}

#[gpui::test]
fn changing_the_cover_does_not_change_the_mode_wallpaper(cx: &mut TestAppContext) {
    let data = tempfile::tempdir().expect("profile");
    let (desk, cx) = desk::tests::start(cx, data.path());
    cx.update(|_, cx| {
        ely_ui::install_theme(cx);
        super::appearance::install(cx).expect("embedded palettes");
        Theme::set_mode_now(ColorMode::Light, cx);
        assert_eq!(cx.theme().colors.bg, rgb(0xfcf8ef).into());
    });
    desk.update(cx, |desk, cx| {
        desk.reimu = true;
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("reimu-cover-reimu").is_some());
    click(cx, "reimu-art-yukari");
    assert!(cx.debug_bounds("reimu-cover-yukari").is_some());
    click(cx, "reimu-home-new");
    let paper = cx.debug_bounds("reimu-wallpaper").expect("wallpaper");
    let reimu = cx.debug_bounds("reimu-wallpaper-reimu").expect("Reimu");
    assert_eq!(reimu, paper);
    cx.update(|_, cx| {
        Theme::set_mode_now(ColorMode::Dark, cx);
        assert_eq!(cx.theme().colors.bg, rgb(0x2e3440).into());
        assert_eq!(cx.theme().colors.surface, rgb(0x3b4252).into());
    });
    cx.run_until_parked();
    let paper = cx.debug_bounds("reimu-wallpaper").expect("wallpaper");
    let yukari = cx.debug_bounds("reimu-wallpaper-yukari").expect("Yukari");
    assert_eq!(yukari, paper);
}

#[test]
fn wallpapers_cover_the_viewport_with_aligned_faces() {
    let source = size(px(1448.0), px(1086.0));
    for (width, height) in [(2560.0, 1440.0), (3440.0, 1440.0), (491.0, 1376.0)] {
        let viewport = Bounds {
            origin: point(px(48.0), px(30.0)),
            size: size(px(width), px(height)),
        };
        let eyes = |art: Art, source_x, source_y, eye_distance| {
            let bounds = art.wallpaper_bounds(viewport, source);
            let tolerance = px(0.01);
            assert!(bounds.left() <= viewport.left() + tolerance);
            assert!(bounds.right() >= viewport.right() - tolerance);
            assert!(bounds.top() <= viewport.top() + tolerance);
            assert!(bounds.bottom() >= viewport.bottom() - tolerance);
            let scale = bounds.size.height / source.height;
            let center = bounds.origin + point(px(source_x), px(source_y)) * scale;
            assert!(viewport.contains(&center));
            (center, px(eye_distance) * scale)
        };
        let (day, day_size) = eyes(Art::Reimu, 924.0, 418.0, 118.0);
        let (night, night_size) = eyes(Art::Yukari, 871.0, 257.0, 97.0);
        assert!((day.x - night.x).abs() < px(1.5));
        assert!((day.y - night.y).abs() < px(1.5));
        assert!((day_size - night_size).abs() < px(1.5));
    }
}

#[gpui::test]
fn the_reimu_controls_open_edit_modes_and_save(cx: &mut TestAppContext) {
    let data = tempfile::tempdir().expect("profile");
    let path = data.path().join("note.md");
    std::fs::write(&path, "# Before\n").expect("sample note");
    let (desk, cx) = desk::tests::start(cx, data.path());
    cx.simulate_resize(size(px(1000.0), px(720.0)));
    cx.update(|_, cx| crate::keys::bind(true, cx));
    desk.update(cx, |desk, cx| {
        desk.reimu = true;
        cx.notify();
    });
    cx.run_until_parked();

    assert!(cx.debug_bounds("reimu-cover").is_some());
    click(cx, "reimu-home-new");
    assert!(desk.read_with(cx, |desk, _| desk.active_view().is_some()));
    desk::tests::open(&desk, &path, cx);
    let view = desk.read_with(cx, |desk, _| desk.active_view().expect("opened note"));
    assert_eq!(
        view.read_with(cx, |view, cx| view.document().read(cx).look().scale),
        1.1,
    );

    click(cx, "toggle source");
    assert_eq!(view.read_with(cx, |view, _| view.mode), Mode::Source);
    view.update(cx, |view, cx| view.adopt("# After\n".into(), cx));
    click(cx, "toggle preview");
    assert_eq!(view.read_with(cx, |view, _| view.mode), Mode::Preview);
    let paper = cx.debug_bounds("reimu-wallpaper").expect("wallpaper");
    let map = cx.debug_bounds("reimu-minimap").expect("document minimap");
    assert!(map.left() >= paper.right());
    assert!(paper.top() < map.top());
    let gohei = cx.debug_bounds("reimu-gohei").expect("rail ornament");
    assert!(gohei.right() <= paper.left());
    assert!(gohei.bottom() <= paper.bottom());
    let outline = cx
        .debug_bounds("mark-outline-dock")
        .expect("bottom outline open by default");
    let files = cx.debug_bounds("segment files").expect("file control");
    assert!(outline.top() > files.bottom());
    assert!(paper.left() <= files.left() && paper.right() >= outline.right());
    assert!(paper.bottom() >= outline.bottom());
    click(cx, "mark-outline-switch");
    assert!(cx.debug_bounds("mark-outline-dock").is_none());
    click(cx, "mark-outline-switch");
    assert!(cx.debug_bounds("mark-outline-dock").is_some());
    click(cx, "toggle minimap");
    assert!(!desk.read_with(cx, |desk, _| desk.look.minimap));
    click(cx, "toggle minimap");
    click(cx, "toggle visual");
    assert_eq!(view.read_with(cx, |view, _| view.mode), Mode::Visual);

    click(cx, "reimu-save");
    assert_eq!(
        std::fs::read_to_string(path).expect("saved note"),
        "# After\n"
    );
    let first = cx.debug_bounds("side-tab-close-0").expect("first tab");
    let body = cx.debug_bounds("shell-body").expect("document area");
    let title = cx.debug_bounds("shell-title").expect("title bar");
    assert!(first.right() < body.left());
    assert_eq!(body.top(), title.bottom());
    cx.simulate_click(
        point(first.left() - px(36.0), first.center().y),
        Modifiers::none(),
    );
    cx.run_until_parked();
    assert_eq!(desk.read_with(cx, |desk, _| desk.core.active()), Some(0));
    cx.simulate_keystrokes("ctrl-tab");
    assert_eq!(desk.read_with(cx, |desk, _| desk.core.active()), Some(1));
    cx.simulate_keystrokes("ctrl-shift-tab");
    assert_eq!(desk.read_with(cx, |desk, _| desk.core.active()), Some(0));
    click(cx, "side-tab-close-1");
    assert_eq!(desk.read_with(cx, |desk, _| desk.core.tabs.len()), 1);
}

#[gpui::test]
fn reading_returns_to_the_editing_mode_and_width(cx: &mut TestAppContext) {
    let data = tempfile::tempdir().expect("profile");
    let path = data.path().join("note.md");
    std::fs::write(&path, "# Sample\n").expect("sample note");
    let (desk, cx) = desk::tests::start(cx, data.path());
    cx.update(|_, cx| {
        ely_ui::commands::bind_keys(cx);
        crate::keys::bind(true, cx);
    });
    desk.update(cx, |desk, cx| {
        desk.reimu = true;
        cx.notify();
    });
    desk::tests::open(&desk, &path, cx);
    let view = desk.read_with(cx, |desk, _| desk.active_view().expect("opened note"));
    let seen = |cx: &mut VisualTestContext| {
        view.read_with(cx, |view, cx| {
            (view.mode, view.document().read(cx).look().measure)
        })
    };
    let edit_width = seen(cx).1;
    assert!(edit_width.is_some());
    click(cx, "toggle wide-edit");
    assert_eq!(seen(cx), (Mode::Visual, None));
    cx.simulate_keystrokes(secondary!("shift-e"));
    assert_eq!(seen(cx), (Mode::Source, None));
    cx.simulate_keystrokes(secondary!("e"));
    assert_eq!(seen(cx), (Mode::Preview, edit_width));
    assert!(desk.read_with(cx, |desk, _| desk.exporting.is_none()));
    click(cx, "toggle wide-preview");
    assert_eq!(seen(cx), (Mode::Preview, None));
    cx.simulate_resize(size(px(640.0), px(420.0)));
    cx.run_until_parked();
    let wide = cx
        .debug_bounds("toggle wide-preview")
        .expect("width toggle");
    let body = cx.debug_bounds("shell-body").expect("editor");
    assert!(wide.bottom() <= body.bottom());
    cx.simulate_resize(size(px(1000.0), px(420.0)));
    cx.run_until_parked();
    let map = cx.debug_bounds("toggle minimap").expect("minimap toggle");
    let body = cx.debug_bounds("shell-body").expect("editor");
    assert!(map.bottom() <= body.bottom());
    cx.simulate_keystrokes(secondary!("e"));
    assert_eq!(seen(cx), (Mode::Source, None));
    click(cx, "toggle wide-edit");
    assert_eq!(seen(cx), (Mode::Source, edit_width));
    cx.simulate_keystrokes(secondary!("shift-e"));
    assert_eq!(seen(cx), (Mode::Visual, edit_width));
    cx.simulate_keystrokes(secondary!("e"));
    assert_eq!(seen(cx), (Mode::Preview, None));
    let saved_look = crate::look::load(&data.path().join("look.json")).expect("saved width");
    assert!(!saved_look.wide_edit);
    assert!(saved_look.wide_preview);
    assert!(!desk.read_with(cx, |desk, _| desk.active_tab().expect("note").modified));
    cx.simulate_keystrokes(ely_ui::commands::SIDE_EXPORT_KEYS);
    assert!(desk.read_with(cx, |desk, _| desk.exporting.is_some()));
}

#[gpui::test]
fn the_short_source_minimap_uses_the_whole_drag_range(cx: &mut TestAppContext) {
    let data = tempfile::tempdir().expect("profile");
    let path = data.path().join("long.md");
    std::fs::write(&path, "# Section\n\nA paragraph.\n\n".repeat(18)).expect("sample note");
    let (desk, cx) = desk::tests::start(cx, data.path());
    cx.simulate_resize(size(px(1200.0), px(720.0)));
    desk.update(cx, |desk, cx| {
        desk.reimu = true;
        cx.notify();
    });
    desk::tests::open(&desk, &path, cx);
    click(cx, "toggle source");
    let view = desk.read_with(cx, |desk, _| desk.active_view().expect("sample note"));
    let scroll = view.read_with(cx, |view, cx| {
        view.document().read(cx).editor().read(cx).scroll_handle()
    });
    assert!(scroll.max_offset().y > px(0.0));
    let progress = || -scroll.offset().y / scroll.max_offset().y;
    let before = view.read_with(cx, |view, cx| view.place(cx));
    click(cx, "reimu-minimap");
    let after = view.read_with(cx, |view, cx| view.place(cx));
    assert!((progress() - 0.5).abs() < 0.03);
    assert_eq!((after.line, after.column), (before.line, before.column));

    let map = cx.debug_bounds("reimu-minimap").expect("document minimap");
    let from = map.center();
    let view_height = scroll.bounds().size.height;
    let shown = view_height / (view_height + scroll.max_offset().y);
    let travel = (map.size.height - px(14.0)) * (1.0 - shown);
    let moved = point(from.x, from.y + px(15.0));
    let to = point(from.x, from.y + travel * 0.2);
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::none());
    assert!((progress() - 0.5).abs() < 0.03);
    cx.simulate_mouse_move(moved, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(to, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert!((progress() - 0.7).abs() < 0.03);
    cx.simulate_click(point(from.x, map.bottom() - px(1.0)), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(progress(), 1.0);
}
