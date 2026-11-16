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
