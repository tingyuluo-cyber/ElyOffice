use std::rc::Rc;

use gpui::{AnyElement, App, InteractiveElement, IntoElement, ParentElement, Window, div};

use super::Draw;
use crate::{forms::Checkbox, tree::Node};

type OnTask = Rc<dyn Fn(usize, bool, &mut Window, &mut App)>;

pub(crate) struct TaskHooks {
    pub markers: Vec<usize>,
    pub toggle: OnTask,
}

impl Draw<'_> {
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
