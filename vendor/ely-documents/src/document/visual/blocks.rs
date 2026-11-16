use super::*;

impl MarkdownDocument {
    /// A block, or the end.
    pub(super) fn block_item(
        &mut self,
        ix: usize,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let editing = self.mode == MarkdownMode::Visual;
        let drawn = editing && ix < self.parsed.blocks.len();
        let on_point = drawn.then(|| self.on_point(ix, cx));
        let tasks = self.task_hooks(ix, cx);
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let (radius, body, mono) = (
            theme.radius(Radius::Md),
            theme.text_size(TextSize::Base) * self.look.scale,
            theme.mono_family.clone(),
        );
        // Focus fades all but the reader's block.
        let faded = self.look.focus && editing_mode(self.mode) && ix != self.current_block(cx);
        let open = self.open.as_ref();
        let Some(block) = self.parsed.blocks.get(ix) else {
            return match (open, editing) {
                (Some(open), _) if open.append => self.field(open, None, cx),
                (_, true) => self
                    .pressable(div().id("more").min_h_8(), None, cx)
                    .into_any_element(),
                (_, false) => div().into_any_element(),
            };
        };
        if let Some(open) = open.filter(|open| open.cell.is_none() && !open.append) {
            let range = &open.range;
            if range.start == block.range.start {
                return self.field(open, Some(block), cx);
            }
            // Shown in the field above.
            if range.start < block.range.start && block.range.end <= range.end {
                return div().into_any_element();
            }
        }
        // A cell open in this block, typed in place.
        let cell = open.and_then(|open| {
            let inside = block.range.start <= open.range.start && open.range.end <= block.range.end;
            Some(OpenCell {
                at: open.cell.filter(|_| inside)?,
                field: std::cell::RefCell::new(Some(
                    div()
                        .debug_selector(|| "markdown-open-cell".to_string())
                        .on_mouse_down(MouseButton::Right, Self::point_in_open(cx))
                        .rounded(radius)
                        .bg(colors.sunken)
                        .child(open.input.clone())
                        .into_any_element(),
                )),
            })
        });
        let shown = match &block.nodes {
            Some(nodes) => div()
                .flex()
                .flex_col()
                .gap_3()
                .text_size(body)
                .line_height(body * LINE)
                .text_color(colors.fg)
                .children(draw_nodes(
                    &("render", ix).into(),
                    nodes,
                    Hooks {
                        color_marks: self.look.color_marks,
                        base: &self.base,
                        pictures: &None,
                        on_link: &self.on_link,
                        on_heading: &self.on_heading,
                        on_point: &on_point,
                        cell: cell.as_ref(),
                        tasks: &tasks,
                    },
                    self.look.scale,
                    cx,
                )),
            None => {
                let raw = &self.parsed.text[block.range.clone()];
                let drawn = StyledText::new(raw.to_string());
                let layout = drawn.layout().clone();
                let shown = div()
                    .text_size(body)
                    .font_family(mono)
                    .text_color(colors.fg_muted)
                    .child(drawn)
                    .into_any_element();
                div().child(point_lines(
                    shown,
                    layout,
                    raw,
                    0,
                    &on_point,
                    (false, false),
                ))
            }
        };
        let block = div()
            .id(("block", ix))
            .debug_selector(move || format!("markdown-block-{ix}"))
            .px_3()
            .py_1()
            .rounded(radius)
            .when(faded, |block| block.opacity(FADED))
            .child(shown);
        match editing {
            true => self.pressable(block, Some(ix), cx).into_any_element(),
            false => block.into_any_element(),
        }
    }
}
