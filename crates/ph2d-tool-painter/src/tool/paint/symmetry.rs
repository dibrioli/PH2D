//! Drawing **symmetry** — the tool-side glue for the brush engine's mirror/radial dab replication
//! (`ph2d_painter_brush::symmetry`). Like seamless [`super::tiling`], symmetry is plain paint state
//! (no undo / pixel touch): the panel toggles it, the engine reads `brush.symmetry` per dab. This
//! module owns the panel-driven setters, the **centre resolution** (X/Y mirror and the radial default
//! pivot on the canvas centre, which the engine can't know), and the two on-canvas **pick** modes —
//! draw the custom mirror line, or pick the radial centre — modelled on the colour-picker button.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use ph2d_painter_brush::{
    MirrorAxis, SYMMETRY_MAX_SEGMENTS, SYMMETRY_MIN_SEGMENTS, SymmetrySettings,
};

/// Which on-canvas geometry the armed pointer gesture sets (the tool paints normally when this is off).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SymmetryPick {
    /// Draw the custom mirror line: Down = first endpoint, drag previews it live, Up commits it.
    Line,
    /// Pick the radial centre with a click.
    Center,
}

impl PainterTool {
    // ── panel-driven setters (plain state, like Tiling — no undo, no pixel touch) ──────────────────

    /// Toggle **Use Symmetry** (the master switch). Off ⇒ painting is byte-identical to no symmetry.
    pub fn toggle_symmetry_enabled(&mut self) {
        self.paint.brush.symmetry.enabled = !self.paint.brush.symmetry.enabled;
        self.resolve_symmetry_geometry();
    }

    /// Toggle **Circular** (radial) vs. mirror symmetry.
    pub fn toggle_symmetry_circular(&mut self) {
        self.paint.brush.symmetry.circular = !self.paint.brush.symmetry.circular;
        self.resolve_symmetry_geometry();
    }

    /// Set the mirror **axis** (X / Y / Custom). X and Y pivot on the canvas centre; Custom keeps the
    /// last drawn line. Selecting X/Y re-arms auto-centre so the axis snaps back to the middle.
    pub fn set_symmetry_axis(&mut self, axis: MirrorAxis) {
        self.paint.brush.symmetry.axis = axis;
        if axis != MirrorAxis::Custom {
            self.paint.symmetry_auto_center = true;
        }
        self.resolve_symmetry_geometry();
    }

    /// Set the radial **segment** count, clamped to the UI range `[3, 12]`.
    pub fn set_symmetry_segments(&mut self, n: u32) {
        self.paint.brush.symmetry.radial_segments =
            n.clamp(SYMMETRY_MIN_SEGMENTS, SYMMETRY_MAX_SEGMENTS);
    }

    /// Reset the whole Symmetry section to defaults (disabled, mirror X, auto-centre).
    pub fn reset_symmetry(&mut self) {
        self.paint.brush.symmetry = SymmetrySettings::default();
        self.paint.symmetry_auto_center = true;
        self.paint.symmetry_pick = None;
        self.paint.symmetry_line_start = None;
        self.resolve_symmetry_geometry();
    }

    // ── canvas pick-mode entry (the colour-picker-button analogue) ─────────────────────────────────

    /// Arm the **draw custom line** mode: the next canvas drag sets the mirror line (and selects the
    /// Custom axis). A second press of the button (or [`Self::cancel_symmetry_pick`]) disarms it.
    pub fn begin_symmetry_pick_line(&mut self) {
        if self.paint.symmetry_pick == Some(SymmetryPick::Line) {
            self.cancel_symmetry_pick();
            return;
        }
        self.paint.symmetry_pick = Some(SymmetryPick::Line);
        self.paint.symmetry_line_start = None;
        self.paint.brush.symmetry.axis = MirrorAxis::Custom;
    }

    /// Arm the **pick radial centre** mode: the next canvas click sets the rosette centre.
    pub fn begin_symmetry_pick_center(&mut self) {
        if self.paint.symmetry_pick == Some(SymmetryPick::Center) {
            self.cancel_symmetry_pick();
            return;
        }
        self.paint.symmetry_pick = Some(SymmetryPick::Center);
    }

    /// Disarm any active symmetry pick mode (resume normal painting).
    pub fn cancel_symmetry_pick(&mut self) {
        self.paint.symmetry_pick = None;
        self.paint.symmetry_line_start = None;
    }

    // ── queries (panel read-back + overlay) ────────────────────────────────────────────────────────

    /// The current symmetry settings — the panel reads this to show the toggles / slider / axis, and
    /// the shell overlay reads it (with the centre already resolved) to draw the dashed guides.
    #[must_use]
    pub fn symmetry(&self) -> SymmetrySettings {
        self.paint.brush.symmetry
    }

    /// Whether a symmetry pick mode is armed (a canvas gesture will set geometry, not paint).
    #[must_use]
    pub fn symmetry_pick_active(&self) -> bool {
        self.paint.symmetry_pick.is_some()
    }

    // ── geometry resolution + canvas pick handling ─────────────────────────────────────────────────

    /// Snap the symmetry **centre** to the canvas centre for the auto-centre modes (X / Y mirror and
    /// the radial default), leaving a user-drawn custom line / picked centre intact. Called whenever
    /// the source size or a setter changes, so every `Stroke` built from `brush` and the overlay read a
    /// current centre without the engine needing to know the canvas dimensions.
    pub(crate) fn resolve_symmetry_geometry(&mut self) {
        if self.paint.symmetry_auto_center {
            let (w, h) = self.source_size;
            self.paint.brush.symmetry.center = [w as f32 * 0.5, h as f32 * 0.5];
        }
    }

    /// Handle a canvas pointer while a pick mode is armed; returns `true` (always consumes the event so
    /// it never paints). Center: a click sets the pivot. Line: Down anchors, Move previews, Up commits.
    pub(crate) fn symmetry_pick_pointer(&mut self, ev: CanvasPointer) -> bool {
        match self.paint.symmetry_pick {
            Some(SymmetryPick::Center) => {
                if ev.phase != PointerPhase::Hover {
                    self.paint.brush.symmetry.center = ev.pos;
                    self.paint.symmetry_auto_center = false;
                }
                if ev.phase == PointerPhase::Up {
                    self.paint.symmetry_pick = None;
                }
                true
            }
            Some(SymmetryPick::Line) => {
                match ev.phase {
                    PointerPhase::Down => self.paint.symmetry_line_start = Some(ev.pos),
                    PointerPhase::Move | PointerPhase::Up => {
                        if let Some(a) = self.paint.symmetry_line_start {
                            self.set_symmetry_custom_line(a, ev.pos);
                        }
                        if ev.phase == PointerPhase::Up {
                            self.paint.symmetry_pick = None;
                            self.paint.symmetry_line_start = None;
                        }
                    }
                    PointerPhase::Hover => {}
                }
                true
            }
            None => false,
        }
    }

    /// Commit a custom mirror line through `a → b`: the line's direction is `b − a` (a degenerate
    /// zero-length drag falls back to a vertical line), its anchor the midpoint. Selects the Custom
    /// axis and drops auto-centre so the line is pinned where the artist drew it.
    fn set_symmetry_custom_line(&mut self, a: [f32; 2], b: [f32; 2]) {
        let d = [b[0] - a[0], b[1] - a[1]];
        let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let dir = if len < 1e-3 {
            [0.0, 1.0]
        } else {
            [d[0] / len, d[1] / len]
        };
        self.paint.brush.symmetry.axis = MirrorAxis::Custom;
        self.paint.brush.symmetry.custom_dir = dir;
        self.paint.brush.symmetry.center = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
        self.paint.symmetry_auto_center = false;
    }
}

#[cfg(test)]
mod tests {
    use crate::tool::PainterTool;
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{
        CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool,
    };
    use ph2d_painter_brush::MirrorAxis;

    fn cp(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
        CanvasPointer {
            pos: [x, y],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        }
    }

    /// The full panel→tool seam EFFECT (the other half of `tests/seam.rs`'s forward proof): the real
    /// `PanelEvent` the panel forwards, fed to `handle_panel_event`, mutates the observable symmetry state.
    #[test]
    fn panel_clicks_drive_symmetry_state() {
        use ph2d_editor_core::tool::PanelEvent;
        let mut t = PainterTool::default();
        assert!(!t.symmetry().enabled, "default off");

        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SYMMETRY_USE));
        assert!(t.symmetry().enabled, "Use toggled symmetry on");

        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SYMMETRY_CIRCULAR));
        assert!(t.symmetry().circular, "Circular toggled on");

        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SYMMETRY_AXIS_Y));
        assert_eq!(t.symmetry().axis, MirrorAxis::Y, "axis set to Y");

        // The segment slider's 0..1 track maps onto 3..12.
        t.handle_panel_event(PanelEvent::SetValue(
            core_ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS,
            1.0,
        ));
        assert_eq!(t.symmetry().radial_segments, 12, "track 1.0 → 12 segments");
        t.handle_panel_event(PanelEvent::SetValue(
            core_ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS,
            0.0,
        ));
        assert_eq!(t.symmetry().radial_segments, 3, "track 0.0 → 3 segments");

        // Reset returns the whole section to defaults.
        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SYMMETRY_RESET));
        assert!(!t.symmetry().enabled && !t.symmetry().circular);
        assert_eq!(t.symmetry().axis, MirrorAxis::X);
    }

    /// "Draw Custom Line" arms the pick mode, and a canvas drag sets the Custom mirror line — the
    /// observable effect of the on-canvas geometry mode (works regardless of the active layer).
    #[test]
    fn draw_line_pick_sets_custom_axis_from_canvas_drag() {
        use ph2d_editor_core::tool::PanelEvent;
        let mut t = PainterTool::default();
        t.handle_panel_event(PanelEvent::Click(
            core_ids::PAINTER_BRUSH_SYMMETRY_DRAW_LINE,
        ));
        assert!(t.symmetry_pick_active(), "Draw-Line armed a pick mode");
        assert_eq!(
            t.symmetry().axis,
            MirrorAxis::Custom,
            "Draw-Line selects Custom"
        );

        // A vertical drag (10,10)→(10,30): the line direction is +y, anchored at the midpoint.
        assert!(t.on_canvas_pointer(cp(10.0, 10.0, PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp(10.0, 30.0, PointerPhase::Up)));
        assert!(!t.symmetry_pick_active(), "pick mode cleared on pen-up");
        let s = t.symmetry();
        assert!(
            (s.custom_dir[0]).abs() < 1e-3 && (s.custom_dir[1] - 1.0).abs() < 1e-3,
            "vertical dir: {:?}",
            s.custom_dir
        );
        assert!(
            (s.center[0] - 10.0).abs() < 1e-3 && (s.center[1] - 20.0).abs() < 1e-3,
            "midpoint centre: {:?}",
            s.center
        );
    }

    /// "Pick Center" arms the pick, and a canvas click sets the radial centre (clearing auto-centre).
    #[test]
    fn pick_center_sets_radial_centre_from_canvas_click() {
        use ph2d_editor_core::tool::PanelEvent;
        let mut t = PainterTool::default();
        t.handle_panel_event(PanelEvent::Click(
            core_ids::PAINTER_BRUSH_SYMMETRY_PICK_CENTER,
        ));
        assert!(t.symmetry_pick_active());
        assert!(t.on_canvas_pointer(cp(5.0, 7.0, PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp(5.0, 7.0, PointerPhase::Up)));
        assert!(!t.symmetry_pick_active(), "cleared on pen-up");
        let s = t.symmetry();
        assert_eq!(s.center, [5.0, 7.0], "picked centre");
    }

    /// The auto-centre modes pin the symmetry centre to the canvas centre: the per-frame heartbeat
    /// (`on_tick` → `paint_tick` → `resolve_symmetry_geometry`) re-pins it after a fresh source bind, so
    /// the overlay guide is correct even before the first stroke.
    #[test]
    fn auto_centre_tracks_the_canvas_centre() {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; 64 * 40 * 4], 64, 40);
        t.on_tick(16.0); // one frame heartbeat resolves the auto centre
        assert_eq!(
            t.symmetry().center,
            [32.0, 20.0],
            "auto centre = canvas centre after a tick"
        );
    }
}
