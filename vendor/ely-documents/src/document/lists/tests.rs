use std::ops::Range;

use super::{
    changes::{apply, difference},
    enter::enter,
    shift::shift,
};

fn shifted(text: &str, needle: &str, deeper: bool) -> String {
    let at = text.find(needle).expect("item") + needle.len();
    shift(text, std::iter::once(at..at).collect::<Vec<_>>(), deeper).0
}

fn entered(text: &str, soft: bool) -> String {
    enter(text, text.len()..text.len(), soft).expect("list").0
}

#[test]
fn nested_numbers_restart_and_join_existing_children() {
    assert_eq!(
        shifted("1. a\n2. b\n3. c\n4. d", "3. c", true),
        "1. a\n2. b\n    1. c\n3. d"
    );
    assert_eq!(
        shifted("1. a\n    1. b\n2. c\n3. d", "2. c", true),
        "1. a\n    1. b\n    2. c\n2. d"
    );
    assert_eq!(
        shifted("1. a\n2. b\n3. c\n4. d\n5. e\n6. ", "6. ", true),
        "1. a\n2. b\n3. c\n4. d\n5. e\n    1. "
    );
}

#[test]
fn outdent_joins_the_parent_and_repairs_both_lists() {
    assert_eq!(
        shifted("1. a\n    1. b\n    2. c\n2. d", "1. b", false),
        "1. a\n2. b\n    1. c\n3. d"
    );
    assert_eq!(
        shifted("9) a\n    1) b\n10) c", "1) b", false),
        "9) a\n10) b\n11) c"
    );
}

#[test]
fn a_parent_keeps_its_children_and_continuation_lines() {
    let text = "1. a\n2. b\n    continued\n    7. child\n3. c";
    let nested = shifted(text, "2. b", true);
    assert_eq!(
        nested,
        "1. a\n    1. b\n        continued\n        7. child\n2. c"
    );
    assert_eq!(shifted(&nested, "1. b", false), text);
}

#[test]
fn quotes_inside_a_moved_item_follow_the_item() {
    let text = "1. a\n2. b\n   > quoted\n3. c";
    let nested = shifted(text, "2. b", true);
    assert_eq!(nested, "1. a\n    1. b\n       > quoted\n2. c");
    assert_eq!(shifted(&nested, "1. b", false), text);
    assert_eq!(
        shifted("99. a\n    1. b\n       > quoted\n100. c", "1. b", false),
        "99. a\n100. b\n     > quoted\n101. c"
    );
}

#[test]
fn no_parent_means_no_orphaned_sublist() {
    assert_eq!(shifted("1. a\n2. b", "1. a", true), "1. a\n2. b");
}

#[test]
fn numbering_is_scoped_to_the_edited_lists() {
    let text = "1. a\n2. b\n\n# Other\n\n7. x\n9. y";
    assert_eq!(
        shifted(text, "2. b", true),
        "1. a\n    1. b\n\n# Other\n\n7. x\n9. y"
    );
}

#[test]
fn quotes_and_nonstandard_indentation_keep_their_structure() {
    assert_eq!(
        shifted("> 1. a\n> 2. b\n> 3. c", "2. b", true),
        "> 1. a\n>     1. b\n> 2. c"
    );
    assert_eq!(
        shifted("1. a\n   1. b\n2. c", "1. b", false),
        "1. a\n2. b\n3. c"
    );
    assert_eq!(
        shifted("100. a\n101. b", "101. b", true),
        "100. a\n     1. b"
    );
}

#[test]
fn enter_continues_items_and_resets_task_boxes() {
    for (text, expected) in [
        ("- a", "- a\n- "),
        ("  * a", "  * a\n  * "),
        ("9. a", "9. a\n10. "),
        ("3) a", "3) a\n4) "),
        ("- [x] done", "- [x] done\n- [ ] "),
        ("> - a", "> - a\n> - "),
        ("-\ta", "-\ta\n-\t"),
        ("- a\n\t- b", "- a\n\t- b\n\t- "),
        ("- - a", "- - a\n  - "),
    ] {
        assert_eq!(entered(text, false), expected, "{text}");
    }
}

#[test]
fn enter_in_the_middle_repairs_later_numbers() {
    let text = "1. 一二\n2. 后面";
    let at = "1. 一".len();
    let (out, range) = enter(text, at..at, false).expect("list");
    assert_eq!(out, "1. 一\n2. 二\n3. 后面");
    assert_eq!(range, "1. 一\n2. ".len().."1. 一\n2. ".len());
}

#[test]
fn an_empty_item_exits_one_level_at_a_time() {
    let text = "1. a\n    1. b\n    2. ";
    let parent = entered(text, false);
    assert_eq!(parent, "1. a\n    1. b\n2. ");
    assert_eq!(entered(&parent, false), "1. a\n    1. b\n");
    assert_eq!(entered("- a\n- [ ] ", false), "- a\n");
    assert_eq!(entered("> - a\n> - ", false), "> - a\n> ");
    assert_eq!(entered("- - ", false), "- ");
    assert_eq!(entered("- item\n  > - ", false), "- item\n  > ");
    assert_eq!(
        shifted("- item\n  > - child", "- child", false),
        "- item\n  > - child"
    );
}

#[test]
fn shift_enter_continues_the_item_without_a_new_marker() {
    assert_eq!(entered("1. a", true), "1. a  \n   ");
    assert_eq!(entered("- [x] a", true), "- [x] a  \n  ");
    assert_eq!(entered("> 1. a", true), "> 1. a  \n>    ");
}

#[test]
fn code_html_and_front_matter_are_not_lists() {
    for text in [
        "text",
        "-a",
        "1.a",
        "* * *",
        "```\n1. a\n```",
        "---\n- a\n---",
    ] {
        let at = text.find("a").map_or(text.len(), |at| at + 1);
        assert_eq!(enter(text, at..at, false), None, "{text}");
    }
    assert_eq!(
        shifted("```\n1. a\n```", "1. a", true),
        "```\n    1. a\n```"
    );
    for text in [
        "- item\n  ```\n  code",
        "- item\n  ```\n  code\n  ```",
        "- item\n  <div>html",
    ] {
        assert_eq!(enter(text, text.len()..text.len(), false), None, "{text}");
    }
}

#[test]
fn plain_text_and_reversed_selections_keep_their_positions() {
    let text = "一行\n\nsecond\nthird\n";
    let start = "一".len();
    let end = "一行\n\nsecond\n".len();
    let (out, ranges) = shift(
        text,
        vec![Range {
            start: end,
            end: start,
        }],
        true,
    );
    assert_eq!(out, "    一行\n\n    second\nthird\n");
    assert_eq!(
        ranges,
        vec![Range {
            start: end + 8,
            end: start + 4
        }]
    );
    assert_eq!(
        shift("  \t- b", std::iter::once(6..6).collect::<Vec<_>>(), false),
        ("- b".into(), std::iter::once(3..3).collect::<Vec<_>>())
    );
}

#[test]
fn crlf_and_multi_digit_carets_are_preserved() {
    assert_eq!(entered("1. a\r\n2. b", false), "1. a\r\n2. b\r\n3. ");
    let text = "9. 父级\n    1. 子项\n10. 后项";
    let at = text.find("子项").expect("child") + "子项".len();
    let (out, ranges) = shift(text, std::iter::once(at..at).collect::<Vec<_>>(), false);
    assert_eq!(out, "9. 父级\n10. 子项\n11. 后项");
    let expected = "9. 父级\n10. 子项".len();
    assert_eq!(
        ranges,
        std::iter::once(expected..expected).collect::<Vec<_>>()
    );
}

#[test]
fn edits_reproduce_the_result_at_character_boundaries() {
    for (before, after) in [
        ("1. 一\n2. 二\n3. 三", "1. 一\n    1. 二\n2. 三"),
        ("9. 中文", "9. 中\n10. 文"),
        ("a\r\nb", "a\r\n\r\nb"),
    ] {
        let mut edits = difference(before, after);
        assert_eq!(apply(before, &mut edits, &mut []), after);
    }
}
