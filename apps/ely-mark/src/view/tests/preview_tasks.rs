use gpui::{Modifiers, MouseButton, TestAppContext};

use super::{Mode, current, open};

#[gpui::test]
fn preview_selection_copies_rendered_text_without_dirtying_the_file(cx: &mut TestAppContext) {
    let original = "**Selected** 文本\n\nSecond paragraph\n";
    let (view, cx) = open(original, cx);
    view.update_in(cx, |view, window, cx| {
        view.switch(Mode::Preview, window, cx)
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("secondary-a secondary-c");
    assert_eq!(
        cx.read_from_clipboard()
            .and_then(|item| item.text())
            .as_deref(),
        Some("Selected 文本\nSecond paragraph")
    );
    view.read_with(cx, |view, cx| {
        assert!(!view.modified());
        assert_eq!(view.document.read(cx).text(cx), original);
        assert_eq!(view.mode, Mode::Preview);
    });
    assert_eq!(current(&view, cx), original);
}

#[gpui::test]
fn preview_task_changes_reach_the_saved_text_and_dirty_state(cx: &mut TestAppContext) {
    let original = "\u{feff}- [ ] 调查依赖项\r\n";
    let (view, cx) = open(original, cx);
    view.update_in(cx, |view, window, cx| {
        view.switch(Mode::Preview, window, cx)
    });
    cx.run_until_parked();
    let at = current(&view, cx).find("[ ]").expect("a task") + 1;
    let selector = Box::leak(format!("markdown-task-toggle-{at}").into_boxed_str());
    let bounds = cx.debug_bounds(selector).expect("the checkbox");
    cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(current(&view, cx), "- [x] 调查依赖项\n");
    view.read_with(cx, |view, _| {
        assert!(view.modified() && view.changed() > 0);
        let doc = view.doc.lock().expect("text");
        assert_eq!(
            doc.source.encode(&doc.current),
            "\u{feff}- [x] 调查依赖项\r\n".as_bytes()
        );
        assert_eq!(view.mode, Mode::Preview);
    });
}
