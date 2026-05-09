//! [`FloatingPanel`] — Procreate-style draggable tool drawer (ADR-0023 §5).
//!
//! Per-tool contextual panel that floats over the canvas. User drags
//! it freely; position persists per `tool_id`. Auto-dismisses when
//! tool changes or 4-finger tap fires Zen mode.

use crate::zones::Rect;

/// Stable id identifying which tool this panel belongs to.
/// Position memory is keyed by this id so reactivating the tool
/// restores the user-chosen position.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ToolId(pub String);

impl ToolId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// Default snap target for newly-created panels (user can drag away).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelAnchor {
    BottomCenter,
    BottomLeft,
    BottomRight,
    TopCenter,
    Free,
}

impl PanelAnchor {
    /// Snap distance in pixels — when user releases drag this close
    /// to an anchor edge, panel re-anchors. Otherwise stays Free.
    pub const SNAP_PX: f32 = 24.0;
}

/// One tab in the optional segmented mode-picker row at the top of
/// the panel (e.g. Procreate Selection: *Automatic / Freehand /
/// Rectangle / Ellipse*).
#[derive(Clone, Debug, PartialEq)]
pub struct PanelTab {
    pub label: String,
    pub icon: Option<&'static str>,
    pub active: bool,
}

/// One action in the action grid below the tabs (e.g. *Add /
/// Remove / Invert / Copy & Paste / Feather / Save & Load /
/// Color Fill / Clear*). Used when the action is essentially a
/// labeled button — no internal state of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct PanelAction {
    pub label: String,
    pub icon: Option<&'static str>,
    pub enabled: bool,
}

/// One control in a tool panel. Lifted enum over the widget set so
/// a single `controls: Vec<PanelControl>` field can carry mixed
/// Sliders / Toggles / RadioGroups / ColorSwatches — Brush wants
/// 3 sliders + 1 swatch; Move wants 2 toggles + 1 radio group.
///
/// `Action` is the label-only fallback (the original `PanelAction`),
/// used for Procreate-style action grids like the Selection panel.
#[derive(Clone, Debug)]
pub enum PanelControl {
    Slider(crate::widget::Slider),
    Toggle(crate::widget::Toggle),
    RadioGroup(crate::widget::RadioGroup<String>),
    ColorSwatch(crate::widget::ColorSwatch),
    Action(PanelAction),
}

impl PanelControl {
    /// Display label routed to the paint pass; pulled from whatever
    /// the wrapped widget calls its label.
    pub fn label(&self) -> &str {
        match self {
            Self::Slider(s) => &s.label,
            Self::Toggle(t) => &t.label,
            Self::RadioGroup(g) => &g.label,
            Self::ColorSwatch(s) => &s.label,
            Self::Action(a) => &a.label,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FloatingPanel {
    pub tool_id: ToolId,
    pub title: String,
    pub anchor: PanelAnchor,
    /// Pixel position when `anchor == Free`. Ignored when anchored.
    pub position: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub tabs: Vec<PanelTab>,
    /// Label-only actions (Procreate Selection-style action grid).
    /// Empty for tools that use the richer [`Self::controls`] vector.
    pub actions: Vec<PanelAction>,
    /// Mixed widgets (Slider/Toggle/RadioGroup/ColorSwatch). When
    /// non-empty, the paint pass renders these instead of `actions`.
    pub controls: Vec<PanelControl>,
    pub collapsed: bool,
    pub visible: bool,
}

impl FloatingPanel {
    /// Construct an empty panel for a tool. Tabs and actions are
    /// pushed by the caller.
    pub fn new(tool_id: ToolId, title: impl Into<String>) -> Self {
        Self {
            tool_id,
            title: title.into(),
            anchor: PanelAnchor::BottomCenter,
            position: (0.0, 0.0),
            width: 480.0, // ~Procreate selection panel width
            height: 96.0, // 1 tab row + 1 action row + padding
            tabs: Vec::new(),
            actions: Vec::new(),
            controls: Vec::new(),
            collapsed: false,
            visible: true,
        }
    }

    pub fn with_tabs(mut self, tabs: Vec<PanelTab>) -> Self {
        self.tabs = tabs;
        self
    }

    pub fn with_actions(mut self, actions: Vec<PanelAction>) -> Self {
        self.actions = actions;
        self
    }

    pub fn with_controls(mut self, controls: Vec<PanelControl>) -> Self {
        self.controls = controls;
        self
    }

    /// Compute the panel's pixel rect within `viewport`. Honors
    /// anchor. Caller is the layout system; this is pure math.
    pub fn rect(&self, viewport: Rect) -> Rect {
        if self.collapsed {
            // Collapsed = chip-icon-sized at anchor edge.
            let chip = 40.0;
            return self.anchor_rect(viewport, chip, chip);
        }
        self.anchor_rect(viewport, self.width, self.height)
    }

    fn anchor_rect(&self, viewport: Rect, w: f32, h: f32) -> Rect {
        let margin = 16.0;
        match self.anchor {
            PanelAnchor::BottomCenter => Rect::new(
                viewport.x + (viewport.w - w) / 2.0,
                viewport.y + viewport.h - h - margin,
                w,
                h,
            ),
            PanelAnchor::BottomLeft => Rect::new(
                viewport.x + margin,
                viewport.y + viewport.h - h - margin,
                w,
                h,
            ),
            PanelAnchor::BottomRight => Rect::new(
                viewport.x + viewport.w - w - margin,
                viewport.y + viewport.h - h - margin,
                w,
                h,
            ),
            PanelAnchor::TopCenter => Rect::new(
                viewport.x + (viewport.w - w) / 2.0,
                viewport.y + margin,
                w,
                h,
            ),
            PanelAnchor::Free => Rect::new(self.position.0, self.position.1, w, h),
        }
    }

    /// User dropped the panel after a drag. If close to an anchor
    /// edge, snap to it; otherwise switch to `Free` anchor and
    /// remember the absolute position.
    pub fn snap_or_free(&mut self, dropped_x: f32, dropped_y: f32, viewport: Rect) {
        let snap = PanelAnchor::SNAP_PX;
        let bottom = viewport.y + viewport.h - self.height;
        let center_x = viewport.x + (viewport.w - self.width) / 2.0;
        let left = viewport.x;
        let right = viewport.x + viewport.w - self.width;
        let top = viewport.y;

        if (dropped_y - bottom).abs() < snap && (dropped_x - center_x).abs() < snap {
            self.anchor = PanelAnchor::BottomCenter;
        } else if (dropped_y - bottom).abs() < snap && (dropped_x - left).abs() < snap {
            self.anchor = PanelAnchor::BottomLeft;
        } else if (dropped_y - bottom).abs() < snap && (dropped_x - right).abs() < snap {
            self.anchor = PanelAnchor::BottomRight;
        } else if (dropped_y - top).abs() < snap && (dropped_x - center_x).abs() < snap {
            self.anchor = PanelAnchor::TopCenter;
        } else {
            self.anchor = PanelAnchor::Free;
            self.position = (dropped_x, dropped_y);
        }
    }

    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }

    pub fn dismiss(&mut self) {
        self.visible = false;
    }
}

/// Convenience constructor for the canonical Procreate Selection
/// panel demo (per ADR-0023 ratification image: tabs Automatic /
/// Freehand / Rectangle / Ellipse + 8 actions Add / Remove / Invert
/// / Copy & Paste / Feather / Save & Load / Color Fill / Clear).
///
/// Useful in tests and as the M12 demo.
pub fn selection_demo_panel() -> FloatingPanel {
    FloatingPanel::new(ToolId::new("selection"), "Selection")
        .with_tabs(vec![
            PanelTab {
                label: "Automatic".into(),
                icon: None,
                active: false,
            },
            PanelTab {
                label: "Freehand".into(),
                icon: None,
                active: true,
            },
            PanelTab {
                label: "Rectangle".into(),
                icon: None,
                active: false,
            },
            PanelTab {
                label: "Ellipse".into(),
                icon: None,
                active: false,
            },
        ])
        .with_actions(vec![
            PanelAction {
                label: "Add".into(),
                icon: Some("+"),
                enabled: true,
            },
            PanelAction {
                label: "Remove".into(),
                icon: Some("−"),
                enabled: true,
            },
            PanelAction {
                label: "Invert".into(),
                icon: None,
                enabled: true,
            },
            PanelAction {
                label: "Copy & Paste".into(),
                icon: None,
                enabled: true,
            },
            PanelAction {
                label: "Feather".into(),
                icon: None,
                enabled: true,
            },
            PanelAction {
                label: "Save & Load".into(),
                icon: None,
                enabled: true,
            },
            PanelAction {
                label: "Color Fill".into(),
                icon: None,
                enabled: true,
            },
            PanelAction {
                label: "Clear".into(),
                icon: None,
                enabled: true,
            },
        ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 1024.0, 768.0)
    }

    #[test]
    fn default_anchor_is_bottom_center() {
        let p = FloatingPanel::new(ToolId::new("t"), "Test");
        assert_eq!(p.anchor, PanelAnchor::BottomCenter);
    }

    #[test]
    fn rect_at_bottom_center_is_horizontally_centered() {
        let p = FloatingPanel::new(ToolId::new("t"), "Test");
        let r = p.rect(viewport());
        let center_x = (1024.0 - p.width) / 2.0;
        assert_eq!(r.x, center_x);
    }

    #[test]
    fn collapsed_panel_is_chip_sized() {
        let mut p = FloatingPanel::new(ToolId::new("t"), "Test");
        p.toggle_collapsed();
        let r = p.rect(viewport());
        assert_eq!(r.w, 40.0);
        assert_eq!(r.h, 40.0);
    }

    #[test]
    fn snap_to_bottom_center_when_close() {
        let mut p = FloatingPanel::new(ToolId::new("t"), "Test");
        p.anchor = PanelAnchor::Free;
        // Drop within snap distance of bottom-center.
        let bottom = 768.0 - p.height;
        let center_x = (1024.0 - p.width) / 2.0;
        p.snap_or_free(center_x + 10.0, bottom + 5.0, viewport());
        assert_eq!(p.anchor, PanelAnchor::BottomCenter);
    }

    #[test]
    fn drop_far_from_anchors_becomes_free() {
        let mut p = FloatingPanel::new(ToolId::new("t"), "Test");
        p.snap_or_free(300.0, 200.0, viewport());
        assert_eq!(p.anchor, PanelAnchor::Free);
        assert_eq!(p.position, (300.0, 200.0));
    }

    #[test]
    fn dismiss_hides() {
        let mut p = FloatingPanel::new(ToolId::new("t"), "Test");
        assert!(p.visible);
        p.dismiss();
        assert!(!p.visible);
    }

    #[test]
    fn selection_demo_matches_procreate_screenshot() {
        // The reference screenshot from Enio's ratification had
        // these exact 4 tabs and 8 actions. Locking the demo so we
        // notice if anyone strips an item by accident.
        let p = selection_demo_panel();
        assert_eq!(p.tool_id, ToolId::new("selection"));
        assert_eq!(p.title, "Selection");
        assert_eq!(p.tabs.len(), 4);
        assert_eq!(p.tabs[1].label, "Freehand");
        assert!(p.tabs[1].active);
        assert_eq!(p.actions.len(), 8);
        let labels: Vec<&str> = p.actions.iter().map(|a| a.label.as_str()).collect();
        assert_eq!(
            labels,
            vec![
                "Add",
                "Remove",
                "Invert",
                "Copy & Paste",
                "Feather",
                "Save & Load",
                "Color Fill",
                "Clear"
            ]
        );
    }
}
