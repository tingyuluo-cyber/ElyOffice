use std::{fs, path::PathBuf};

use ely_office::prelude::*;
use ely_ui::{commands, settings::panels};
use gpui::{ExternalPaths, FileDropEvent, TestAppContext, VisualTestContext};

use crate::desk::tests::start;

#[gpui::test]
fn the_folder_picker_reveals_markdown_and_the_file_picker_opens_it(cx: &mut TestAppContext) {
    let data = tempfile::tempdir().expect("profile");
    let files = tempfile::tempdir().expect("files");
    let folder = files.path().join("我的 笔记");
    fs::create_dir_all(folder.join("子目录")).expect("subfolder");
    let note = folder.join("子目录/测试.MD");
    fs::write(&note, "# Markdown\n").expect("note");
    fs::write(folder.join("skip.txt"), "ignored").expect("other file");
    let (desk, window) = start(cx, data.path());
    window.update(|_, cx| panels::show(true, false, cx));

    let root = desk.read_with(window, |desk, _| desk.core.root.upgrade().expect("root"));
    window.update(|window, cx| {
        root.update(cx, |root, cx| root.run_from_palette("file.open_folder", window, cx));
    });
    cx.simulate_path_prompt_response(|options| {
        assert!(options.directories && !options.files && !options.multiple);
        Some(vec![folder.clone()])
    });
    window.run_until_parked();
    assert!(window.update(|_, cx| panels::shown(true, cx)));
    desk.read_with(window, |desk, _| {
        let shown = desk.folder.as_ref().expect("shown folder");
        assert_eq!(
            shown.root,
            dunce::canonicalize(&folder).expect("real folder")
        );
        assert_eq!(shown.files, ["子目录/测试.MD"]);
    });

    window.dispatch_action(commands::Open);
    cx.simulate_path_prompt_response(|options| {
        assert!(options.files && options.multiple);
        Some(vec![note.clone()])
    });
    window.run_until_parked();
    desk.read_with(window, |desk, _| {
        let tab = desk.active_tab().expect("opened note");
        assert_eq!(tab.session.path(), Some(note.as_path()));
        assert_eq!(desk.text_of(tab.id).as_deref(), Some("# Markdown\n"));
    });
}

fn drop_on_body(cx: &mut VisualTestContext, paths: Vec<PathBuf>) {
    let position = cx.debug_bounds("shell-body").expect("drop area").center();
    cx.simulate_event(FileDropEvent::Entered {
        position,
        paths: ExternalPaths(paths.into()),
    });
    cx.simulate_event(FileDropEvent::Submit { position });
    cx.run_until_parked();
}

#[gpui::test]
fn drops_open_notes_on_welcome_and_in_the_editor_and_keep_picture_attachments(
    cx: &mut TestAppContext,
) {
    let data = tempfile::tempdir().expect("profile");
    let files = tempfile::tempdir().expect("files");
    let first = files.path().join("first.md");
    let second = files.path().join("second.markdown");
    let picture = files.path().join("shot.png");
    fs::write(&first, "# First\n").expect("first");
    fs::write(&second, "# Second\n").expect("second");
    fs::write(&picture, b"pixels").expect("picture");
    let (desk, cx) = start(cx, data.path());
    desk.update(cx, |desk, cx| {
        desk.reimu = true;
        cx.notify();
    });
    cx.run_until_parked();

    drop_on_body(cx, vec![first.clone()]);
    let original = desk.read_with(cx, |desk, _| desk.active_tab().expect("first tab").id);
    drop_on_body(cx, vec![second.clone(), picture]);
    desk.read_with(cx, |desk, _| {
        let tab = desk.active_tab().expect("second tab");
        assert_eq!(tab.session.path(), Some(second.as_path()));
        assert_eq!(desk.core.tabs.len(), 2);
        assert!(
            desk.text_of(original)
                .expect("first note")
                .contains("assets/shot.png")
        );
    });
    assert!(files.path().join("assets/shot.png").is_file());

    drop_on_body(cx, vec![files.path().to_path_buf()]);
    desk.read_with(cx, |desk, _| {
        assert_eq!(
            desk.folder.as_ref().expect("dropped folder").files,
            ["first.md", "second.markdown"]
        );
    });
}
