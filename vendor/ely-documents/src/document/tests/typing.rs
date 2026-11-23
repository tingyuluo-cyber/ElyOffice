use gpui::{Focusable as _, Modifiers, TestAppContext};

use super::{open, press, text};
use crate::MarkdownMode;

#[gpui::test]
fn repeated_stars_wrap_the_same_selection_in_both_editing_modes(cx: &mut TestAppContext) {
    let (document, cx) = open("中文 and words", MarkdownMode::Source, cx);
    for mode in [MarkdownMode::Source, MarkdownMode::Visual] {
        document.update_in(cx, |document, window, cx| {
            document.set_text("中文 and words", cx);
            document.set_mode(mode, window, cx);
        });
        if mode == MarkdownMode::Visual {
            cx.run_until_parked();
            press(0, Modifiers::none(), cx);
            document.update(cx, |document, cx| {
                document
                    .open
                    .as_ref()
                    .expect("open")
                    .input
                    .update(cx, |input, cx| input.select(0..6, cx));
            });
        } else {
            let editor = document.read_with(cx, |document, _| document.editor().clone());
            editor.update(cx, |editor, cx| editor.select(std::iter::once(0..6), cx));
            let focus = editor.read_with(cx, |editor, cx| editor.focus_handle(cx));
            cx.update(|window, cx| window.focus(&focus, cx));
        }
        for count in 1..=3 {
            cx.simulate_keystrokes("shift-8->*");
            cx.run_until_parked();
            let stars = "*".repeat(count);
            assert_eq!(text(&document, cx), format!("{stars}中文{stars} and words"));
        }
    }
}

#[gpui::test]
fn selected_stars_and_bare_carets_type_together(cx: &mut TestAppContext) {
    let (document, cx) = open("猫和狗", MarkdownMode::Source, cx);
    let editor = document.read_with(cx, |document, _| document.editor().clone());
    editor.update(cx, |editor, cx| editor.select([0..3, 6..9, 9..9], cx));
    let focus = editor.read_with(cx, |editor, cx| editor.focus_handle(cx));
    cx.update(|window, cx| window.focus(&focus, cx));
    cx.simulate_keystrokes("shift-8->*");
    assert_eq!(text(&document, cx), "*猫*和*狗**");
    cx.simulate_keystrokes("shift-8->*");
    assert_eq!(text(&document, cx), "**猫**和**狗****");
}

#[gpui::test]
fn source_backticks_preserve_selected_crlf_and_inner_fences(cx: &mut TestAppContext) {
    let body = "甲\r\n```\r\n乙";
    let original = format!("前 {body} 后\r\n");
    let (document, cx) = open(&original, MarkdownMode::Source, cx);
    let editor = document.read_with(cx, |document, _| document.editor().clone());
    let start = "前 ".len();
    editor.update(cx, |editor, cx| {
        editor.select(
            [original.len()..original.len(), start..start + body.len()],
            cx,
        );
    });
    let focus = editor.read_with(cx, |editor, cx| editor.focus_handle(cx));
    cx.update(|window, cx| window.focus(&focus, cx));
    for count in 1..=2 {
        cx.simulate_keystrokes("`->`");
        assert_eq!(
            document.read_with(cx, |document, cx| document.selected(cx)),
            body
        );
        let ticks = "`".repeat(count);
        assert_eq!(
            text(&document, cx),
            format!("前 {ticks}{body}{ticks} 后\r\n{ticks}")
        );
    }
    cx.simulate_keystrokes("`->`");
    assert_eq!(
        text(&document, cx),
        format!("前 \r\n````\r\n{body}\r\n````\r\n 后\r\n```")
    );
}

#[gpui::test]
fn moving_to_another_selection_starts_a_new_backtick_pair(cx: &mut TestAppContext) {
    let (document, cx) = open("alpha beta", MarkdownMode::Source, cx);
    let editor = document.read_with(cx, |document, _| document.editor().clone());
    let focus = editor.read_with(cx, |editor, cx| editor.focus_handle(cx));
    cx.update(|window, cx| window.focus(&focus, cx));
    editor.update(cx, |editor, cx| editor.select(std::iter::once(0..5), cx));
    cx.simulate_keystrokes("`->` `->`");
    editor.update(cx, |editor, cx| editor.select(std::iter::once(10..14), cx));
    cx.simulate_keystrokes("`->`");
    assert_eq!(text(&document, cx), "``alpha`` `beta`");
}
