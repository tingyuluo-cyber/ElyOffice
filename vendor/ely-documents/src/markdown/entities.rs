use std::ops::Range;

use crate::tree::html;

/// A complete named reference without its semicolon, in Markdown prose.
pub(crate) fn shorthand(text: &str) -> Option<(&'static str, usize)> {
    let body = text.strip_prefix('&')?;
    let len = body
        .bytes()
        .take(html::LONGEST + 1)
        .take_while(u8::is_ascii_alphanumeric)
        .count();
    if !(1..=html::LONGEST).contains(&len) || body[len..].starts_with(';') {
        return None;
    }
    Some((html::named(&body[..len], true)?, len + 1))
}

/// Whether Markdown escapes the byte at `at` with a backslash.
pub(crate) fn escaped(source: &str, at: usize) -> bool {
    let slashes = source.as_bytes()[..at]
        .iter()
        .rev()
        .take_while(|&&byte| byte == b'\\')
        .count();
    !slashes.is_multiple_of(2)
}

/// References in a verbatim parser event, with offsets relative to the event.
pub(super) fn shorthands(
    source: &str,
    span: Range<usize>,
) -> impl Iterator<Item = (Range<usize>, &'static str)> {
    source[span.clone()]
        .match_indices('&')
        .filter_map(move |(at, _)| {
            let start = span.start + at;
            let (value, len) = shorthand(&source[start..])?;
            (start + len <= span.end && !escaped(source, start)).then_some((at..at + len, value))
        })
}
