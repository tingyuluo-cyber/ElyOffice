use gpui::TestAppContext;

use super::completed_inline;
use crate::{
    render::style,
    theme::{ActiveTheme, Mode, Theme},
    tree::{Inline, Node, parse},
};

fn paragraph(source: &str) -> Inline {
    let [Node::Paragraph(inline)] = &parse(source)[..] else {
        panic!("a paragraph")
    };
    inline.clone()
}

#[gpui::test]
fn completed_text_matches_explicit_markdown_strike_in_both_themes(cx: &mut TestAppContext) {
    cx.update(crate::init_for_tests);
    let plain = paragraph("中文 **bold** *italic* `code` [link](https://example.com)");
    let explicit = paragraph("~~中文 **bold** *italic* `code` [link](https://example.com)~~");
    let completed = completed_inline(&plain, true);
    assert_eq!(completed.text, explicit.text);
    assert_eq!(completed.links, plain.links);
    assert_eq!(completed.sources, plain.sources);
    for mode in [Mode::Light, Mode::Dark] {
        cx.update(|cx| {
            Theme::set_mode_now(mode, cx);
            let theme = cx.theme();
            let styles = |inline: &Inline| {
                crate::editor::stack(style::highlights(
                    inline,
                    &theme.colors,
                    theme.underline_thickness(),
                    true,
                ))
            };
            assert_eq!(styles(&completed), styles(&explicit));
        });
    }
}

#[test]
fn unchecking_removes_completion_styling_but_keeps_explicit_strikes() {
    let original = paragraph("open ~~already struck~~ text");
    let before = original.clone();
    let completed = completed_inline(&original, true);
    assert!(completed.styles.iter().any(|(range, style)| {
        *range == (0..original.text.len()) && *style == crate::tree::Style::Strike
    }));
    assert_eq!(completed_inline(&original, false).as_ref(), &before);
    assert_eq!(
        original, before,
        "rendering never mutates the parsed source"
    );
}
