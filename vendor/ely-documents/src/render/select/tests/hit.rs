use gpui::{Entity, StyledText};

use super::*;
use crate::render::select::{Gather, Selection, Spot};

struct Columns {
    selection: Entity<Selection>,
}

impl Render for Columns {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let gather = Gather::new(self.selection.clone(), Rc::new(|_, _, _| {}), cx);
        let mut columns = Vec::new();
        for words in [["Left top", "Left bottom"], ["Right top", "Right bottom"]] {
            let mut rows = Vec::new();
            for words in words {
                let text = StyledText::new(words);
                gather.push(words, text.layout().clone(), &[]);
                rows.push(div().child(text));
            }
            columns.push(div().w(px(150.)).flex().flex_col().gap_4().children(rows));
        }
        gather.finish(cx);
        div().flex().children(columns)
    }
}

#[gpui::test]
fn pointer_uses_the_nearest_text_in_both_columns_and_between_rows(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    let (body, cx) = cx.add_window_view(|_, cx| Columns {
        selection: cx.new(|cx| Selection::new(cx)),
    });
    cx.run_until_parked();
    body.read_with(cx, |body, cx| {
        let selection = body.selection.read(cx);
        let right = selection.texts[2].layout().expect("drawn").bounds();
        assert_eq!(
            selection.spot(right.origin + point(px(1.), px(1.))),
            Some(Spot { text: 2, at: 0 })
        );
        let first = selection.texts[0].layout().expect("drawn").bounds();
        let near_end = point(first.left() + px(1.), first.bottom() + px(1.));
        assert_eq!(
            selection.spot(near_end),
            Some(Spot {
                text: 0,
                at: "Left top".len()
            })
        );
    });
}

#[gpui::test]
fn clicking_an_exact_line_boundary_lands_on_the_next_line(cx: &mut TestAppContext) {
    cx.update(Theme::init);
    struct Lines(gpui::TextLayout);
    impl Render for Lines {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let text = StyledText::new("First\nSecond\nThird");
            self.0 = text.layout().clone();
            div().child(text)
        }
    }
    let (body, cx) = cx.add_window_view(|_, _| Lines(gpui::TextLayout::default()));
    cx.run_until_parked();
    let layout = body.read_with(cx, |body, _| body.0.clone());
    let at = layout.bounds().origin + point(px(0.), layout.line_height());
    assert_eq!(
        layout.index_for_position(at).unwrap_or_else(|at| at),
        "First\n".len()
    );
}
