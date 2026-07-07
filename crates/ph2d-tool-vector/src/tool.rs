//! [`VectorTool`] — the Style model for the Vector drawing tool.
//!
//! The tool is deliberately thin: it holds the current stroke colour, fill
//! colour, and stroke width. The real UI is the **docked** `ph2d-panel-vector`
//! (a `Panel<State>`, right-docked in the Inspector slot while the tool is
//! active) — tool `FloatingPanel`s are unpainted in this app, so the panel is a
//! separate crate that drives the tool through the generic `ToolPanelEvent`
//! channel + colour-picker read-back (mirror of the Padding tool+panel pair).
//!
//! The shell's `vector_bridge` reads this Style each frame (downcast via
//! [`Tool::as_any_mut`]) to restyle newly drawn paths and — on a picker pick /
//! Fill-None — recolour the selected path.
//!
//! ## Colour approach
//!
//! Docked panels CAN drive the shared OKLCH (Blender) colour picker (unlike
//! tool `FloatingPanel`s): the panel paints a `ColorSwatch` + calls
//! `register_picker_swatch`, the shell reads the picked colour back and feeds it
//! through [`VectorTool::set_stroke_rgba`] / [`VectorTool::set_fill_rgba`]. The
//! [`PALETTE`] below is retained as a curated preset list (seeds the defaults);
//! the picker is the live path.

use ph2d_editor_core::floating_panel::{FloatingPanel, PanelAnchor, ToolId};
use ph2d_editor_core::ids;
use ph2d_editor_core::tool::{PanelEvent, Tool};

use crate::params::{DrawMode, VectorStyleSnapshot, slider_to_px, slider_to_sides};

/// Curated stroke / fill preset palette: `(key, label, sRGB8)`. Retained as the
/// seed source for the tool's defaults (and a stable named-colour reference);
/// the live colour path is the OKLCH picker driven by the docked panel.
pub const PALETTE: &[(&str, &str, [u8; 4])] = &[
    ("white", "White", [240, 240, 245, 255]),
    ("black", "Black", [20, 20, 24, 255]),
    ("gray", "Gray", [130, 130, 138, 255]),
    ("red", "Red", [220, 60, 60, 255]),
    ("orange", "Orange", [230, 140, 40, 255]),
    ("yellow", "Yellow", [235, 205, 50, 255]),
    ("green", "Green", [70, 190, 90, 255]),
    ("cyan", "Cyan", [60, 190, 205, 255]),
    ("blue", "Blue", [90, 150, 230, 255]),
    ("purple", "Purple", [160, 110, 220, 255]),
];

/// Fill "None" colour (alpha 0 = no visible fill on close).
const FILL_NONE: [u8; 4] = [0, 0, 0, 0];

/// Default stroke width in screen pixels (matches the old `PenTool` default).
pub const DEFAULT_STROKE_WIDTH_PX: f64 = 3.0;

/// Default polygon side count (a pentagon reads clearly as "polygon").
pub const DEFAULT_POLYGON_SIDES: u32 = 5;

/// Look up a palette colour by key (defaults only — the live path is the picker).
fn color_of(key: &str) -> Option<[u8; 4]> {
    PALETTE
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, _, c)| *c)
}

/// Vector drawing tool — Style + draw-mode model only.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorTool {
    stroke: [u8; 4],
    /// Fill applied on close; alpha 0 ⇒ no fill.
    fill: [u8; 4],
    /// Stroke width in screen pixels, held in `WIDTH_MIN_PX..=WIDTH_MAX_PX`.
    stroke_width_px: f64,
    /// Canvas gesture: Pen (draw + edit) vs a drag-to-size shape. The shell
    /// mirrors this each frame to route canvas input (`vector_bridge`).
    mode: DrawMode,
    /// Sides for `DrawMode::Polygon`, held in `SIDES_MIN..=SIDES_MAX`.
    polygon_sides: u32,
    /// Set when a colour changes → the shell recolours the selected path.
    /// Drained by [`Self::take_apply_to_selected`].
    apply_to_selected: bool,
}

impl Default for VectorTool {
    fn default() -> Self {
        Self {
            stroke: color_of("white").unwrap_or([240, 240, 245, 255]),
            fill: color_of("blue").unwrap_or([90, 150, 230, 255]),
            stroke_width_px: DEFAULT_STROKE_WIDTH_PX,
            mode: DrawMode::Pen,
            polygon_sides: DEFAULT_POLYGON_SIDES,
            apply_to_selected: false,
        }
    }
}

impl VectorTool {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Current stroke colour (sRGB8).
    #[must_use]
    pub fn stroke_rgba(&self) -> [u8; 4] {
        self.stroke
    }

    /// Current fill colour (sRGB8); alpha 0 ⇒ no fill on close.
    #[must_use]
    pub fn fill_rgba(&self) -> [u8; 4] {
        self.fill
    }

    /// Current stroke width in screen pixels.
    #[must_use]
    pub fn stroke_width_px(&self) -> f64 {
        self.stroke_width_px
    }

    /// Current canvas draw-mode (the shell mirrors this to route input).
    #[must_use]
    pub fn mode(&self) -> DrawMode {
        self.mode
    }

    /// Current polygon side count (only meaningful in `DrawMode::Polygon`).
    #[must_use]
    pub fn polygon_sides(&self) -> u32 {
        self.polygon_sides
    }

    /// Set the stroke colour (picker read-back) + flag the selected path for
    /// recolour. `a = 0` is accepted (a fully-transparent stroke is unusual but
    /// not rejected here — the panel drives opaque picks).
    pub fn set_stroke_rgba(&mut self, rgba: [u8; 4]) {
        self.stroke = rgba;
        self.apply_to_selected = true;
    }

    /// Set the fill colour (picker read-back) + flag the selected path for
    /// recolour. `a = 0` ⇒ "None" (no fill).
    pub fn set_fill_rgba(&mut self, rgba: [u8; 4]) {
        self.fill = rgba;
        self.apply_to_selected = true;
    }

    /// Project the current Style into the snapshot the docked panel paints.
    #[must_use]
    pub fn ui_snapshot(&self) -> VectorStyleSnapshot {
        VectorStyleSnapshot {
            stroke: self.stroke,
            fill: self.fill,
            stroke_width_px: self.stroke_width_px,
            mode: self.mode,
            polygon_sides: self.polygon_sides,
        }
    }

    /// Drain the "recolour the selected path" request (set on any colour change).
    pub fn take_apply_to_selected(&mut self) -> bool {
        std::mem::take(&mut self.apply_to_selected)
    }
}

impl Tool for VectorTool {
    fn id(&self) -> ToolId {
        ToolId::new("vector")
    }

    fn label(&self) -> &str {
        "Vector"
    }

    fn icon_slug(&self) -> &str {
        "vector"
    }

    fn build_panel(&self) -> FloatingPanel {
        // The real UI is the docked `ph2d-panel-vector` crate; tool
        // `FloatingPanel`s are unpainted (input-dispatch only) in this app. A
        // minimal empty panel shell is returned so `Tool::build_panel` has a
        // value — it carries no controls (mirror of `PaddingTool`).
        let mut panel = FloatingPanel::new(self.id(), "Vector");
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        // Docked-panel control ids are the shared `ph2d_editor_core::ids::VECTOR_*`
        // chrome NodeIds (the panel forwards `SetValue` / `Click` over
        // `ToolPanelEvent`; the swatch colours arrive via the setters above,
        // driven by the picker read-back in `vector_bridge`).
        match event {
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_WIDTH => {
                self.stroke_width_px = slider_to_px(v as f32);
                // Also restyle the selected path (mirror of a colour change), so
                // the width slider affects the path you're looking at — not just
                // the next one drawn.
                self.apply_to_selected = true;
            }
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_SIDES => {
                self.polygon_sides = slider_to_sides(v as f32);
            }
            PanelEvent::Click(id) if id == ids::VECTOR_FILL_NONE => {
                self.fill = FILL_NONE;
                self.apply_to_selected = true;
            }
            // Draw-mode segmented row: switches the canvas gesture. No recolour
            // (mode is not a Style change) — the shell reads `mode()` to route.
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_PEN => self.mode = DrawMode::Pen,
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_RECT => self.mode = DrawMode::Rectangle,
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_ELLIPSE => {
                self.mode = DrawMode::Ellipse
            }
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_POLYGON => {
                self.mode = DrawMode::Polygon
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
    use crate::params::{WIDTH_MAX_PX, WIDTH_MIN_PX};
    use ph2d_a11y::NodeId;

    #[test]
    fn fresh_tool_defaults() {
        let t = VectorTool::new();
        assert_eq!(t.stroke_rgba(), [240, 240, 245, 255]); // white
        assert_eq!(t.fill_rgba(), [90, 150, 230, 255]); // blue
        assert_eq!(t.stroke_width_px(), DEFAULT_STROKE_WIDTH_PX);
    }

    #[test]
    fn width_slider_maps_normalized_to_px() {
        let mut t = VectorTool::new();
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_WIDTH, 0.0));
        assert_eq!(t.stroke_width_px(), WIDTH_MIN_PX);
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_WIDTH, 1.0));
        assert_eq!(t.stroke_width_px(), WIDTH_MAX_PX);
    }

    #[test]
    fn set_stroke_sets_colour_and_flags_apply() {
        let mut t = VectorTool::new();
        assert!(!t.take_apply_to_selected());
        t.set_stroke_rgba([220, 60, 60, 255]);
        assert_eq!(t.stroke_rgba(), [220, 60, 60, 255]);
        assert!(t.take_apply_to_selected());
        assert!(!t.take_apply_to_selected(), "drained");
    }

    #[test]
    fn set_fill_sets_colour_and_flags_apply() {
        let mut t = VectorTool::new();
        t.set_fill_rgba([70, 190, 90, 255]);
        assert_eq!(t.fill_rgba(), [70, 190, 90, 255]);
        assert!(t.take_apply_to_selected());
    }

    #[test]
    fn fill_none_click_sets_transparent_and_flags_apply() {
        let mut t = VectorTool::new();
        Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::VECTOR_FILL_NONE));
        assert_eq!(t.fill_rgba()[3], 0);
        assert!(t.take_apply_to_selected());
    }

    #[test]
    fn foreign_node_id_ignored() {
        let mut t = VectorTool::new();
        let before = t.clone();
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(NodeId(999), 0.5));
        Tool::handle_panel_event(&mut t, PanelEvent::Click(NodeId(999)));
        assert_eq!(t, before);
    }

    #[test]
    fn mode_buttons_switch_the_draw_mode() {
        let mut t = VectorTool::new();
        assert_eq!(t.mode(), DrawMode::Pen); // default
        for (id, want) in [
            (ids::VECTOR_MODE_RECT, DrawMode::Rectangle),
            (ids::VECTOR_MODE_ELLIPSE, DrawMode::Ellipse),
            (ids::VECTOR_MODE_POLYGON, DrawMode::Polygon),
            (ids::VECTOR_MODE_PEN, DrawMode::Pen),
        ] {
            Tool::handle_panel_event(&mut t, PanelEvent::Click(id));
            assert_eq!(t.mode(), want);
        }
        // Mode change is NOT a Style edit → never flags a recolour.
        assert!(!t.take_apply_to_selected());
    }

    #[test]
    fn sides_slider_maps_normalized_to_sides() {
        use crate::params::{SIDES_MAX, SIDES_MIN};
        let mut t = VectorTool::new();
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_SIDES, 0.0));
        assert_eq!(t.polygon_sides(), SIDES_MIN);
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_SIDES, 1.0));
        assert_eq!(t.polygon_sides(), SIDES_MAX);
    }

    #[test]
    fn ui_snapshot_round_trips_style() {
        let mut t = VectorTool::new();
        t.set_stroke_rgba([1, 2, 3, 255]);
        t.set_fill_rgba([4, 5, 6, 255]);
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_WIDTH, 0.5));
        Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::VECTOR_MODE_POLYGON));
        Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_SIDES, 1.0));
        let s = t.ui_snapshot();
        assert_eq!(s.stroke, [1, 2, 3, 255]);
        assert_eq!(s.fill, [4, 5, 6, 255]);
        assert_eq!(s.stroke_width_px, t.stroke_width_px());
        assert_eq!(s.mode, DrawMode::Polygon);
        assert_eq!(s.polygon_sides, t.polygon_sides());
    }

    #[test]
    fn empty_panel_has_no_controls() {
        let t = VectorTool::new();
        let panel = t.build_panel();
        assert!(panel.controls.is_empty());
    }

    #[test]
    fn id_label_icon_stable() {
        let t = VectorTool::new();
        assert_eq!(t.id(), ToolId::new("vector"));
        assert_eq!(t.label(), "Vector");
        assert_eq!(t.icon_slug(), "vector");
    }
}
