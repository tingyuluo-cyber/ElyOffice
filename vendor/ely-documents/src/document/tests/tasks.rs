use gpui::{Entity, Modifiers, MouseButton, TestAppContext, VisualTestContext};

use super::{MarkdownDocument, MarkdownMode, open, text};

fn click(at: usize, cx: &mut VisualTestContext) {
    let selector = Box::leak(format!("markdown-task-toggle-{at}").into_boxed_str());
    let bounds = cx.debug_bounds(selector).expect("a task checkbox");
    cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

fn mode(document: &Entity<MarkdownDocument>, mode: MarkdownMode, cx: &mut VisualTestContext) {
    document.update_in(cx, |document, window, cx| {
        document.set_mode(mode, window, cx)
    });
    cx.run_until_parked();
}

#[gpui::test]
fn preview_checks_and_unchecks_only_the_marker_and_shares_undo(cx: &mut TestAppContext) {
    let original = "# 清单\r\n\r\n- [ ] 有必要调查一下目前不在项目中调用的依赖项\r\n";
    let (document, cx) = open(original, MarkdownMode::Preview, cx);
    let at = original.find("[ ]").expect("the marker") + 1;
    click(at, cx);
    let checked = original.replacen("[ ]", "[x]", 1);
    assert_eq!(text(&document, cx), checked);
    assert!(document.read_with(cx, |document, _| document.open.is_none()));
    click(at, cx);
    assert_eq!(text(&document, cx), original);
    mode(&document, MarkdownMode::Source, cx);
    cx.simulate_keystrokes("secondary-z");
    assert_eq!(text(&document, cx), checked, "each toggle is one undo step");
    cx.simulate_keystrokes("secondary-z");
    assert_eq!(text(&document, cx), original);
    cx.simulate_keystrokes("secondary-shift-z");
    assert_eq!(text(&document, cx), checked);
    mode(&document, MarkdownMode::Preview, cx);
    click(at, cx);
    assert_eq!(text(&document, cx), original);
}

#[gpui::test]
fn nested_quoted_numbered_and_empty_tasks_target_their_own_markers(cx: &mut TestAppContext) {
    let original = "- [ ] same\n  - [X] same\n- [ ]\n\n> 1. [ ] same\n> 2. [x] `same`\n\n-\n  [ ] next line\n\n```md\n- [ ] code\n```\n\nText [ ] literal.\n";
    let (document, cx) = open(original, MarkdownMode::Preview, cx);
    let expected = super::super::parse::parsed(original);
    let markers: Vec<_> = expected
        .blocks
        .iter()
        .flat_map(|block| block.tasks.clone())
        .collect();
    assert_eq!(markers.len(), 6, "only parsed task boxes");
    let mut changed = original.to_string();
    for at in markers {
        click(at, cx);
        let done = &changed[at..at + 1] == " ";
        changed.replace_range(at..at + 1, if done { "x" } else { " " });
        assert_eq!(text(&document, cx), changed, "marker at {at}");
    }
}

#[gpui::test]
fn preview_task_can_be_toggled_from_the_keyboard(cx: &mut TestAppContext) {
    let original = "- [ ] Keyboard\n";
    let (document, cx) = open(original, MarkdownMode::Preview, cx);
    cx.update(|window, cx| {
        let focus = document.read(cx).focus.clone();
        window.focus(&focus, cx);
        window.focus_next(cx);
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("space");
    cx.simulate_event(gpui::KeyUpEvent {
        keystroke: gpui::Keystroke::parse("space").expect("space"),
    });
    cx.run_until_parked();
    assert_eq!(text(&document, cx), "- [x] Keyboard\n");
    cx.simulate_keystrokes("enter");
    cx.simulate_event(gpui::KeyUpEvent {
        keystroke: gpui::Keystroke::parse("enter").expect("enter"),
    });
    cx.run_until_parked();
    assert_eq!(text(&document, cx), original, "focus survives the redraw");
}

#[gpui::test]
fn a_stale_task_callback_cannot_change_replaced_text_or_another_mode(cx: &mut TestAppContext) {
    let original = "- [ ] Task\n";
    let (document, cx) = open(original, MarkdownMode::Preview, cx);
    let hooks = document.update(cx, |document, cx| {
        document.task_hooks(0, cx).expect("tasks")
    });
    mode(&document, MarkdownMode::Visual, cx);
    cx.update(|window, cx| (hooks.toggle)(3, true, window, cx));
    assert_eq!(text(&document, cx), original);
    mode(&document, MarkdownMode::Preview, cx);
    let hooks = document.update(cx, |document, cx| {
        document.task_hooks(0, cx).expect("tasks")
    });
    document.update(cx, |document, cx| document.set_text("- [ ] Other\n", cx));
    cx.update(|window, cx| (hooks.toggle)(3, true, window, cx));
    assert_eq!(text(&document, cx), "- [ ] Other\n");
    let hooks = document.update(cx, |document, cx| {
        document.task_hooks(0, cx).expect("tasks")
    });
    let long = format!(
        "- [ ] Other\n\n{}",
        "prose\n\n".repeat(super::AT_ONCE / 7 + 1)
    );
    document.update(cx, |document, cx| document.set_text(long.clone(), cx));
    cx.update(|window, cx| (hooks.toggle)(3, true, window, cx));
    assert_eq!(
        text(&document, cx),
        long,
        "a pending parse keeps old callbacks inert"
    );
}

#[gpui::test]
fn task_targets_follow_edits_and_body_clicks_stay_read_only(cx: &mut TestAppContext) {
    let (document, cx) = open("Before\n\n- [ ] Task\n", MarkdownMode::Preview, cx);
    super::press(1, Modifiers::none(), cx);
    assert_eq!(text(&document, cx), "Before\n\n- [ ] Task\n");
    let updated = "A longer introduction\n\n- [ ] Task\n";
    document.update(cx, |document, cx| document.set_text(updated, cx));
    cx.run_until_parked();
    click(updated.find("[ ]").expect("task") + 1, cx);
    assert_eq!(text(&document, cx), updated.replace("[ ]", "[x]"));
}
