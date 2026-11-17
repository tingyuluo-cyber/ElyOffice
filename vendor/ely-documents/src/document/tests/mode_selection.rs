use gpui::{Entity, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};

use super::{MarkdownDocument, MarkdownMode, frames, open, text};

fn switch(document: &Entity<MarkdownDocument>, mode: MarkdownMode, cx: &mut VisualTestContext) {
    document.update_in(cx, |document, window, cx| {
        document.set_mode(mode, window, cx)
    });
    frames(cx);
}

fn selected(document: &Entity<MarkdownDocument>, cx: &mut VisualTestContext) -> String {
    document.read_with(cx, |document, cx| document.selected(cx))
}

#[gpui::test]
fn source_selection_reveals_and_selects_the_same_preview_text(cx: &mut TestAppContext) {
    let source = (0..200)
        .map(|ix| format!("Paragraph {ix}: **中文** tail\n\n"))
        .collect::<String>();
    let (document, cx) = open(&source, MarkdownMode::Source, cx);
    let at = source.find("Paragraph 150:").unwrap() + "Paragraph 150: **".len();
    document.update(cx, |document, cx| {
        document.editor.update(cx, |editor, cx| {
            editor.select(std::iter::once(at..at + "中文".len()), cx)
        });
    });
    switch(&document, MarkdownMode::Preview, cx);
    assert_eq!(selected(&document, cx), "中文");
    assert!(
        cx.debug_bounds("markdown-block-150").is_some(),
        "the selected paragraph is drawn"
    );
    switch(&document, MarkdownMode::Source, cx);
    assert_eq!(selected(&document, cx), "中文");
    assert_eq!(text(&document, cx), source);
}

#[gpui::test]
fn preview_drag_maps_both_directions_and_a_plain_click_to_source(cx: &mut TestAppContext) {
    let source = "**First 中文**\n\nSecond *text*";
    let (document, cx) = open(source, MarkdownMode::Preview, cx);
    let first = cx.debug_bounds("markdown-block-0").expect("drawn");
    let last = cx.debug_bounds("markdown-block-1").expect("drawn");
    let start = point(first.left() + px(13.), first.top() + px(8.));
    let end = point(last.right() - px(14.), last.top() + px(8.));
    for (anchor, head) in [(start, end), (end, start)] {
        cx.simulate_mouse_down(anchor, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(head, Some(MouseButton::Left), Modifiers::none());
        cx.simulate_mouse_up(head, MouseButton::Left, Modifiers::none());
        switch(&document, MarkdownMode::Source, cx);
        assert_eq!(selected(&document, cx), "First 中文**\n\nSecond *text");
        let picked = document.read_with(cx, |document, cx| document.editor.read(cx).primary());
        assert_eq!(
            picked.head < picked.anchor,
            anchor == end,
            "selection direction survives"
        );
        switch(&document, MarkdownMode::Preview, cx);
        assert_eq!(selected(&document, cx), "First 中文\nSecond text");
    }
    cx.simulate_click(start, Modifiers::none());
    switch(&document, MarkdownMode::Source, cx);
    assert_eq!(selected(&document, cx), "");
    assert_eq!(
        document.read_with(cx, |document, cx| document.editor.read(cx).primary().head),
        2
    );
    assert_eq!(text(&document, cx), source);
}

#[gpui::test]
fn formatted_nested_code_and_table_selections_round_trip(cx: &mut TestAppContext) {
    let source = "---\r\ntitle: 中文\r\n---\r\n\r\n- [ ] **任务**\r\n  - nested &amp; `code`\r\n\r\n> quoted *文字*\r\n\r\n| A | B |\r\n| - | - |\r\n| cell | 中文表格 |\r\n\r\n```rust\r\nlet 代码 = 1;\r\n```\r\n\r\nBefore $x^2$ after\r\n";
    let (document, cx) = open(source, MarkdownMode::Source, cx);
    for word in [
        "中文",
        "任务",
        "nested",
        "code",
        "文字",
        "中文表格",
        "代码",
        "x^2",
    ] {
        let at = source.find(word).unwrap();
        document.update(cx, |document, cx| {
            document.editor.update(cx, |editor, cx| {
                editor.select(std::iter::once(at..at + word.len()), cx)
            });
        });
        switch(&document, MarkdownMode::Preview, cx);
        let preview = match word {
            "任务" => "[ ] 任务",
            "nested" => "  ◦ nested",
            word => word,
        };
        assert_eq!(selected(&document, cx), preview, "preview {word}");
        switch(&document, MarkdownMode::Source, cx);
        assert_eq!(selected(&document, cx), word, "source {word}");
    }
    assert!(!document.read_with(cx, |document, cx| document.editor.read(cx).can_undo()));
    assert_eq!(text(&document, cx), source);
}

#[gpui::test]
fn visual_selection_keeps_both_ends_when_its_block_is_committed(cx: &mut TestAppContext) {
    let source = "Before **中文** after";
    let (document, cx) = open(source, MarkdownMode::Visual, cx);
    let at = source.find("中文").unwrap();
    document.update_in(cx, |document, window, cx| {
        document.show_match(at..at + "中文".len(), window, cx)
    });
    assert_eq!(selected(&document, cx), "中文");
    switch(&document, MarkdownMode::Preview, cx);
    assert_eq!(selected(&document, cx), "中文");
    switch(&document, MarkdownMode::Visual, cx);
    assert_eq!(selected(&document, cx), "中文");
    assert_eq!(text(&document, cx), source);
}

#[gpui::test]
fn selection_waits_for_a_long_parse_and_drops_when_the_text_changes(cx: &mut TestAppContext) {
    let source = "Paragraph **中文**\n\n".repeat(super::AT_ONCE / 18 + 1);
    let (document, cx) = open(&source, MarkdownMode::Source, cx);
    let at = source.rfind("中文").unwrap();
    document.update(cx, |document, cx| {
        document.editor.update(cx, |editor, cx| {
            editor.edit([(at..at, "新增".to_owned())], cx);
        });
    });
    document.update(cx, |document, cx| {
        document.editor.update(cx, |editor, cx| {
            let at = at + "新增".len();
            editor.select(std::iter::once(at..at + "中文".len()), cx)
        });
    });
    switch(&document, MarkdownMode::Preview, cx);
    assert_eq!(selected(&document, cx), "中文");
    document.update(cx, |document, cx| document.set_text("Replacement", cx));
    frames(cx);
    assert_eq!(selected(&document, cx), "");
}

#[gpui::test]
fn switching_without_a_selection_keeps_the_scroll_and_source_caret(cx: &mut TestAppContext) {
    let source = (0..100)
        .map(|ix| format!("# Heading {ix}\n\n"))
        .collect::<String>();
    let (document, cx) = open(&source, MarkdownMode::Source, cx);
    document.update_in(cx, |document, window, cx| {
        document.set_top_line(40, window, cx)
    });
    frames(cx);
    switch(&document, MarkdownMode::Preview, cx);
    switch(&document, MarkdownMode::Source, cx);
    assert_eq!(
        document.read_with(cx, |document, cx| document.caret(cx)),
        (0, 0)
    );
    assert_eq!(
        document.read_with(cx, |document, cx| document.top_line(cx)),
        40
    );
    assert_eq!(selected(&document, cx), "");
}

#[gpui::test]
fn switching_reveals_selected_text_inside_one_soft_wrapped_line(cx: &mut TestAppContext) {
    use gpui::{Bounds, EntityInputHandler as _, size};
    let source = format!("{}**Target** tail", "Long paragraph words ".repeat(500));
    let (document, cx) = open(&source, MarkdownMode::Source, cx);
    cx.simulate_resize(size(px(400.), px(300.)));
    frames(cx);
    let at = source.find("Target").unwrap();
    document.update(cx, |document, cx| {
        document.editor.update(cx, |editor, cx| {
            editor.select(std::iter::once(at..at + 6), cx)
        });
    });
    switch(&document, MarkdownMode::Preview, cx);
    document.read_with(cx, |document, cx| {
        let selection = document.preview.as_ref().expect("preview").state.read(cx);
        let (anchor, head) = selection.ends().expect("selected");
        let position = selection
            .position(anchor.min(head))
            .expect("drawn selection");
        assert!(
            document.list.viewport_bounds().contains(&position),
            "preview selection is visible: {position:?}"
        );
    });
    switch(&document, MarkdownMode::Source, cx);
    document.update_in(cx, |document, window, cx| {
        document.editor.update(cx, |editor, cx| {
            let bounds = editor
                .bounds_for_range(at..at, Bounds::default(), window, cx)
                .expect("source selection");
            assert!(
                editor.scroll_handle().bounds().contains(&bounds.origin),
                "source selection is visible: {bounds:?}"
            );
        });
    });
}
