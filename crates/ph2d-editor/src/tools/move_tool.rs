//! [`MoveTool`] — second concrete tool. Model: snap toggles + axis lock.
//!
//! Panel: one "Transform" tab plus 2 Toggles (Snap Grid / Snap Pixel)
//! and a 3-option RadioGroup for Lock Axis (None/X/Y). The widget
//! states reflect the model; mouse interaction (next PR) writes back.

use crate::floating_panel::{FloatingPanel, PanelAnchor, PanelControl, PanelTab, ToolId};
use crate::tool::Tool;
use crate::widget::{RadioGroup, RadioOption, Toggle};
use ph2d_a11y::NodeId;

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
        let mut snap_grid = Toggle::new(NodeId(201), "Snap Grid");
        snap_grid.on = self.snap_to_grid;
        let mut snap_pixel = Toggle::new(NodeId(202), "Snap Pixel");
        snap_pixel.on = self.snap_to_pixel;

        let lock_options = vec![
            RadioOption {
                value: "none".to_string(),
                label: "None".to_string(),
                id: NodeId(210),
            },
            RadioOption {
                value: "x".to_string(),
                label: "X".to_string(),
                id: NodeId(211),
            },
            RadioOption {
                value: "y".to_string(),
                label: "Y".to_string(),
                id: NodeId(212),
            },
        ];
        let mut lock = RadioGroup::new(NodeId(203), "Lock Axis", lock_options);
        lock.select(match self.lock_axis {
            LockAxis::None => "none".to_string(),
            LockAxis::X => "x".to_string(),
            LockAxis::Y => "y".to_string(),
        });

        let mut panel = FloatingPanel::new(self.id(), "Move")
            .with_tabs(vec![PanelTab {
                label: "Transform".into(),
                icon: None,
                active: true,
            }])
            .with_controls(vec![
                PanelControl::Toggle(snap_grid),
                PanelControl::Toggle(snap_pixel),
                PanelControl::RadioGroup(lock),
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
    fn panel_has_one_tab_and_three_controls() {
        let p = MoveTool::default().build_panel();
        assert_eq!(p.tool_id, ToolId::new("move"));
        assert_eq!(p.title, "Move");
        assert_eq!(p.anchor, PanelAnchor::BottomCenter);
        assert_eq!(p.tabs.len(), 1);
        assert!(p.tabs[0].active);
        assert_eq!(p.tabs[0].label, "Transform");
        assert_eq!(p.controls.len(), 3);
        assert!(matches!(p.controls[0], PanelControl::Toggle(_)));
        assert!(matches!(p.controls[2], PanelControl::RadioGroup(_)));
        let labels: Vec<&str> = p.controls.iter().map(|c| c.label()).collect();
        assert_eq!(labels, vec!["Snap Grid", "Snap Pixel", "Lock Axis"]);
    }
}
