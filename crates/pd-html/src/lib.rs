//! HTML tokenization and parsing boundaries.

use pd_dom::Document;

/// Parses raw HTML into a DOM document.
#[derive(Debug, Default)]
pub struct HtmlParser;

impl HtmlParser {
    pub fn parse(&self, input: &str) -> Document {
        let summary = summarize_document(input);

        Document {
            title: summary.title,
            root: if summary.node_count > 0 { 1 } else { 0 },
            node_count: summary.node_count,
            text_bytes: summary.text_bytes,
        }
    }
}

#[derive(Debug, Default)]
struct HtmlSummary {
    title: String,
    node_count: u32,
    text_bytes: u32,
}

fn summarize_document(input: &str) -> HtmlSummary {
    let bytes = input.as_bytes();
    let mut idx = 0_usize;
    let mut node_count = 0_u32;
    let mut text_bytes = 0_u32;
    let mut title: Option<String> = None;

    while idx < bytes.len() {
        if bytes[idx] != b'<' {
            let next = find_byte(bytes, idx, b'<').unwrap_or(bytes.len());
            text_bytes = text_bytes.saturating_add(count_visible_text_bytes(&input[idx..next]));
            idx = next;
            continue;
        }

        if starts_with(bytes, idx, b"<!--") {
            idx = skip_comment(bytes, idx);
            continue;
        }

        if starts_with(bytes, idx, b"<!") {
            idx = skip_to_gt(bytes, idx.saturating_add(2));
            continue;
        }

        if starts_with(bytes, idx, b"<?") {
            idx = skip_processing_instruction(bytes, idx);
            continue;
        }

        let Some((tag, next_idx)) = parse_tag(bytes, idx) else {
            idx = idx.saturating_add(1);
            continue;
        };

        if !tag.is_end {
            node_count = node_count.saturating_add(1);

            if tag.name == "title" {
                let (raw_title, after_title) =
                    read_raw_text_until_end_tag(input, next_idx, "title");
                if title.is_none() {
                    let collapsed = collapse_whitespace(raw_title);
                    if !collapsed.is_empty() {
                        title = Some(collapsed);
                    }
                }
                idx = after_title;
                continue;
            }

            if !tag.self_closing && (tag.name == "script" || tag.name == "style") {
                let (_, after_raw) = read_raw_text_until_end_tag(input, next_idx, &tag.name);
                idx = after_raw;
                continue;
            }
        }

        idx = next_idx;
    }

    HtmlSummary {
        title: title.unwrap_or_default(),
        node_count,
        text_bytes,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedTag {
    name: String,
    is_end: bool,
    self_closing: bool,
}

fn parse_tag(bytes: &[u8], start: usize) -> Option<(ParsedTag, usize)> {
    if bytes.get(start).copied() != Some(b'<') {
        return None;
    }

    let mut idx = start.saturating_add(1);
    let mut is_end = false;
    if bytes.get(idx).copied() == Some(b'/') {
        is_end = true;
        idx = idx.saturating_add(1);
    }

    idx = skip_spaces(bytes, idx);
    let name_start = idx;
    while idx < bytes.len() && is_tag_name_char(bytes[idx]) {
        idx = idx.saturating_add(1);
    }

    if idx == name_start {
        return None;
    }

    let name = String::from_utf8_lossy(&bytes[name_start..idx]).to_ascii_lowercase();
    let mut in_single = false;
    let mut in_double = false;

    while idx < bytes.len() {
        let byte = bytes[idx];
        if in_single {
            if byte == b'\'' {
                in_single = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_double {
            if byte == b'"' {
                in_double = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        match byte {
            b'\'' => in_single = true,
            b'"' => in_double = true,
            b'>' => {
                let mut lookback = idx;
                while lookback > start && bytes[lookback - 1].is_ascii_whitespace() {
                    lookback -= 1;
                }
                let self_closing = lookback > start && bytes[lookback - 1] == b'/';
                return Some((
                    ParsedTag {
                        name,
                        is_end,
                        self_closing,
                    },
                    idx.saturating_add(1),
                ));
            }
            _ => {}
        }

        idx = idx.saturating_add(1);
    }

    None
}

fn read_raw_text_until_end_tag<'a>(
    input: &'a str,
    start: usize,
    tag_name: &str,
) -> (&'a str, usize) {
    let bytes = input.as_bytes();
    let tag_bytes = tag_name.as_bytes();
    let mut idx = start;

    while idx < bytes.len() {
        if bytes[idx] == b'<'
            && bytes.get(idx.saturating_add(1)).copied() == Some(b'/')
            && starts_with_ignore_ascii_case(bytes, idx.saturating_add(2), tag_bytes)
            && tag_name_boundary(bytes, idx.saturating_add(2 + tag_bytes.len()))
        {
            if let Some((_, end_idx)) = parse_tag(bytes, idx) {
                return (&input[start..idx], end_idx);
            }
        }

        idx = idx.saturating_add(1);
    }

    (&input[start..], bytes.len())
}

fn count_visible_text_bytes(segment: &str) -> u32 {
    segment
        .chars()
        .filter(|ch| !ch.is_control())
        .fold(0_u32, |acc, ch| acc.saturating_add(ch.len_utf8() as u32))
}

fn collapse_whitespace(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn skip_comment(bytes: &[u8], start: usize) -> usize {
    find_subslice(bytes, start.saturating_add(4), b"-->")
        .map(|end| end.saturating_add(3))
        .unwrap_or(bytes.len())
}

fn skip_processing_instruction(bytes: &[u8], start: usize) -> usize {
    if let Some(end) = find_subslice(bytes, start.saturating_add(2), b"?>") {
        return end.saturating_add(2);
    }

    skip_to_gt(bytes, start.saturating_add(2))
}

fn skip_to_gt(bytes: &[u8], mut idx: usize) -> usize {
    while idx < bytes.len() {
        if bytes[idx] == b'>' {
            return idx.saturating_add(1);
        }
        idx = idx.saturating_add(1);
    }

    bytes.len()
}

fn tag_name_boundary(bytes: &[u8], idx: usize) -> bool {
    match bytes.get(idx).copied() {
        None => true,
        Some(byte) => byte.is_ascii_whitespace() || byte == b'>' || byte == b'/',
    }
}

fn skip_spaces(bytes: &[u8], mut idx: usize) -> usize {
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx = idx.saturating_add(1);
    }
    idx
}

fn is_tag_name_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':')
}

fn starts_with(bytes: &[u8], idx: usize, pattern: &[u8]) -> bool {
    let end = idx.saturating_add(pattern.len());
    end <= bytes.len() && bytes[idx..end] == *pattern
}

fn starts_with_ignore_ascii_case(bytes: &[u8], idx: usize, pattern: &[u8]) -> bool {
    let end = idx.saturating_add(pattern.len());
    if end > bytes.len() {
        return false;
    }

    bytes[idx..end]
        .iter()
        .zip(pattern.iter())
        .all(|(left, right)| left.eq_ignore_ascii_case(right))
}

fn find_subslice(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from >= bytes.len() {
        return None;
    }

    bytes[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| from + offset)
}

fn find_byte(bytes: &[u8], from: usize, byte: u8) -> Option<usize> {
    bytes[from..]
        .iter()
        .position(|candidate| *candidate == byte)
        .map(|offset| from + offset)
}

#[cfg(test)]
mod tests {
    use super::HtmlParser;

    #[test]
    fn parses_title_and_root() {
        let parser = HtmlParser;
        let doc =
            parser.parse("<html><head><title> Pixel Dust </title></head><body>Hi</body></html>");
        assert_eq!(doc.title, "Pixel Dust");
        assert!(doc.has_root());
        assert!(doc.node_count >= 3);
    }

    #[test]
    fn handles_documents_without_title() {
        let parser = HtmlParser;
        let doc = parser.parse("plain text only");
        assert_eq!(doc.title, "");
        assert!(!doc.has_root());
        assert!(doc.text_bytes > 0);
    }

    #[test]
    fn skips_script_and_style_raw_text_in_text_count() {
        let parser = HtmlParser;
        let doc = parser.parse(
            "<html><body>Hello<script>var x = 42;</script><style>body{color:red}</style>World</body></html>",
        );

        assert!(doc.text_bytes >= 10);
        assert!(doc.text_bytes < 25);
    }

    #[test]
    fn parses_case_insensitive_title_with_attributes() {
        let parser = HtmlParser;
        let doc = parser.parse("<TiTlE data-a='1'>   Hello    PixelDust </tItLe>");
        assert_eq!(doc.title, "Hello PixelDust");
    }
}
