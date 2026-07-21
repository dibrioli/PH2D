//! The Wet Paint **authored settings** — the checkbox (the persistent ARM), the W3 curated
//! knobs and their panel routing. Split from `wetpaint.rs` (workspace file-LOC cap), the exact
//! sibling pattern of `watercolor_settings.rs`: this file owns the AUTHORED state and its doors;
//! `wetpaint.rs` owns the SESSION (display-state) that reconciles against it.

use super::*;

/// The W3 **curated knobs** — SPEC §16's ~40-knob tuning table curated to the
/// seven an artist reaches for (the rest stay the engine's named constants).
/// Authored, persistent tool state (the session is display-state and dies on
/// any foreign mutation — knobs living in the engine would forget themselves
/// on every undo): the session **reconciles** the engine against this every
/// batch and tick, the same law as the paper. Defaults are the SPEC's boot
/// values, so an untouched section is byte-identical to W2.
/// `f64` on purpose — the engine's own precision, so [`WetKnobs::DEFAULT`]
/// can equal the engine's boot values EXACTLY (an `f32` copy of `0.4` is
/// `0.4000000059604645` in `f64`, and the reconcile would push that noise
/// into untouched knobs the first time any other knob moves). The panel's
/// `f32` rows convert at the paint boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WetKnobs {
    /// Water per dab (engine `sliders.water`, `0..1`).
    pub water: f64,
    /// Pigment per dab (`Knob::PigmentPerDab`, SPEC §10 gain, `0..2000`).
    pub pigment: f64,
    /// Settled-paint pickup — the dirty brush (`Knob::Pickup`, `0..0.2`).
    pub pickup: f64,
    /// Evaporation multiplier — how fast the water dries (`Knob::Evaporation`, `0..8`).
    pub dry_speed: f64,
    /// Edge darkening — the watercolor rim (`Knob::EdgeDarkening`, `0..200`).
    pub edge_darkening: f64,
    /// Gravity — the run/drip pull, straight down (`Knob::Gravity`, `0..0.05`).
    pub gravity: f64,
    /// The wet eraser's lift per pass (engine `sliders.erase`, `0..1`).
    pub erase: f64,
}

impl WetKnobs {
    /// SPEC §16 boot defaults — MUST match the engine's own boot state
    /// (`Sliders::default` + `KNOB_DEFS`), or a fresh session would get
    /// reconciled away from the very defaults it booted with. A `const` so
    /// the panel's `FALLBACK_BRUSH` (a `const` item) can carry it.
    pub const DEFAULT: Self = Self {
        water: 1.0,
        pigment: 600.0,
        pickup: 0.005,
        dry_speed: 1.0,
        edge_darkening: 50.0,
        gravity: 0.005,
        erase: 0.4,
    };
}

impl Default for WetKnobs {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl PainterTool {
    /// The Wet Paint checkbox's setter (the panel's Enable + the smoke's
    /// arm). Arming while holding the plain Brush enters the mode on the
    /// spot; disarming while wet exits to the plain Brush — and the exit's
    /// teardown ends the session, which IS the bake. From any other tool
    /// the flag just flips: the next `"brush"` honours it.
    pub fn set_wetpaint_armed(&mut self, on: bool) {
        if self.paint.wetpaint.armed == on {
            return;
        }
        self.paint.wetpaint.armed = on;
        match self.paint.paint_mode {
            PaintMode::Paint if on && !self.paint.eraser => self.set_paint_tool_mode("brush"),
            PaintMode::WetPaint if !on => self.set_paint_tool_mode("brush"),
            _ => {}
        }
    }

    /// The checkbox Click ([`ph2d_editor_core::ids::PAINTER_WETPAINT_ENABLE`]).
    pub fn toggle_wetpaint_armed(&mut self) {
        self.set_wetpaint_armed(!self.paint.wetpaint.armed);
    }

    /// The section reset — the Watercolor reset's exact semantics: restore
    /// the section's defaults INCLUDING the enable (disarming bakes the live
    /// water) AND the W3 knobs (the reconcile carries them into any live
    /// session's engine).
    pub fn reset_brush_wetpaint(&mut self) {
        self.paint.wetpaint.knobs = WetKnobs::default();
        self.set_wetpaint_armed(false);
    }

    /// The authored W3 knob values (the panel snapshot's source).
    #[must_use]
    pub fn wet_knobs(&self) -> WetKnobs {
        self.paint.wetpaint.knobs
    }

    /// Route one W3 knob `SetValue` to its clamped field. Ranges are the
    /// engine's own (SPEC §16 / `KNOB_DEFS` slider ranges; sliders `0..1`).
    fn set_wet_knob_value(&mut self, id: ph2d_a11y::NodeId, v: f64) -> bool {
        use ph2d_editor_core::ids as core_ids;
        let k = &mut self.paint.wetpaint.knobs;
        match id {
            x if x == core_ids::PAINTER_WETPAINT_WATER => k.water = v.clamp(0.0, 1.0),
            x if x == core_ids::PAINTER_WETPAINT_PIGMENT => k.pigment = v.clamp(0.0, 2000.0),
            x if x == core_ids::PAINTER_WETPAINT_PICKUP => k.pickup = v.clamp(0.0, 0.2),
            x if x == core_ids::PAINTER_WETPAINT_DRY_SPEED => k.dry_speed = v.clamp(0.0, 8.0),
            x if x == core_ids::PAINTER_WETPAINT_EDGE => k.edge_darkening = v.clamp(0.0, 200.0),
            x if x == core_ids::PAINTER_WETPAINT_GRAVITY => k.gravity = v.clamp(0.0, 0.05),
            x if x == core_ids::PAINTER_WETPAINT_ERASE => k.erase = v.clamp(0.0, 1.0),
            _ => return false,
        }
        true
    }

    /// Route the Wet Paint section controls (Enable + reset) from the layers
    /// panel's generic channel — sibling of `route_brush_watercolor_event`.
    pub(crate) fn route_brush_wetpaint_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WETPAINT_ENABLE => {
                self.toggle_wetpaint_armed();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WETPAINT_RESET => {
                self.reset_brush_wetpaint();
                true
            }
            PanelEvent::SetValue(id, v) => self.set_wet_knob_value(*id, *v),
            _ => false,
        }
    }
}
