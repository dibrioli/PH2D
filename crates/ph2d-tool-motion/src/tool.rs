//! `MotionTool` — the concrete Motion Nodes tool type behind the `motion` pill.
//!
//! A thin activation handle (ADR-0040): it carries no document state — the
//! `MotionDoc`, transport and persistent `Cook` live in the shell's
//! `MotionState`, driven by `render_loop::motion_bridge`. The tool's job is to
//! (a) be a registrable `Tool` so the pill activates it, and (b) exist as a
//! concrete type the bridge can downcast for future tool-scoped settings. M0.T9
//! ships this skeleton; graph interaction lands in M1.

use ph2d_editor_core::floating_panel::{FloatingPanel, PanelAnchor, ToolId};
use ph2d_editor_core::tool::Tool;

/// The Motion Nodes tool. Zero-size for now — activation-only in M0.
#[derive(Default)]
pub struct MotionTool;

impl MotionTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for MotionTool {
    fn id(&self) -> ToolId {
        ToolId::new("motion")
    }

    fn label(&self) -> &str {
        "Motion"
    }

    fn icon_slug(&self) -> &str {
        "motion-nodes"
    }

    fn build_panel(&self) -> FloatingPanel {
        // Tool `FloatingPanel`s are unpainted in this app (input-dispatch only);
        // the real UI is the docked `ph2d-panel-motion-graph` / `-params` crates,
        // shown via `panel_visible` from the bridge. Return an empty shell so
        // `Tool::build_panel` has a value (mirror of `VectorTool`).
        let mut panel = FloatingPanel::new(self.id(), "Motion");
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_label_icon_stable() {
        let t = MotionTool::new();
        assert_eq!(t.id(), ToolId::new("motion"));
        assert_eq!(t.label(), "Motion");
        assert_eq!(t.icon_slug(), "motion-nodes");
    }

    #[test]
    fn empty_panel_has_no_controls() {
        let t = MotionTool::new();
        let panel = t.build_panel();
        assert!(panel.controls.is_empty());
    }

    #[test]
    fn as_any_mut_downcasts_to_self() {
        let mut t = MotionTool::new();
        assert!(t.as_any_mut().downcast_mut::<MotionTool>().is_some());
    }
}
