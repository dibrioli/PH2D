//! **Watercolor** section setters + panel router (the wet-media look: edge darkening + granulation +
//! pigment build-up; no fluid sim — `docs/Painter/08_plano_aquarela_edge_grain_pigment.md`). The single
//! clamp source for those UI edits, mirroring [`super::jitter_settings`]. A submodule of `paint` so it
//! shares `PainterTool`'s private `paint.brush` access. The stored values are consumed at stamp time
//! (granulation / pigment, `ph2d_painter_brush::dab`) and at stroke end (edge darkening).

use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::BrushSpec;

impl PainterTool {
    /// Route the Watercolor section controls (master enable + Pigment toggle + section reset, and the
    /// Edge / Spread / Granulation / Mix sliders) from the layers panel's generic channel to the
    /// setters below. Returns `true` when it consumed the event. Mirrors
    /// [`Self::route_brush_jitter_event`]; called from `handle_panel_event` before the main match.
    pub(crate) fn route_brush_watercolor_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        match event {
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_ENABLE => {
                self.toggle_brush_watercolor();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_PIGMENT => {
                self.toggle_brush_pigment();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_RESET => {
                self.reset_brush_watercolor();
                true
            }
            PanelEvent::SetValue(id, v) => {
                let v = *v as f32;
                match *id {
                    x if x == core_ids::PAINTER_WATERCOLOR_EDGE => {
                        self.set_brush_edge_gain(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_SPREAD => {
                        self.set_brush_edge_spread(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_GRANULATION => {
                        self.set_brush_granulation(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_MIX => {
                        self.set_brush_pigment_mix(v);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Toggle the **Wet edges** master enable — gates the whole section (edge / granulation / pigment).
    /// Off (default) makes a stroke byte-identical to a plain brush.
    pub fn toggle_brush_watercolor(&mut self) {
        self.paint.brush.watercolor = !self.paint.brush.watercolor;
    }

    /// Toggle **Pigment** (subtractive Kubelka–Munk wet-on-wet colour mixing).
    pub fn toggle_brush_pigment(&mut self) {
        self.paint.brush.pigment = !self.paint.brush.pigment;
    }

    /// Set the **Edge** darkening gain (the wet-edge "fringe"), clamped to `0..=8`.
    pub fn set_brush_edge_gain(&mut self, v: f32) {
        self.paint.brush.edge_gain = v.clamp(0.0, 8.0);
    }

    /// Set the **Spread** (edge-darkening blur radius in canvas px), clamped to `1..=24`.
    pub fn set_brush_edge_spread(&mut self, v: f32) {
        self.paint.brush.edge_spread = v.clamp(1.0, 24.0);
    }

    /// Set the **Granulation** amount (paper-tooth deposit gate), clamped to `0..=1`.
    pub fn set_brush_granulation(&mut self, v: f32) {
        self.paint.brush.granulation = v.clamp(0.0, 1.0);
    }

    /// Set the **Mix** (subtractive pigment amount), clamped to `0..=1`.
    pub fn set_brush_pigment_mix(&mut self, v: f32) {
        self.paint.brush.pigment_mix = v.clamp(0.0, 1.0);
    }

    /// Reset the **Watercolor** section to defaults (section off; all params neutral). Plain paint
    /// state — no undo / pixel touch, like the other section resets.
    pub fn reset_brush_watercolor(&mut self) {
        let d = BrushSpec::default();
        let b = &mut self.paint.brush;
        b.watercolor = d.watercolor;
        b.edge_gain = d.edge_gain;
        b.edge_spread = d.edge_spread;
        b.granulation = d.granulation;
        b.pigment = d.pigment;
        b.pigment_mix = d.pigment_mix;
    }
}

#[cfg(test)]
mod tests {
    use crate::tool::PainterTool;
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};

    /// The full panel→tool seam EFFECT (the other half of the panel's `tests/seam.rs` forward proof):
    /// the exact `PanelEvent`s the panel forwards, fed to `handle_panel_event`, mutate the observable
    /// brush state (read back through the published `BrushSettings` snapshot). Also pins the clamps.
    #[test]
    fn panel_events_drive_watercolor_state() {
        let mut t = PainterTool::default();
        assert!(!t.brush_settings().watercolor, "default off");

        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_ENABLE));
        assert!(t.brush_settings().watercolor, "Wet edges toggled on");

        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_EDGE, 3.0));
        assert_eq!(t.brush_settings().edge_gain, 3.0, "Edge slider set");

        t.handle_panel_event(PanelEvent::SetValue(
            core_ids::PAINTER_WATERCOLOR_GRANULATION,
            0.5,
        ));
        assert_eq!(t.brush_settings().granulation, 0.5, "Granulation set");

        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_PIGMENT));
        assert!(t.brush_settings().pigment, "Pigment toggled on");

        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_MIX, 0.75));
        assert_eq!(t.brush_settings().pigment_mix, 0.75, "Mix set");

        // Clamp: Edge caps at 8, Spread at 24.
        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_EDGE, 99.0));
        assert_eq!(t.brush_settings().edge_gain, 8.0, "Edge clamped to 8");
        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_WATERCOLOR_SPREAD, 99.0));
        assert_eq!(t.brush_settings().edge_spread, 24.0, "Spread clamped to 24");

        // Reset returns the whole section to defaults (neutral again).
        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_WATERCOLOR_RESET));
        let b = t.brush_settings();
        assert!(
            !b.watercolor && !b.pigment && b.edge_gain == 0.0 && b.granulation == 0.0,
            "reset restored the Watercolor section to defaults"
        );
    }
}
