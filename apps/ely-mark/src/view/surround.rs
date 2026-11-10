use gpui::{Context, KeyDownEvent, Window};

use super::{MarkView, Mode};

impl MarkView {
    pub(super) fn surround_selection(
        &mut self,
        event: &KeyDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = &event.keystroke;
        if event.prefer_character_input
            || key.modifiers.control
            || key.modifiers.platform
            || key.modifiers.alt
        {
            return;
        }
        let Some(typed) = key.key_char.as_deref() else {
            return;
        };
        let (open, close) = match typed {
            "*" | "_" | "`" | "$" => (typed, typed),
            "~" => ("~~", "~~"),
            "[" => ("[", "]"),
            "(" => ("(", ")"),
            "{" => ("{", "}"),
            _ => return,
        };

        let wrapped = match self.mode {
            Mode::Preview => false,
            Mode::Visual => self.document.update(cx, |document, cx| {
                let selected = document.selected(cx);
                !selected.is_empty() && document.insert(&format!("{open}{selected}{close}"), cx)
            }),
            Mode::Source => {
                let editor = self.document.read(cx).editor().clone();
                editor.update(cx, |editor, cx| {
                    if editor
                        .selections()
                        .iter()
                        .all(|selection| selection.is_empty())
                    {
                        return false;
                    }
                    let edits: Vec<_> = editor
                        .selections()
                        .iter()
                        .map(|selection| {
                            let range = selection.range();
                            let replacement = if range.is_empty() {
                                typed.to_owned()
                            } else {
                                format!("{open}{}{close}", &editor.text()[range.clone()])
                            };
                            (range, replacement)
                        })
                        .collect();
                    editor.edit(edits, cx);
                    true
                })
            }
        };
        if wrapped {
            log::debug!("mark: surrounded the selection");
            cx.stop_propagation();
        }
    }
}
