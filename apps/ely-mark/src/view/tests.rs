use std::sync::{Arc, Mutex};

use gpui::{AppContext as _, Entity, TestAppContext, VisualTestContext};

use super::{MarkView, Mode, Reported};
use crate::engine::{Text, mark};
use crate::text::Source;

fn open<'a>(
    text: &str,
    cx: &'a mut TestAppContext,
) -> (Entity<MarkView>, &'a mut VisualTestContext) {
    cx.update(ely_gpui_component::init_for_tests);
    cx.update(|cx| {
        ely_ui::settings::install_defaults(cx);
        ely_ui::i18n::install_as("en-US", cx);
    });
    let source = Source::decode(text.as_bytes().to_vec()).expect("reads");
    let saved = mark(&source.text);
    let doc = Arc::new(Mutex::new(Text::new(source)));
    let (view, cx) = cx.add_window_view(move |window, cx| MarkView::new(doc, saved, window, cx));
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    (view, cx)
}

#[gpui::test]
fn typing_reaches_the_text_the_shell_saves(cx: &mut TestAppContext) {
    let (view, cx) = open("# A\n\nB\n", cx);
    let outline: Vec<(usize, String, usize)> = view.read_with(cx, |view, cx| {
        view.outline(cx)
            .into_iter()
            .map(|heading| (heading.level, heading.title, heading.line))
            .collect()
    });
    assert_eq!(outline, [(1, "A".to_string(), 0)]);
    view.update_in(cx, |view, window, cx| view.switch(Mode::Source, window, cx));
    let focus = view.read_with(cx, |view, _| view.focus.clone());
    cx.update(|window, cx| window.focus(&focus, cx));
    cx.run_until_parked();
    cx.simulate_input("x");
    let (bytes, modified, changed) = view.read_with(cx, |view, _| {
        (view.bytes(), view.modified(), view.changed())
    });
    assert_eq!(bytes, "x# A\n\nB\n".len());
    assert!(modified && changed > 0);
}

fn current(view: &Entity<MarkView>, cx: &mut VisualTestContext) -> String {
    view.read_with(cx, |view, _| {
        view.doc.lock().expect("the text lock").current.clone()
    })
}

fn focused(view: &Entity<MarkView>, cx: &mut VisualTestContext) {
    let focus = view.read_with(cx, |view, _| view.focus.clone());
    cx.update(|window, cx| window.focus(&focus, cx));
    cx.run_until_parked();
}

#[gpui::test]
fn visual_typing_surrounds_the_selection_in_one_undo(cx: &mut TestAppContext) {
    let original = "前 我是猫 后\n";
    let (view, cx) = open(original, cx);
    let picked = "前 ".len().."前 我是猫".len();
    let document = view.read_with(cx, |view, _| view.document().clone());
    document.update_in(cx, |document, window, cx| {
        document.show_match(picked.clone(), window, cx);
    });
    cx.run_until_parked();
    assert_eq!(document.read_with(cx, |doc, cx| doc.selected(cx)), "我是猫");
    cx.simulate_keystrokes("shift-8->*");
    assert_eq!(current(&view, cx), "前 *我是猫* 后\n");
    cx.simulate_keystrokes(ely_gpui_component::secondary!("z"));
    assert_eq!(current(&view, cx), original);

    document.update_in(cx, |document, window, cx| document.show_match(picked, window, cx));
    cx.run_until_parked();
    cx.simulate_input("x");
    assert_eq!(current(&view, cx), "前 x 后\n");
}

#[gpui::test]
fn source_typing_surrounds_each_selection_and_types_at_bare_carets(cx: &mut TestAppContext) {
    let original = "猫和狗\n";
    let (view, cx) = open(original, cx);
    view.update_in(cx, |view, window, cx| view.switch(Mode::Source, window, cx));
    focused(&view, cx);
    let editor = view.read_with(cx, |view, cx| view.document().read(cx).editor().clone());
    editor.update(cx, |editor, cx| {
        editor.select(
            [0.."猫".len(), "猫和".len().."猫和狗".len(), original.len()..original.len()],
            cx,
        );
    });
    cx.simulate_input("[");
    assert_eq!(current(&view, cx), "[猫]和[狗]\n[");
    cx.simulate_keystrokes(ely_gpui_component::secondary!("z"));
    assert_eq!(current(&view, cx), original);

    editor.update(cx, |editor, cx| editor.select(std::iter::once(0..0), cx));
    cx.simulate_input("*");
    assert_eq!(current(&view, cx), "*猫和狗\n");
}

#[gpui::test]
fn source_tab_indents_lines_and_keeps_the_selection(cx: &mut TestAppContext) {
    let original = "一行\n\nsecond\nthird\n";
    let (view, cx) = open(original, cx);
    cx.update(|_, cx| crate::keys::bind(false, cx));
    view.update_in(cx, |view, window, cx| {
        view.switch(Mode::Source, window, cx);
        view.reveal((0, 1), window, cx);
    });
    focused(&view, cx);
    cx.simulate_keystrokes("tab");
    assert_eq!(current(&view, cx), "    一行\n\nsecond\nthird\n");
    assert_eq!(view.read_with(cx, |view, cx| view.place(cx).column), 5);
    cx.simulate_keystrokes("shift-tab");
    assert_eq!(current(&view, cx), original);

    let editor = view.read_with(cx, |view, cx| view.document().read(cx).editor().clone());
    let start = "一".len();
    let end = "一行\n\nsecond\n".len();
    editor.update(cx, |editor, cx| {
        editor.select([std::ops::Range { start: end, end: start }], cx);
    });
    cx.simulate_keystrokes("tab");
    assert_eq!(current(&view, cx), "    一行\n\n    second\nthird\n");
    let kept = editor.read_with(cx, |editor, _| editor.primary());
    assert_eq!((kept.anchor, kept.head), (end + 8, start + 4));
    cx.simulate_keystrokes(ely_gpui_component::secondary!("z"));
    assert_eq!(current(&view, cx), original);
}

#[gpui::test]
fn source_list_keys_keep_each_levels_numbers(cx: &mut TestAppContext) {
    let original = "1. 一级\n2. 二级\n3. 三级\n4. 四级";
    let (view, cx) = open(original, cx);
    cx.update(|_, cx| crate::keys::bind(false, cx));
    view.update_in(cx, |view, window, cx| {
        view.switch(Mode::Source, window, cx);
        view.reveal((2, 5), window, cx);
    });
    focused(&view, cx);
    cx.simulate_keystrokes("tab");
    assert_eq!(current(&view, cx), "1. 一级\n2. 二级\n    1. 三级\n3. 四级");
    cx.simulate_keystrokes("shift-tab");
    assert_eq!(current(&view, cx), original);
    cx.simulate_keystrokes(ely_gpui_component::secondary!("z"));
    assert_eq!(current(&view, cx), "1. 一级\n2. 二级\n    1. 三级\n3. 四级");
    cx.simulate_keystrokes(ely_gpui_component::secondary!("z"));
    assert_eq!(current(&view, cx), original);
}

#[gpui::test]
fn source_list_enter_tab_and_empty_enter_form_a_complete_flow(cx: &mut TestAppContext) {
    let (view, cx) = open("1. 父项", cx);
    cx.update(|_, cx| crate::keys::bind(false, cx));
    view.update_in(cx, |view, window, cx| {
        view.switch(Mode::Source, window, cx);
        view.reveal((0, 5), window, cx);
    });
    focused(&view, cx);
    cx.simulate_keystrokes("enter tab");
    assert_eq!(current(&view, cx), "1. 父项\n    1. ");
    cx.simulate_input("子项");
    cx.simulate_keystrokes("enter");
    assert_eq!(current(&view, cx), "1. 父项\n    1. 子项\n    2. ");
    cx.simulate_keystrokes("enter");
    assert_eq!(current(&view, cx), "1. 父项\n    1. 子项\n2. ");
    cx.simulate_keystrokes("enter");
    assert_eq!(current(&view, cx), "1. 父项\n    1. 子项\n");
}

#[gpui::test]
fn source_shift_enter_stays_in_the_same_item(cx: &mut TestAppContext) {
    let (view, cx) = open("1. a", cx);
    cx.update(|_, cx| crate::keys::bind(false, cx));
    view.update_in(cx, |view, window, cx| {
        view.switch(Mode::Source, window, cx);
        view.reveal((0, 4), window, cx);
    });
    focused(&view, cx);
    cx.simulate_keystrokes("shift-enter");
    cx.simulate_input("continued");
    assert_eq!(current(&view, cx), "1. a  \n   continued");
    cx.simulate_keystrokes("enter");
    assert_eq!(current(&view, cx), "1. a  \n   continued\n2. ");
}

#[gpui::test]
fn large_source_tab_keeps_plain_indentation_without_reading_a_tree(cx: &mut TestAppContext) {
    let original = format!(
        "1. item\n{}",
        "x".repeat(ely_gpui_component::documents::LARGEST_READ)
    );
    let (view, cx) = open(&original, cx);
    view.update_in(cx, |view, window, cx| view.reveal((0, 7), window, cx));
    focused(&view, cx);
    cx.simulate_keystrokes("tab");
    assert_eq!(current(&view, cx), format!("    {original}"));
    cx.simulate_keystrokes("shift-tab");
    assert_eq!(current(&view, cx), original);
}

#[gpui::test]
fn switching_modes_writes_no_byte(cx: &mut TestAppContext) {
    let (view, cx) = open("# A\r\n\r\n* b\r\n", cx);
    for mode in [Mode::Source, Mode::Preview, Mode::Visual, Mode::Source] {
        view.update_in(cx, |view, window, cx| view.switch(mode, window, cx));
        cx.run_until_parked();
    }
    let written = view.read_with(cx, |view, _| {
        let doc = view.doc.lock().expect("the text lock");
        doc.source.encode(&doc.current)
    });
    assert_eq!(written, b"# A\r\n\r\n* b\r\n");
    let (modified, changed) = view.read_with(cx, |view, _| (view.modified(), view.changed()));
    assert!(!modified && changed == 0);
}

#[gpui::test]
fn undo_reaches_across_modes(cx: &mut TestAppContext) {
    let (view, cx) = open("# A\n", cx);
    view.update_in(cx, |view, window, cx| view.switch(Mode::Source, window, cx));
    focused(&view, cx);
    cx.simulate_input("x");
    assert_eq!(current(&view, cx), "x# A\n");
    view.update_in(cx, |view, window, cx| view.switch(Mode::Visual, window, cx));
    focused(&view, cx);
    cx.simulate_keystrokes("secondary-z");
    cx.run_until_parked();
    assert_eq!(
        current(&view, cx),
        "# A\n",
        "the source edit undone in visual"
    );
}

#[gpui::test]
fn undo_brings_back_a_text_emptied_in_source(cx: &mut TestAppContext) {
    let (view, cx) = open("Seed\n", cx);
    view.update_in(cx, |view, window, cx| view.switch(Mode::Source, window, cx));
    view.update(cx, |view, cx| view.adopt(String::new(), cx));
    view.update_in(cx, |view, window, cx| view.switch(Mode::Visual, window, cx));
    focused(&view, cx);
    cx.simulate_keystrokes("secondary-z");
    cx.run_until_parked();
    assert_eq!(current(&view, cx), "Seed\n");
}

#[gpui::test]
fn a_new_empty_file_takes_typing_in_visual(cx: &mut TestAppContext) {
    let (view, cx) = open("", cx);
    assert_eq!(view.read_with(cx, |view, _| view.mode), Mode::Visual);
    focused(&view, cx);
    cx.simulate_input("Hi");
    assert_eq!(current(&view, cx), "Hi");
    assert!(view.read_with(cx, |view, _| view.modified()));
}

#[gpui::test]
fn a_long_file_shows_once_its_blocks_are_read(cx: &mut TestAppContext) {
    let long = "para\n\n".repeat(40_000);
    let (view, cx) = open(&long, cx);
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(1));
    cx.run_until_parked();
    let (shown, read) = view.read_with(cx, |view, cx| {
        (view.shown, view.document.read(cx).read_now(cx))
    });
    assert!(shown && read, "shown with its blocks read");
}

#[gpui::test]
fn a_refresh_is_saved_text_and_keeps_no_undo(cx: &mut TestAppContext) {
    let (view, cx) = open("one\n", cx);
    view.update(cx, |view, cx| view.adopt("edited\n".into(), cx));
    cx.run_until_parked();
    assert!(view.read_with(cx, |view, _| view.modified()));
    let disk = Source::decode(b"from disk\n".to_vec()).expect("reads");
    view.update(cx, |view, cx| view.refresh(disk, cx));
    cx.run_until_parked();
    assert_eq!(current(&view, cx), "from disk\n");
    assert!(!view.read_with(cx, |view, _| view.modified()));
    view.update_in(cx, |view, window, cx| view.switch(Mode::Source, window, cx));
    let focus = view.read_with(cx, |view, _| view.focus.clone());
    cx.update(|window, cx| window.focus(&focus, cx));
    cx.run_until_parked();
    cx.simulate_keystrokes("secondary-z");
    cx.run_until_parked();
    assert_eq!(current(&view, cx), "from disk\n", "undo stops at the disk");
}

#[gpui::test]
fn the_shell_hears_a_load_it_subscribed_to(cx: &mut TestAppContext) {
    cx.update(ely_gpui_component::init_for_tests);
    cx.update(|cx| {
        ely_ui::settings::install_defaults(cx);
        ely_ui::i18n::install_as("en-US", cx);
    });
    let heard = std::rc::Rc::new(std::cell::Cell::new(0));
    let (count, held) = (
        heard.clone(),
        std::rc::Rc::new(std::cell::RefCell::new(None)),
    );
    let holder = held.clone();
    let (_root, cx) = cx.add_window_view(move |window, cx| {
        let source = Source::decode(b"# Opened\n".to_vec()).expect("reads");
        let saved = mark(&source.text);
        let doc = Arc::new(Mutex::new(Text::new(source)));
        let view = cx.new(|cx| MarkView::new(doc, saved, window, cx));
        // As the shell: subscribed after the view is made.
        let count = count.clone();
        cx.subscribe(&view, move |_, _, _: &Reported, _| {
            count.set(count.get() + 1)
        })
        .detach();
        holder.replace(Some(view));
        gpui::Empty
    });
    cx.run_until_parked();
    assert!(heard.get() > 0);
}

#[gpui::test]
fn a_text_grown_past_the_limit_goes_to_source(cx: &mut TestAppContext) {
    let (view, cx) = open("# Old\n", cx);
    let long = "x".repeat(ely_gpui_component::documents::LARGEST_READ + 1);
    view.update(cx, |view, cx| view.adopt(long, cx));
    cx.run_until_parked();
    assert!(view.read_with(cx, |view, _| view.large()));
    assert_eq!(view.read_with(cx, |view, _| view.mode), Mode::Source);
    view.update_in(cx, |view, window, cx| {
        view.switch(Mode::Preview, window, cx)
    });
    assert_eq!(view.read_with(cx, |view, _| view.mode), Mode::Source);
    let outline = view.read_with(cx, |view, cx| view.outline(cx));
    assert!(outline.is_empty(), "no stale heading: {outline:?}");
}

#[gpui::test]
fn a_large_file_stays_in_source(cx: &mut TestAppContext) {
    let large = ely_gpui_component::documents::LARGEST_READ / 2 + 1;
    let (view, cx) = open(&"a\n".repeat(large), cx);
    assert_eq!(view.read_with(cx, |view, _| view.mode), Mode::Source);
    view.update_in(cx, |view, window, cx| view.switch(Mode::Visual, window, cx));
    assert_eq!(view.read_with(cx, |view, _| view.mode), Mode::Source);
}

#[gpui::test]
fn an_edit_keeps_the_rest_of_the_file_byte_for_byte(cx: &mut TestAppContext) {
    let bytes = b"\xEF\xBB\xBF# A\r\n\r\nB\r\n".to_vec();
    let (view, cx) = open(std::str::from_utf8(&bytes).expect("UTF-8"), cx);
    view.update_in(cx, |view, window, cx| view.switch(Mode::Source, window, cx));
    let focus = view.read_with(cx, |view, _| view.focus.clone());
    cx.update(|window, cx| window.focus(&focus, cx));
    cx.run_until_parked();
    cx.simulate_input("x");
    let written = view.read_with(cx, |view, _| {
        let doc = view.doc.lock().expect("the text lock");
        doc.source.encode(&doc.current)
    });
    assert_eq!(written, b"\xEF\xBB\xBFx# A\r\n\r\nB\r\n");
}

#[gpui::test]
fn a_web_link_opens_outside(cx: &mut TestAppContext) {
    let (view, cx) = open("x\n", cx);
    view.update(cx, |view, cx| view.follow("https://ely.dev", cx));
    assert_eq!(cx.opened_url().as_deref(), Some("https://ely.dev"));
}

#[gpui::test]
fn a_file_link_is_refused_and_said(cx: &mut TestAppContext) {
    let (view, cx) = open("x\n", cx);
    let said = std::rc::Rc::new(std::cell::Cell::new(false));
    let heard = said.clone();
    cx.update(|_, cx| {
        cx.subscribe(&view, move |_, _: &super::Failed, _| heard.set(true))
            .detach()
    });
    view.update(cx, |view, cx| view.follow("file:///etc/hosts", cx));
    cx.run_until_parked();
    assert!(said.get() && cx.opened_url().is_none());
}

/// A GIF whose frame never decodes.
fn broken_gif() -> Vec<u8> {
    let gif =
        "47494638396101000100800000000000ffffff21f90401000000002c0000000001000100000202ffff003b";
    (0..gif.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&gif[at..at + 2], 16).expect("hex"))
        .collect()
}

#[gpui::test]
fn a_broken_gif_ends_and_its_error_names_no_path(cx: &mut TestAppContext) {
    let (_view, cx) = open("x\n", cx);
    let folder = std::env::temp_dir().join(format!("ely-marker-{}", std::process::id()));
    std::fs::create_dir_all(&folder).expect("a folder");
    let path = folder.join("broken.gif");
    std::fs::write(&path, broken_gif()).expect("the gif");
    let resource = gpui::Resource::Path(path.into());
    let loaded = loop {
        let asked = cx.update(|_, cx| cx.fetch_asset::<gpui::ImageAssetLoader>(&resource));
        if let Some(loaded) = asked {
            break loaded;
        }
        cx.run_until_parked();
    };
    std::fs::remove_dir_all(&folder).expect("cleaned");
    let error = loaded.expect_err("a broken frame fails the load");
    assert!(!format!("{error:?}").contains("ely-marker"), "{error:?}");
}

#[gpui::test]
fn a_broken_inline_gif_ends(cx: &mut TestAppContext) {
    let (_view, cx) = open("x\n", cx);
    let renderer = cx.update(|_, cx| cx.svg_renderer());
    let image = gpui::Image::from_bytes(gpui::ImageFormat::Gif, broken_gif());
    assert!(image.to_image_data(renderer).is_err());
}

#[gpui::test]
fn a_wheel_beside_the_text_scrolls_it(cx: &mut TestAppContext) {
    use gpui::{ScrollDelta, ScrollEvent, point, px};
    let (data, files) = (tempfile::tempdir().expect("data"), tempfile::tempdir().expect("files"));
    let path = files.path().join("long.md");
    std::fs::write(&path, (0..300).map(|n| format!("Paragraph {n}.

")).collect::<String>()).expect("a file");
    let (desk, cx) = crate::desk::tests::start(cx, data.path());
    crate::desk::tests::open(&desk, &path, cx);
    let view = desk.read_with(cx, |desk, _| desk.active_view().expect("a tab"));
    cx.run_until_parked();
    // Left of the first block is margin, not text.
    let block = cx.debug_bounds("markdown-block-0").expect("the first block");
    let top = |cx: &mut VisualTestContext| cx.update(|_, app| view.read(app).place(app).top);
    let before = top(cx);
    cx.simulate_event(ScrollEvent {
        position: point(block.left() - px(20.), block.center().y),
        delta: ScrollDelta::Pixels(point(px(0.), px(-600.))),
        ..Default::default()
    });
    cx.run_until_parked();
    assert!(top(cx) > before, "the margin's wheel moved the text");
}
