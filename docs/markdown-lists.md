# Markdown list editing

Source and visual editing use the same implementation in `vendor/ely-documents/src/document/lists`.

| Key | Behavior |
| --- | --- |
| Enter | Continue a sibling item and renumber the affected list. New tasks start unchecked. |
| Tab | Nest under the preceding sibling. A new numbered sublist starts at 1; an existing sublist continues its numbering. |
| Shift+Tab | Move to the parent level and continue that list's numbering. |
| Enter on an empty item | Move up one level; at the outer level, remove the list marker. |
| Shift+Enter | Insert a line break within the same item, without another marker. |

Moving a parent also moves its continuation lines and children. A first item without a preceding sibling stays at its current level. List editing preserves quote prefixes, numbered-list delimiters, existing list starts, CRLF, selections and undo. Number changes are limited to affected sibling lists. Code blocks, math, HTML and front matter do not create list continuations.

Files above `LARGEST_READ` remain source-only. Tab and Shift+Tab use plain line indentation without parsing a Markdown tree or renumbering lists.

The local crate is from Ely-GPUI-Components commit `e0f355ac1a669c24336244bcaa8131f7f0e4f1b5`, with its MIT license preserved. The Git dependency patch keeps these interactions in the shared component rather than duplicating them in an application.

Validation:

```text
cargo test --locked -p ely-documents --features test-support
cargo test --locked -p ely-mark --bin ely-mark
```

Interaction reference: [Obsidian smart lists](https://obsidian.md/help/settings) and [list line breaks](https://obsidian.md/help/Editing+and+formatting/Basic+formatting+syntax).
