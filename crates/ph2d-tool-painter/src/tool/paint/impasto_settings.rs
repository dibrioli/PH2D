//! **Impasto** section setters + the panel-event route (`docs/Painter/16…` §5, T3.5).
//!
//! Mirrors `watercolor_settings.rs` — WITHOUT touching it. The section has two halves and they belong
//! to different owners: **Impasto** is per-BRUSH (how this brush lays paint down, so it rides in
//! `BrushSpec` and follows the brush around) and **Lighting** is per-CANVAS (one light for the whole
//! document, like the paper colour or the drying time, so it lives in `PaintState`). Getting that
//! split wrong would give every brush its own sun.

use super::PaintMode;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::{BrushSpec, DepthSource, DrawTo};

/// Widest relief a brush may deposit, either way. Signed: `+` lifts, `−` carves. // CLAMP-OK
const DEPTH_MIN: f32 = -1.0;
const DEPTH_MAX: f32 = 1.0;
/// The light's elevation range in whole degrees. Floor above 0: a grazing light makes the flat-surface
/// response — the divisor the whole relative-shading model rests on — go to zero. // CLAMP-OK
const ELEV_MIN: f32 = 5.0;
const ELEV_MAX: f32 = 90.0;
/// Azimuth wraps, so it is a full turn. // CLAMP-OK
const ANGLE_MAX: f32 = 360.0;

impl PainterTool {
    /// Route the Impasto section's controls from the panel's generic channel to the setters below.
    /// Returns `true` when it consumed the event. Called from `handle_panel_event`.
    pub(crate) fn route_brush_impasto_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        match event {
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_ENABLE => {
                self.toggle_brush_impasto();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_RESET => {
                self.reset_brush_impasto();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_SOURCE_UNIFORM => {
                self.set_brush_impasto_source(DepthSource::Uniform.to_u8());
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_SOURCE_GRAIN => {
                self.set_brush_impasto_source(DepthSource::Grain.to_u8());
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_DRAW_BOTH => {
                self.set_brush_impasto_draw_to(DrawTo::ColorAndDepth.to_u8());
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_DRAW_COLOR => {
                self.set_brush_impasto_draw_to(DrawTo::Color.to_u8());
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_DRAW_DEPTH => {
                self.set_brush_impasto_draw_to(DrawTo::Depth.to_u8());
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_SHOW => {
                self.toggle_impasto_show();
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_DEPTH => {
                self.set_brush_impasto_depth(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_SMOOTHING => {
                self.set_brush_impasto_smoothing(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_BODY => {
                self.set_brush_impasto_body(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_LIGHT_ANGLE => {
                self.set_impasto_light_angle(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_LIGHT_ELEV => {
                self.set_impasto_light_elevation(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_SHINE => {
                self.set_impasto_shine(*v as f32);
                true
            }
            _ => false,
        }
    }

    /// Whether the **Impasto** section applies to the current mode — the §1.2 matrix, in one predicate.
    ///
    /// The panel asks this to decide whether to PAINT the card, and a card that is not painted registers
    /// no hit, so the ids are inert. Hidden under: the **Watercolor** wash (a separate implementation,
    /// and thin paint besides — the wash short-circuits before the height pass ever runs), **Inpaint** (a
    /// heal disc that ignores the brush entirely), **Mask** (a grayscale channel has no body), and
    /// **Smear / Blur / Clone** (they move paint that is already down; dragging relief around is `Plow`,
    /// named and deferred). Publishing this ONE predicate — rather than re-deriving the disjunction in
    /// the panel — is what keeps the UI and the engine from disagreeing about when Impasto is live.
    #[must_use]
    pub fn impasto_applies(&self) -> bool {
        matches!(self.paint.paint_mode, PaintMode::Paint)
            && !self.watercolor_render_active()
            && !self.paint.eraser
    }

    /// Toggle the section's master switch.
    pub fn toggle_brush_impasto(&mut self) {
        self.paint.brush.impasto = !self.paint.brush.impasto;
        // Ticking it back ON re-derives the last stroke's body at the current Depth. Ticking it OFF does
        // NOT delete relief that is already painted — the switch governs what the BRUSH deposits, and
        // silently erasing an artist's sculpting because they unticked a checkbox would be indefensible.
        self.refresh_live_relief();
    }

    /// **Depth** — signed thickness one full-coverage dab lays down. Applies to the NEXT stroke and,
    /// live, to the LAST one: the artist lays a stroke and then dials the thickness in while looking at
    /// it, like every other property in this panel (Enio 2026-07-12). See `impasto::refresh_live_relief`.
    pub fn set_brush_impasto_depth(&mut self, v: f32) {
        self.paint.brush.impasto_depth = v.clamp(DEPTH_MIN, DEPTH_MAX);
        self.refresh_live_relief();
    }

    /// **Smoothing** — how far the deposit settles at stroke end. Live on the last stroke too (the raw
    /// envelope is kept unsettled precisely so a new value can be applied to it, not on top of the old).
    pub fn set_brush_impasto_smoothing(&mut self, v: f32) {
        self.paint.brush.impasto_smoothing = v.clamp(0.0, 1.0);
        self.refresh_live_relief();
    }

    /// **Body** — the cross-section dial (`1` = level film with a wall; `0` = the relief obeys the
    /// falloff: the perfectly rounded ridge of Enio's smoke). Applies from the NEXT stroke: the
    /// profile is baked into the deposit per pixel, and the stored envelope no longer carries the
    /// raw silhouette to re-derive it from (unlike Depth, which is a pure rescale).
    pub fn set_brush_impasto_body(&mut self, v: f32) {
        self.paint.brush.impasto_body = v.clamp(0.0, 1.0);
    }

    /// **Depth Source** from its wire discriminant (the segmented group's option).
    pub fn set_brush_impasto_source(&mut self, wire: u8) {
        self.paint.brush.impasto_source = DepthSource::from_u8(wire);
    }

    /// **Draw To** from its wire discriminant (the segmented group's option).
    pub fn set_brush_impasto_draw_to(&mut self, wire: u8) {
        self.paint.brush.impasto_draw_to = DrawTo::from_u8(wire);
    }

    /// **Show Impasto** — light the relief (canvas-level). Invalidates the composite, since the whole
    /// preview changes.
    pub fn toggle_impasto_show(&mut self) {
        self.paint.impasto_show = !self.paint.impasto_show;
        self.invalidate_composite();
    }

    /// **Light Angle** (azimuth, whole degrees; wraps).
    pub fn set_impasto_light_angle(&mut self, v: f32) {
        self.paint.impasto_light_angle_deg = (v.clamp(0.0, ANGLE_MAX) as u16) % 360;
        self.invalidate_composite();
    }

    /// **Elevation** (whole degrees above the canvas plane).
    pub fn set_impasto_light_elevation(&mut self, v: f32) {
        self.paint.impasto_light_elev_deg = v.clamp(ELEV_MIN, ELEV_MAX) as u16;
        self.invalidate_composite();
    }

    /// **Shine** — the specular highlight on the crests.
    pub fn set_impasto_shine(&mut self, v: f32) {
        self.paint.impasto_shine = v.clamp(0.0, 1.0);
        self.invalidate_composite();
    }

    /// Reset the whole section to the defaults — the brush half from [`BrushSpec::default`], the canvas
    /// half from the tool's. Does NOT touch the relief already painted: Reset is for the SETTINGS, and
    /// silently deleting the artist's sculpting because they clicked a settings reset would be a
    /// spectacular way to lose someone's afternoon.
    pub fn reset_brush_impasto(&mut self) {
        let d = BrushSpec::default();
        let b = &mut self.paint.brush;
        b.impasto = d.impasto;
        b.impasto_depth = d.impasto_depth;
        b.impasto_source = d.impasto_source;
        b.impasto_draw_to = d.impasto_draw_to;
        b.impasto_smoothing = d.impasto_smoothing;
        b.impasto_body = d.impasto_body;
        self.paint.impasto_show = true;
        self.paint.impasto_light_angle_deg = 135;
        self.paint.impasto_light_elev_deg = 45;
        self.paint.impasto_shine = 0.3;
        self.invalidate_composite();
    }
}
