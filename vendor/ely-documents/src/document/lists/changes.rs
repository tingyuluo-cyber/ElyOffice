use std::ops::Range;

pub(super) type Edit = (Range<usize>, String);

pub(super) fn moved(at: usize, edits: &[Edit]) -> usize {
    let mut delta = 0isize;
    for (range, text) in edits {
        if at < range.start {
            break;
        }
        if at < range.end {
            return (range.start as isize + delta) as usize + (at - range.start).min(text.len());
        }
        delta += text.len() as isize - range.len() as isize;
    }
    (at as isize + delta) as usize
}

pub(super) fn apply(text: &str, edits: &mut [Edit], selections: &mut [Range<usize>]) -> String {
    edits.sort_by_key(|(range, _)| (range.start, range.end));
    for range in selections {
        *range = moved(range.start, edits)..moved(range.end, edits);
    }
    let mut out = String::new();
    let mut at = 0;
    for (range, written) in edits {
        out.push_str(&text[at..range.start]);
        out.push_str(written);
        at = range.end;
    }
    out.push_str(&text[at..]);
    out
}

pub(super) fn difference(before: &str, after: &str) -> Vec<Edit> {
    let mut at = 0;
    let mut edits: Vec<Edit> = Vec::new();
    for change in similar::TextDiff::from_lines(before, after).iter_all_changes() {
        match change.tag() {
            similar::ChangeTag::Equal => at += change.value().len(),
            tag => {
                if edits.last().is_none_or(|(range, _)| range.end != at) {
                    edits.push((at..at, String::new()));
                }
                let Some((range, written)) = edits.last_mut() else {
                    continue;
                };
                if tag == similar::ChangeTag::Delete {
                    at += change.value().len();
                    range.end = at;
                } else {
                    written.push_str(change.value());
                }
            }
        }
    }
    for (range, written) in &mut edits {
        let old = &before[range.clone()];
        let prefix = old
            .chars()
            .zip(written.chars())
            .take_while(|(a, b)| a == b)
            .map(|(ch, _)| ch.len_utf8())
            .sum::<usize>();
        let suffix = old[prefix..]
            .chars()
            .rev()
            .zip(written[prefix..].chars().rev())
            .take_while(|(a, b)| a == b)
            .map(|(ch, _)| ch.len_utf8())
            .sum::<usize>();
        range.start += prefix;
        range.end -= suffix;
        *written = written[prefix..written.len() - suffix].to_owned();
    }
    edits
}
