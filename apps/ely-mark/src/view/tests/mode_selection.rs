use gpui::{Modifiers, TestAppContext, VisualTestContext, px, size};

use crate::{desk, view::Mode};

fn click(selector: &'static str, cx: &mut VisualTestContext) {
    let bounds = cx.debug_bounds(selector).expect("mode control");
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
}

#[gpui::test]
fn reimu_mode_buttons_keep_the_selected_text(cx: &mut TestAppContext) {
    let data = tempfile::tempdir().expect("profile");
    let path = data.path().join("selection.md");
    let source = "# 标题\n\nBefore **选中文字** after\n";
    std::fs::write(&path, source).expect("note");
    let (desk, cx) = desk::tests::start(cx, data.path());
    cx.simulate_resize(size(px(1000.), px(720.)));
    desk.update(cx, |desk, cx| {
        desk.reimu = true;
        cx.notify();
    });
    desk::tests::open(&desk, &path, cx);
    let view = desk.read_with(cx, |desk, _| desk.active_view().expect("note"));
    let document = view.read_with(cx, |view, _| view.document().clone());
    let at = source.find("选中文字").expect("selected text");
    document.update_in(cx, |document, window, cx| {
        document.show_match(at..at + "选中文字".len(), window, cx);
    });
    cx.run_until_parked();
    for (button, mode) in [
        ("toggle preview", Mode::Preview),
        ("toggle source", Mode::Source),
        ("toggle visual", Mode::Visual),
    ] {
        click(button, cx);
        assert_eq!(view.read_with(cx, |view, _| view.mode), mode);
        assert_eq!(
            document.read_with(cx, |document, cx| document.selected(cx)),
            "选中文字"
        );
    }
    assert!(!view.read_with(cx, |view, _| view.modified()));
    assert_eq!(std::fs::read_to_string(path).expect("saved note"), source);
}
