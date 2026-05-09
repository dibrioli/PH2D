//! [`MoveTool`] — second concrete tool. Model: snap toggles + axis lock.
//!
//! Panel: one "Transform" tab plus three placeholder actions tagging
//! the toggles and lock-axis radio. The widget-primitives PR
//! converts the placeholders into checkboxes + a 3-state segmented
//! control bound to the model fields below.

use crate::floating_panel::{FloatingPanel, PanelAction, PanelAnchor, PanelTab, ToolId};
use crate::tool::Tool;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum LockAxis {
    #[default]
    None,
    X,
    Y,
}

#[derive(Clone, Debug)]
pub struct MoveTool {
    pub snap_to_grid: bool,
    pub snap_to_pixel: bool,
    pub lock_axis: LockAxis,
}

impl Default for MoveTool {
    fn default() -> Self {
        Self {
            snap_to_grid: true,
            snap_to_pixel: true,
            lock_axis: LockAxis::None,
        }
    }
}

impl Tool for MoveTool {
    fn id(&self) -> ToolId {
        ToolId::new("move")
    }

    fn label(&self) -> &str {
        "Move"
    }

    fn icon_slug(&self) -> &str {
        "move"
    }

    fn build_panel(&self) -> FloatingPanel {
        let mut panel = FloatingPanel::new(self.id(), "Move")
            .with_tabs(vec![PanelTab {
                label: "Transform".into(),
                icon: None,
                active: true,
            }])
            .with_actions(vec![
                PanelAction {
                    label: "Snap Grid".into(),
                    icon: None,
                    enabled: true,
                },
                PanelAction {
                    label: "Snap Pixel".into(),
                    icon: None,
                    enabled: true,
                },
                PanelAction {
                    label: "Lock Axis".into(),
                    icon: None,
                    enabled: true,
                },
            ]);
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_snap_on_lock_none() {
        let m = MoveTool::default();
        assert!(m.snap_to_grid);
        assert!(m.snap_to_pixel);
        assert_eq!(m.lock_axis, LockAxis::None);
    }

    #[test]
    fn id_label_icon_are_move() {
        let m = MoveTool::default();
        assert_eq!(m.id(), ToolId::new("move"));
        assert_eq!(m.label(), "Move");
        assert_eq!(m.icon_slug(), "move");
    }

    #[test]
    fn panel_has_one_tab_and_three_actions() {
        let p = MoveTool::default().build_panel();
        assert_eq!(p.tool_id, ToolId::new("move"));
        assert_eq!(p.title, "Move");
        assert_eq!(p.anchor, PanelAnchor::BottomCenter);
        assert_eq!(p.tabs.len(), 1);
        assert!(p.tabs[0].active);
        assert_eq!(p.tabs[0].label, "Transform");
        let labels: Vec<&str> = p.actions.iter().map(|a| a.label.as_str()).collect();
        assert_eq!(labels, vec!["Snap Grid", "Snap Pixel", "Lock Axis"]);
    }
}
