//! [`MoveTool`] — second concrete tool. Model: snap toggles + axis lock.
//!
//! Panel: one "Transform" tab plus 2 Toggles (Snap Grid / Snap Pixel)
//! and a 3-option RadioGroup for Lock Axis (None/X/Y). The widget
//! states reflect the model; mouse interaction (next PR) writes back.

use crate::floating_panel::{FloatingPanel, PanelAnchor, PanelControl, PanelTab, ToolId};
use crate::tool::{PanelEvent, Tool};
use crate::widget::{RadioGroup, RadioOption, Toggle};
use ph2d_a11y::NodeId;

const SNAP_GRID_NODE: NodeId = NodeId(201);
const SNAP_PIXEL_NODE: NodeId = NodeId(202);
const LOCK_AXIS_NODE: NodeId = NodeId(203);

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
        let mut snap_grid = Toggle::new(SNAP_GRID_NODE, "Snap Grid");
        snap_grid.on = self.snap_to_grid;
        let mut snap_pixel = Toggle::new(SNAP_PIXEL_NODE, "Snap Pixel");
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
        let mut lock = RadioGroup::new(LOCK_AXIS_NODE, "Lock Axis", lock_options);
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

    fn handle_panel_event(&mut self, event: PanelEvent) {
        match event {
            PanelEvent::Toggle(id, on) if id == SNAP_GRID_NODE => self.snap_to_grid = on,
            PanelEvent::Toggle(id, on) if id == SNAP_PIXEL_NODE => self.snap_to_pixel = on,
            PanelEvent::SelectOption(id, v) if id == LOCK_AXIS_NODE => {
                self.lock_axis = match v.as_str() {
                    "x" => LockAxis::X,
                    "y" => LockAxis::Y,
                    _ => LockAxis::None,
                };
            }
            _ => {}
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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
