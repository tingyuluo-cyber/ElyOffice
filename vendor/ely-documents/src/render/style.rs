use gpui::{FontStyle, HighlightStyle, Pixels, StrikethroughStyle, UnderlineStyle};

use super::HEAVY;
use crate::{
    theme::Palette,
    tree::{Inline, Style},
};

pub(super) fn highlights(
    inline: &Inline,
    colors: &Palette,
    rule: Pixels,
    color_marks: bool,
) -> Vec<(std::ops::Range<usize>, HighlightStyle)> {
    let mut marks: Vec<_> = inline.styles.iter().collect();
    if color_marks {
        marks.sort_by_key(|(_, style)| *style == Style::Strong);
    }
    marks
        .into_iter()
        .map(|(range, style)| (range.clone(), highlight(style, colors, rule, color_marks)))
        .collect()
}

/// How one inline style draws.
pub(super) fn highlight(
    style: &Style,
    colors: &Palette,
    rule: Pixels,
    color_marks: bool,
) -> HighlightStyle {
    match style {
        Style::Strong => HighlightStyle {
            color: color_marks.then_some(colors.syntax.function),
            font_weight: Some(HEAVY),
            ..HighlightStyle::default()
        },
        Style::Emphasis => HighlightStyle {
            color: color_marks.then_some(colors.syntax.keyword),
            font_style: Some(FontStyle::Italic),
            ..HighlightStyle::default()
        },
        Style::Strike => HighlightStyle {
            strikethrough: Some(StrikethroughStyle {
                thickness: rule,
                color: Some(colors.fg_muted),
            }),
            ..HighlightStyle::default()
        },
        Style::Code | Style::Math => HighlightStyle {
            background_color: Some(colors.sunken),
            color: Some(if *style == Style::Math {
                colors.syntax.function
            } else {
                colors.fg
            }),
            ..HighlightStyle::default()
        },
        Style::Link => HighlightStyle {
            color: Some(colors.link),
            underline: Some(UnderlineStyle {
                thickness: rule,
                color: Some(colors.link.opacity(0.4)),
                wavy: false,
            }),
            ..HighlightStyle::default()
        },
        Style::Footnote => HighlightStyle {
            color: Some(colors.fg_subtle),
            ..HighlightStyle::default()
        },
        Style::Underline => HighlightStyle {
            underline: Some(UnderlineStyle {
                thickness: rule,
                color: None,
                wavy: false,
            }),
            ..HighlightStyle::default()
        },
        Style::Mark => HighlightStyle {
            background_color: Some(colors.warning_subtle),
            ..HighlightStyle::default()
        },
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use crate::theme::{ActiveTheme as _, Mode, Theme};

    #[gpui::test]
    fn marked_source_and_preview_share_distinct_colors(cx: &mut gpui::TestAppContext) {
        cx.update(crate::init_for_tests);
        for mode in [Mode::Light, Mode::Dark] {
            cx.update(|cx| {
                Theme::set_mode_now(mode, cx);
                let theme = cx.theme();
                let strong = highlight(
                    &Style::Strong,
                    &theme.colors,
                    theme.underline_thickness(),
                    true,
                );
                let italic = highlight(
                    &Style::Emphasis,
                    &theme.colors,
                    theme.underline_thickness(),
                    true,
                );
                assert_ne!(strong.color, italic.color);
                assert_eq!(strong.font_weight, Some(HEAVY));
                assert_eq!(italic.font_style, Some(FontStyle::Italic));
                let source = crate::markdown::highlights("**粗体** *斜体*", cx, true);
                assert!(
                    source.iter().any(|(_, style)| style.weight == Some(HEAVY)
                        && Some(style.color) == strong.color)
                );
                assert!(
                    source
                        .iter()
                        .any(|(_, style)| style.italic && Some(style.color) == italic.color)
                );
                assert!(
                    highlight(
                        &Style::Strong,
                        &theme.colors,
                        theme.underline_thickness(),
                        false
                    )
                    .color
                    .is_none()
                );
                for written in ["***文字***", "**外 *文字* 外**", "*外 **文字** 外*"] {
                    let source = crate::markdown::highlights(written, cx, true)
                        .into_iter()
                        .map(|(range, style)| {
                            (
                                range,
                                HighlightStyle {
                                    color: Some(style.color),
                                    ..Default::default()
                                },
                            )
                        })
                        .collect();
                    let source = crate::editor::stack(source);
                    let at = written.find("文字").expect("source text");
                    let source = source
                        .iter()
                        .find(|(range, _)| range.contains(&at))
                        .expect("source color")
                        .1
                        .color;
                    let nodes = crate::tree::parse(written);
                    let [crate::tree::Node::Paragraph(inline)] = &nodes[..] else {
                        panic!("paragraph")
                    };
                    let preview = crate::editor::stack(highlights(
                        inline,
                        &theme.colors,
                        theme.underline_thickness(),
                        true,
                    ));
                    let at = inline.text.find("文字").expect("drawn text");
                    let preview = preview
                        .iter()
                        .find(|(range, _)| range.contains(&at))
                        .expect("preview color")
                        .1
                        .color;
                    assert_eq!(source, preview, "{written}");
                    assert_eq!(source, strong.color);
                }
            });
        }
    }
}
