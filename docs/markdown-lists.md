# Markdown list editing

Source and visual editing use the same implementation in `vendor/ely-documents/src/document/lists`.

| Key | Behavior |
| --- | --- |
| Enter | Continue a sibling item and renumber the affected list. New tasks start unchecked. |
| Enter before a list marker | Insert an empty item before it and renumber its siblings. |
| Tab | Nest under the preceding sibling. A new numbered sublist starts at 1; an existing sublist continues its numbering. |
| Shift+Tab | Move to the parent level and continue that list's numbering. |
| Enter on an empty item | Move up one level; at the outer level, remove the list marker. |
| Shift+Enter | Insert a line break within the same item, without another marker. |

Moving a parent also moves its continuation lines and children. A first item without a preceding sibling stays at its current level. List editing preserves quote prefixes, numbered-list delimiters, existing list starts, CRLF, selections and undo. Number changes are limited to affected sibling lists. Code blocks, math, HTML and front matter do not create list continuations.

Preview task checkboxes toggle `[ ]`, `[x]` and `[X]` in the document. Each click changes only the marker's middle byte and creates one step in the shared source/visual undo history. Nested, quoted, numbered and empty tasks work the same way. Tab reaches the checkbox; Space or Enter toggles it. Task text stays read-only in preview. Callbacks from an older parse or mode are ignored. See the [0.1.21 review and validation](reviews/0.1.21-preview-tasks.md).

In preview, completed task text uses the same strikethrough styling as `~~text~~`. Unchecking removes only the completion styling; explicit Markdown strikes remain. Each nested item uses its own checked state. Rendering preserves source text, selection offsets, inline formatting and links. See the [0.1.23 review and validation](reviews/0.1.23-completed-tasks.md).

Files above `LARGEST_READ` remain source-only. Tab and Shift+Tab use plain line indentation without parsing a Markdown tree or renumbering lists.

The local crate is from Ely-GPUI-Components commit `e0f355ac1a669c24336244bcaa8131f7f0e4f1b5`, with its MIT license preserved. The Git dependency patch keeps these interactions in the shared component rather than duplicating them in an application.

The same pinned `ely-editor` is vendored for source soft wrapping. Window width and font size change the drawn rows without inserting newlines into the file. Mouse positions, IME bounds, vertical navigation, copies and undo use the original source offsets. The wrap cache changes only with text, width or font changes; line highlighting is shared within each frame.

Typing `*` with selected text adds a star on each side and keeps the text selected, so another `*` adds the second pair. Reimu uses `Ctrl+/` for source mode; `Ctrl+Shift+E` has no default binding. Windows drawing retains DirectWrite's actual fallback face, including simulated italics, so glyph indices are drawn with their original font.

ElyMark colors bold text with the theme's blue and italic text with its magenta. Source, open editing and preview share those colors in light and dark themes. Combined bold and italic uses the bold color and retains both styles. Other document component users keep their existing text colors.

Validation:

```text
cargo test --locked -p ely-documents --features test-support
cargo test --locked -p ely-editor --features test-support
cargo test --locked -p ely-mark --bin ely-mark
cargo test --locked -p gpui-pre-windows --lib
```

Interaction reference: [Obsidian smart lists](https://obsidian.md/help/settings) and [list line breaks](https://obsidian.md/help/Editing+and+formatting/Basic+formatting+syntax).

Validation on Windows, 2026-11-16:

- 545 document, 67 editor, 231 ReimuMark and 9 DirectWrite tests passed. ElyMark tests use the installed Git through `ELY_GIT`.
- Clippy with warnings denied, file/comment/line-ending checks and record checks passed.
- The running build showed readable Chinese italics and source soft wrapping. The original installed build showed corrupted italic glyphs on the same text.
- Independent code review found and verified fixes for cached UTF-8 highlight spans, wrap-boundary affinity including multiple cursors, nested list prefixes and combined bold/italic color priority.
- The formal external review script was blocked by automatic approval because it launches Claude with permission checks disabled. No formal script verdict is claimed.
- Broader OS integration suites were not green: the UI suite passed 94/96 (tray icon refused and an Omarchy watcher timing assertion); Windows platform tests passed 22/25 (clipboard and credential access). DirectWrite's 9 tests passed separately.

ReimuMark 0.1.20 preserves the standalone profile and icon from the fork's main branch. The extracted portable package reached its smoke-test readiness event without ELY_MARK_REIMU. Its editor matches the release build and its PDF worker matches the pinned runtime.

Portable ZIP SHA-256: `1a9b342bb02feb24b5f05ba6ce424567b172212e80d82fd88ab87f626115ed22`.
