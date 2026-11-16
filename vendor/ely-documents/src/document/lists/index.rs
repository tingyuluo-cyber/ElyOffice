use std::ops::Range;

use crate::markdown::{file_tree, mdast::Node, span};

#[derive(Clone)]
pub(super) struct Item {
    pub row: usize,
    pub marker: Range<usize>,
    pub body: usize,
    pub position: Range<usize>,
    pub group: usize,
    pub parent: Option<usize>,
}

pub(super) struct Group {
    pub start: u32,
    pub ordered: bool,
    pub items: Vec<usize>,
}

pub(super) struct Index {
    pub items: Vec<Item>,
    pub groups: Vec<Group>,
    raw: Vec<Range<usize>>,
}

pub(super) fn row(text: &str, at: usize) -> usize {
    text.as_bytes()[..at]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |at| at + 1)
}

pub(super) fn end(text: &str, at: usize) -> usize {
    text[at..]
        .find(['\r', '\n'])
        .map_or(text.len(), |end| at + end)
}

pub(super) fn columns(text: &str) -> usize {
    text.chars().fold(0, |col, ch| match ch {
        '\t' => (col / 4 + 1) * 4,
        _ => col + 1,
    })
}

pub(super) fn padding(text: &str, start: usize, marker: usize) -> usize {
    text[start..marker].rfind('>').map_or(start, |at| {
        let at = start + at + 1;
        at + usize::from(text.as_bytes().get(at) == Some(&b' '))
    })
}

pub(super) fn container(text: &str, start: usize, quotes: usize) -> usize {
    let mut at = start;
    for _ in 0..quotes {
        at += text[at..].len() - text[at..].trim_start_matches([' ', '\t']).len();
        if text.as_bytes().get(at) != Some(&b'>') {
            return start;
        }
        at += 1;
        at += usize::from(text.as_bytes().get(at) == Some(&b' '));
    }
    at
}

impl Index {
    pub fn new(text: &str) -> Self {
        let mut index = Self {
            items: Vec::new(),
            groups: Vec::new(),
            raw: Vec::new(),
        };
        index.visit(text, &file_tree(text), None);
        index
    }

    fn visit(&mut self, text: &str, node: &Node, parent: Option<usize>) {
        if matches!(
            node,
            Node::Code(_) | Node::Math(_) | Node::Html(_) | Node::Yaml(_) | Node::Toml(_)
        ) {
            self.raw.push(span(node));
            return;
        }
        if let Node::List(list) = node {
            let group = self.groups.len();
            self.groups.push(Group {
                start: list.start.unwrap_or(1),
                ordered: list.ordered,
                items: Vec::new(),
            });
            for node in &list.children {
                let position = span(node);
                let raw = &text[position.clone()];
                let start = position.start + raw.len() - raw.trim_start_matches([' ', '\t']).len();
                let line = &text[start..end(text, start)];
                let width = if list.ordered {
                    line.bytes().take_while(u8::is_ascii_digit).count() + 1
                } else {
                    1
                };
                let past = start + width;
                let body = past + text[past..end(text, past)].len()
                    - text[past..end(text, past)]
                        .trim_start_matches([' ', '\t'])
                        .len();
                let id = self.items.len();
                self.items.push(Item {
                    row: row(text, start),
                    marker: start..past,
                    body,
                    position,
                    group,
                    parent,
                });
                self.groups[group].items.push(id);
                for child in node.children().into_iter().flatten() {
                    self.visit(text, child, Some(id));
                }
            }
            return;
        }
        for child in node.children().into_iter().flatten() {
            self.visit(text, child, parent);
        }
    }

    pub fn at(&self, at: usize) -> Option<usize> {
        if self
            .raw
            .iter()
            .any(|range| range.start <= at && at <= range.end)
        {
            return None;
        }
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.marker.start <= at && at <= item.position.end)
            .min_by_key(|(_, item)| item.position.len())
            .map(|(id, _)| id)
    }

    pub fn parent(&self, text: &str, id: usize) -> Option<usize> {
        let item = &self.items[id];
        item.parent.filter(|parent| {
            let parent = &self.items[*parent];
            text[item.row..item.marker.start].matches('>').count()
                == text[parent.row..parent.marker.start].matches('>').count()
        })
    }
}
