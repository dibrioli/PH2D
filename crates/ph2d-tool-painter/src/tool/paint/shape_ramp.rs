//! **Shape Colour Ramp** editing setters — thin wrappers over the reusable `ph2d_color::ColorRamp`
//! (the twin of the Grain ramp in `ramp.rs`). The Shape ramp OWNS the painted colour when its **B&W**
//! filter is off, or is the grayscale **tone** remap of the silhouette when on (auto-on with a Grain).
//! Split from `ramp.rs` for the LOC cap. Every mutation flags both LUTs dirty + bumps
//! `shape_ramp_version` so the cached stamp re-bakes (the tone path bakes it into the silhouette mask).

use crate::tool::PainterTool;
use ph2d_color::{RampColorMode, RampInterp, RampStop};

impl PainterTool {
    /// Mark the Shape ramp changed: re-bake the LUTs + invalidate the cached stamp. The colour LUT is
    /// flagged too because the Shape ramp may be the colour owner (`bw` off).
    fn shape_ramp_changed(&mut self) {
        self.paint.shape_ramp_dirty = true;
        self.paint.texture_ramp_dirty = true;
        self.paint.shape_ramp_version = self.paint.shape_ramp_version.wrapping_add(1);
    }

    /// Toggle whether the Shape Colour Ramp drives the paint.
    pub fn toggle_shape_ramp_enabled(&mut self) {
        self.paint.shape_color_ramp_enabled = !self.paint.shape_color_ramp_enabled;
        self.shape_ramp_changed();
    }

    /// Toggle the Shape ramp **B&W** filter: off ⇒ the ramp owns the painted colour, on ⇒ the ramp is
    /// the grayscale tone remap of the silhouette (the Grain, if any, owns colour).
    pub fn toggle_shape_ramp_bw(&mut self) {
        self.paint.shape_color_ramp_bw = !self.paint.shape_color_ramp_bw;
        self.shape_ramp_changed();
    }

    /// Set the Shape ramp colour-interpolation space (`RampColorMode` wire discriminant).
    pub fn set_shape_ramp_mode(&mut self, m: u8) {
        self.paint.shape_color_ramp.color_mode = RampColorMode::from_u8(m);
        self.shape_ramp_changed();
    }

    /// Set the Shape ramp interpolation mode (`RampInterp` wire discriminant).
    pub fn set_shape_ramp_interp(&mut self, i: u8) {
        self.paint.shape_color_ramp.interp = RampInterp::from_u8(i);
        self.shape_ramp_changed();
    }

    /// Move the Shape-ramp stop with stable `id` to normalized position `pos` (tracked by id, so it may
    /// be dragged **past its neighbours** and keep its identity).
    pub fn shape_ramp_move_stop(&mut self, id: u8, pos: f32) {
        if let Some(idx) = self.paint.shape_color_ramp.index_of_id(id) {
            self.paint
                .shape_color_ramp
                .set_position(idx, pos.clamp(0.0, 1.0));
            self.shape_ramp_changed();
        }
    }

    /// Set the colour of the Shape-ramp stop with stable `id` from straight-sRGB **RGBA** bytes (the
    /// picker's space). RGB → the ramp's LINEAR space; alpha straight (`a / 255`).
    pub fn shape_ramp_set_stop_color(&mut self, id: u8, rgba: [u8; 4]) {
        let Some(idx) = self.paint.shape_color_ramp.index_of_id(id) else {
            return;
        };
        let lin = ph2d_color::srgb::srgb_to_linear_byte;
        let a = f32::from(rgba[3]) / 255.0;
        self.paint
            .shape_color_ramp
            .set_color(idx, [lin(rgba[0]), lin(rgba[1]), lin(rgba[2]), a]);
        self.shape_ramp_changed();
    }

    /// Add a Shape-ramp stop at the midpoint of the **largest gap**, coloured by the ramp there;
    /// returns its index.
    pub fn shape_ramp_add_stop(&mut self) -> usize {
        let pos = super::ramp::largest_gap_midpoint(
            self.paint.shape_color_ramp.stops().iter().map(|s| s.pos),
            self.paint.shape_color_ramp.len(),
        );
        let color = self.paint.shape_color_ramp.eval(pos);
        let i = self
            .paint
            .shape_color_ramp
            .add_stop(RampStop::new(pos, color));
        self.shape_ramp_changed();
        i
    }

    /// Remove the last (highest-position) Shape-ramp stop — the panel's "−" (≥1 stop always kept).
    pub fn shape_ramp_remove_last_stop(&mut self) {
        let last = self.paint.shape_color_ramp.len().saturating_sub(1);
        self.paint.shape_color_ramp.remove_stop(last);
        self.shape_ramp_changed();
    }

    /// **Invert** the Shape ramp left↔right (mirror stop positions). The invert button.
    pub fn shape_ramp_invert(&mut self) {
        self.paint.shape_color_ramp.invert();
        self.shape_ramp_changed();
    }

    /// Reset the Shape Colour section to defaults: ramp off, the default gradient, B&W off.
    pub fn reset_shape_ramp(&mut self) {
        self.paint.shape_color_ramp = ph2d_color::ColorRamp::default();
        self.paint.shape_color_ramp_enabled = false;
        self.paint.shape_color_ramp_bw = false;
        self.paint.shape_camadas = Default::default(); // Texture Color só, Falloff Color de fábrica
        self.shape_ramp_changed();
    }

    /// Replace the Shape Colour Ramp (`ph2d_color::ColorRamp`); re-bakes the LUTs. Symmetry with
    /// `set_texture_ramp` (panel seed-path + tests).
    pub fn set_shape_color_ramp(&mut self, ramp: ph2d_color::ColorRamp) {
        self.paint.shape_color_ramp = ramp;
        self.shape_ramp_changed();
    }

    /// Enable / disable the Shape Colour Ramp.
    pub fn set_shape_ramp_enabled(&mut self, on: bool) {
        self.paint.shape_color_ramp_enabled = on;
        self.shape_ramp_changed();
    }

    /// The Shape Colour Ramp (read-only) — the snapshot builder reads its stops.
    #[must_use]
    pub fn shape_color_ramp(&self) -> &ph2d_color::ColorRamp {
        &self.paint.shape_color_ramp
    }

    /// Whether the Shape ramp B&W filter is on (read-only; the snapshot + tests).
    #[must_use]
    pub fn shape_ramp_bw(&self) -> bool {
        self.paint.shape_color_ramp_bw
    }

    /// The whole Shape Color section, copied — the live brush preview carries it to its own tool.
    pub(super) fn shape_color_estado(&self) -> EstadoDaShapeColor {
        let p = &self.paint;
        EstadoDaShapeColor {
            ligada: p.shape_color_ramp_enabled,
            bw: p.shape_color_ramp_bw,
            rampa: p.shape_color_ramp.clone(),
            camadas: p.shape_camadas.clone(),
        }
    }

    /// Install a [`Self::shape_color_estado`] copy; re-bakes the LUTs (the copied ones are never trusted).
    pub(super) fn set_shape_color_estado(&mut self, e: &EstadoDaShapeColor) {
        let p = &mut self.paint;
        p.shape_color_ramp_enabled = e.ligada;
        p.shape_color_ramp_bw = e.bw;
        p.shape_color_ramp.clone_from(&e.rampa);
        p.shape_camadas.clone_from(&e.camadas);
        self.shape_ramp_changed();
    }
}

/// The Shape Color section as one value (on, B&W, the Texture Color ramp, the layers).
#[derive(Clone, Debug)]
pub(super) struct EstadoDaShapeColor {
    ligada: bool,
    bw: bool,
    rampa: ph2d_color::ColorRamp,
    camadas: super::shape_camadas::CamadasDoShape,
}
