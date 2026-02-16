//! Renderer process pipeline from HTML source to frame output.

use pd_css::CssParser;
use pd_html::HtmlParser;
use pd_js::JsRuntime;
use pd_layout::LayoutEngine;
use pd_render::Frame;
use pd_render::Renderer;

/// Dedicated renderer process.
#[derive(Debug, Default)]
pub struct RendererProcess {
    html: HtmlParser,
    css: CssParser,
    js: JsRuntime,
    layout: LayoutEngine,
    render: Renderer,
}

impl RendererProcess {
    pub fn render_document(&self, html_source: &str, css_source: &str) -> Frame {
        let document = self.html.parse(html_source);
        self.js.run_bootstrap_scripts(&document);

        let stylesheet = self.css.parse(css_source);
        let layout_tree = self.layout.compute(&document, &stylesheet);

        self.render.render(&layout_tree)
    }
}

#[cfg(test)]
mod tests {
    use super::RendererProcess;

    #[test]
    fn pipeline_renders_non_empty_documents() {
        let renderer = RendererProcess::default();
        let frame = renderer.render_document(
            "<html><head><title>PixelDust</title></head><body><p>Hello</p></body></html>",
            "body { color: red; } p { margin: 8px; }",
        );
        assert!(frame.draw_calls > 0);
    }

    #[test]
    fn pipeline_handles_empty_input() {
        let renderer = RendererProcess::default();
        let frame = renderer.render_document("", "");
        assert_eq!(frame.draw_calls, 0);
    }
}
