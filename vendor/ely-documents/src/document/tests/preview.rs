use gpui::{ClipboardItem, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};

use super::{frames, open as open_document, text};
use crate::MarkdownMode;

fn open<'a>(
    text: &str,
    mode: MarkdownMode,
    cx: &'a mut TestAppContext,
) -> (
    gpui::Entity<super::MarkdownDocument>,
    &'a mut VisualTestContext,
) {
    let (document, cx) = open_document(text, mode, cx);
    document.update_in(cx, |document, window, cx| window.focus(&document.focus, cx));
    (document, cx)
}

fn bounds(cx: &mut VisualTestContext, ix: usize) -> gpui::Bounds<gpui::Pixels> {
    cx.debug_bounds(Box::leak(format!("markdown-block-{ix}").into_boxed_str()))
        .expect("drawn")
}

fn copy(cx: &mut VisualTestContext) -> String {
    cx.write_to_clipboard(ClipboardItem::new_string("before".into()));
    cx.simulate_keystrokes("secondary-c");
    cx.read_from_clipboard()
        .and_then(|item| item.text())
        .expect("clipboard text")
}

fn drag(
    cx: &mut VisualTestContext,
    from: gpui::Point<gpui::Pixels>,
    to: gpui::Point<gpui::Pixels>,
) {
    cx.simulate_mouse_down(from, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(to, Some(MouseButton::Left), Modifiers::none());
    cx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
    frames(cx);
}

#[gpui::test]
fn preview_drag_copies_rendered_text_in_both_directions_without_editing(cx: &mut TestAppContext) {
    let source = "**One** two\n\n三 *four*\n";
    let (document, cx) = open(source, MarkdownMode::Preview, cx);
    let first = bounds(cx, 0);
    let last = bounds(cx, 1);
    let start = point(first.left() + px(13.0), first.top() + px(8.0));
    let end = point(last.right() - px(14.0), last.top() + px(8.0));
    drag(cx, start, end);
    assert_eq!(copy(cx), "One two\n三 four");
    assert_eq!(
        document.read_with(cx, |document, cx| document.selected(cx)),
        "One two\n三 four"
    );
    drag(cx, end, start);
    assert_eq!(copy(cx), "One two\n三 four");
    cx.simulate_input("replacement");
    cx.simulate_keystrokes("backspace delete secondary-v secondary-x");
    assert_eq!(text(&document, cx), source);
    assert!(document.read_with(cx, |document, _| document.open.is_none()));
}

#[gpui::test]
fn preview_select_all_includes_offscreen_blocks_and_survives_scroll(cx: &mut TestAppContext) {
    let source = (0..200)
        .map(|ix| format!("Paragraph {ix}\n\n"))
        .collect::<String>();
    let (document, cx) = open(&source, MarkdownMode::Preview, cx);
    let expected = (0..200)
        .map(|ix| format!("Paragraph {ix}"))
        .collect::<Vec<_>>()
        .join("\n");
    cx.simulate_keystrokes("secondary-a");
    assert_eq!(copy(cx), expected);
    document.update_in(cx, |document, window, cx| {
        document.set_top_line(350, window, cx)
    });
    frames(cx);
    assert_eq!(copy(cx), expected);
    let visible = document.read_with(cx, |document, _| document.list.logical_scroll_top().item_ix);
    assert!(visible > 100);
    let block = bounds(cx, visible + 1);
    let start = point(block.left() + px(13.0), block.top() + px(8.0));
    let end = point(block.right() - px(14.0), block.top() + px(8.0));
    drag(cx, start, end);
    assert_eq!(copy(cx), format!("Paragraph {}", visible + 1));
}

#[gpui::test]
fn preview_copies_lists_tables_code_frontmatter_and_inline_math(cx: &mut TestAppContext) {
    let source = "---\ntitle: Note\n---\n\n# 标题\n\n- first\n  - nested\n- [x] done\n\n| A | B |\n| - | - |\n| 一 | two |\n\n```rust\nlet x = 1;\n```\n\nBefore $x^2$ after\n\n[site]: https://example.com\n";
    let (_, cx) = open(source, MarkdownMode::Preview, cx);
    cx.simulate_keystrokes("secondary-a");
    assert_eq!(
        copy(cx),
        "title: Note\n标题\n• first\n  ◦ nested\n[x] done\nA\nB\n一\ntwo\nlet x = 1;\nBefore x^2 after\n[site]: https://example.com"
    );
}

#[gpui::test]
fn preview_selection_survives_modes_and_clears_on_document_change(cx: &mut TestAppContext) {
    let (document, cx) = open("Old **text**", MarkdownMode::Preview, cx);
    cx.simulate_keystrokes("secondary-a");
    assert_eq!(copy(cx), "Old text");
    document.update_in(cx, |document, window, cx| {
        document.set_mode(MarkdownMode::Source, window, cx)
    });
    document.update_in(cx, |document, window, cx| {
        document.set_mode(MarkdownMode::Preview, window, cx)
    });
    frames(cx);
    assert_eq!(copy(cx), "Old text");
    cx.simulate_keystrokes("secondary-a");
    document.update(cx, |document, cx| document.set_text("New 文本", cx));
    frames(cx);
    assert_eq!(copy(cx), "before");
    cx.simulate_keystrokes("secondary-a");
    assert_eq!(copy(cx), "New 文本");
}

#[gpui::test]
fn preview_shift_click_extends_and_right_click_preserves_selection(cx: &mut TestAppContext) {
    let (_, cx) = open("First\n\nSecond", MarkdownMode::Preview, cx);
    let first = bounds(cx, 0);
    let last = bounds(cx, 1);
    cx.simulate_click(
        point(first.left() + px(13.0), first.top() + px(8.0)),
        Modifiers::none(),
    );
    cx.simulate_click(
        point(last.right() - px(14.0), last.top() + px(8.0)),
        Modifiers {
            shift: true,
            ..Modifiers::none()
        },
    );
    assert_eq!(copy(cx), "First\nSecond");
    cx.simulate_mouse_down(last.center(), MouseButton::Right, Modifiers::none());
    cx.simulate_mouse_up(last.center(), MouseButton::Right, Modifiers::none());
    assert_eq!(copy(cx), "First\nSecond");
    cx.simulate_click(last.center(), Modifiers::none());
    assert_eq!(copy(cx), "before");
}

#[gpui::test]
fn preview_drags_through_inline_math_without_copying_layout_glue(cx: &mut TestAppContext) {
    let (_, cx) = open("Before $x^2$ 中文 after", MarkdownMode::Preview, cx);
    let block = bounds(cx, 0);
    drag(
        cx,
        point(block.left() + px(13.0), block.top() + px(8.0)),
        point(block.right() - px(14.0), block.top() + px(8.0)),
    );
    assert_eq!(copy(cx), "Before x^2 中文 after");
}

#[gpui::test]
fn preview_dragging_a_link_selects_instead_of_opening_it(cx: &mut TestAppContext) {
    let (_, cx) = open(
        "[Selected link](https://example.com)",
        MarkdownMode::Preview,
        cx,
    );
    let block = bounds(cx, 0);
    drag(
        cx,
        point(block.left() + px(13.0), block.top() + px(8.0)),
        point(block.left() + px(65.0), block.top() + px(8.0)),
    );
    let copied = copy(cx);
    assert!(
        "Selected link".starts_with(&copied) && copied.len() > 2,
        "{copied}"
    );
    assert_eq!(cx.opened_url(), None);
}
