//! JavaScript runtime integration surface.

use boa_engine::Context;
use boa_engine::Source;
use pd_dom::Document;

const BOOTSTRAP_ENV: &str = r#"
globalThis.window = globalThis;
globalThis.self = globalThis;
globalThis.global = globalThis;
globalThis.navigator = {
  userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
  language: "en-US",
  languages: ["en-US", "en"],
  platform: "Win32",
  sendBeacon: function () { return true; }
};
globalThis.console = {
  log: function () {},
  warn: function () {},
  error: function () {}
};
globalThis.performance = {
  now: function () { return Date.now(); },
  timeOrigin: 0,
  mark: function () {},
  measure: function () {},
  getEntriesByType: function () { return []; }
};
globalThis.__pd_timer_queue = [];
globalThis.__pd_timer_cancelled = {};
globalThis.__pd_next_timer_id = 1;
globalThis.setTimeout = function (callback, _delay) {
  var cb = callback;
  if (typeof cb !== "function") {
    var src = String(callback);
    cb = function () { (0, eval)(src); };
  }
  var id = globalThis.__pd_next_timer_id++;
  globalThis.__pd_timer_queue.push({ id: id, cb: cb });
  return id;
};
globalThis.clearTimeout = function (id) {
  globalThis.__pd_timer_cancelled[String(id)] = true;
};
globalThis.setInterval = function (callback, delay) {
  return globalThis.setTimeout(callback, delay);
};
globalThis.clearInterval = globalThis.clearTimeout;
globalThis.requestAnimationFrame = function (callback) {
  return globalThis.setTimeout(function () {
    if (typeof callback === "function") {
      callback(globalThis.performance.now());
    }
  }, 16);
};
globalThis.cancelAnimationFrame = globalThis.clearTimeout;
globalThis.matchMedia = function (query) {
  return {
    media: String(query || ""),
    matches: false,
    onchange: null,
    addListener: function () {},
    removeListener: function () {},
    addEventListener: function () {},
    removeEventListener: function () {},
    dispatchEvent: function () { return true; }
  };
};
globalThis.queueMicrotask = function (callback) {
  return globalThis.setTimeout(callback, 0);
};
globalThis.__pd_flush_timers = function (limit) {
  var maxRuns = Number(limit) || 0;
  if (maxRuns < 1) {
    maxRuns = 1;
  }
  var runs = 0;
  while (globalThis.__pd_timer_queue.length > 0 && runs < maxRuns) {
    var task = globalThis.__pd_timer_queue.shift();
    if (!task) {
      continue;
    }
    var cancelled = !!globalThis.__pd_timer_cancelled[String(task.id)];
    delete globalThis.__pd_timer_cancelled[String(task.id)];
    if (!cancelled) {
      task.cb();
    }
    runs++;
  }
  return runs;
};
"#;

/// Script payload to execute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptSource {
    pub origin: String,
    pub source: String,
}

/// Minimal host-side snapshot used by JS Phase-1 DOM shims.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JsHostEnvironment {
    pub page_url: String,
    pub document_title: String,
    pub cookie_header: String,
    pub elements_by_id: Vec<JsHostElement>,
}

/// ID-indexed element metadata exposed to JS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsHostElement {
    pub id: String,
    pub tag_name: String,
    pub text_content: String,
    pub attributes: Vec<(String, String)>,
}

/// Runtime hardening knobs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsRuntimeConfig {
    /// Soft limit: when exceeded, execution continues but a runtime warning is recorded.
    pub max_scripts: usize,
    /// Preferred script-size budget in bytes.
    pub max_script_bytes: usize,
    pub max_error_messages: usize,
    pub recursion_limit: usize,
    pub stack_size_limit: usize,
    pub loop_iteration_limit: u64,
}

impl Default for JsRuntimeConfig {
    fn default() -> Self {
        Self {
            max_scripts: 128,
            max_script_bytes: 2 * 1024 * 1024,
            max_error_messages: 24,
            recursion_limit: 64,
            stack_size_limit: 1024,
            loop_iteration_limit: 100_000,
        }
    }
}

/// Per-script execution error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError {
    pub origin: String,
    pub message: String,
}

/// Runtime outcome summary.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JsExecutionReport {
    pub scripts_seen: usize,
    pub scripts_executed: usize,
    pub scripts_failed: usize,
    pub scripts_skipped: usize,
    pub errors: Vec<ScriptError>,
}

/// Runtime execution output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JsExecutionOutput {
    pub report: JsExecutionReport,
    pub document_title: Option<String>,
    pub location_href: Option<String>,
    pub document_cookie: Option<String>,
}

/// Script engine facade.
#[derive(Debug, Clone, Default)]
pub struct JsRuntime {
    config: JsRuntimeConfig,
}

impl JsRuntime {
    pub fn new(config: JsRuntimeConfig) -> Self {
        Self { config }
    }

    pub fn execute_scripts(&self, scripts: &[ScriptSource]) -> JsExecutionReport {
        self.execute_scripts_with_host(&JsHostEnvironment::default(), scripts)
            .report
    }

    pub fn execute_scripts_with_host(
        &self,
        host: &JsHostEnvironment,
        scripts: &[ScriptSource],
    ) -> JsExecutionOutput {
        if scripts.is_empty() {
            return JsExecutionOutput {
                report: JsExecutionReport::default(),
                document_title: Some(host.document_title.clone()),
                location_href: Some(host.page_url.clone()),
                document_cookie: Some(host.cookie_header.clone()),
            };
        }

        let mut report = JsExecutionReport {
            scripts_seen: scripts.len(),
            ..JsExecutionReport::default()
        };

        let mut context = Context::default();
        context
            .runtime_limits_mut()
            .set_recursion_limit(self.config.recursion_limit);
        context
            .runtime_limits_mut()
            .set_stack_size_limit(self.config.stack_size_limit);
        context
            .runtime_limits_mut()
            .set_loop_iteration_limit(self.config.loop_iteration_limit);
        if let Err(error) = context.eval(Source::from_bytes(BOOTSTRAP_ENV.as_bytes())) {
            report.scripts_failed = 1;
            report.errors.push(ScriptError {
                origin: "bootstrap".to_owned(),
                message: error.to_string(),
            });
            report.scripts_skipped = scripts.len();
            return JsExecutionOutput {
                report,
                document_title: None,
                location_href: None,
                document_cookie: None,
            };
        }

        let host_bootstrap = build_host_bootstrap(host);
        if let Err(error) = context.eval(Source::from_bytes(host_bootstrap.as_bytes())) {
            report.scripts_failed = report.scripts_failed.saturating_add(1);
            report.errors.push(ScriptError {
                origin: "host-bootstrap".to_owned(),
                message: error.to_string(),
            });
            report.scripts_skipped = scripts.len();
            return JsExecutionOutput {
                report,
                document_title: None,
                location_href: None,
                document_cookie: None,
            };
        }

        if scripts.len() > self.config.max_scripts
            && report.errors.len() < self.config.max_error_messages
        {
            report.errors.push(ScriptError {
                origin: "runtime".to_owned(),
                message: format!(
                    "script count {} exceeded soft limit {}; continuing",
                    scripts.len(),
                    self.config.max_scripts
                ),
            });
        }

        let hard_cap = hard_script_byte_cap(self.config.max_script_bytes);
        for script in scripts {
            let source_bytes = script.source.as_bytes();
            let source_len = source_bytes.len();
            if source_len > hard_cap {
                report.scripts_skipped = report.scripts_skipped.saturating_add(1);
                continue;
            }

            match context.eval(Source::from_bytes(source_bytes)) {
                Ok(_) => {
                    report.scripts_executed = report.scripts_executed.saturating_add(1);
                    let _ = context.eval(Source::from_bytes(
                        b"(typeof __pd_flush_timers === 'function') ? __pd_flush_timers(128) : 0;",
                    ));
                }
                Err(error) => {
                    report.scripts_failed = report.scripts_failed.saturating_add(1);
                    if report.errors.len() < self.config.max_error_messages {
                        report.errors.push(ScriptError {
                            origin: script.origin.clone(),
                            message: if source_len > self.config.max_script_bytes {
                                format!(
                                    "oversized script ({} bytes, preferred <= {}) failed: {error}",
                                    source_len, self.config.max_script_bytes
                                )
                            } else {
                                error.to_string()
                            },
                        });
                    }
                }
            }
        }

        JsExecutionOutput {
            report,
            document_title: read_document_title(&mut context),
            location_href: read_location_href(&mut context),
            document_cookie: read_document_cookie(&mut context),
        }
    }

    pub fn run_bootstrap_scripts(&self, _document: &Document) {
        // DOM bindings are introduced in a later milestone.
    }
}

fn read_document_title(context: &mut Context) -> Option<String> {
    let value = context
        .eval(Source::from_bytes(
            b"(typeof document === 'object' && document !== null && 'title' in document) ? String(document.title) : ''",
        ))
        .ok()?;
    let js_string = value.to_string(context).ok()?;
    Some(js_string.to_std_string_escaped())
}

fn read_location_href(context: &mut Context) -> Option<String> {
    let value = context
        .eval(Source::from_bytes(
            b"(typeof location === 'object' && location !== null && 'href' in location) ? String(location.href) : ''",
        ))
        .ok()?;
    let js_string = value.to_string(context).ok()?;
    Some(js_string.to_std_string_escaped())
}

fn read_document_cookie(context: &mut Context) -> Option<String> {
    let value = context
        .eval(Source::from_bytes(
            b"(typeof document === 'object' && document !== null && 'cookie' in document) ? String(document.cookie) : ''",
        ))
        .ok()?;
    let js_string = value.to_string(context).ok()?;
    Some(js_string.to_std_string_escaped())
}

fn build_host_bootstrap(host: &JsHostEnvironment) -> String {
    let location = js_string_literal(&host.page_url);
    let title = js_string_literal(&host.document_title);
    let cookie_header = js_string_literal(&host.cookie_header);
    let elements = build_elements_by_id_object(&host.elements_by_id);

    format!(
        r##"
(function() {{
  function __pd_makeEventTarget(target) {{
    const listeners = Object.create(null);
    target.addEventListener = function(type, handler) {{
      const key = String(type || "");
      if (!key || typeof handler !== "function") {{
        return;
      }}
      if (!listeners[key]) {{
        listeners[key] = [];
      }}
      listeners[key].push(handler);
    }};
    target.removeEventListener = function(type, handler) {{
      const key = String(type || "");
      const arr = listeners[key];
      if (!arr || typeof handler !== "function") {{
        return;
      }}
      const index = arr.indexOf(handler);
      if (index >= 0) {{
        arr.splice(index, 1);
      }}
    }};
    target.dispatchEvent = function(event) {{
      const evt = event || {{}};
      const key = String(evt.type || "");
      const arr = listeners[key];
      if (!arr || arr.length === 0) {{
        return true;
      }}
      for (let i = 0; i < arr.length; i += 1) {{
        arr[i].call(this, evt);
      }}
      return true;
    }};
    return target;
  }}

  const __pd_cookie_store = Object.create(null);
  const __pd_cookie_seed = {cookie_header};
  if (typeof __pd_cookie_seed === "string" && __pd_cookie_seed.length > 0) {{
    const pairs = __pd_cookie_seed.split(";");
    for (let i = 0; i < pairs.length; i += 1) {{
      const part = pairs[i].trim();
      if (!part) {{
        continue;
      }}
      const eq = part.indexOf("=");
      if (eq <= 0) {{
        continue;
      }}
      const name = part.slice(0, eq).trim();
      const value = part.slice(eq + 1).trim();
      if (name) {{
        __pd_cookie_store[name] = value;
      }}
    }}
  }}

  function __pd_cookie_string() {{
    const names = Object.keys(__pd_cookie_store);
    if (names.length === 0) {{
      return "";
    }}
    const out = [];
    for (let i = 0; i < names.length; i += 1) {{
      const name = names[i];
      out.push(name + "=" + __pd_cookie_store[name]);
    }}
    return out.join("; ");
  }}

  function __pd_set_cookie(input) {{
    if (input == null) {{
      return;
    }}
    const raw = String(input).trim();
    if (!raw) {{
      return;
    }}
    const first = raw.split(";")[0];
    const eq = first.indexOf("=");
    if (eq <= 0) {{
      return;
    }}
    const name = first.slice(0, eq).trim();
    const value = first.slice(eq + 1).trim();
    if (!name) {{
      return;
    }}
    __pd_cookie_store[name] = value;
  }}

  const __pd_elements = {elements};
  function __pd_clone(node) {{
    if (!node) {{
      return null;
    }}
    const el = __pd_makeEventTarget({{
      id: node.id,
      tagName: node.tagName,
      textContent: node.textContent,
      innerText: node.textContent,
      style: {{}},
      getAttribute: function(name) {{
        const key = String(name);
        return Object.prototype.hasOwnProperty.call(node.attributes, key)
          ? node.attributes[key]
          : null;
      }},
      setAttribute: function(name, value) {{
        node.attributes[String(name)] = String(value);
      }},
      appendChild: function() {{}},
      removeChild: function() {{}}
    }});
    return el;
  }}

  globalThis.location = __pd_makeEventTarget({{
    href: {location},
    assign: function(next) {{
      this.href = String(next || "");
    }},
    replace: function(next) {{
      this.href = String(next || "");
    }},
    reload: function() {{}},
    toString: function() {{ return this.href; }}
  }});

  const __pd_document = __pd_makeEventTarget({{
    title: {title},
    URL: {location},
    documentURI: {location},
    readyState: "complete",
    body: __pd_makeEventTarget({{}}),
    documentElement: __pd_makeEventTarget({{}}),
    location: globalThis.location,
    getElementById: function(id) {{
      if (id == null) {{
        return null;
      }}
      return __pd_clone(__pd_elements[String(id)]);
    }},
    querySelector: function(selector) {{
      if (typeof selector !== "string") {{
        return null;
      }}
      if (selector.startsWith("#")) {{
        return this.getElementById(selector.slice(1));
      }}
      return null;
    }},
    querySelectorAll: function(selector) {{
      const node = this.querySelector(selector);
      return node ? [node] : [];
    }},
    createElement: function(tag) {{
      return __pd_makeEventTarget({{
        tagName: String(tag || "").toUpperCase(),
        textContent: "",
        style: {{}},
        children: [],
        appendChild: function(child) {{
          this.children.push(child);
        }},
        setAttribute: function(name, value) {{
          this[String(name)] = String(value);
        }},
        getAttribute: function(name) {{
          const key = String(name);
          return Object.prototype.hasOwnProperty.call(this, key) ? this[key] : null;
        }}
      }});
    }}
  }});
  Object.defineProperty(__pd_document, "cookie", {{
    configurable: true,
    enumerable: true,
    get: function() {{
      return __pd_cookie_string();
    }},
    set: function(value) {{
      __pd_set_cookie(value);
    }}
  }});

  globalThis.document = __pd_document;
  globalThis.window = __pd_makeEventTarget(globalThis.window || globalThis);
  globalThis.window.location = globalThis.location;
  globalThis.window.document = __pd_document;
  globalThis.__pd_get_cookie_string = function() {{
    return __pd_cookie_string();
  }};
}})();
"##
    )
}

fn build_elements_by_id_object(elements: &[JsHostElement]) -> String {
    let mut out = String::from("{");
    for (index, element) in elements.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        let key = js_string_literal(&element.id);
        let tag_name = js_string_literal(&element.tag_name);
        let text_content = js_string_literal(&element.text_content);
        let attributes = build_attributes_object(&element.attributes);
        out.push_str(&format!(
            "{key}:{{id:{key},tagName:{tag_name},textContent:{text_content},attributes:{attributes}}}"
        ));
    }
    out.push('}');
    out
}

fn build_attributes_object(attributes: &[(String, String)]) -> String {
    let mut out = String::from("{");
    for (index, (name, value)) in attributes.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{}:{}",
            js_string_literal(name),
            js_string_literal(value)
        ));
    }
    out.push('}');
    out
}

fn js_string_literal(input: &str) -> String {
    format!("{input:?}")
}

fn hard_script_byte_cap(preferred: usize) -> usize {
    let scaled = preferred.saturating_mul(4);
    let bounded = scaled.min(16 * 1024 * 1024);
    bounded.max(preferred)
}

#[cfg(test)]
mod tests {
    use super::{JsHostElement, JsHostEnvironment, JsRuntime, JsRuntimeConfig, ScriptSource};

    #[test]
    fn executes_scripts_against_host_document() {
        let runtime = JsRuntime::new(JsRuntimeConfig::default());
        let host = JsHostEnvironment {
            page_url: "https://example.test/".to_owned(),
            document_title: "Before".to_owned(),
            cookie_header: String::new(),
            elements_by_id: vec![JsHostElement {
                id: "hero".to_owned(),
                tag_name: "DIV".to_owned(),
                text_content: "hello".to_owned(),
                attributes: vec![("class".to_owned(), "banner".to_owned())],
            }],
        };
        let scripts = vec![ScriptSource {
            origin: "inline:1".to_owned(),
            source: "document.title = document.getElementById('hero').textContent + ' world';"
                .to_owned(),
        }];

        let output = runtime.execute_scripts_with_host(&host, &scripts);
        assert_eq!(output.report.scripts_executed, 1);
        assert_eq!(output.document_title.as_deref(), Some("hello world"));
    }

    #[test]
    fn does_not_hard_skip_when_script_count_exceeds_soft_limit() {
        let runtime = JsRuntime::new(JsRuntimeConfig {
            max_scripts: 1,
            max_script_bytes: 8 * 1024,
            ..JsRuntimeConfig::default()
        });
        let scripts = vec![
            ScriptSource {
                origin: "inline:1".to_owned(),
                source: "globalThis.__pd_count = (globalThis.__pd_count || 0) + 1;".to_owned(),
            },
            ScriptSource {
                origin: "inline:2".to_owned(),
                source: "globalThis.__pd_count = (globalThis.__pd_count || 0) + 1;".to_owned(),
            },
        ];

        let output = runtime.execute_scripts_with_host(&JsHostEnvironment::default(), &scripts);
        assert_eq!(output.report.scripts_executed, 2);
        assert_eq!(output.report.scripts_skipped, 0);
    }

    #[test]
    fn attempts_moderately_oversized_script() {
        let runtime = JsRuntime::new(JsRuntimeConfig {
            max_scripts: 8,
            max_script_bytes: 1024,
            ..JsRuntimeConfig::default()
        });
        let filler = " ".repeat(1800);
        let script = format!("{};globalThis.__pd_big = 7;", filler);
        let output = runtime.execute_scripts_with_host(
            &JsHostEnvironment::default(),
            &[ScriptSource {
                origin: "inline:big".to_owned(),
                source: script,
            }],
        );
        assert_eq!(output.report.scripts_skipped, 0);
        assert_eq!(output.report.scripts_executed, 1);
    }

    #[test]
    fn supports_timer_callbacks_for_dom_updates() {
        let runtime = JsRuntime::new(JsRuntimeConfig::default());
        let scripts = vec![ScriptSource {
            origin: "inline:timer".to_owned(),
            source: "setTimeout(function(){ document.title = 'after-timer'; }, 0);".to_owned(),
        }];

        let output = runtime.execute_scripts_with_host(&JsHostEnvironment::default(), &scripts);
        assert_eq!(output.report.scripts_failed, 0);
        assert_eq!(output.document_title.as_deref(), Some("after-timer"));
    }

    #[test]
    fn captures_cookie_and_location_mutations() {
        let runtime = JsRuntime::new(JsRuntimeConfig::default());
        let host = JsHostEnvironment {
            page_url: "https://example.test/start".to_owned(),
            document_title: "Before".to_owned(),
            cookie_header: "sid=abc".to_owned(),
            elements_by_id: Vec::new(),
        };
        let scripts = vec![ScriptSource {
            origin: "inline:cookie".to_owned(),
            source: "document.cookie='token=xyz; path=/'; location.replace('/next');".to_owned(),
        }];

        let output = runtime.execute_scripts_with_host(&host, &scripts);
        assert_eq!(output.report.scripts_failed, 0);
        assert_eq!(output.location_href.as_deref(), Some("/next"));
        assert!(
            output
                .document_cookie
                .as_deref()
                .is_some_and(|cookie| cookie.contains("sid=abc") && cookie.contains("token=xyz"))
        );
    }

    #[test]
    fn exposes_performance_and_animation_frame_shims() {
        let runtime = JsRuntime::new(JsRuntimeConfig::default());
        let scripts = vec![ScriptSource {
            origin: "inline:raf".to_owned(),
            source: "if (typeof performance === 'object' && typeof requestAnimationFrame === 'function') { requestAnimationFrame(function(){ document.title = 'raf-ok'; }); }".to_owned(),
        }];

        let output = runtime.execute_scripts_with_host(&JsHostEnvironment::default(), &scripts);
        assert_eq!(output.report.scripts_failed, 0);
        assert_eq!(output.document_title.as_deref(), Some("raf-ok"));
    }
}
