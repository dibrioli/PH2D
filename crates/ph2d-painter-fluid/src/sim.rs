//! [`FluidSim`] descriptor (ADR-0049 §2.4, cap ≤ 12) + [`GravitySource`]
//! (§2.6, cap ≤ 6).
//!
//! `FluidSim` is the CPU-side *descriptor* of a live fluid field — id, size,
//! tuning, lifecycle flags. The GPU resources (storage textures, bind groups,
//! pipeline) live in the (uncapped) solver, so `FluidSim` stays a small,
//! serializable value the tool can hold without a GPU dependency.

use crate::params::{FLUID_PARAMS_VERSION, FluidParams};
use serde::{Deserialize, Serialize};

/// Where the (future) gravity vector comes from (ADR-0049 §2.6, cap ≤ 6).
///
/// **Dormant in v1.** The shipped solver is gated diffusion-advection, which has
/// no gravity term (it advects along paper-slope + water-gradient). The enum is
/// kept — capped, forward-compatible — for a later Shallow-Water / "tilt the
/// device and the ink runs" extension (ADR-0049 §2.2 gyroscope). v1 treats every
/// variant as "no gravity".
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum GravitySource {
    /// No directional bias — the wash spreads isotropically (the v1 behavior).
    #[default]
    None,
    /// A fixed vector configured in Painter Preferences (normalized dir + magnitude).
    Fixed { dir_x: f32, dir_y: f32, magnitude: f32 },
    /// Read per-frame from `PlatformHost::gyroscope()`; `None` from the host ⇒
    /// treated as [`GravitySource::None`].
    DeviceGyroscope,
    // === 3 slots of headroom (cap ≤ 6) ===
}

/// Descriptor of a live wet-on-wet fluid field (ADR-0049 §2.4, cap ≤ 12).
/// 7 fields, 5 headroom.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FluidSim {
    /// The canvas this field belongs to (`ph2d_painter_stroke::CanvasId.0`; kept
    /// as a plain `u64` so the crate needn't depend on the stroke crate).
    pub canvas_id: u64,
    /// Grid resolution = canvas / `WET_FIELD_SCALE`. Held so the solver can size
    /// its textures + the bilinear upsample matches the tool's composite.
    pub fluid_size: (u32, u32),
    /// Gated diffusion-advection tuning (mirrors the CPU reference).
    pub params: FluidParams,
    /// Dormant in v1 (see [`GravitySource`]).
    pub gravity_source: GravitySource,
    /// Diffusion sub-steps per frame (≥ 1; > 1 trades cost for a faster bloom).
    pub steps_per_frame: u32,
    /// Drained per frame by the driver — `false` once the field dries out.
    pub active: bool,
    /// HR-14 schema version (v1 = 1).
    pub version: u32,
    // === 5 slots of headroom (cap ≤ 12) ===
}

impl FluidSim {
    /// A fresh field descriptor for `canvas_id` at grid resolution `fluid_size`,
    /// with default tuning, gravity off, one step/frame, active.
    #[must_use]
    pub fn new(canvas_id: u64, fluid_size: (u32, u32)) -> Self {
        Self {
            canvas_id,
            fluid_size,
            params: FluidParams::default(),
            gravity_source: GravitySource::None,
            steps_per_frame: 1,
            active: true,
            version: FLUID_PARAMS_VERSION,
        }
    }
}
