#[cfg(test)]
mod tests {
    use super::{
        allow_page_script_source, allow_subresource_request, cookie_domain_matches,
        decode_text_response, format_js_error, format_script_origin, normalize_input_url,
        parse_charset_from_content_type, parse_charset_from_html_prefix, parse_set_cookie_header,
        same_navigation_target, same_origin, truncate_preview_text,
    };
    use pd_browser::Browser;

    #[test]
    fn parses_charset_from_content_type_header() {
        let content_type = "text/html; charset=ISO-8859-1";
        let parsed = parse_charset_from_content_type(content_type);
        assert_eq!(parsed.as_deref(), Some("ISO-8859-1"));
    }

    #[test]
    fn prefers_meta_charset_for_html() {
        let html = "<html><head><meta charset=\"UTF-8\"></head><body>hello</body></html>";
        let parsed = parse_charset_from_html_prefix(html.as_bytes());
        assert_eq!(parsed.as_deref(), Some("UTF-8"));
    }

    #[test]
    fn decodes_html_using_meta_charset_before_header_charset() {
        let html = b"<html><head><meta charset=\"UTF-8\"></head><body>\xE2\x82\xAC</body></html>";
        let decoded = decode_text_response(html, "text/html; charset=ISO-8859-1");
        assert!(decoded.contains("\u{20AC}"));
    }

    #[test]
    fn truncates_preview_without_breaking_utf8() {
        let text = "abc\u{20AC}";
        let truncated = truncate_preview_text(text, 5);
        assert!(truncated.is_char_boundary(truncated.len()));
    }

    #[test]
    fn normalizes_exaple_typo_host() {
        let normalized = normalize_input_url("exaple.com/docs?a=1".to_owned());
        assert_eq!(normalized, "https://example.com/docs?a=1");
    }

    #[test]
    fn keeps_example_host_when_valid() {
        let normalized = normalize_input_url("https://example.com/".to_owned());
        assert_eq!(normalized, "https://example.com/");
    }

    #[test]
    fn same_origin_checks_scheme_host_and_port() {
        assert!(same_origin(
            "https://example.com/docs",
            "https://example.com/other"
        ));
        assert!(!same_origin("https://example.com/", "http://example.com/"));
        assert!(!same_origin(
            "https://example.com/",
            "https://cdn.example.com/"
        ));
    }

    #[test]
    fn accepts_large_one_line_script_payload() {
        let script = "x".repeat(512 * 1024);
        assert!(allow_page_script_source(&script));
    }

    #[test]
    fn rejects_extremely_large_script_payload() {
        let script = "x".repeat(10 * 1024 * 1024);
        assert!(!allow_page_script_source(&script));
    }

    #[test]
    fn parses_set_cookie_domain_and_deletion() {
        let parsed = parse_set_cookie_header(
            "sid=abc; Domain=.google.com; Max-Age=3600",
            "www.google.com",
        );
        assert!(parsed.is_some());
        let parsed = parsed.expect("cookie should parse");
        assert_eq!(parsed.domain, "google.com");
        assert_eq!(parsed.name, "sid");
        assert_eq!(parsed.value, "abc");
        assert!(!parsed.delete);

        let deleted = parse_set_cookie_header("sid=; Max-Age=0", "www.google.com");
        assert!(deleted.is_some_and(|cookie| cookie.delete));
    }

    #[test]
    fn cookie_domain_matching_supports_parent_domains() {
        assert!(cookie_domain_matches("www.google.com", "google.com"));
        assert!(cookie_domain_matches("google.com", "google.com"));
        assert!(!cookie_domain_matches("badgoogle.com", "google.com"));
    }

    #[test]
    fn navigation_target_comparison_ignores_minor_url_formatting() {
        assert!(same_navigation_target(
            "https://example.com/search?q=rust",
            "https://example.com/search?q=rust#section"
        ));
        assert!(!same_navigation_target(
            "https://example.com/search?q=rust",
            "https://example.com/search?q=rust+lang"
        ));
    }

    #[test]
    fn subresource_policy_allows_cross_origin_https_assets() {
        let browser = Browser::new().unwrap_or_else(|_| unreachable!());
        assert!(allow_subresource_request(
            &browser,
            "https://www.google.com/",
            "https://www.gstatic.com/myscript.js"
        ));
    }

    #[test]
    fn subresource_policy_blocks_https_to_http_downgrade() {
        let browser = Browser::new().unwrap_or_else(|_| unreachable!());
        assert!(!allow_subresource_request(
            &browser,
            "https://www.example.com/",
            "http://cdn.example.com/app.js"
        ));
    }

    #[test]
    fn script_origin_log_formatting_drops_query_noise() {
        let formatted = format_script_origin(
            "https://www.google.com/xjs/_/js/k=xjs.hd.en/path?very=long&token=abc#frag",
        );
        assert!(formatted.starts_with("https://www.google.com/xjs/_/js/k=xjs.hd.en/path"));
        assert!(!formatted.contains("token="));
        assert!(!formatted.contains("#frag"));
    }

    #[test]
    fn js_error_messages_are_sanitized_for_ui() {
        let message = format_js_error(
            "https://example.com/script.js?huge=true",
            "TypeError:\n  cannot\n   read\tproperty   'x' of undefined",
        );
        assert!(message.contains("TypeError: cannot read property 'x' of undefined"));
        assert!(!message.contains('\n'));
        assert!(!message.contains("?huge=true"));
    }
}
