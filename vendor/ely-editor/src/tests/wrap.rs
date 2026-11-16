use gpui::{Bounds, EntityInputHandler, TestAppContext, point, px, size};

use super::editor;
use crate::{cursor::Motion, layout::Row};

#[gpui::test]
fn short_wrapped_rows_keep_the_caret_at_their_own_end(cx: &mut TestAppContext) {
    let (editor, cx) = editor("abcdefghij X abcdefghijklmnop", cx);
    cx.run_until_parked();
    editor.update_in(cx, |editor, window, cx| {
        editor.options.soft_wrap = true;
        editor.frame.rows = vec![Row::Line(0); 3];
        editor.frame.segments = vec![0..10, 10..12, 12..editor.text().len()];
        editor.select(std::iter::once(8..8), cx);
        editor.motion(Motion::Down, false, cx);
        assert_eq!(editor.primary().head, 12);
        assert_eq!(editor.caret_row(12), Some(1));
        let bounds = editor
            .bounds_for_range(12..12, Bounds::default(), window, cx)
            .expect("short row");
        assert_eq!(bounds.origin.y, editor.metrics.top + editor.metrics.line);
        editor.motion(Motion::Down, false, cx);
        assert_eq!(editor.caret_row(editor.primary().head), Some(2));
        editor.motion(Motion::Up, false, cx);
        assert_eq!(editor.caret_row(editor.primary().head), Some(1));
        let position = point(
            editor.metrics.left + editor.metrics.advance * 8.,
            editor.metrics.top + editor.metrics.line * 1.5,
        );
        let at = editor
            .offset_at(position, window, cx)
            .expect("short row hit");
        assert_eq!(at, 12);
        editor.select(std::iter::once(at..at), cx);
        editor.set_upstream_at(at, position);
        assert_eq!(editor.caret_row(at), Some(1));
    });
}

#[gpui::test]
fn each_cursor_keeps_its_own_wrap_affinity(cx: &mut TestAppContext) {
    let (editor, cx) = editor("abcdefghij X long\nabcdefghij Y long", cx);
    cx.run_until_parked();
    editor.update(cx, |editor, cx| {
        editor.options.soft_wrap = true;
        editor.frame.rows = vec![
            Row::Line(0),
            Row::Line(0),
            Row::Line(0),
            Row::Line(1),
            Row::Line(1),
            Row::Line(1),
        ];
        editor.frame.segments = vec![
            0..10,
            10..12,
            12..16,
            17..27,
            27..29,
            29..editor.text().len(),
        ];
        editor.select([8..8, 25..25], cx);
        editor.motion(Motion::Down, false, cx);
        assert_eq!(editor.caret_row(12), Some(1));
        assert_eq!(editor.caret_row(29), Some(4));
        editor.select([12..12, 29..29, 2..2], cx);
        assert_eq!(editor.caret_row(12), Some(1));
        assert_eq!(editor.caret_row(29), Some(4));
    });
}

#[gpui::test]
fn wrapped_highlight_cache_never_survives_an_ime_edit(cx: &mut TestAppContext) {
    let (editor, cx) = editor("**ab**", cx);
    editor.update(cx, |editor, cx| {
        editor.options.soft_wrap = true;
        cx.notify();
    });
    cx.run_until_parked();
    editor.update_in(cx, |editor, window, cx| {
        editor
            .frame
            .colors
            .borrow_mut()
            .insert(0, vec![(4..6, Default::default())]);
        editor.select(std::iter::once(2..2), cx);
        editor.replace_and_mark_text_in_range(None, "中", None, window, cx);
        for (range, _) in editor.styles(0, cx) {
            assert!(editor.text().is_char_boundary(range.start));
            assert!(editor.text().is_char_boundary(range.end));
        }
        assert!(
            editor
                .bounds_for_range(3..3, Bounds::default(), window, cx)
                .is_some()
        );
    });
}

#[gpui::test]
fn prose_wraps_on_resize_and_zoom_without_changing_the_source(cx: &mut TestAppContext) {
    let source = "中文 words 😄 combining e\u{301} ".repeat(18);
    let (editor, cx) = editor(&source, cx);
    editor.update(cx, |editor, cx| {
        editor.options.soft_wrap = true;
        cx.notify();
    });
    cx.simulate_resize(size(px(600.), px(800.)));
    cx.run_until_parked();
    let wide = editor.read_with(cx, |editor, _| editor.frame.rows.len());
    assert!(wide > 1);
    cx.simulate_resize(size(px(320.), px(800.)));
    cx.run_until_parked();
    let narrow = editor.read_with(cx, |editor, _| editor.frame.rows.len());
    assert!(narrow > wide, "{wide} then {narrow}");
    editor.update(cx, |editor, cx| editor.set_scale(2., cx));
    cx.run_until_parked();
    editor.update_in(cx, |editor, window, cx| {
        assert!(editor.frame.rows.len() > narrow);
        let joined: String = editor
            .frame
            .segments
            .iter()
            .map(|range| &editor.text()[range.clone()])
            .collect();
        assert_eq!(joined, source);
        assert_eq!(editor.text(), source);
        let second = editor.frame.segments[1].start;
        editor.select(std::iter::once(0..0), cx);
        editor.motion(Motion::Down, false, cx);
        assert_eq!(editor.primary().head, second);
        let utf16 = source[..second].encode_utf16().count();
        let bounds = editor
            .bounds_for_range(utf16..utf16, Bounds::default(), window, cx)
            .expect("wrapped caret");
        assert!(bounds.origin.y > editor.metrics.top);
        let hit = editor.offset_at(
            point(bounds.origin.x, bounds.origin.y + editor.metrics.line / 2.),
            window,
            cx,
        );
        assert_eq!(hit, Some(second));
        editor.select(std::iter::once(0..source.len()), cx);
        editor.copy(cx);
        assert_eq!(
            cx.read_from_clipboard()
                .and_then(|item| item.text())
                .as_deref(),
            Some(source.as_str())
        );
        editor.insert("changed", false, cx);
        editor.undo(cx);
        assert_eq!(editor.text(), source);
        assert!(editor.frame.rows.iter().all(|row| *row == Row::Line(0)));
    });
}
