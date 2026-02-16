//! Layout engine entry points (style resolution + box tree).

use pd_css::StyleSheet;
use pd_dom::Document;

/// Simplified layout tree root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutTree {
    pub width: u32,
    pub height: u32,
}

/// Computes visual layout from DOM and styles.
#[derive(Debug, Default)]
pub struct LayoutEngine;

impl LayoutEngine {
    pub fn compute(&self, document: &Document, stylesheet: &StyleSheet) -> LayoutTree {
        if !document.has_root() && document.text_bytes == 0 {
            return LayoutTree {
                width: 0,
                height: 0,
            };
        }

        let base_width = if document.has_root() { 800 } else { 640 };
        let style_width = (stylesheet.rule_count() as u32).saturating_mul(4).min(400);
        let title_width = (document.title.chars().count() as u32)
            .saturating_mul(2)
            .min(220);

        let base_height = if document.has_root() { 600 } else { 200 };
        let node_height = document.node_count.saturating_mul(10).min(1400);
        let text_height = (document.text_bytes / 4).min(1800);
        let style_height = (stylesheet.rule_count() as u32).saturating_mul(12).min(480);

        LayoutTree {
            width: base_width + style_width + title_width,
            height: base_height + node_height + text_height + style_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutEngine;
    use pd_css::CssParser;
    use pd_dom::Document;

    #[test]
    fn empty_document_has_no_layout() {
        let engine = LayoutEngine;
        let doc = Document::empty();
        let css = CssParser.parse("");
        let tree = engine.compute(&doc, &css);
        assert_eq!(tree.width, 0);
        assert_eq!(tree.height, 0);
    }

    #[test]
    fn non_empty_document_produces_viewport() {
        let engine = LayoutEngine;
        let doc = Document {
            title: "PixelDust".to_owned(),
            root: 1,
            node_count: 12,
            text_bytes: 320,
        };
        let css = CssParser.parse("body{color:red} .card{padding:8px}");
        let tree = engine.compute(&doc, &css);
        assert!(tree.width >= 800);
        assert!(tree.height >= 600);
    }
}
