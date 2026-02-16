//! Paint and compositing command generation.

use pd_layout::LayoutTree;

/// Output frame metadata produced by rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub draw_calls: usize,
}

/// Converts layout trees into drawable output.
#[derive(Debug, Default)]
pub struct Renderer;

impl Renderer {
    pub fn render(&self, layout: &LayoutTree) -> Frame {
        if layout.width == 0 || layout.height == 0 {
            return Frame { draw_calls: 0 };
        }

        let area = (layout.width as usize).saturating_mul(layout.height as usize);
        let area_calls = (area / 120_000).max(1);
        let complexity_calls = ((layout.width + layout.height) as usize / 300).max(1);

        Frame {
            draw_calls: area_calls + complexity_calls,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Renderer;
    use pd_layout::LayoutTree;

    #[test]
    fn empty_layout_has_zero_draw_calls() {
        let renderer = Renderer;
        let frame = renderer.render(&LayoutTree {
            width: 0,
            height: 0,
        });
        assert_eq!(frame.draw_calls, 0);
    }

    #[test]
    fn non_empty_layout_produces_draw_calls() {
        let renderer = Renderer;
        let frame = renderer.render(&LayoutTree {
            width: 1200,
            height: 900,
        });
        assert!(frame.draw_calls > 0);
    }
}
