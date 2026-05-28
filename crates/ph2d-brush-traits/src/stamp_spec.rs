//! Per-stamp spec.
//!
//! Per [ADR-0067 §2.2](../../../../docs/architecture/decisions/0067-brush-traits-decoupling.md).
//! Cap: exactly 7 fields; expansion requires an ADR-0067 amendment.

use glam::Vec2;

/// One stamp event: where the brush should hit the target, with full
/// input-device modulation.
///
/// Layout intentionally matches the Painter `Stamp` struct (ADR-0044
/// §2.7) at the field-naming level so the `impl BrushEngine for
/// PainterBrush` adapter is a near-trivial passthrough.
///
/// Field cap: **7** (ADR-0067 §2.6). Adding an 8th field requires an
/// `0067-amendment-N.md` ADR.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StampSpec {
    /// Stamp position in target-texture pixel coordinates.
    pub pos: Vec2,

    /// Unit tangent along the stroke at this stamp (drives elongation
    /// / azimuth-aware stamps).
    pub tangent: Vec2,

    /// Normalized pressure in `[0.0, 1.0]`. `1.0` = full pressure.
    pub pressure: f32,

    /// Tilt in radians per axis, range `[-π/2, π/2]`.
    pub tilt: Vec2,

    /// Azimuth (barrel rotation) in radians, range `[0.0, 2π)`.
    pub azimuth: f32,

    /// Per-stamp random offset (target-pixel units) for natural
    /// scatter — pre-computed by the stroke generator so the
    /// `BrushEngine` stays deterministic.
    pub jitter: Vec2,

    /// Multiplier on the brush's base radius. Typically `1.0`;
    /// `> 1.0` for "size scrubber" UX; `< 1.0` for pressure
    /// inverse modulation.
    pub size_scale: f32,
}

impl Default for StampSpec {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            tangent: Vec2::X,
            pressure: 1.0,
            tilt: Vec2::ZERO,
            azimuth: 0.0,
            jitter: Vec2::ZERO,
            size_scale: 1.0,
        }
    }
}
