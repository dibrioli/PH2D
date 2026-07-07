//! The `BrushSettings` **snapshot builder** — packs the tool's live brush state (incl. the Grain
//! Colour Ramp + the Shape Value Ramp) into the panel's `Copy` snapshot each frame. Split from
//! `brush_settings.rs` for the LOC cap; the struct + setters stay there.

use super::brush_settings::{BrushSettings, PANEL_RAMP_STOPS, size_px_to_norm};
use crate::tool::PainterTool;
use ph2d_painter_brush::{Falloff, FalloffPoint, MAX_FALLOFF_POINTS, eval_falloff_curve};

impl BrushSettings {
    /// `true` when the active paint operation processes existing pixels instead of painting the brush
    /// colour — a pure **Smear** or **Blur** rail tool. The panel hides the colour / blend / Colour-Ramp
    /// / Randomize-Color / Accumulate / Eraser controls and restricts the Stroke Method to the
    /// incremental methods for either. **Composite Brush** is excluded: it contains a Brush layer that
    /// paints colour, so every control stays visible (the colour ones just drive the Brush layer).
    /// (Defined here, beside the snapshot builder, so `brush_settings.rs` stays under the file-LOC cap.)
    #[must_use]
    pub fn paints_no_color(&self) -> bool {
        (self.is_smear || self.is_blur || self.is_clone || self.is_inpaint)
            && !self.composite_enabled
    }
}

/// Strength of the brush's active falloff at normalized distance `t` (`0` = centre, `1` = rim), for
/// the panel's live curve preview — the editable points for `Custom`, else the [`Falloff`] formula.
/// Lives beside the snapshot it reads (moved from `brush_settings` for the file-LOC cap).
#[must_use]
pub fn brush_falloff_weight_at(s: &BrushSettings, t: f32) -> f32 {
    if s.falloff == Falloff::Custom.to_u8() {
        eval_falloff_curve(&s.falloff_points[..s.falloff_len as usize], t)
    } else {
        Falloff::from_u8(s.falloff).weight(t)
    }
}

impl PainterTool {
    /// Snapshot the active brush for the panel's Brush section.
    #[must_use]
    pub fn brush_settings(&self) -> BrushSettings {
        let b = &self.paint.brush;
        // Snapshot the Custom curve's control points into the Copy array (panel plots + places handles).
        let mut falloff_points = [FalloffPoint::default(); MAX_FALLOFF_POINTS];
        let pts = b.custom_falloff.points();
        falloff_points[..pts.len()].copy_from_slice(pts);
        // Snapshot the Grain Colour Ramp's stops (LINEAR → display sRGB; the 6th slot is the stable id).
        let ramp = self.texture_ramp();
        let mut texture_ramp_stops = [[0.0f32; 6]; PANEL_RAMP_STOPS];
        let ramp_count = ramp.stops().len().min(PANEL_RAMP_STOPS);
        let srgb = |x: f32| f32::from(ph2d_color::srgb::linear_to_srgb_byte(x)) / 255.0;
        for (slot, s) in texture_ramp_stops.iter_mut().zip(ramp.stops()) {
            *slot = [
                s.pos,
                srgb(s.color[0]),
                srgb(s.color[1]),
                srgb(s.color[2]),
                s.color[3],
                f32::from(s.id),
            ];
        }
        // Snapshot the Shape Colour Ramp's stops (LINEAR → display sRGB; the 6th slot is the stable id).
        let sramp = self.shape_color_ramp();
        let mut shape_color_ramp_stops = [[0.0f32; 6]; PANEL_RAMP_STOPS];
        let sramp_count = sramp.stops().len().min(PANEL_RAMP_STOPS);
        for (slot, s) in shape_color_ramp_stops.iter_mut().zip(sramp.stops()) {
            *slot = [
                s.pos,
                srgb(s.color[0]),
                srgb(s.color[1]),
                srgb(s.color[2]),
                s.color[3],
                f32::from(s.id),
            ];
        }
        // Per-layer blend + the brush-only per-layer opacity arrays (the "B" chip + the opacity box).
        let (shape_layer_blend, shape_layer_opacity) = self.paint.shape_layers.panel_blend_arrays();
        BrushSettings {
            size_px: b.radius_px,
            size_norm: size_px_to_norm(b.radius_px),
            strength: b.strength,
            falloff: b.falloff.to_u8(),
            falloff_points,
            falloff_len: b.custom_falloff.len() as u8,
            color: b.color,
            blend: b.blend.to_u8(),
            eraser: self.paint.eraser,
            is_smear: self.is_smear_mode(),
            is_blur: self.is_blur_mode(),
            is_clone: self.is_clone_mode(),
            is_mask: self.is_mask_mode(),
            is_inpaint: self.is_inpaint_mode(),
            is_selection: self.is_selection_mode(),
            selection_mode: self.selection_mode(),
            selection_op: self.selection_bool_op(),
            selection_threshold: self.selection_threshold(),
            selection_stabilizer: self.selection_stabilizer(),
            selection_feather: self.selection_feather(),
            selection_edit: self.selection_edit_mode(),
            selection_overlay_opacity: self.selection_overlay_opacity(),
            is_deform: self.is_deform_mode(),
            deform_mode: self.paint.deform.mode,
            deform_size_norm: self.paint.deform.size_norm,
            // Same squared-track mapping as `warp::deform_radius_px` (kept in sync; the cursor ring reads it).
            deform_size_px: {
                let t = self.paint.deform.size_norm.clamp(0.0, 1.0);
                super::BRUSH_SIZE_MIN_PX
                    + t * t * (super::BRUSH_SIZE_MAX_PX - super::BRUSH_SIZE_MIN_PX)
            },
            deform_pressure: self.paint.deform.pressure,
            deform_distortion: self.paint.deform.distortion,
            deform_momentum: self.paint.deform.momentum,
            deform_strength: self.paint.deform.strength,
            deform_temperament: self.paint.deform.temperament,
            deform_transform_mode: self.paint.deform.transform_mode,
            selection_offset: self.selection_offset(),
            inpaint_patch: self.paint.inpaint_patch_norm,
            inpaint_quality: self.paint.inpaint_quality_norm,
            inpaint_search: self.paint.inpaint_search_norm,
            mask_brush: self.mask_brush(),
            mask_overlay_color: self.mask_overlay_color(),
            clone_has_source: self.clone_has_source(),
            clone_aligned: self.clone_aligned(),
            clone_sample_armed: self.clone_sample_armed(),
            composite_enabled: self.composite_enabled(),
            composite_ops: self.composite_ops_u8(),
            composite_strength: self.composite_strengths(),
            tiling: self.paint.tiling,
            repeat_image: self.paint.repeat_image,
            symmetry_enabled: b.symmetry.enabled,
            symmetry_circular: b.symmetry.circular,
            symmetry_axis: b.symmetry.axis.to_u8(),
            symmetry_segments: b.symmetry.segments(),
            symmetry_pick_line: self.paint.symmetry_pick == Some(super::SymmetryPick::Line),
            symmetry_pick_center: self.paint.symmetry_pick == Some(super::SymmetryPick::Center),
            stroke_method: b.stroke_method.to_u8(),
            stroke_op_mode: self.paint.stroke_op_mode.to_wire(),
            spacing: b.spacing,
            offset: self.paint.shape_offset_norm,
            offset_trim: self.paint.offset_trim,
            // Simplify shows for Free Hand, after a point is added, OR on any CLOSED editable curve — a
            // converted Ellipse/Polygon / merged curve is closed + dense and is exactly what Simplify reduces
            // (Enio 2026-07-05: the button was hidden on a converted curve until you added a point).
            can_simplify: self.paint.curve.as_ref().is_some_and(|ed| {
                ed.editing
                    && (b.stroke_method == ph2d_painter_brush::StrokeMethod::FreeHand
                        || ed.added_point
                        || (ed.model.closed && ed.model.is_curve()))
            }),
            // A curve with points exists whenever the point editor is editing (Curve / Free Hand, or a
            // Ellipse/Polygon converted via Edit) — gates the Save-As-Object button.
            has_drawn_curve: self.paint.curve.as_ref().is_some_and(|ed| ed.editing),
            space_attenuation: b.space_attenuation,
            accumulate: b.accumulate,
            link_shared: self.paint.link_shared_settings,
            line_show_dimensions: self.paint.line_show_dimensions,
            jitter: b.jitter,
            jitter_absolute_px: b.jitter_absolute_px,
            jitter_unit: b.jitter_unit.to_u8(),
            dash_ratio: b.dash_ratio,
            dash_samples: b.dash_samples,
            input_samples: b.input_samples,
            stabilizer: b.stabilizer,
            airbrush_rate_s: b.airbrush_rate_s,
            edge_to_edge: b.edge_to_edge,
            texture_kind: b.texture.kind.to_u8(),
            texture_mapping: b.texture.mapping.to_u8(),
            texture_angle_deg: b.texture.angle_deg,
            texture_rake: b.texture.rake,
            texture_random: b.texture.random_angle,
            texture_offset: b.texture.offset,
            texture_size: b.texture.size,
            stencil_offset: b.texture.stencil_offset,
            stencil_size: b.texture.stencil_size,
            stencil_angle_deg: b.texture.stencil_angle_deg,
            texture_params: b.texture.params,
            grain_depth: b.grain_depth,
            shape_kind: b.shape.kind.to_u8(),
            shape_has_image: self.paint.shape_image.is_some(),
            shape_angle_deg: b.shape.angle_deg,
            shape_rake: b.shape.rake,
            shape_random: b.shape.random_angle,
            shape_offset: b.shape.offset,
            shape_size: b.shape.size,
            shape_params: b.shape.params,
            shape_layer_count: self.paint.shape_layers.len().min(u8::MAX as usize) as u8,
            document_layer_count: self.capturable_layer_count().min(u8::MAX as usize) as u8,
            shape_per_layer_color: self.paint.shape_layers.per_layer_color(),
            shape_layer_color_on: self.paint.shape_layers.snapshot().1,
            shape_layer_color: self.paint.shape_layers.snapshot().2,
            shape_layer_blend,
            shape_layer_opacity,
            dab_flatten: b.dab_flatten,
            dab_angle_deg: b.dab_angle_deg,
            texture_ramp_enabled: self.paint.texture_ramp_enabled,
            texture_ramp_bw: self.paint.texture_ramp_bw,
            texture_ramp_mode: ramp.color_mode.to_u8(),
            texture_ramp_interp: ramp.interp.to_u8(),
            texture_ramp_stops,
            texture_ramp_stop_count: ramp_count as u8,
            texture_ramp_alpha_mode: self.paint.texture_ramp_alpha_mode.to_u8(),
            shape_color_ramp_enabled: self.paint.shape_color_ramp_enabled,
            shape_color_ramp_bw: self.paint.shape_color_ramp_bw,
            shape_color_ramp_mode: sramp.color_mode.to_u8(),
            shape_color_ramp_interp: sramp.interp.to_u8(),
            shape_color_ramp_stops,
            shape_color_ramp_stop_count: sramp_count as u8,
            shape_color_ramp_alpha_mode: self.paint.shape_color_ramp_alpha_mode.to_u8(),
            color_jitter_enabled: b.color_jitter_enabled,
            color_jitter: [b.color_jitter_hue, b.color_jitter_sat, b.color_jitter_val],
            jitter_scale: b.jitter_scale,
            jitter_rotate: b.jitter_rotate,
            jitter_spacing: b.jitter_spacing,
            watercolor: b.watercolor,
            edge_gain: b.edge_gain,
            edge_spread: b.edge_spread,
            granulation: b.granulation,
            pigment: b.pigment,
            pigment_mix: b.pigment_mix,
            fill: b.fill,
            depth: b.depth,
            warp: b.warp,
            wet_smudge: b.wet_smudge,
            wet_rewet: b.wet_rewet,
            paper_kind: b.paper.kind.to_u8(),
            paper_mapping: b.paper.mapping.to_u8(),
            paper_rake: b.paper.rake,
            paper_random: b.paper.random_angle,
            paper_angle: b.paper.angle_deg,
            paper_offset: b.paper.offset,
            paper_size: b.paper.size,
            paper_depth: b.paper_depth,
            paper_params: b.paper.params,
            granulation_use_paper: b.granulation_use_paper,
            paper_color: self.paint.paper_color,
        }
    }

    /// Set the **Offset** slider track (`0..1`, `0.5` = no offset) — perpendicular path offset for shapes.
    /// While a shape is open, the first value of a drag opens a coalesced undo transaction (the whole drag
    /// folds into one step, flushed by the next gesture / bake / undo).
    pub fn set_brush_offset(&mut self, norm: f32) {
        if self.is_editing_shape() {
            self.begin_offset_txn();
        }
        self.paint.shape_offset_norm = norm.clamp(0.0, 1.0);
    }

    /// Toggle **Trim** — cut the offset spine's self-intersections (Offset card checkbox). Live next frame.
    pub fn toggle_offset_trim(&mut self) {
        self.paint.offset_trim = !self.paint.offset_trim;
    }

    /// The EFFECTIVE offset (px) applied to the pristine base geometry: the accumulated offset from prior
    /// Apply & Keep presses PLUS the live slider. The whole pipeline (overlay, fill, materialise bake) reads
    /// this, so the offset is always a single offset of the base — never an offset-of-an-offset.
    pub(crate) fn shape_offset_px(&self) -> f32 {
        self.paint.shape_offset_base_px + self.slider_offset_px()
    }

    /// The Offset SLIDER's own contribution mapped to px: `(norm−0.5)·2·MAX` (`MAX = 100`), `0` at the
    /// centred `0.5` track. The accumulator ([`Self::shape_offset_px`]) adds prior committed offsets on top.
    pub(crate) fn slider_offset_px(&self) -> f32 {
        (self.paint.shape_offset_norm - 0.5) * 2.0 * 100.0
    }

    /// **Apply & Keep**'s offset step: fold the live slider into the accumulator and re-centre the slider —
    /// the EFFECTIVE offset (and so the displayed curve) is unchanged, but the slider is free to push further
    /// in the same direction. Does NOT touch the geometry (the base curve stays pristine).
    pub(crate) fn accumulate_offset(&mut self) {
        self.paint.shape_offset_base_px += self.slider_offset_px();
        self.paint.shape_offset_norm = 0.5;
    }
}
