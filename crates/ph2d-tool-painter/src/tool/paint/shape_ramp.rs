//! **Shape value-ramp** editing setters (B&W tonal remap of the silhouette) — thin wrappers over the
//! reusable `ph2d_color::ValueRamp` that the panel's grayscale gradient editor drives. Plus the
//! **invert** setters for both ramps. Split from `ramp.rs` for the LOC cap. Every Shape-ramp mutation
//! flags the LUT dirty + bumps `shape_ramp_version` so the cached stamp re-bakes
//! (see `stamp_cache::ensure_shape_ramp_lut` + `StampKey`).

use crate::tool::PainterTool;
use ph2d_color::{RampInterp, ValueStop};

impl PainterTool {
    /// Mark the Shape ramp changed: re-bake the LUT + invalidate the cached stamp.
    fn shape_ramp_changed(&mut self) {
        self.paint.shape_ramp_dirty = true;
        self.paint.shape_ramp_version = self.paint.shape_ramp_version.wrapping_add(1);
    }

    /// Toggle whether the Shape value ramp remaps the silhouette tone.
    pub fn toggle_shape_ramp_enabled(&mut self) {
        self.paint.shape_ramp_enabled = !self.paint.shape_ramp_enabled;
        self.shape_ramp_changed();
    }

    /// Set the Shape ramp interpolation mode (`RampInterp` wire discriminant).
    pub fn set_shape_ramp_interp(&mut self, i: u8) {
        self.paint.shape_ramp.interp = RampInterp::from_u8(i);
        self.shape_ramp_changed();
    }

    /// Move the Shape-ramp stop with stable `id` to normalized position `pos` (tracked by id, so it may
    /// be dragged **past its neighbours** and keep its identity).
    pub fn shape_ramp_move_stop(&mut self, id: u8, pos: f32) {
        if let Some(idx) = self.paint.shape_ramp.index_of_id(id) {
            self.paint.shape_ramp.set_position(idx, pos.clamp(0.0, 1.0));
            self.shape_ramp_changed();
        }
    }

    /// Set the grayscale **value** (`0..1`) of the Shape-ramp stop with stable `id`.
    pub fn shape_ramp_set_stop_value(&mut self, id: u8, value: f32) {
        if let Some(idx) = self.paint.shape_ramp.index_of_id(id) {
            self.paint.shape_ramp.set_value(idx, value);
            self.shape_ramp_changed();
        }
    }

    /// Add a Shape-ramp stop at the midpoint of the **largest gap**, valued by the current ramp there;
    /// returns its index.
    pub fn shape_ramp_add_stop(&mut self) -> usize {
        let pos = super::ramp::largest_gap_midpoint(
            self.paint.shape_ramp.stops().iter().map(|s| s.pos),
            self.paint.shape_ramp.len(),
        );
        let value = self.paint.shape_ramp.eval(pos);
        let i = self.paint.shape_ramp.add_stop(ValueStop::new(pos, value));
        self.shape_ramp_changed();
        i
    }

    /// Remove the last (highest-position) Shape-ramp stop — the panel's "−" (≥1 stop always kept).
    pub fn shape_ramp_remove_last_stop(&mut self) {
        let last = self.paint.shape_ramp.len().saturating_sub(1);
        self.paint.shape_ramp.remove_stop(last);
        self.shape_ramp_changed();
    }

    /// **Invert** the Shape ramp left↔right (mirror stop positions). The invert button.
    pub fn shape_ramp_invert(&mut self) {
        self.paint.shape_ramp.invert();
        self.shape_ramp_changed();
    }

    /// Reset the Shape-tone section to defaults: ramp off, the identity (black→white) gradient.
    pub fn reset_shape_ramp(&mut self) {
        self.paint.shape_ramp = ph2d_color::ValueRamp::default();
        self.paint.shape_ramp_enabled = false;
        self.shape_ramp_changed();
    }

    /// **Invert** the Grain Color Ramp left↔right (mirror stop positions). The invert button.
    pub fn ramp_invert(&mut self) {
        self.paint.texture_ramp.invert();
        self.paint.texture_ramp_dirty = true;
    }

    /// The Shape value ramp (read-only) — the snapshot builder reads its stops.
    #[must_use]
    pub fn shape_ramp(&self) -> &ph2d_color::ValueRamp {
        &self.paint.shape_ramp
    }
}
