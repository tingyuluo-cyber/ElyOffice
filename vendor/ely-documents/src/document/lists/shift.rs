use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

use super::{
    changes::{Edit, apply, moved},
    index::{Index, columns, container, end, padding, row},
};

fn cut(text: &str, columns: usize) -> usize {
    let mut width = 0;
    for (at, ch) in text.char_indices() {
        if width >= columns {
            return at;
        }
        width = match ch {
            ' ' => width + 1,
            '\t' => (width / 4 + 1) * 4,
            _ => return at,
        };
    }
    text.len()
}

pub(super) fn renumber(
    before: &Index,
    text: &str,
    edits: &[Edit],
    affected: &[usize],
    moving: &[usize],
    selections: &mut [Range<usize>],
) -> String {
    let after = Index::new(text);
    let moved_items: BTreeSet<_> = moving
        .iter()
        .map(|id| moved(before.items[*id].marker.start, edits))
        .collect();
    let peers: BTreeSet<_> = affected
        .iter()
        .flat_map(|id| {
            before.groups[before.items[*id].group]
                .items
                .iter()
                .map(|peer| moved(before.items[*peer].marker.start, edits))
        })
        .collect();
    let mut numbers = Vec::new();
    let mut continuation: BTreeMap<(usize, usize), isize> = BTreeMap::new();
    for group in &after.groups {
        if !group.ordered
            || !group
                .items
                .iter()
                .any(|id| peers.contains(&after.items[*id].marker.start))
        {
            continue;
        }
        let start = group
            .items
            .iter()
            .find_map(|id| {
                let marker = after.items[*id].marker.start;
                if moved_items.contains(&marker) {
                    return None;
                }
                before
                    .items
                    .iter()
                    .find(|item| moved(item.marker.start, edits) == marker)
                    .map(|item| before.groups[item.group].start)
            })
            .unwrap_or(1);
        for (offset, id) in group.items.iter().enumerate() {
            let item = &after.items[*id];
            let range = item.marker.start..item.marker.end - 1;
            let number = (u64::from(start) + offset as u64).to_string();
            if text[range.clone()] != number {
                let delta = number.len() as isize - range.len() as isize;
                if delta != 0 {
                    let quotes = text[item.row..item.marker.start].matches('>').count();
                    let mut at = item.row;
                    let last = row(text, item.position.end.saturating_sub(1).max(at));
                    while at < last {
                        at += text[at..].find('\n').map_or(text.len() - at, |end| end + 1);
                        *continuation.entry((at, quotes)).or_default() += delta;
                    }
                }
                numbers.push((range, number));
            }
        }
    }
    for ((at, quotes), delta) in continuation {
        let line = &text[at..end(text, at)];
        if delta == 0 || line.trim().is_empty() {
            continue;
        }
        let start = container(text, at, quotes);
        let rest = &text[start..end(text, start)];
        let marker = start + rest.len() - rest.trim_start_matches([' ', '\t']).len();
        let width = columns(&text[start..marker]);
        let width = (width as isize + delta).max(0) as usize;
        numbers.push((start..marker, " ".repeat(width)));
    }
    apply(text, &mut numbers, selections)
}

fn selected_rows(text: &str, selections: &[Range<usize>]) -> BTreeSet<usize> {
    let mut rows = BTreeSet::new();
    let multiline = selections
        .iter()
        .any(|range| row(text, range.start) != row(text, range.end));
    for range in selections {
        let (first, last) = (range.start.min(range.end), range.start.max(range.end));
        let last = row(text, last.saturating_sub(usize::from(first != last)));
        let mut at = row(text, first);
        loop {
            if !multiline || !text[at..end(text, at)].trim().is_empty() {
                rows.insert(at);
            }
            if at >= last {
                break;
            }
            at += text[at..].find('\n').map_or(text.len() - at, |end| end + 1);
        }
    }
    rows
}

pub(super) fn plain(
    text: &str,
    mut selections: Vec<Range<usize>>,
    deeper: bool,
) -> (Vec<Edit>, Vec<Range<usize>>) {
    let edits: Vec<_> = selected_rows(text, &selections)
        .into_iter()
        .filter_map(|at| {
            if deeper {
                Some((at..at, "    ".to_owned()))
            } else {
                let bytes = cut(&text[at..end(text, at)], 4);
                (bytes != 0).then(|| (at..at + bytes, String::new()))
            }
        })
        .collect();
    for range in &mut selections {
        *range = moved(range.start, &edits)..moved(range.end, &edits);
    }
    (edits, selections)
}

pub(super) fn shift(
    text: &str,
    mut selections: Vec<Range<usize>>,
    deeper: bool,
) -> (String, Vec<Range<usize>>) {
    let index = Index::new(text);
    let mut rows = selected_rows(text, &selections);
    let selected: BTreeSet<_> = index
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| rows.contains(&item.row))
        .map(|(id, _)| id)
        .collect();
    let moving: Vec<_> = selected
        .iter()
        .copied()
        .filter(|id| {
            let mut parent = index.items[*id].parent;
            while let Some(id) = parent {
                if selected.contains(&id) {
                    return false;
                }
                parent = index.items[id].parent;
            }
            true
        })
        .collect();
    let mut shifts = BTreeMap::new();
    let mut changed = Vec::new();
    for id in moving {
        let item = &index.items[id];
        let width = if deeper {
            let previous = index.groups[item.group]
                .items
                .iter()
                .copied()
                .take_while(|peer| *peer != id)
                .filter(|peer| !selected.contains(peer))
                .last();
            let Some(previous) = previous else {
                let mut at = item.row;
                while at < item.position.end {
                    rows.remove(&at);
                    let next = text[at..].find('\n').map_or(text.len(), |end| at + end + 1);
                    if next == at {
                        break;
                    }
                    at = next;
                }
                continue;
            };
            let previous = &index.items[previous];
            columns(&text[previous.marker.start..previous.body]).max(4)
        } else {
            index.parent(text, id).map_or_else(
                || {
                    let start = padding(text, item.row, item.marker.start);
                    columns(&text[start..item.marker.start]).min(4)
                },
                |parent| {
                    let parent = &index.items[parent];
                    columns(&text[item.row..item.marker.start])
                        .saturating_sub(columns(&text[parent.row..parent.marker.start]))
                },
            )
        };
        let mut at = item.row;
        let quotes = text[item.row..item.marker.start].matches('>').count();
        let last = row(text, item.position.end.saturating_sub(1).max(item.row));
        loop {
            let line = &text[at..end(text, at)];
            if width != 0 && !line.trim().is_empty() {
                shifts.insert(at, (container(text, at, quotes), width));
            }
            rows.remove(&at);
            if at >= last {
                break;
            }
            at += text[at..].find('\n').map_or(text.len() - at, |end| end + 1);
        }
        if width != 0 {
            changed.push(id);
        }
    }
    for at in rows {
        shifts.insert(at, (at, 4));
    }
    let mut edits: Vec<Edit> = shifts
        .into_iter()
        .filter_map(|(_, (at, width))| {
            if deeper {
                Some((at..at, " ".repeat(width)))
            } else {
                let bytes = cut(&text[at..end(text, at)], width);
                (bytes != 0).then(|| (at..at + bytes, String::new()))
            }
        })
        .collect();
    let groups: BTreeSet<_> = changed.iter().map(|id| index.items[*id].group).collect();
    for group in groups {
        if !index.groups[group].ordered {
            continue;
        }
        for id in &index.groups[group].items {
            let item = &index.items[*id];
            let range = item.marker.start..item.marker.end - 1;
            let seed = format!("{:0>width$}", 1, width = range.len());
            if text[range.clone()] != seed {
                edits.push((range, seed));
            }
        }
    }
    let shifted = apply(text, &mut edits, &mut selections);
    let shifted = renumber(
        &index,
        &shifted,
        &edits,
        &changed,
        &changed,
        &mut selections,
    );
    (shifted, selections)
}
