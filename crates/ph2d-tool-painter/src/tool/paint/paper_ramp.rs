//! **Paper Colors** ramp editing setters — the watercolor Paper slot's colour ramp (twin of the Shape /
//! Grain ramps). Maps the paper tooth (`[0, 1]`) to a colour so the substrate the wash sits on can be
//! tinted (cream / tan / grey papers), instead of the fixed warm-white. The baked LUT is consumed by the
//! render-path base tint ([`super::watercolor_render`]). Every mutation flags the LUT dirty for a re-bake.

use crate::tool::PainterTool;
use ph2d_color::{RampColorMode, RampInterp, RampStop};

impl PainterTool {
    /// Mark the Paper ramp changed → re-bake the LUT on next use.
    fn invalidate_paper_ramp(&mut self) {
        self.paint.paper_ramp_dirty = true;
    }

    /// Toggle whether the Paper Colors ramp tints the substrate.
    pub fn toggle_paper_ramp_enabled(&mut self) {
        self.paint.paper_color_ramp_enabled = !self.paint.paper_color_ramp_enabled;
        self.invalidate_paper_ramp();
    }

    /// Toggle the Paper ramp **B&W** filter (grayscale tint — the tooth shading with no hue).
    pub fn toggle_paper_ramp_bw(&mut self) {
        self.paint.paper_color_ramp_bw = !self.paint.paper_color_ramp_bw;
        self.invalidate_paper_ramp();
    }

    /// Set the Paper ramp colour-interpolation space (`RampColorMode` wire discriminant).
    pub fn set_paper_ramp_mode(&mut self, m: u8) {
        self.paint.paper_color_ramp.color_mode = RampColorMode::from_u8(m);
        self.invalidate_paper_ramp();
    }

    /// Set the Paper ramp interpolation mode (`RampInterp` wire discriminant).
    pub fn set_paper_ramp_interp(&mut self, i: u8) {
        self.paint.paper_color_ramp.interp = RampInterp::from_u8(i);
        self.invalidate_paper_ramp();
    }

    /// Set the Paper ramp alpha action (`RampAlphaMode` wire discriminant). The paper is an opaque base,
    /// so the render-path ignores it — stored for editor parity only.
    pub fn set_paper_ramp_alpha_mode(&mut self, m: u8) {
        self.paint.paper_color_ramp_alpha_mode = ph2d_painter_brush::RampAlphaMode::from_u8(m);
    }

    /// Move the Paper-ramp stop with stable `id` to normalized position `pos` (tracked by id).
    pub fn paper_ramp_move_stop(&mut self, id: u8, pos: f32) {
        if let Some(idx) = self.paint.paper_color_ramp.index_of_id(id) {
            self.paint
                .paper_color_ramp
                .set_position(idx, pos.clamp(0.0, 1.0));
            self.invalidate_paper_ramp();
        }
    }

    /// Set the colour of the Paper-ramp stop with stable `id` from straight-sRGB **RGBA** bytes.
    pub fn paper_ramp_set_stop_color(&mut self, id: u8, rgba: [u8; 4]) {
        let Some(idx) = self.paint.paper_color_ramp.index_of_id(id) else {
            return;
        };
        let lin = ph2d_color::srgb::srgb_to_linear_byte;
        let a = f32::from(rgba[3]) / 255.0;
        self.paint
            .paper_color_ramp
            .set_color(idx, [lin(rgba[0]), lin(rgba[1]), lin(rgba[2]), a]);
        self.invalidate_paper_ramp();
    }

    /// Add a Paper-ramp stop at the midpoint of the largest gap, coloured by the ramp there; returns its index.
    pub fn paper_ramp_add_stop(&mut self) -> usize {
        let pos = super::ramp::largest_gap_midpoint(
            self.paint.paper_color_ramp.stops().iter().map(|s| s.pos),
            self.paint.paper_color_ramp.len(),
        );
        let color = self.paint.paper_color_ramp.eval(pos);
        let i = self
            .paint
            .paper_color_ramp
            .add_stop(RampStop::new(pos, color));
        self.invalidate_paper_ramp();
        i
    }

    /// Remove the last (highest-position) Paper-ramp stop — the panel's "−" (≥1 stop always kept).
    pub fn paper_ramp_remove_last_stop(&mut self) {
        let last = self.paint.paper_color_ramp.len().saturating_sub(1);
        self.paint.paper_color_ramp.remove_stop(last);
        self.invalidate_paper_ramp();
    }

    /// **Invert** the Paper ramp left↔right (mirror stop positions).
    pub fn paper_ramp_invert(&mut self) {
        self.paint.paper_color_ramp.invert();
        self.invalidate_paper_ramp();
    }

    /// Reset the Paper Colors section to defaults: ramp off, default gradient, B&W off, alpha off.
    pub fn reset_paper_ramp(&mut self) {
        self.paint.paper_color_ramp = ph2d_color::ColorRamp::default();
        self.paint.paper_color_ramp_enabled = false;
        self.paint.paper_color_ramp_bw = false;
        self.paint.paper_color_ramp_alpha_mode = ph2d_painter_brush::RampAlphaMode::None;
        self.invalidate_paper_ramp();
    }

    /// The Paper Colors ramp (read-only) — the snapshot builder reads its stops.
    #[must_use]
    pub fn paper_color_ramp(&self) -> &ph2d_color::ColorRamp {
        &self.paint.paper_color_ramp
    }

    /// Whether the Paper ramp B&W filter is on (read-only; the snapshot).
    #[must_use]
    pub fn paper_ramp_bw(&self) -> bool {
        self.paint.paper_color_ramp_bw
    }

    /// Ensure the baked 256-entry **linear** RGBA LUT of the Paper ramp is current (rebuild if dirty).
    /// Returns `None` when the ramp is off (→ the render-path uses the fixed warm-white paper). The B&W
    /// filter desaturates each entry to its luminance (a grayscale tint).
    pub(super) fn ensure_paper_ramp_lut(&mut self) -> Option<&[[f32; 4]]> {
        if !self.paint.paper_color_ramp_enabled {
            return None;
        }
        if self.paint.paper_ramp_dirty || self.paint.paper_ramp_lut.len() != 256 {
            let bw = self.paint.paper_color_ramp_bw;
            let ramp = &self.paint.paper_color_ramp;
            let lut: Vec<[f32; 4]> = (0..256)
                .map(|i| {
                    let mut c = ramp.eval(i as f32 / 255.0); // linear RGBA
                    if bw {
                        let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
                        c = [l, l, l, c[3]];
                    }
                    c
                })
                .collect();
            self.paint.paper_ramp_lut = lut;
            self.paint.paper_ramp_dirty = false;
        }
        Some(&self.paint.paper_ramp_lut)
    }
}
