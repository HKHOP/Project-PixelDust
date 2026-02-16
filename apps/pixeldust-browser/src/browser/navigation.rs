use super::*;

pub(super) fn execute_navigation(
    raw_url: &str,
    trust_store: TrustStoreSelection,
    ocsp_required: bool,
    cache: Arc<Mutex<HttpCache>>,
) -> Result<PageView, String> {
    let browser = pd_browser::Browser::new().map_err(|error| error.to_string())?;
    let policy = browser
        .network
        .tls_policy
        .clone()
        .with_trust_store_mode(trust_store.as_policy_mode())
        .with_ocsp_stapling_required(ocsp_required);

    let mut client = browser
        .network
        .http11_client_with_tls_policy(policy.clone())
        .map_err(|error| error.to_string())?;
    let mut current_url = raw_url.to_owned();
    let mut js_redirects_remaining = MAX_PAGE_JS_REDIRECTS;

    loop {
        let page = fetch_with_redirects(
            &browser,
            &mut client,
            &policy,
            &current_url,
            MAX_REDIRECTS,
            &cache,
        )?;

        let is_html = page.content_type.to_ascii_lowercase().contains("text/html")
            || page
                .content_type
                .to_ascii_lowercase()
                .contains("application/xhtml+xml");

        let decoded_body = decode_text_response(&page.body, &page.content_type);
        let body_preview = truncate_preview_text(&decoded_body, MAX_BODY_PREVIEW_BYTES);
        let mut html_document = None;
        let mut static_text_fallback = None;
        let mut decoded_images = Vec::new();
        let mut subresource_stats = SubresourceStats::default();
        let mut js_execution = JsExecutionStats::default();
        let mut renderer_draw_calls = None;
        let mut js_redirect_target: Option<String> = None;

        if is_html {
            js_execution.enabled = true;
            let mut document = simple_html::HtmlDocument::parse(&decoded_body);
            let manifest = document.collect_subresources(&page.final_url);
            subresource_stats.inline_style_tags = document.inline_style_tag_count();
            let mut stylesheet_sources = String::new();
            let mut script_sources = Vec::new();

            for stylesheet_url in manifest.stylesheets.iter().take(MAX_STYLESHEET_FETCHES) {
                if !allow_subresource_request(&browser, &page.final_url, stylesheet_url) {
                    subresource_stats.blocked = subresource_stats.blocked.saturating_add(1);
                    continue;
                }

                let stylesheet = fetch_with_redirects(
                    &browser,
                    &mut client,
                    &policy,
                    stylesheet_url,
                    MAX_SUBRESOURCE_REDIRECTS,
                    &cache,
                );
                let Ok(stylesheet) = stylesheet else {
                    continue;
                };

                if !is_success_status(stylesheet.status_code) {
                    continue;
                }

                if !is_css_content_type(&stylesheet.content_type, &stylesheet.final_url) {
                    continue;
                }

                let source = decode_text_response(&stylesheet.body, &stylesheet.content_type);
                document.append_stylesheet_source(&source);
                stylesheet_sources.push_str(&source);
                stylesheet_sources.push('\n');
                subresource_stats.stylesheets_loaded =
                    subresource_stats.stylesheets_loaded.saturating_add(1);
            }

            let pipeline_renderer = pd_renderer::RendererProcess::default();
            let frame = pipeline_renderer.render_document(&decoded_body, &stylesheet_sources);
            renderer_draw_calls = Some(frame.draw_calls);

            subresource_stats.css_rules_total = document.css_rule_count();
            let script_plan = document.collect_script_descriptors(&page.final_url);
            let total_scripts = script_plan.len();
            let overflow_scripts = total_scripts.saturating_sub(MAX_SCRIPT_FETCHES);
            let mut budget_skipped_scripts = 0_usize;
            let mut inline_index = 0_usize;

            for descriptor in script_plan.into_iter().take(MAX_SCRIPT_FETCHES) {
                match descriptor {
                    simple_html::ScriptDescriptor::Inline { source } => {
                        inline_index = inline_index.saturating_add(1);
                        if source.trim().is_empty() {
                            continue;
                        }
                        if !allow_page_script_source(&source) {
                            budget_skipped_scripts = budget_skipped_scripts.saturating_add(1);
                            continue;
                        }
                        script_sources.push(ScriptSource {
                            origin: format!("inline-script:{inline_index}"),
                            source,
                        });
                    }
                    simple_html::ScriptDescriptor::External { url } => {
                        if !allow_subresource_request(&browser, &page.final_url, &url) {
                            subresource_stats.blocked = subresource_stats.blocked.saturating_add(1);
                            continue;
                        }

                        let script = fetch_with_redirects(
                            &browser,
                            &mut client,
                            &policy,
                            &url,
                            MAX_SUBRESOURCE_REDIRECTS,
                            &cache,
                        );
                        let Ok(script) = script else {
                            continue;
                        };

                        if is_success_status(script.status_code) {
                            subresource_stats.scripts_loaded =
                                subresource_stats.scripts_loaded.saturating_add(1);
                        }

                        if !is_success_status(script.status_code) {
                            continue;
                        }

                        if !is_javascript_content_type(&script.content_type, &script.final_url) {
                            continue;
                        }

                        let source = decode_text_response(&script.body, &script.content_type);
                        if source.trim().is_empty() {
                            continue;
                        }
                        if !allow_page_script_source(&source) {
                            budget_skipped_scripts = budget_skipped_scripts.saturating_add(1);
                            continue;
                        }

                        script_sources.push(ScriptSource {
                            origin: format_script_origin(&script.final_url),
                            source,
                        });
                    }
                }
            }

            if !script_sources.is_empty() {
                let host = JsHostEnvironment {
                    page_url: page.final_url.clone(),
                    document_title: document.title.clone().unwrap_or_default(),
                    cookie_header: cookie_header_for_url(&cache, &page.final_url),
                    elements_by_id: document
                        .collect_id_elements(256)
                        .into_iter()
                        .map(|element| JsHostElement {
                            id: element.id,
                            tag_name: element.tag_name,
                            text_content: element.text_content,
                            attributes: element.attributes,
                        })
                        .collect(),
                };
                let js_runtime = JsRuntime::new(page_js_runtime_config());
                let output = js_runtime.execute_scripts_with_host(&host, &script_sources);
                js_execution = js_stats_from_report(true, output.report);
                js_execution.scripts_seen = js_execution
                    .scripts_seen
                    .saturating_add(overflow_scripts)
                    .saturating_add(budget_skipped_scripts);
                js_execution.scripts_skipped = js_execution
                    .scripts_skipped
                    .saturating_add(overflow_scripts)
                    .saturating_add(budget_skipped_scripts);

                if let Some(cookie_snapshot) = output.document_cookie.as_deref() {
                    merge_document_cookie_snapshot(&cache, &page.final_url, cookie_snapshot);
                }

                if let Some(new_title) = output
                    .document_title
                    .map(|title| title.trim().to_owned())
                    .filter(|title| !title.is_empty())
                {
                    document.title = Some(new_title);
                }

                js_redirect_target = output
                    .location_href
                    .as_deref()
                    .and_then(|href| resolve_js_location(&page.final_url, href))
                    .filter(|next| !same_navigation_target(next, &page.final_url));
            } else if overflow_scripts > 0 || budget_skipped_scripts > 0 {
                js_execution.scripts_seen = total_scripts;
                js_execution.scripts_skipped =
                    overflow_scripts.saturating_add(budget_skipped_scripts);
            }

            for image_url in manifest.images.iter().take(MAX_IMAGE_FETCHES) {
                if !allow_subresource_request(&browser, &page.final_url, image_url) {
                    subresource_stats.blocked = subresource_stats.blocked.saturating_add(1);
                    continue;
                }

                let image = fetch_with_redirects(
                    &browser,
                    &mut client,
                    &policy,
                    image_url,
                    MAX_SUBRESOURCE_REDIRECTS,
                    &cache,
                );
                let Ok(image) = image else {
                    continue;
                };

                if !is_success_status(image.status_code) {
                    continue;
                }

                if let Some(decoded) =
                    decode_image_asset(&image.final_url, &image.content_type, &image.body)
                {
                    decoded_images.push(decoded);
                    subresource_stats.images_loaded =
                        subresource_stats.images_loaded.saturating_add(1);
                }
            }

            if document.renderable_text_len() == 0 {
                let fallback = document.static_text_fallback(MAX_STATIC_FALLBACK_CHARS);
                if !fallback.is_empty() {
                    static_text_fallback = Some(fallback);
                }
            }

            html_document = Some(document);
        }

        if let Some(next_url) = js_redirect_target {
            if js_redirects_remaining > 0 {
                js_redirects_remaining = js_redirects_remaining.saturating_sub(1);
                current_url = next_url;
                continue;
            }

            if js_execution.errors.len() < MAX_JS_ERROR_LOGS {
                js_execution.errors.push(format_js_error(
                    "runtime",
                    &format!("js redirect limit reached while navigating to {next_url}"),
                ));
            }
        }

        let title = html_document
            .as_ref()
            .and_then(|doc| doc.title.clone())
            .or_else(|| extract_html_title(&body_preview));

        return Ok(PageView {
            final_url: page.final_url,
            status_code: page.status_code,
            http_version: page.http_version,
            content_type: page.content_type,
            headers: page.headers,
            body_bytes: page.body.len(),
            body_preview,
            title,
            html_document,
            static_text_fallback,
            decoded_images,
            subresource_stats,
            js_execution,
            renderer_draw_calls,
        });
    }
}

fn fetch_with_redirects(
    browser: &pd_browser::Browser,
    client: &mut Http11Client,
    policy: &pd_net::tls::StrictTlsPolicy,
    raw_url: &str,
    max_redirects: usize,
    cache: &Arc<Mutex<HttpCache>>,
) -> Result<FetchedResponse, String> {
    let mut current_url = raw_url.to_owned();
    let mut redirects_followed = 0_usize;

    loop {
        let cached = lookup_cache(cache, &current_url);
        if let CacheLookup::Fresh(response) = cached {
            return Ok(response);
        }

        let request_policy = effective_tls_policy_for_request(policy, &current_url);
        let mut prepared = browser
            .network
            .prepare_get_with_tls_policy(&current_url, &request_policy)
            .map_err(|error| error.to_string())?;
        attach_cookie_header(cache, &current_url, &mut prepared.request.headers)?;

        if let CacheLookup::Stale {
            etag,
            last_modified,
            ..
        } = &cached
        {
            add_conditional_request_headers(
                &mut prepared.request.headers,
                etag.as_deref(),
                last_modified.as_deref(),
            )?;
        }

        let response = client
            .execute(prepared)
            .map_err(|error| error.to_string())?;
        let headers: Vec<(String, String)> = response
            .headers
            .iter()
            .map(|header| (header.name.clone(), header.value.clone()))
            .collect();
        let status_code = response.status.as_u16();
        store_response_cookies(cache, &current_url, &headers);

        if status_code == 304 {
            if let CacheLookup::Stale { cached, .. } = cached {
                refresh_cached_metadata(cache, &current_url, &headers);
                return Ok(cached);
            }
        }

        if is_redirect_status(status_code) {
            let location = headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("location"))
                .map(|(_, value)| value.clone());

            if let Some(location) = location {
                if redirects_followed >= max_redirects {
                    return Err(format!(
                        "Too many redirects (>{max_redirects}) while loading {raw_url}"
                    ));
                }

                current_url = resolve_redirect_url(&current_url, &location)?;
                redirects_followed = redirects_followed.saturating_add(1);
                continue;
            }
        }

        let content_type = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| "unknown".to_owned());

        let fetched = FetchedResponse {
            final_url: current_url,
            status_code,
            http_version: response.version.as_str().to_owned(),
            headers,
            content_type,
            body: response.body,
        };

        maybe_store_cache_entry(cache, &fetched);
        return Ok(fetched);
    }
}

fn effective_tls_policy_for_request(
    base: &pd_net::tls::StrictTlsPolicy,
    request_url: &str,
) -> pd_net::tls::StrictTlsPolicy {
    let mut out = base.clone();
    if is_local_network_url(request_url) {
        // Local development endpoints commonly run over plain HTTP and/or without OCSP/SNI.
        out.https_only_mode = false;
        out.require_ocsp_stapling = false;
        out.require_sni = false;
    }
    out
}

fn lookup_cache(cache: &Arc<Mutex<HttpCache>>, url: &str) -> CacheLookup {
    let guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let Some(entry) = guard.entries.get(url) else {
        return CacheLookup::Miss;
    };

    if entry.is_fresh() {
        return CacheLookup::Fresh(entry.response.clone());
    }

    if entry.etag.is_some() || entry.last_modified.is_some() {
        return CacheLookup::Stale {
            cached: entry.response.clone(),
            etag: entry.etag.clone(),
            last_modified: entry.last_modified.clone(),
        };
    }

    CacheLookup::Miss
}

fn add_conditional_request_headers(
    headers: &mut Vec<Header>,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<(), String> {
    if let Some(value) = etag {
        headers.push(Header::new("If-None-Match", value).map_err(|error| error.to_string())?);
    }
    if let Some(value) = last_modified {
        headers.push(Header::new("If-Modified-Since", value).map_err(|error| error.to_string())?);
    }
    Ok(())
}

fn maybe_store_cache_entry(cache: &Arc<Mutex<HttpCache>>, response: &FetchedResponse) {
    if !is_success_status(response.status_code) {
        return;
    }

    let cache_control = header_value(&response.headers, "cache-control").unwrap_or_default();
    if contains_cache_directive(cache_control, "no-store") {
        return;
    }

    let max_age = parse_max_age(cache_control);
    let etag = header_value(&response.headers, "etag").map(ToOwned::to_owned);
    let last_modified = header_value(&response.headers, "last-modified").map(ToOwned::to_owned);

    if max_age.is_none() && etag.is_none() && last_modified.is_none() {
        return;
    }

    let mut guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if guard.entries.len() >= MAX_CACHE_ENTRIES {
        let oldest = guard
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.stored_at)
            .map(|(key, _)| key.clone());
        if let Some(oldest_key) = oldest {
            guard.entries.remove(&oldest_key);
        }
    }

    guard.entries.insert(
        response.final_url.clone(),
        CachedResponse {
            response: response.clone(),
            etag,
            last_modified,
            max_age,
            stored_at: Instant::now(),
        },
    );
}

fn refresh_cached_metadata(
    cache: &Arc<Mutex<HttpCache>>,
    url: &str,
    response_headers: &[(String, String)],
) {
    let mut guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let mut remove_entry = false;
    if let Some(cache_control) = header_value(response_headers, "cache-control") {
        if contains_cache_directive(cache_control, "no-store") {
            remove_entry = true;
        }
    }

    if remove_entry {
        guard.entries.remove(url);
        return;
    }

    let Some(entry) = guard.entries.get_mut(url) else {
        return;
    };

    if let Some(cache_control) = header_value(response_headers, "cache-control") {
        if let Some(max_age) = parse_max_age(cache_control) {
            entry.max_age = Some(max_age);
        }
    }

    if let Some(etag) = header_value(response_headers, "etag") {
        entry.etag = Some(etag.to_owned());
    }
    if let Some(last_modified) = header_value(response_headers, "last-modified") {
        entry.last_modified = Some(last_modified.to_owned());
    }

    entry.stored_at = Instant::now();
}

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn contains_cache_directive(value: &str, directive: &str) -> bool {
    value
        .split(',')
        .map(str::trim)
        .any(|token| token.eq_ignore_ascii_case(directive))
}

fn parse_max_age(cache_control: &str) -> Option<Duration> {
    for directive in cache_control.split(',').map(str::trim) {
        let Some((name, value)) = directive.split_once('=') else {
            continue;
        };

        if !name.trim().eq_ignore_ascii_case("max-age") {
            continue;
        }

        let trimmed = value.trim().trim_matches('"');
        if let Ok(seconds) = trimmed.parse::<u64>() {
            return Some(Duration::from_secs(seconds));
        }
    }

    None
}

impl CachedResponse {
    fn is_fresh(&self) -> bool {
        let Some(max_age) = self.max_age else {
            return false;
        };

        self.stored_at.elapsed() < max_age
    }
}

fn is_success_status(status: u16) -> bool {
    (200..=299).contains(&status)
}

fn allow_subresource_request(
    browser: &pd_browser::Browser,
    document_url: &str,
    candidate_url: &str,
) -> bool {
    let Ok(candidate) = Url::parse(candidate_url) else {
        return false;
    };
    if !matches!(candidate.scheme(), "http" | "https") {
        return false;
    }

    let Some(host) = candidate.host_str() else {
        return false;
    };

    if browser.privacy.should_block_host(host) {
        return false;
    }

    if !browser.security.enforce_site_isolation {
        return true;
    }

    if same_origin(document_url, candidate_url) {
        return true;
    }

    // Keep downgrade protections while allowing cross-origin HTTPS subresources
    // (required by modern pages that split assets across dedicated hosts/CDNs).
    let Ok(document) = Url::parse(document_url) else {
        return false;
    };
    if !matches!(document.scheme(), "http" | "https") {
        return false;
    }
    if document.scheme() == "https" && candidate.scheme() != "https" {
        return false;
    }

    true
}

fn same_origin(left: &str, right: &str) -> bool {
    let Ok(left) = Url::parse(left) else {
        return false;
    };
    let Ok(right) = Url::parse(right) else {
        return false;
    };

    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn is_css_content_type(content_type: &str, final_url: &str) -> bool {
    if content_type.to_ascii_lowercase().contains("text/css") {
        return true;
    }

    final_url.to_ascii_lowercase().contains(".css")
}

fn is_javascript_content_type(content_type: &str, final_url: &str) -> bool {
    let lower = content_type.to_ascii_lowercase();
    if lower.contains("javascript")
        || lower.contains("ecmascript")
        || lower.contains("application/x-javascript")
    {
        return true;
    }

    let url_lower = final_url.to_ascii_lowercase();
    url_lower.contains(".js") || url_lower.contains(".mjs")
}

fn format_script_origin(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "script".to_owned();
    }

    if let Ok(mut url) = Url::parse(trimmed) {
        url.set_query(None);
        url.set_fragment(None);
        return clamp_log_text(&url.to_string(), MAX_JS_ERROR_ORIGIN_CHARS);
    }

    clamp_log_text(trimmed, MAX_JS_ERROR_ORIGIN_CHARS)
}

fn format_js_error(origin: &str, message: &str) -> String {
    let clean_origin = format_script_origin(origin);
    let clean_message = normalize_log_whitespace(message);
    let clean_message = clamp_log_text(&clean_message, MAX_JS_ERROR_MESSAGE_CHARS);
    format!("{clean_origin}: {clean_message}")
}

fn normalize_log_whitespace(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn clamp_log_text(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    if input.chars().count() <= max_chars {
        return input.to_owned();
    }

    let mut clipped = input.chars().take(max_chars).collect::<String>();
    clipped.push_str("...");
    clipped
}

fn js_stats_from_report(enabled: bool, report: JsExecutionReport) -> JsExecutionStats {
    let errors = report
        .errors
        .into_iter()
        .map(|error| format_js_error(&error.origin, &error.message))
        .collect::<Vec<_>>();

    JsExecutionStats {
        enabled,
        scripts_seen: report.scripts_seen,
        scripts_executed: report.scripts_executed,
        scripts_failed: report.scripts_failed,
        scripts_skipped: report.scripts_skipped,
        event_dispatches: 0,
        event_failures: 0,
        errors,
    }
}

pub(super) fn dispatch_dom_events(
    page: &mut PageView,
    events: &[simple_html::DomEventRequest],
) -> Option<String> {
    if events.is_empty() {
        return None;
    }
    let document = page.html_document.as_ref()?;

    let mut event_scripts = Vec::new();
    for (index, event) in events.iter().take(MAX_DOM_EVENTS_PER_FRAME).enumerate() {
        if event.inline_handler.len() > MAX_INLINE_EVENT_HANDLER_BYTES {
            page.js_execution.event_failures = page.js_execution.event_failures.saturating_add(1);
            if page.js_execution.errors.len() < MAX_JS_ERROR_LOGS {
                page.js_execution.errors.push(format_js_error(
                    &format!(
                        "dom-event:{}:{}",
                        match event.kind {
                            simple_html::DomEventKind::Click => "click",
                            simple_html::DomEventKind::Input => "input",
                            simple_html::DomEventKind::Submit => "submit",
                        },
                        index + 1
                    ),
                    &format!(
                        "inline handler too large ({} bytes)",
                        event.inline_handler.len()
                    ),
                ));
            }
            continue;
        }

        let event_type = match event.kind {
            simple_html::DomEventKind::Click => "click",
            simple_html::DomEventKind::Input => "input",
            simple_html::DomEventKind::Submit => "submit",
        };
        let target_id = event.target_id.as_deref().unwrap_or("");
        let script = build_inline_event_script(event_type, target_id, &event.inline_handler);
        event_scripts.push(ScriptSource {
            origin: format!("dom-event:{}:{}", event_type, index + 1),
            source: script,
        });
    }

    if event_scripts.is_empty() {
        return None;
    }

    let host = JsHostEnvironment {
        page_url: page.final_url.clone(),
        document_title: page.title.clone().unwrap_or_default(),
        cookie_header: String::new(),
        elements_by_id: document
            .collect_id_elements(256)
            .into_iter()
            .map(|element| JsHostElement {
                id: element.id,
                tag_name: element.tag_name,
                text_content: element.text_content,
                attributes: element.attributes,
            })
            .collect(),
    };

    let runtime = JsRuntime::new(event_js_runtime_config());
    let output = runtime.execute_scripts_with_host(&host, &event_scripts);
    page.js_execution.event_dispatches = page
        .js_execution
        .event_dispatches
        .saturating_add(events.len().min(MAX_DOM_EVENTS_PER_FRAME));
    page.js_execution.event_failures = page
        .js_execution
        .event_failures
        .saturating_add(output.report.scripts_failed);

    for error in output.report.errors {
        if page.js_execution.errors.len() >= MAX_JS_ERROR_LOGS {
            break;
        }
        page.js_execution
            .errors
            .push(format_js_error(&error.origin, &error.message));
    }

    if let Some(new_title) = output
        .document_title
        .map(|title| title.trim().to_owned())
        .filter(|title| !title.is_empty())
    {
        page.title = Some(new_title.clone());
        if let Some(doc) = page.html_document.as_mut() {
            doc.title = Some(new_title);
        }
    }

    output
        .location_href
        .as_deref()
        .and_then(|href| resolve_js_location(&page.final_url, href))
}

fn allow_page_script_source(source: &str) -> bool {
    if source.is_empty() {
        return false;
    }
    if source.len() > MAX_PAGE_SCRIPT_HARD_BYTES {
        return false;
    }

    // Reject embedded NUL bytes that typically indicate binary/non-text payloads.
    !source.as_bytes().contains(&0)
}

fn page_js_runtime_config() -> JsRuntimeConfig {
    JsRuntimeConfig {
        max_scripts: 128,
        max_script_bytes: MAX_PAGE_SCRIPT_BYTES,
        max_error_messages: 64,
        recursion_limit: 96,
        stack_size_limit: 2048,
        loop_iteration_limit: 500_000,
    }
}

fn event_js_runtime_config() -> JsRuntimeConfig {
    JsRuntimeConfig {
        max_scripts: MAX_DOM_EVENTS_PER_FRAME,
        max_script_bytes: MAX_INLINE_EVENT_HANDLER_BYTES + 1024,
        max_error_messages: 24,
        recursion_limit: 32,
        stack_size_limit: 512,
        loop_iteration_limit: 25_000,
    }
}

fn build_inline_event_script(event_type: &str, target_id: &str, handler: &str) -> String {
    let handler_literal = js_string_literal(handler);
    let target_literal = js_string_literal(target_id);
    format!(
        r#"
(function() {{
  const __pd_target_id = {target_literal};
  const __pd_target = __pd_target_id ? document.getElementById(__pd_target_id) : null;
  const __pd_event = {{
    type: {event_type:?},
    target: __pd_target,
    currentTarget: __pd_target
  }};
  const __pd_handler_src = {handler_literal};
  const __pd_handler = Function("event", __pd_handler_src);
  __pd_handler.call(__pd_target || document, __pd_event);
}})();
"#
    )
}

fn js_string_literal(input: &str) -> String {
    format!("{input:?}")
}

fn resolve_js_location(base_url: &str, href: &str) -> Option<String> {
    let trimmed = href.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(absolute) = Url::parse(trimmed)
        && matches!(absolute.scheme(), "http" | "https")
    {
        return Some(absolute.to_string());
    }

    let base = Url::parse(base_url).ok()?;
    let joined = base.join(trimmed).ok()?;
    match joined.scheme() {
        "http" | "https" => Some(joined.to_string()),
        _ => None,
    }
}

fn same_navigation_target(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }

    let Ok(left_url) = Url::parse(left) else {
        return false;
    };
    let Ok(right_url) = Url::parse(right) else {
        return false;
    };

    left_url.scheme() == right_url.scheme()
        && left_url.host_str() == right_url.host_str()
        && left_url.port_or_known_default() == right_url.port_or_known_default()
        && left_url.path() == right_url.path()
        && left_url.query() == right_url.query()
}

fn cookie_header_for_url(cache: &Arc<Mutex<HttpCache>>, request_url: &str) -> String {
    let Ok(parsed) = Url::parse(request_url) else {
        return String::new();
    };
    let Some(host) = parsed.host_str().map(|host| host.to_ascii_lowercase()) else {
        return String::new();
    };

    let guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let mut domain_entries = guard
        .cookies
        .iter()
        .filter(|(domain, _)| cookie_domain_matches(&host, domain))
        .collect::<Vec<_>>();
    domain_entries.sort_by(|(left, _), (right, _)| {
        right
            .len()
            .cmp(&left.len())
            .then_with(|| left.as_str().cmp(right.as_str()))
    });

    let mut selected = HashMap::<String, String>::new();
    for (_, cookies) in domain_entries {
        for (name, value) in cookies {
            if !selected.contains_key(name) {
                selected.insert(name.clone(), value.clone());
            }
        }
    }

    let mut pairs = selected.into_iter().collect::<Vec<_>>();
    pairs.sort_by(|(left_name, _), (right_name, _)| left_name.cmp(right_name));
    pairs
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

fn attach_cookie_header(
    cache: &Arc<Mutex<HttpCache>>,
    request_url: &str,
    headers: &mut Vec<Header>,
) -> Result<(), String> {
    let cookie = cookie_header_for_url(cache, request_url);
    if cookie.is_empty() {
        return Ok(());
    }

    headers.retain(|header| !header.name.eq_ignore_ascii_case("cookie"));
    if let Ok(cookie_header) = Header::new("Cookie", &cookie) {
        headers.push(cookie_header);
    }
    Ok(())
}

fn merge_document_cookie_snapshot(
    cache: &Arc<Mutex<HttpCache>>,
    page_url: &str,
    cookie_snapshot: &str,
) {
    let Ok(parsed_url) = Url::parse(page_url) else {
        return;
    };
    let Some(host) = parsed_url.host_str().and_then(normalize_cookie_domain) else {
        return;
    };

    if cookie_snapshot.trim().is_empty() {
        return;
    }

    let mut guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    for chunk in cookie_snapshot.split(';') {
        let entry = chunk.trim();
        if entry.is_empty() {
            continue;
        }
        let Some((name, value)) = entry.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        upsert_cookie(&mut guard, &host, name, value.trim());
    }
}

fn store_response_cookies(
    cache: &Arc<Mutex<HttpCache>>,
    request_url: &str,
    response_headers: &[(String, String)],
) {
    let Ok(parsed_url) = Url::parse(request_url) else {
        return;
    };
    let Some(default_domain) = parsed_url.host_str().and_then(normalize_cookie_domain) else {
        return;
    };

    let mut parsed_cookies = Vec::new();
    for (name, value) in response_headers {
        if !name.eq_ignore_ascii_case("set-cookie") {
            continue;
        }
        if let Some(cookie) = parse_set_cookie_header(value, &default_domain) {
            parsed_cookies.push(cookie);
        }
    }

    if parsed_cookies.is_empty() {
        return;
    }

    let mut guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    for cookie in parsed_cookies {
        if cookie.delete {
            if let Some(domain_entry) = guard.cookies.get_mut(&cookie.domain) {
                domain_entry.remove(&cookie.name);
                if domain_entry.is_empty() {
                    guard.cookies.remove(&cookie.domain);
                }
            }
            continue;
        }

        upsert_cookie(&mut guard, &cookie.domain, &cookie.name, &cookie.value);
    }
}

#[derive(Debug, Clone)]
struct ParsedSetCookie {
    domain: String,
    name: String,
    value: String,
    delete: bool,
}

fn parse_set_cookie_header(input: &str, default_domain: &str) -> Option<ParsedSetCookie> {
    let mut segments = input.split(';');
    let first = segments.next()?.trim();
    let (name, value) = first.split_once('=')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    let mut domain = default_domain.to_owned();
    let mut delete = value.trim().is_empty();

    for raw_attr in segments {
        let attr = raw_attr.trim();
        if attr.is_empty() {
            continue;
        }

        let (attr_name, attr_value) = attr
            .split_once('=')
            .map(|(name, value)| (name.trim(), value.trim()))
            .unwrap_or((attr, ""));

        if attr_name.eq_ignore_ascii_case("domain") {
            if let Some(normalized) = normalize_cookie_domain(attr_value) {
                domain = normalized;
            }
            continue;
        }

        if attr_name.eq_ignore_ascii_case("max-age")
            && attr_value
                .parse::<i64>()
                .ok()
                .is_some_and(|value| value <= 0)
        {
            delete = true;
        }
    }

    Some(ParsedSetCookie {
        domain,
        name: name.to_owned(),
        value: value.trim().to_owned(),
        delete,
    })
}

fn normalize_cookie_domain(input: &str) -> Option<String> {
    let normalized = input.trim().trim_start_matches('.').to_ascii_lowercase();
    if normalized.is_empty() || normalized.chars().any(char::is_whitespace) {
        None
    } else {
        Some(normalized)
    }
}

fn cookie_domain_matches(host: &str, domain: &str) -> bool {
    host == domain
        || (host.len() > domain.len()
            && host.ends_with(domain)
            && host.as_bytes()[host.len() - domain.len() - 1] == b'.')
}

fn upsert_cookie(cache: &mut HttpCache, domain: &str, name: &str, value: &str) {
    if !cache.cookies.contains_key(domain)
        && cache.cookies.len() >= MAX_COOKIE_DOMAINS
        && let Some(evicted) = cache.cookies.keys().next().cloned()
    {
        cache.cookies.remove(&evicted);
    }

    let cookies = cache.cookies.entry(domain.to_owned()).or_default();
    if !cookies.contains_key(name)
        && cookies.len() >= MAX_COOKIES_PER_DOMAIN
        && let Some(evicted) = cookies.keys().next().cloned()
    {
        cookies.remove(&evicted);
    }

    cookies.insert(name.to_owned(), value.to_owned());
}

fn decode_text_response(body: &[u8], content_type: &str) -> String {
    let charset = detect_response_charset(body, content_type);
    if let Some(label) = charset {
        if let Some(encoding) = Encoding::for_label(label.as_bytes()) {
            let (decoded, _, _) = encoding.decode(body);
            return decoded.into_owned();
        }
    }

    String::from_utf8_lossy(body).to_string()
}

fn detect_response_charset(body: &[u8], content_type: &str) -> Option<String> {
    let is_html = content_type.to_ascii_lowercase().contains("text/html")
        || content_type
            .to_ascii_lowercase()
            .contains("application/xhtml+xml");

    if is_html {
        if let Some(meta_charset) = parse_charset_from_html_prefix(body) {
            return Some(meta_charset);
        }
    }

    parse_charset_from_content_type(content_type)
}

fn parse_charset_from_content_type(content_type: &str) -> Option<String> {
    for part in content_type.split(';').skip(1) {
        let Some((name, value)) = part.split_once('=') else {
            continue;
        };
        if !name.trim().eq_ignore_ascii_case("charset") {
            continue;
        }

        let label = value.trim().trim_matches('"').trim_matches('\'');
        if !label.is_empty() {
            return Some(label.to_owned());
        }
    }

    None
}

fn parse_charset_from_html_prefix(body: &[u8]) -> Option<String> {
    let prefix_len = body.len().min(8192);
    let prefix = String::from_utf8_lossy(&body[..prefix_len]);
    let lower = prefix.to_ascii_lowercase();
    let mut search_start = 0_usize;

    while let Some(relative) = lower[search_start..].find("charset=") {
        let charset_start = search_start + relative + "charset=".len();
        let remainder = &prefix[charset_start..];
        if let Some(label) = parse_charset_label(remainder) {
            return Some(label);
        }
        search_start = charset_start;
    }

    None
}

fn parse_charset_label(input: &str) -> Option<String> {
    let trimmed = input.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let mut chars = trimmed.chars();
    let first = chars.next()?;

    if first == '"' || first == '\'' {
        let rest = &trimmed[first.len_utf8()..];
        let end = rest.find(first)?;
        let label = rest[..end].trim();
        return if label.is_empty() {
            None
        } else {
            Some(label.to_owned())
        };
    }

    let end = trimmed
        .find(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\'' | ';' | '>' | '/'))
        .unwrap_or(trimmed.len());
    let label = trimmed[..end].trim();
    if label.is_empty() {
        None
    } else {
        Some(label.to_owned())
    }
}

fn truncate_preview_text(input: &str, max_bytes: usize) -> String {
    if input.len() <= max_bytes {
        return input.to_owned();
    }

    let mut end = max_bytes.min(input.len());
    while end > 0 && !input.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    input[..end].to_owned()
}

fn decode_image_asset(url: &str, content_type: &str, body: &[u8]) -> Option<DecodedImageAsset> {
    let content_type = content_type.to_ascii_lowercase();
    let lower_url = url.to_ascii_lowercase();
    if is_svg_image_candidate(&content_type, &lower_url, body) {
        if let Some((width, height, rgba)) = decode_svg_image(body) {
            return Some(DecodedImageAsset {
                url: url.to_owned(),
                width,
                height,
                rgba,
            });
        }
    }

    if !(content_type.starts_with("image/")
        || lower_url.ends_with(".png")
        || lower_url.ends_with(".jpg")
        || lower_url.ends_with(".jpeg")
        || lower_url.ends_with(".webp"))
    {
        return None;
    }

    let decoded = image::load_from_memory(body).ok()?;
    let (width, height) = decoded.dimensions();
    let width_usize = usize::try_from(width).ok()?;
    let height_usize = usize::try_from(height).ok()?;
    let pixels = width_usize.checked_mul(height_usize)?;
    if pixels == 0 || pixels > MAX_IMAGE_PIXELS {
        return None;
    }

    let rgba = decoded.to_rgba8().into_raw();

    Some(DecodedImageAsset {
        url: url.to_owned(),
        width: width_usize,
        height: height_usize,
        rgba,
    })
}

fn is_svg_image_candidate(content_type: &str, lower_url: &str, body: &[u8]) -> bool {
    if content_type.contains("image/svg+xml") {
        return true;
    }

    let url_path = strip_query_and_fragment(lower_url);
    if url_path.ends_with(".svg") {
        return true;
    }

    looks_like_svg_document(body)
}

fn strip_query_and_fragment(url: &str) -> &str {
    let before_fragment = match url.split_once('#') {
        Some((prefix, _)) => prefix,
        None => url,
    };
    match before_fragment.split_once('?') {
        Some((prefix, _)) => prefix,
        None => before_fragment,
    }
}

fn looks_like_svg_document(body: &[u8]) -> bool {
    let mut bytes = body;
    if let Some(remaining) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        bytes = remaining;
    }

    while let Some(first) = bytes.first() {
        if !first.is_ascii_whitespace() {
            break;
        }
        bytes = &bytes[1..];
    }

    if bytes.starts_with(b"<svg") {
        return true;
    }

    if bytes.starts_with(b"<?xml") {
        return bytes.windows(4).any(|window| window == b"<svg");
    }

    false
}

fn decode_svg_image(body: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(body, &options).ok()?;
    let size = tree.size().to_int_size();
    let width_u32 = size.width();
    let height_u32 = size.height();
    let width = usize::try_from(width_u32).ok()?;
    let height = usize::try_from(height_u32).ok()?;
    let pixels = width.checked_mul(height)?;
    if pixels == 0 || pixels > MAX_IMAGE_PIXELS {
        return None;
    }

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width_u32, height_u32)?;
    let mut pixmap_mut = pixmap.as_mut();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap_mut,
    );
    let rgba = pixmap.data().to_vec();
    Some((width, height, rgba))
}

pub(super) fn normalize_input_url(input: String) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return DEFAULT_URL.to_owned();
    }

    let candidate = if trimmed.contains("://") {
        trimmed.to_owned()
    } else {
        let default_scheme = if is_local_network_input(trimmed) {
            "http"
        } else {
            "https"
        };
        format!("{default_scheme}://{trimmed}")
    };

    correct_known_host_typo(candidate)
}

fn is_local_network_input(input: &str) -> bool {
    let probe = format!("http://{input}");
    let Ok(parsed) = Url::parse(&probe) else {
        return false;
    };
    parsed.host_str().is_some_and(is_local_network_host)
}

fn is_local_network_url(input: &str) -> bool {
    let Ok(parsed) = Url::parse(input) else {
        return false;
    };
    parsed.host_str().is_some_and(is_local_network_host)
}

fn is_local_network_host(host: &str) -> bool {
    let normalized = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }

    if normalized == "localhost"
        || normalized.ends_with(".localhost")
        || normalized.ends_with(".local")
    {
        return true;
    }

    let Ok(ip) = normalized.parse::<std::net::IpAddr>() else {
        return false;
    };

    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_unspecified()
        }
        std::net::IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unique_local()
                || v6.is_unicast_link_local()
                || v6.is_unspecified()
        }
    }
}

fn correct_known_host_typo(candidate: String) -> String {
    let Ok(mut parsed) = Url::parse(&candidate) else {
        return candidate;
    };

    let Some(host) = parsed.host_str() else {
        return candidate;
    };

    let replacement = match host.to_ascii_lowercase().as_str() {
        "exaple.com" => Some("example.com"),
        "www.exaple.com" => Some("www.example.com"),
        _ => None,
    };

    if let Some(replacement) = replacement {
        let _ = parsed.set_host(Some(replacement));
        return parsed.to_string();
    }

    candidate
}

fn extract_html_title(document: &str) -> Option<String> {
    let lower = document.to_ascii_lowercase();
    let open = lower.find("<title>")?;
    let close = lower.find("</title>")?;
    if close <= open + 7 {
        return None;
    }

    let title = document[(open + 7)..close].trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_owned())
    }
}

fn is_redirect_status(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

fn resolve_redirect_url(base_url: &str, location: &str) -> Result<String, String> {
    if location.starts_with("http://") || location.starts_with("https://") {
        return Ok(location.to_owned());
    }

    let base = Url::parse(base_url).map_err(|error| error.to_string())?;
    let joined = base.join(location).map_err(|error| error.to_string())?;
    match joined.scheme() {
        "http" | "https" => Ok(joined.to_string()),
        _ => Err(format!(
            "unsupported redirect target scheme '{}'",
            joined.scheme()
        )),
    }
}

#[cfg(test)]
include!("tests.rs");
