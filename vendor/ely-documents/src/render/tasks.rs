use std::{borrow::Cow, rc::Rc};

use gpui::{AnyElement, App, InteractiveElement, IntoElement, ParentElement, Window, div};

use super::Draw;
use crate::{
    forms::Checkbox,
    tree::{Inline, Node, Style},
};

type OnTask = Rc<dyn Fn(usize, bool, &mut Window, &mut App)>;

pub(crate) struct TaskHooks {
    pub markers: Vec<usize>,
    pub toggle: OnTask,
}

impl Draw<'_> {
    pub(super) fn task_nodes(
        &self,
        task: Option<bool>,
        body: &[Node],
        cx: &App,
    ) -> Vec<AnyElement> {
        // Nested items use their own completion state.
        let previous = self
            .done
            .replace(task == Some(true) && self.hooks.tasks.is_some());
        let nodes = self.nodes(body, cx);
        self.done.set(previous);
        nodes
    }

    pub(super) fn task(&self, done: bool, body: &[Node]) -> Option<AnyElement> {
        let ix = self.tasks.get();
        self.tasks.set(ix + 1);
        let hooks = self.hooks.tasks.as_ref()?;
        let at = *hooks.markers.get(ix)?;
        let toggle = hooks.toggle.clone();
        let name = body
            .iter()
            .find_map(|node| match node {
                Node::Paragraph(text) | Node::Heading(_, text) if !text.text.is_empty() => {
                    Some(text.text.clone())
                }
                Node::Image { alt, .. } if !alt.is_empty() => Some(alt.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "Task".to_string());
        Some(
            div()
                .debug_selector(move || format!("markdown-task-toggle-{at}"))
                .child(
                    Checkbox::new(self.key("task"), done)
                        .name(name)
                        .on_change(move |done, window, cx| toggle(at, done, window, cx)),
                )
                .into_any_element(),
        )
    }
}

/// Completion shares Markdown's strike styling without changing source text.
pub(super) fn completed_inline(inline: &Inline, done: bool) -> Cow<'_, Inline> {
    if !done || inline.text.is_empty() {
        return Cow::Borrowed(inline);
    }
    let mut shown = inline.clone();
    shown.styles.push((0..shown.text.len(), Style::Strike));
    Cow::Owned(shown)
}

#[cfg(all(test, feature = "test-support"))]
mod tests;
