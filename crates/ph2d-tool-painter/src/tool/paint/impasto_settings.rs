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
/// A lamp may be pushed to twice full — a rig wants headroom for a key that carries the picture while
/// the fills sit under 1. Above this the relative ratio saturates against its own `clamp(0, 2)` anyway,
/// so a higher ceiling would be a knob with nothing left to give. // CLAMP-OK
const LIGHT_INTENSITY_MAX: f32 = 2.0;
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
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_PUSH => {
                self.set_brush_impasto_push(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_PLOW => {
                self.set_brush_impasto_plow(*v as f32);
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
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_LIGHT_POWER => {
                self.set_impasto_light_intensity(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_SHINE => {
                self.set_impasto_shine(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_ROUGHNESS => {
                self.set_impasto_roughness(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_METALLIC => {
                self.set_impasto_metallic(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_IMPASTO_WAX => {
                self.set_impasto_wax(*v as f32);
                true
            }
            // The lamp SELECTOR — four chips, one router arm each. Spelled out rather than derived from
            // an index: the arch-gate that pairs a widget id with its handler reads THIS, and a loop over
            // ids it cannot see is how a chip that does nothing gets shipped.
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_LIGHT_1 => {
                self.select_impasto_light(0);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_LIGHT_2 => {
                self.select_impasto_light(1);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_LIGHT_3 => {
                self.select_impasto_light(2);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_LIGHT_4 => {
                self.select_impasto_light(3);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_IMPASTO_LIGHT_ON => {
                self.toggle_impasto_light_on();
                true
            }
            // The lamp's colour, read back from the shared OKLCH picker as "r,g,b" (sRGB bytes).
            PanelEvent::SelectOption(id, v) if *id == core_ids::PAINTER_IMPASTO_LIGHT_COLOR => {
                let mut it = v.split(',').filter_map(|p| p.trim().parse::<u8>().ok());
                if let (Some(r), Some(g), Some(b)) = (it.next(), it.next(), it.next()) {
                    self.set_impasto_light_color([
                        f32::from(r) / 255.0,
                        f32::from(g) / 255.0,
                        f32::from(b) / 255.0,
                    ]);
                }
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

    /// Whether the **Plow** control applies: the Smear, and only the Smear.
    ///
    /// The knife is the one place where a mode that deposits NO paint still has something to say about
    /// the relief — it moves what is already there. Blur and Clone do not (they have no displacement to
    /// speak of), Mask has no body, and the wash is a separate implementation. So the Body card is
    /// hidden here and a single row takes its place, which is also the honest UI: in the Smear there is
    /// no Depth to set, because nothing is being laid down.
    #[must_use]
    pub fn impasto_plow_applies(&self) -> bool {
        matches!(self.paint.paint_mode, PaintMode::Smear)
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
    /// falloff: the perfectly rounded ridge). Live on the last stroke too: the stroke stores the paint
    /// it laid, and the profile is a pure function of it.
    pub fn set_brush_impasto_body(&mut self, v: f32) {
        self.paint.brush.impasto_body = v.clamp(0.0, 1.0);
        self.refresh_live_relief();
    }

    /// **Push** — how much of the paint already on the canvas this brush shoves aside (volume
    /// conservation). Live on the last stroke too, and that is not an accident of the implementation but
    /// the reason for it: the displacement is a pure function of `(ground, footprint)`, so it re-derives
    /// like every other knob in the card — and re-deriving never erodes the same ground twice.
    pub fn set_brush_impasto_push(&mut self, v: f32) {
        self.paint.brush.impasto_push = v.clamp(0.0, 1.0);
        self.refresh_live_relief();
    }

    /// **Plow** — how strongly the Smear drags existing relief (the palette knife). Applies from the
    /// next smear: it is a displacement, not a deposit, so there is nothing to re-derive after the fact.
    pub fn set_brush_impasto_plow(&mut self, v: f32) {
        self.paint.brush.impasto_plow = v.clamp(0.0, 1.0);
    }

    /// **Depth Source** from its wire discriminant (the segmented group's option). Live on the last
    /// stroke too — the grain each dab sampled is stored beside the paint, so flipping to Grain carves
    /// the very grooves that stroke would have left, and flipping back fills them.
    pub fn set_brush_impasto_source(&mut self, wire: u8) {
        self.paint.brush.impasto_source = DepthSource::from_u8(wire);
        self.refresh_live_relief();
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

    /// **Light Angle** (azimuth, whole degrees; wraps) — of the SELECTED lamp.
    pub fn set_impasto_light_angle(&mut self, v: f32) {
        self.paint.impasto_rig.current_mut().angle_deg = (v.clamp(0.0, ANGLE_MAX) as u16) % 360;
        self.invalidate_composite();
    }

    /// **Elevation** (whole degrees above the canvas plane) — of the SELECTED lamp.
    pub fn set_impasto_light_elevation(&mut self, v: f32) {
        self.paint.impasto_rig.current_mut().elev_deg = v.clamp(ELEV_MIN, ELEV_MAX) as u16;
        self.invalidate_composite();
    }

    /// **Intensity** of the selected lamp (`0..2`). A fill at 0.3 against a key at 1 is the everyday rig.
    pub fn set_impasto_light_intensity(&mut self, v: f32) {
        self.paint.impasto_rig.current_mut().intensity = v.clamp(0.0, LIGHT_INTENSITY_MAX);
        self.invalidate_composite();
    }

    /// The selected lamp's **colour** (linear RGB). White is neutral; the shading is relative, so a
    /// coloured lamp tints the paint exactly where it TILTS and leaves flat paint alone.
    pub fn set_impasto_light_color(&mut self, rgb: [f32; 3]) {
        self.paint.impasto_rig.current_mut().color = [
            rgb[0].clamp(0.0, 1.0),
            rgb[1].clamp(0.0, 1.0),
            rgb[2].clamp(0.0, 1.0),
        ];
        self.invalidate_composite();
    }

    /// Pick which lamp the Lighting card edits. Changes no pixel — it is editing state — so it does NOT
    /// invalidate the composite.
    pub fn select_impasto_light(&mut self, i: u8) {
        self.paint.impasto_rig.selected = i.min(super::impasto_rig::MAX_LIGHTS as u8 - 1);
    }

    /// Switch the selected lamp on or off. **Light 0 (the key) cannot be switched off** — a canvas with
    /// Show Impasto ticked and no light at all is an unlit canvas wearing a lit canvas's UI, and "Show
    /// Impasto" already IS that switch. Turning the key off would be a second, worse one.
    pub fn toggle_impasto_light_on(&mut self) {
        if self.paint.impasto_rig.selected == 0 {
            return;
        }
        let l = self.paint.impasto_rig.current_mut();
        l.on = !l.on;
        self.invalidate_composite();
    }

    /// **Shine** — how much the highlight is worth. A property of the PAINT (see `material.rs`), so it
    /// is a brush setting and it is baked into the canvas with the stroke.
    pub fn set_impasto_shine(&mut self, v: f32) {
        self.paint.brush.impasto_shine = v.clamp(0.0, 1.0);
        self.rebake_live_material();
    }

    /// **Roughness** — how BROAD the highlight is. `0` = a tight glint (wet varnish); `1` = a wide soft
    /// sheen (dry chalk). The knob that did not exist: the exponent was the constant `SHININESS = 24`.
    pub fn set_impasto_roughness(&mut self, v: f32) {
        self.paint.brush.impasto_roughness = v.clamp(0.0, 1.0);
        self.rebake_live_material();
    }

    /// **Metallic** — whose colour the highlight takes: the LAMP's (`0`, a dielectric: oil, gouache) or
    /// the PAINT's own (`1`, a conductor: gold leaf, iridescent).
    pub fn set_impasto_metallic(&mut self, v: f32) {
        self.paint.brush.impasto_metallic = v.clamp(0.0, 1.0);
        self.rebake_live_material();
    }

    /// **Wax** — the soft terminator of paint the light enters and leaves nearby (wrap lighting, not a
    /// subsurface simulation — see `material::Material::wax`).
    pub fn set_impasto_wax(&mut self, v: f32) {
        self.paint.brush.impasto_wax = v.clamp(0.0, 1.0);
        self.rebake_live_material();
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
        b.impasto_plow = d.impasto_plow;
        // The MATERIAL is the brush's too — Reset puts the paint back to neutral oil.
        b.impasto_shine = d.impasto_shine;
        b.impasto_roughness = d.impasto_roughness;
        b.impasto_metallic = d.impasto_metallic;
        b.impasto_wax = d.impasto_wax;
        self.paint.impasto_show = true;
        self.paint.impasto_rig = Default::default();
        self.invalidate_composite();
    }
}
