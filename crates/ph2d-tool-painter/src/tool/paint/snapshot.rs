//! The `BrushSettings` **snapshot builder** — packs the tool's live brush state (incl. the Grain
//! Colour Ramp + the Shape Value Ramp) into the panel's `Copy` snapshot each frame. Split from
//! `brush_settings.rs` for the LOC cap; the struct + setters stay there.

use super::brush_settings::{BrushSettings, PANEL_RAMP_STOPS, size_px_to_norm};
use crate::tool::PainterTool;
use ph2d_painter_brush::{FalloffPoint, MAX_FALLOFF_POINTS};

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
            tiling: self.paint.tiling,
            repeat_image: self.paint.repeat_image,
            stroke_method: b.stroke_method.to_u8(),
            spacing: b.spacing,
            space_attenuation: b.space_attenuation,
            accumulate: b.accumulate,
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
        }
    }
}
