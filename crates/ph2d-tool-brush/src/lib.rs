#![forbid(unsafe_code)]
//! [`BrushTool`] — the painter tool (modal, palette). Model:
//! size/opacity/flow/color.
//!
//! Panel layout follows Procreate's brush HUD: one "Tip" tab plus a
//! row of three Sliders (Size / Opacity / Flow) + one ColorSwatch.
//! The slider values reflect the model fields; mouse drag writes back
//! into them via `handle_panel_event`.
//!
//! ADR-0040 T-close: isolated crate. [`make`] is the codegen entry
//! (`ph2d-tool-sync` → `register_all_tools`); `BrushTool::is_default`
//! marks it the editor's boot/fallback tool.

use ph2d_a11y::NodeId;
use ph2d_editor_core::floating_panel::{
    FloatingPanel, PanelAnchor, PanelControl, PanelTab, ToolId,
};
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::widget::{ColorSwatch, Slider};

/// Construct the Brush tool as a boxed trait object for the behavior
/// `ToolRegistry`. Codegen target for `ph2d-tool-sync`.
pub fn make() -> Box<dyn Tool> {
    Box::new(BrushTool::default())
}

const SIZE_NODE: NodeId = NodeId(101);
const OPACITY_NODE: NodeId = NodeId(102);
const FLOW_NODE: NodeId = NodeId(103);
const COLOR_NODE: NodeId = NodeId(104);

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
        let mut size = Slider::new(SIZE_NODE, "Size");
        size.set_value(self.size / 100.0); // size 0..100 px → slider 0..1
        let mut opacity = Slider::new(OPACITY_NODE, "Opacity");
        opacity.set_value(self.opacity);
        let mut flow = Slider::new(FLOW_NODE, "Flow");
        flow.set_value(self.flow);
        let swatch = ColorSwatch::new(COLOR_NODE, "Color", self.color_rgba);

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

    fn handle_panel_event(&mut self, event: PanelEvent) {
        match event {
            PanelEvent::SetValue(id, v) if id == SIZE_NODE => {
                // Slider is normalized 0..=1; tool stores px in 0..=100.
                self.size = (v.clamp(0.0, 1.0) * 100.0) as f32;
            }
            PanelEvent::SetValue(id, v) if id == OPACITY_NODE => {
                self.opacity = v.clamp(0.0, 1.0) as f32;
            }
            PanelEvent::SetValue(id, v) if id == FLOW_NODE => {
                self.flow = v.clamp(0.0, 1.0) as f32;
            }
            // ColorSwatch click would open a picker — out of scope here.
            _ => {}
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    /// Brush is the editor's default / fallback tool (ADR-0040 T-close).
    fn is_default(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brush_is_the_default_tool() {
        assert!(BrushTool::default().is_default());
    }

    #[test]
    fn make_builds_a_brush() {
        let t = make();
        assert_eq!(t.id(), ToolId::new("brush"));
        assert!(t.is_default());
    }

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
