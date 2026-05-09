//! [`BrushTool`] — first concrete tool. Model: size/opacity/flow/color.
//!
//! Panel layout follows Procreate's brush HUD: one "Tip" tab plus a
//! row of three Sliders (Size / Opacity / Flow) + one ColorSwatch.
//! The slider values reflect the model fields; mouse drag (next PR)
//! will write back into them.

use crate::floating_panel::{FloatingPanel, PanelAnchor, PanelControl, PanelTab, ToolId};
use crate::tool::Tool;
use crate::widget::{ColorSwatch, Slider};
use ph2d_a11y::NodeId;

#[derive(Clone, Debug)]
pub struct BrushTool {
    pub size: f32,
    pub opacity: f32,
    pub flow: f32,
    pub color_rgba: [u8; 4],
}

impl Default for BrushTool {
    fn default() -> Self {
        Self {
            size: 24.0,
            opacity: 1.0,
            flow: 1.0,
            color_rgba: [255, 255, 255, 255],
        }
    }
}

impl Tool for BrushTool {
    fn id(&self) -> ToolId {
        ToolId::new("brush")
    }

    fn label(&self) -> &str {
        "Brush"
    }

    fn icon_slug(&self) -> &str {
        "brush"
    }

    fn build_panel(&self) -> FloatingPanel {
        let mut size = Slider::new(NodeId(101), "Size");
        size.set_value(self.size / 100.0); // size 0..100 px → slider 0..1
        let mut opacity = Slider::new(NodeId(102), "Opacity");
        opacity.set_value(self.opacity);
        let mut flow = Slider::new(NodeId(103), "Flow");
        flow.set_value(self.flow);
        let swatch = ColorSwatch::new(NodeId(104), "Color", self.color_rgba);

        let mut panel = FloatingPanel::new(self.id(), "Brush")
            .with_tabs(vec![PanelTab {
                label: "Tip".into(),
                icon: None,
                active: true,
            }])
            .with_controls(vec![
                PanelControl::Slider(size),
                PanelControl::Slider(opacity),
                PanelControl::Slider(flow),
                PanelControl::ColorSwatch(swatch),
            ]);
        panel.anchor = PanelAnchor::BottomCenter;
        panel.width = 480.0;
        panel.height = 96.0;
        panel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_procreate_brush_baseline() {
        let b = BrushTool::default();
        assert_eq!(b.size, 24.0);
        assert_eq!(b.opacity, 1.0);
        assert_eq!(b.flow, 1.0);
        assert_eq!(b.color_rgba, [255, 255, 255, 255]);
    }

    #[test]
    fn id_label_icon_are_brush() {
        let b = BrushTool::default();
        assert_eq!(b.id(), ToolId::new("brush"));
        assert_eq!(b.label(), "Brush");
        assert_eq!(b.icon_slug(), "brush");
    }

    #[test]
    fn panel_has_one_tab_and_four_controls() {
        let p = BrushTool::default().build_panel();
        assert_eq!(p.tool_id, ToolId::new("brush"));
        assert_eq!(p.title, "Brush");
        assert_eq!(p.anchor, PanelAnchor::BottomCenter);
        assert_eq!(p.tabs.len(), 1);
        assert!(p.tabs[0].active);
        assert_eq!(p.tabs[0].label, "Tip");
        assert_eq!(p.controls.len(), 4);
        assert!(matches!(p.controls[0], PanelControl::Slider(_)));
        assert!(matches!(p.controls[3], PanelControl::ColorSwatch(_)));
        let labels: Vec<&str> = p.controls.iter().map(|c| c.label()).collect();
        assert_eq!(labels, vec!["Size", "Opacity", "Flow", "Color"]);
    }
}
