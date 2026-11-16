use super::Drawn;
use crate::tree::Node;

/// Texts in the renderer's traversal order, including blocks outside the viewport.
pub(crate) fn content(nodes: &[Node], out: &mut Vec<Drawn>) {
    collect(nodes, 0, &mut String::new(), out);
}

fn push(text: &str, lead: &mut String, out: &mut Vec<Drawn>) {
    out.push(Drawn {
        lead: std::mem::take(lead),
        text: text.to_owned(),
        layout: None,
        links: Vec::new(),
        mapped: None,
    });
}

fn collect(nodes: &[Node], depth: usize, lead: &mut String, out: &mut Vec<Drawn>) {
    for node in nodes {
        match node {
            Node::Paragraph(inline) | Node::Heading(_, inline) => push(&inline.text, lead, out),
            Node::Code { language, text, .. } if language.as_deref() != Some("mermaid") => {
                push(text, lead, out);
            }
            Node::FrontMatter { text, .. } => push(text, lead, out),
            Node::Quote(body) | Node::Footnote { body, .. } | Node::Alert { body, .. } => {
                collect(body, depth, lead, out);
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
                    collect(body, depth + 1, lead, out);
                    lead.clear();
                }
            }
            Node::Table { head, rows, .. } => {
                for cell in head.iter().chain(rows.iter().flatten()) {
                    collect(cell, depth, lead, out);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn raw(text: &str, out: &mut Vec<Drawn>) {
    push(text, &mut String::new(), out);
}
