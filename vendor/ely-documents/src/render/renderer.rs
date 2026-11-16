use super::*;

impl RenderOnce for MarkdownRenderer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let gather = self.on_menu.clone().map(|on_menu| {
            let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| Selection::new(cx));
            Gather::new(state, on_menu, cx)
        });
        let theme = cx.theme();
        let parsed;
        let nodes: &[Node] = match &self.blocks {
            Some(blocks) => blocks,
            None => {
                parsed = parse(&self.source);
                &parsed
            }
        };
        let draw = Draw {
            id: &self.id,
            hooks: Hooks {
                color_marks: false,
                base: &self.base,
                pictures: &self.pictures,
                on_link: &self.on_link,
                on_heading: &None,
                on_point: &None,
                cell: None,
                tasks: &None,
                select: None,
            },
            next: std::cell::Cell::new(0),
            sections: &self.sections,
            headings: std::cell::Cell::new(0),
            code: &self.code,
            body: self.size,
            scale: 1.0,
            lists: std::cell::Cell::new(0),
            select: gather.as_ref(),
            table: std::cell::Cell::new(false),
            tables: std::cell::Cell::new(0),
            tasks: std::cell::Cell::new(0),
        };
        let body = div()
            .flex()
            .flex_col()
            .gap_3()
            .text_size(theme.text_size(self.size))
            .line_height(theme.text_size(self.size) * LINE)
            .children(draw.nodes(nodes, cx));
        match (gather, self.on_menu) {
            (Some(gather), Some(on_menu)) => {
                let state = gather.finish(cx);
                select::selectable(body, state, on_menu, cx)
            }
            _ => body,
        }
    }
}

/// Opens `url` through the owner, else the system.
pub(super) fn open_link(on_link: &Option<OnLink>, url: &str, window: &mut Window, cx: &mut App) {
    // The address is the document's: never logged.
    log::info!("markdown: a link opens");
    match on_link {
        Some(on_link) => on_link(url, window, cx),
        None => cx.open_url(url),
    }
}

/// Parsed `nodes` drawn under `id`, with the owner's hooks.
pub(crate) fn draw_nodes(
    id: &ElementId,
    nodes: &[Node],
    hooks: Hooks,
    scale: f32,
    cx: &App,
) -> Vec<AnyElement> {
    let select = hooks.select;
    let draw = Draw {
        id,
        hooks,
        next: std::cell::Cell::new(0),
        sections: &None,
        headings: std::cell::Cell::new(0),
        code: &None,
        body: TextSize::Base,
        scale,
        lists: std::cell::Cell::new(0),
        select,
        table: std::cell::Cell::new(false),
        tables: std::cell::Cell::new(0),
        tasks: std::cell::Cell::new(0),
    };
    draw.nodes(nodes, cx)
}
