//! CSS tokenization and stylesheet model.

/// Style rules compiled from source CSS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleSheet {
    pub rules: Vec<String>,
}

impl StyleSheet {
    pub fn empty() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

/// Parses CSS source text.
#[derive(Debug, Default)]
pub struct CssParser;

impl CssParser {
    pub fn parse(&self, input: &str) -> StyleSheet {
        let sanitized = strip_comments_preserve_strings(input);
        let mut rules = Vec::new();
        parse_rules_recursive(&sanitized, &mut rules);
        StyleSheet { rules }
    }
}

fn parse_rules_recursive(input: &str, out: &mut Vec<String>) {
    let mut cursor = 0_usize;

    while let Some((selector_raw, body_raw, next_cursor)) = next_rule_block(input, cursor) {
        cursor = next_cursor;

        let selector = normalize_ws(selector_raw);
        if selector.is_empty() {
            continue;
        }

        if is_grouping_at_rule(&selector) {
            parse_rules_recursive(body_raw, out);
            continue;
        }

        let declarations = normalize_declarations(body_raw);
        if declarations.is_empty() {
            continue;
        }

        out.push(format!("{selector}{{{declarations}}}"));
    }
}

fn next_rule_block(input: &str, from: usize) -> Option<(&str, &str, usize)> {
    let start = skip_rule_separators(input, from);
    if start >= input.len() {
        return None;
    }

    let open = find_top_level_open_brace(input, start)?;
    let close = find_matching_brace(input, open)?;
    let selector = &input[start..open];
    let body = &input[open + 1..close];

    Some((selector, body, close + 1))
}

fn skip_rule_separators(input: &str, mut idx: usize) -> usize {
    while idx < input.len() {
        let byte = input.as_bytes()[idx];
        if byte.is_ascii_whitespace() || byte == b';' {
            idx = idx.saturating_add(1);
            continue;
        }
        break;
    }

    idx
}

fn find_top_level_open_brace(input: &str, from: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut idx = from;
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut paren_depth = 0_u32;
    let mut bracket_depth = 0_u32;

    while idx < bytes.len() {
        let byte = bytes[idx];

        if in_single {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'\'' {
                in_single = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_double {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'"' {
                in_double = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        match byte {
            b'\'' => in_single = true,
            b'"' => in_double = true,
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'{' if paren_depth == 0 && bracket_depth == 0 => return Some(idx),
            _ => {}
        }

        idx = idx.saturating_add(1);
    }

    None
}

fn find_matching_brace(input: &str, open_brace: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.get(open_brace).copied() != Some(b'{') {
        return None;
    }

    let mut idx = open_brace.saturating_add(1);
    let mut in_single = false;
    let mut in_double = false;
    let mut in_comment = false;
    let mut escape = false;
    let mut depth = 1_u32;

    while idx < bytes.len() {
        let byte = bytes[idx];
        let next = bytes.get(idx.saturating_add(1)).copied();

        if in_comment {
            if byte == b'*' && next == Some(b'/') {
                in_comment = false;
                idx = idx.saturating_add(2);
                continue;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_single {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'\'' {
                in_single = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_double {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'"' {
                in_double = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if byte == b'/' && next == Some(b'*') {
            in_comment = true;
            idx = idx.saturating_add(2);
            continue;
        }

        match byte {
            b'\'' => in_single = true,
            b'"' => in_double = true,
            b'{' => depth = depth.saturating_add(1),
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }

        idx = idx.saturating_add(1);
    }

    None
}

fn is_grouping_at_rule(selector: &str) -> bool {
    let lower = selector.to_ascii_lowercase();
    lower.starts_with("@media")
        || lower.starts_with("@supports")
        || lower.starts_with("@layer")
        || lower.starts_with("@document")
}

fn strip_comments_preserve_strings(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut idx = 0_usize;
    let mut out = Vec::with_capacity(input.len());
    let mut in_single = false;
    let mut in_double = false;
    let mut in_comment = false;
    let mut escape = false;

    while idx < bytes.len() {
        let byte = bytes[idx];
        let next = bytes.get(idx.saturating_add(1)).copied();

        if in_comment {
            if byte == b'*' && next == Some(b'/') {
                in_comment = false;
                idx = idx.saturating_add(2);
                continue;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_single {
            out.push(byte);
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'\'' {
                in_single = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_double {
            out.push(byte);
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'"' {
                in_double = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if byte == b'/' && next == Some(b'*') {
            in_comment = true;
            idx = idx.saturating_add(2);
            continue;
        }

        if byte == b'\'' {
            in_single = true;
            out.push(byte);
            idx = idx.saturating_add(1);
            continue;
        }

        if byte == b'"' {
            in_double = true;
            out.push(byte);
            idx = idx.saturating_add(1);
            continue;
        }

        out.push(byte);
        idx = idx.saturating_add(1);
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn normalize_ws(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_declarations(input: &str) -> String {
    let mut normalized = Vec::new();
    for declaration in split_top_level(input, ';') {
        let trimmed = declaration.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Some(colon_idx) = find_top_level_colon(trimmed) else {
            continue;
        };

        let name = normalize_ws(trimmed[..colon_idx].trim());
        let value = normalize_value(trimmed[colon_idx + 1..].trim());
        if name.is_empty() || value.is_empty() {
            continue;
        }

        normalized.push(format!("{name}:{value}"));
    }

    normalized.join(";")
}

fn split_top_level(input: &str, delimiter: char) -> Vec<&str> {
    let bytes = input.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0_usize;
    let mut idx = 0_usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut paren_depth = 0_u32;
    let mut bracket_depth = 0_u32;

    while idx < bytes.len() {
        let byte = bytes[idx];

        if in_single {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'\'' {
                in_single = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_double {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'"' {
                in_double = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        match byte {
            b'\'' => in_single = true,
            b'"' => in_double = true,
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {
                if byte == delimiter as u8 && paren_depth == 0 && bracket_depth == 0 {
                    parts.push(&input[start..idx]);
                    start = idx.saturating_add(1);
                }
            }
        }

        idx = idx.saturating_add(1);
    }

    if start <= input.len() {
        parts.push(&input[start..]);
    }

    parts
}

fn find_top_level_colon(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut idx = 0_usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut paren_depth = 0_u32;
    let mut bracket_depth = 0_u32;

    while idx < bytes.len() {
        let byte = bytes[idx];

        if in_single {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'\'' {
                in_single = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_double {
            if !escape && byte == b'\\' {
                escape = true;
            } else if !escape && byte == b'"' {
                in_double = false;
            } else {
                escape = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        match byte {
            b'\'' => in_single = true,
            b'"' => in_double = true,
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b':' if paren_depth == 0 && bracket_depth == 0 => return Some(idx),
            _ => {}
        }

        idx = idx.saturating_add(1);
    }

    None
}

fn normalize_value(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_single = false;
    let mut in_double = false;
    let mut last_was_space = false;
    let mut escape = false;

    for ch in input.chars() {
        if in_single {
            out.push(ch);
            if !escape && ch == '\\' {
                escape = true;
            } else if !escape && ch == '\'' {
                in_single = false;
            } else {
                escape = false;
            }
            continue;
        }

        if in_double {
            out.push(ch);
            if !escape && ch == '\\' {
                escape = true;
            } else if !escape && ch == '"' {
                in_double = false;
            } else {
                escape = false;
            }
            continue;
        }

        if ch == '\'' {
            in_single = true;
            last_was_space = false;
            out.push(ch);
            continue;
        }

        if ch == '"' {
            in_double = true;
            last_was_space = false;
            out.push(ch);
            continue;
        }

        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
            continue;
        }

        last_was_space = false;
        out.push(ch);
    }

    out.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::CssParser;

    #[test]
    fn parses_simple_rules() {
        let parser = CssParser;
        let sheet = parser.parse("body { color: red; } .card { padding: 8px; }");
        assert_eq!(sheet.rule_count(), 2);
        assert_eq!(sheet.rules[0], "body{color:red}");
        assert_eq!(sheet.rules[1], ".card{padding:8px}");
    }

    #[test]
    fn strips_comments_and_invalid_rules() {
        let parser = CssParser;
        let sheet = parser.parse("/* x */ p { font-size: 14px; } bad-rule {} div { }");
        assert_eq!(sheet.rule_count(), 1);
        assert_eq!(sheet.rules[0], "p{font-size:14px}");
    }

    #[test]
    fn parses_nested_media_rules() {
        let parser = CssParser;
        let sheet = parser.parse(
            "@media screen and (min-width: 800px) { .hero { margin: 0 auto; } .title { color: #fff; } }",
        );
        assert_eq!(sheet.rule_count(), 2);
        assert_eq!(sheet.rules[0], ".hero{margin:0 auto}");
        assert_eq!(sheet.rules[1], ".title{color:#fff}");
    }

    #[test]
    fn keeps_semicolons_inside_function_values() {
        let parser = CssParser;
        let sheet = parser.parse(
            r#".icon { background-image: url("data:image/svg+xml;utf8,<svg></svg>"); color: red; }"#,
        );
        assert_eq!(sheet.rule_count(), 1);
        assert_eq!(
            sheet.rules[0],
            r#".icon{background-image:url("data:image/svg+xml;utf8,<svg></svg>");color:red}"#
        );
    }
}
