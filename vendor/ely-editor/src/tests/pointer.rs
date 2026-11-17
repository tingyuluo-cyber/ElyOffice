use gpui::{Modifiers, TestAppContext, point, px, size};

use super::editor;

#[gpui::test]
fn clicks_follow_the_drawn_rows_after_scrolling_and_zoom(cx: &mut TestAppContext) {
    let source = (0..200)
        .map(|ix| format!("Line {ix} 中文\n"))
        .collect::<String>();
    let (editor, cx) = editor(&source, cx);
    editor.update(cx, |editor, cx| {
        editor.options.soft_wrap = true;
        editor.set_scale(1.37, cx);
    });
    cx.simulate_resize(size(px(480.), px(500.)));
    cx.run_until_parked();
    editor.update(cx, |editor, cx| {
        editor
            .scroll_handle()
            .set_offset(point(px(0.), px(-1013.5)));
        cx.notify();
    });
    cx.run_until_parked();
    let mut checked = 0;
    for line in 40..55 {
        let Some(bounds) =
            cx.debug_bounds(Box::leak(format!("editor-line-{line}").into_boxed_str()))
        else {
            continue;
        };
        for fraction in [0.1, 0.5, 0.9] {
            let x = editor.read_with(cx, |editor, _| editor.metrics.left + editor.metrics.advance);
            let position = point(x, bounds.top() + bounds.size.height * fraction);
            cx.simulate_click(position, Modifiers::none());
            assert_eq!(
                editor.read_with(cx, |editor, _| editor.buffer.line_of(editor.primary().head)),
                line
            );
            checked += 1;
        }
    }
    assert!(checked >= 9, "several visible rows were clicked");
}
