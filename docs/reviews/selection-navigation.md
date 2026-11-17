# ReimuMark 0.1.24 selection navigation

Date: 2026-11-17

Scope: selected text across source, preview and open visual blocks; pointer placement.

Review findings:

- Mode changes previously discarded preview selections and synchronized only the top source line. Preview now indexes each selectable text's existing inline or line source mapping. Both selection ends and their direction carry across modes. An untouched mapped selection retains its exact original Markdown span, including formatting markers.
- Selection boundaries at the edge of formatted text use the preceding piece for the range end. This preserves the last selected character before a closing marker. Chinese text, CRLF, nested tasks, quotes, tables, code, front matter and inline math have regression coverage.
- Scroll placement first brings the source line's block into view, then reveals the selected drawn row. Returning to source uses the editor's current soft-wrap rows after its width settles. This avoids leaving a selection offscreen inside a long single-line paragraph. Mode, placement, selection and preview identity checks reject stale callbacks.
- Pointer lookup previously stopped when traversal reached a lower text rectangle. Table cells can contain several paragraphs, so that traversal does not sort rectangles by height. Lookup now compares all live rectangles by vertical and horizontal distance. Gaps use the nearest text boundary.
- GPUI's text lookup treated an exact line boundary as part of the preceding line. Its wrapped-line lookup then returned offset zero. A half-open line interval sends that boundary to the following line instead. Both pointer regressions were observed failing before their fixes.
- The Reimu toolbar is covered by actual GPUI mouse clicks. Selection changes and mode switches leave saved text and undo history intact. No document text was added to logging.

Validation:

- Document and editor suites: 568 document tests, 68 editor tests and 67 documentation tests passed.
- Application suite: 235 tests passed with the installed Git and the pinned PDF worker.
- Clippy with warnings denied passed for documents, editor and application targets.
- Comment, changed-file length, line-ending and patch-whitespace checks passed. The Bash wrapper could not start in the Windows sandbox; its checks were run through Python and native PowerShell instead. The existing large GPUI text module received a one-line comparison fix.

Release validation:

- Built `ReimuMark-0.1.24-windows-x64.zip` using the pinned, SHA-256-verified upstream installer. The extracted executable matches the release build; the PDF worker matches the installed runtime.
- The extracted package printed `ready ElyMark` and exited 0 using an isolated profile without `ELY_MARK_REIMU`.
- The 0.1.24 application suite passed all 235 tests with the package's PDF worker.
- Verified the open user document against its saved text before quitting. Backed up the old executable as `D:\ReimuMark\ReimuMark.0.1.23.previous.exe`, installed 0.1.24 and observed its restored document tab and version startup event.
- In the installed UI, dragging two preview lines and switching to source copied exactly the corresponding Markdown, including Chinese and bold markers. The original document file stayed unchanged. Bidirectional switching and long wrapped paragraphs are covered by the component and toolbar regression suites.

Portable ZIP SHA-256: `2732bdf7da56f48de8093c6a5f89a19076adf0cc91424f7e9f866d664b65fe9f`.
