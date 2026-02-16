#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrustStoreSelection {
    WebPkiOnly,
    WebPkiAndOs,
}

impl TrustStoreSelection {
    fn label(self) -> &'static str {
        match self {
            Self::WebPkiOnly => "WebPKI only",
            Self::WebPkiAndOs => "WebPKI + OS roots",
        }
    }

    fn as_policy_mode(self) -> TrustStoreMode {
        match self {
            Self::WebPkiOnly => TrustStoreMode::WebPkiOnly,
            Self::WebPkiAndOs => TrustStoreMode::WebPkiAndOs,
        }
    }
}

#[derive(Debug, Clone)]
struct PageView {
    final_url: String,
    status_code: u16,
    http_version: String,
    content_type: String,
    headers: Vec<(String, String)>,
    body_bytes: usize,
    body_preview: String,
    title: Option<String>,
    html_document: Option<simple_html::HtmlDocument>,
    static_text_fallback: Option<String>,
    decoded_images: Vec<DecodedImageAsset>,
    subresource_stats: SubresourceStats,
    js_execution: JsExecutionStats,
    renderer_draw_calls: Option<usize>,
}

#[derive(Debug, Clone)]
struct DecodedImageAsset {
    url: String,
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
struct SubresourceStats {
    stylesheets_loaded: usize,
    inline_style_tags: usize,
    css_rules_total: usize,
    scripts_loaded: usize,
    images_loaded: usize,
    blocked: usize,
}

#[derive(Debug, Clone, Default)]
struct JsExecutionStats {
    enabled: bool,
    scripts_seen: usize,
    scripts_executed: usize,
    scripts_failed: usize,
    scripts_skipped: usize,
    event_dispatches: usize,
    event_failures: usize,
    errors: Vec<String>,
}

#[derive(Debug, Clone)]
struct FetchedResponse {
    final_url: String,
    status_code: u16,
    http_version: String,
    headers: Vec<(String, String)>,
    content_type: String,
    body: Vec<u8>,
}

#[derive(Debug, Clone)]
struct CachedResponse {
    response: FetchedResponse,
    etag: Option<String>,
    last_modified: Option<String>,
    max_age: Option<Duration>,
    stored_at: Instant,
}

#[derive(Debug, Default)]
struct HttpCache {
    entries: HashMap<String, CachedResponse>,
    cookies: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone)]
enum CacheLookup {
    Fresh(FetchedResponse),
    Stale {
        cached: FetchedResponse,
        etag: Option<String>,
        last_modified: Option<String>,
    },
    Miss,
}

#[derive(Debug)]
struct NavigationResult {
    request_id: u64,
    url: String,
    add_to_history: bool,
    result: Result<PageView, String>,
}

#[derive(Debug, Clone)]
struct RuntimeWorkerStatus {
    role: ProcessRole,
    pid: u32,
    running: bool,
    exit_code: Option<i32>,
}

struct BrowserUiApp {
    address_input: String,
    current_url: Option<String>,
    page_view: Option<PageView>,
    status_line: String,
    last_error: Option<String>,
    trust_store: TrustStoreSelection,
    ocsp_required: bool,
    history: Vec<String>,
    history_index: Option<usize>,
    next_request_id: u64,
    inflight_request_id: Option<u64>,
    nav_receiver: Option<mpsc::Receiver<NavigationResult>>,
    show_navigation_details: bool,
    image_textures: HashMap<String, egui::TextureHandle>,
    form_state: HashMap<String, String>,
    cache: Arc<Mutex<HttpCache>>,
    runtime: Option<pd_browser::BrowserRuntime>,
    runtime_workers: Vec<RuntimeWorkerStatus>,
    runtime_restarts: usize,
    runtime_last_error: Option<String>,
    runtime_last_poll: Instant,
}

