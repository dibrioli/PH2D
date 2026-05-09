//! [`BrushTool`] — first concrete tool. Model: size/opacity/flow/color.
//!
//! Panel layout follows Procreate's brush HUD: one "Tip" tab plus a
//! row of placeholder actions tagging the four parameters. Once the
//! widget-primitives PR lands, the placeholders swap for Slider +
//! ColorSwatch widgets driven by the same model fields below.

use crate::floating_panel::{FloatingPanel, PanelAction, PanelAnchor, PanelTab, ToolId};
use crate::tool::Tool;

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
        let mut panel = FloatingPanel::new(self.id(), "Brush")
            .with_tabs(vec![PanelTab {
                label: "Tip".into(),
                icon: None,
                active: true,
            }])
            .with_actions(vec![
                PanelAction {
                    label: "Size".into(),
                    icon: None,
                    enabled: true,
                },
                PanelAction {
                    label: "Opacity".into(),
                    icon: None,
                    enabled: true,
                },
                PanelAction {
                    label: "Flow".into(),
                    icon: None,
                    enabled: true,
                },
                PanelAction {
                    label: "Color".into(),
                    icon: None,
                    enabled: true,
                },
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
    fn panel_has_one_tab_and_four_actions() {
        let p = BrushTool::default().build_panel();
        assert_eq!(p.tool_id, ToolId::new("brush"));
        assert_eq!(p.title, "Brush");
        assert_eq!(p.anchor, PanelAnchor::BottomCenter);
        assert_eq!(p.width, 480.0);
        assert_eq!(p.height, 96.0);
        assert_eq!(p.tabs.len(), 1);
        assert!(p.tabs[0].active);
        assert_eq!(p.tabs[0].label, "Tip");
        let labels: Vec<&str> = p.actions.iter().map(|a| a.label.as_str()).collect();
        assert_eq!(labels, vec!["Size", "Opacity", "Flow", "Color"]);
    }
}
