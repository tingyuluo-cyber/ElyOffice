use super::Drawn;
use crate::tree::{Inline, Node, lines_at};

/// A selectable text's original Markdown, relative to its block.
pub(crate) enum Source {
    Inline(Inline),
    Lines { text: String, from: usize },
    Raw(String),
}

impl Source {
    pub(crate) fn text(&self) -> &str {
        match self {
            Self::Inline(inline) => &inline.text,
            Self::Lines { text, .. } | Self::Raw(text) => text,
        }
    }

    pub(crate) fn at(&self, source: &str, at: usize, upstream: bool) -> Option<usize> {
        match self {
            Self::Inline(inline) if upstream => inline.source_end(source, at),
            Self::Inline(inline) => inline.source_at(source, at),
            Self::Lines { text, from } => lines_at(source, *from, text, at),
            Self::Raw(text) => (at <= text.len() && text.is_char_boundary(at)).then_some(at),
        }
    }
}

/// Texts in the renderer's traversal order, including blocks outside the viewport.
pub(crate) fn content(nodes: &[Node], out: &mut Vec<Drawn>, sources: &mut Vec<Source>) {
    collect(nodes, 0, &mut String::new(), out, sources);
}

fn push(source: Source, lead: &mut String, out: &mut Vec<Drawn>, sources: &mut Vec<Source>) {
    out.push(Drawn {
        lead: std::mem::take(lead),
        text: source.text().to_owned(),
        layout: None,
        links: Vec::new(),
        mapped: None,
    });
    sources.push(source);
}

fn collect(
    nodes: &[Node],
    depth: usize,
    lead: &mut String,
    out: &mut Vec<Drawn>,
    sources: &mut Vec<Source>,
) {
    for node in nodes {
        match node {
            Node::Paragraph(inline) | Node::Heading(_, inline) => {
                push(Source::Inline(inline.clone()), lead, out, sources)
            }
            Node::Code {
                language,
                text,
                from,
            } if language.as_deref() != Some("mermaid") => {
                push(
                    Source::Lines {
                        text: text.clone(),
                        from: *from,
                    },
                    lead,
                    out,
                    sources,
                );
            }
            Node::FrontMatter { text, from } => push(
                Source::Lines {
                    text: text.clone(),
                    from: *from,
                },
                lead,
                out,
                sources,
            ),
            Node::Quote(body) | Node::Footnote { body, .. } | Node::Alert { body, .. } => {
                collect(body, depth, lead, out, sources);
            }
            Node::List { start, items } => {
                for (ix, (task, body)) in items.iter().enumerate() {
                    let marker = match (task, start) {
                        (Some(true), _) => "[x]".to_string(),
                        (Some(false), _) => "[ ]".to_string(),
                        (None, Some(start)) => format!("{}.", start + ix as u64),
                        (None, None) => ["•", "◦", "▪"][depth.min(2)].to_string(),
                    };
                    *lead = format!("{}{marker} ", "  ".repeat(depth));
                    collect(body, depth + 1, lead, out, sources);
                    lead.clear();
                }
            }
            Node::Table { head, rows, .. } => {
                for cell in head.iter().chain(rows.iter().flatten()) {
                    collect(cell, depth, lead, out, sources);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn raw(text: &str, out: &mut Vec<Drawn>, sources: &mut Vec<Source>) {
    push(
        Source::Raw(text.to_owned()),
        &mut String::new(),
        out,
        sources,
    );
}
