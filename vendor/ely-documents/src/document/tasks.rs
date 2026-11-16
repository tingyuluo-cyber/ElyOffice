use std::{rc::Rc, sync::Arc};

use gpui::Context;

use super::MarkdownDocument;
use crate::{
    MarkdownMode,
    markdown::{mdast::Node, walk},
    render::TaskHooks,
};

/// Only parsed task items contribute markers; code and literal brackets do not.
pub(super) fn markers(text: &str, node: &Node) -> Vec<usize> {
    let mut nodes = Vec::new();
    walk(node, &mut nodes);
    nodes
        .into_iter()
        .filter_map(|node| {
            let Node::ListItem(item) = node else {
                return None;
            };
            item.checked?;
            let end = item
                .children
                .first()
                .map_or(item.position.end, |node| node.position().start);
            let prefix = &text[item.position.start..end];
            let at = prefix
                .as_bytes()
                .windows(3)
                .rposition(|box_| matches!(box_, b"[ ]" | b"[x]" | b"[X]"))?;
            Some(item.position.start + at + 1)
        })
        .collect()
}

impl MarkdownDocument {
    pub(super) fn task_hooks(&self, ix: usize, cx: &Context<Self>) -> Option<TaskHooks> {
        if self.mode != MarkdownMode::Preview {
            return None;
        }
        let block = self.parsed.blocks.get(ix)?;
        if block.tasks.is_empty() {
            return None;
        }
        let (document, read) = (cx.entity().downgrade(), self.parsed.clone());
        Some(TaskHooks {
            markers: block.tasks.clone(),
            toggle: Rc::new(move |at, done, _, cx| {
                let _ = document.update(cx, |document, cx| {
                    // A slow read or a queued click must not edit a different task.
                    if document.mode != MarkdownMode::Preview
                        || !Arc::ptr_eq(&read, &document.parsed)
                        || !document.read_now(cx)
                    {
                        return;
                    }
                    let written = &document.parsed.text[at - 1..at + 2];
                    if !matches!(written, "[ ]" | "[x]" | "[X]") || (written != "[ ]") == done {
                        return;
                    }
                    document.editor.update(cx, |editor, cx| {
                        editor.edit([(at..at + 1, if done { "x" } else { " " }.to_string())], cx);
                    });
                });
            }),
        })
    }
}
