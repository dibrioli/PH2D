//! Texture **Color Ramp** editing setters — thin wrappers over the reusable `ph2d_color::ColorRamp`
//! that the panel's gradient editor drives. Split from `brush_settings.rs` for the LOC cap. Every
//! mutation flags the LUT dirty so the next ramped stamp re-bakes (see `stamp_cache::ensure_ramp_lut`).

use crate::tool::PainterTool;
use ph2d_color::{RampColorMode, RampInterp, RampStop};

impl PainterTool {
    /// Set the ramp colour-interpolation space (`RampColorMode` wire discriminant).
    pub fn set_texture_ramp_mode(&mut self, m: u8) {
        self.paint.texture_ramp.color_mode = RampColorMode::from_u8(m);
        self.paint.texture_ramp_dirty = true;
    }

    /// Set the ramp interpolation mode (`RampInterp` wire discriminant).
    pub fn set_texture_ramp_interp(&mut self, i: u8) {
        self.paint.texture_ramp.interp = RampInterp::from_u8(i);
        self.paint.texture_ramp_dirty = true;
    }

    /// Move ramp stop `idx` to normalized position `pos`, **clamped strictly between its neighbours**
    /// so it never crosses them — the drag keeps a stable index (no re-sort), matching gradient-editor
    /// convention. Returns the (unchanged) index.
    pub fn ramp_move_stop(&mut self, idx: usize, pos: f32) -> usize {
        const EPS: f32 = 1e-3;
        let stops = self.paint.texture_ramp.stops();
        if idx >= stops.len() {
            return idx;
        }
        let lo = if idx > 0 { stops[idx - 1].pos } else { 0.0 };
        let hi = if idx + 1 < stops.len() {
            stops[idx + 1].pos
        } else {
            1.0
        };
        let clamped = if hi - lo > 2.0 * EPS {
            pos.clamp(lo + EPS, hi - EPS)
        } else {
            (lo + hi) * 0.5
        };
        let new_i = self.paint.texture_ramp.set_position(idx, clamped);
        self.paint.texture_ramp_dirty = true;
        new_i
    }

    /// Set ramp stop `idx`'s colour from straight-sRGB bytes (the colour picker's space) — converted
    /// to the ramp's LINEAR space. The stop's existing alpha is preserved.
    pub fn ramp_set_stop_color(&mut self, idx: usize, srgb: [u8; 3]) {
        let lin = ph2d_color::srgb::srgb_to_linear_byte;
        let a = self
            .paint
            .texture_ramp
            .stops()
            .get(idx)
            .map_or(1.0, |s| s.color[3]);
        self.paint
            .texture_ramp
            .set_color(idx, [lin(srgb[0]), lin(srgb[1]), lin(srgb[2]), a]);
        self.paint.texture_ramp_dirty = true;
    }

    /// Toggle whether the Color Ramp drives the paint colour.
    pub fn toggle_texture_ramp_enabled(&mut self) {
        self.paint.texture_ramp_enabled = !self.paint.texture_ramp_enabled;
    }

    /// Add a ramp stop at the midpoint of the **largest gap** between existing stops, coloured by the
    /// current ramp there (so it sits on the gradient + spreads out without a position control yet);
    /// returns its index.
    pub fn ramp_add_stop(&mut self) -> usize {
        let stops = self.paint.texture_ramp.stops();
        let pos = match stops.len() {
            0 => 0.5,
            1 => {
                if stops[0].pos < 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
            _ => {
                // Largest gap between consecutive stops (and the edges to 0 / 1).
                let mut best = (0.0f32, -1.0f32); // (midpoint, gap)
                let mut prev = 0.0;
                for s in stops {
                    if s.pos - prev > best.1 {
                        best = ((prev + s.pos) * 0.5, s.pos - prev);
                    }
                    prev = s.pos;
                }
                if 1.0 - prev > best.1 {
                    best.0 = (prev + 1.0) * 0.5;
                }
                best.0
            }
        };
        let color = self.paint.texture_ramp.eval(pos);
        let i = self.paint.texture_ramp.add_stop(RampStop::new(pos, color));
        self.paint.texture_ramp_dirty = true;
        i
    }

    /// Remove ramp stop `idx` (the ramp always keeps at least one stop).
    pub fn ramp_remove_stop(&mut self, idx: usize) {
        self.paint.texture_ramp.remove_stop(idx);
        self.paint.texture_ramp_dirty = true;
    }

    /// Remove the last (highest-position) ramp stop — the panel's "−" with no per-stop selection yet.
    pub fn ramp_remove_last_stop(&mut self) {
        let last = self.paint.texture_ramp.len().saturating_sub(1);
        self.ramp_remove_stop(last);
    }
}
