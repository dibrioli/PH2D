//! ⭐⭐ **WHERE A SPATIAL SLIDER'S EXTENT LIVES** — px on the image grid, or a SURFACE's own
//! units (the 3D piece, `docs/3D/30` §14), shown as % of its size. ONE answer for the three
//! slider mappings (thumb, setter, number) and for the seed of a surface's new adjustment.
//!
//! ⚠️ A surface radius is stored in the surface's units (the world: the blur is the same at
//! `8x` and `32x`); the NUMBER reads as % of the surface's size (its bounding diagonal) because
//! a bare world length means nothing to the artist. The slider keeps the grid's proportions: a
//! full travel of a slot whose grid maximum is `px_max` reaches
//! `px_max / SPATIAL_PX_MAX · SURFACE_RADIUS_MAX · size`.

use super::params::SPATIAL_PX_MAX;
use super::*;

/// The radius a full travel of a blur slider reaches on a surface, as a fraction of its size
/// (the bounding diagonal). Its resource is the GPU's memory traffic: the surface low-pass costs
/// `~0,23 ms` per polynomial term over `754 k` samples (RTX 5060 Ti) and the degree grows with
/// the radius over the sample spacing. Measured on the factory piece (`docs/3D/30` §14), one
/// Gaussian step at the full travel:
///
/// | full travel | `8x` | `16x` | `32x` | `64x` |
/// |---|---|---|---|---|
/// | `5 %` of the diagonal | `2,1 ms` | `6,9 ms` | `111,9 ms` ✗ | `732 ms` |
/// | **`2,5 %`** | `1,0 ms` | `3,2 ms` | **`52,7 ms`** | `340 ms` |
///
/// ⇒ `2,5 %` keeps the full travel under the W6 criterion (`100 ms` at `32x`) with `1,9×` margin.
pub const SURFACE_RADIUS_MAX: f32 = 0.025;

/// Where a spatial extent lives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpatialUnits {
    /// The image grid: px (the 2D).
    Pixels,
    /// A surface's own units; `size` is its bounding diagonal in those units.
    Surface { size: f32 },
}

impl SpatialUnits {
    /// The extent the full travel of a slot whose grid maximum is `px_max` maps to.
    #[must_use]
    pub fn extent(self, px_max: f32) -> f32 {
        match self {
            Self::Pixels => px_max,
            Self::Surface { size } => px_max / SPATIAL_PX_MAX * SURFACE_RADIUS_MAX * size,
        }
    }

    /// The number that slot shows: px, or % of the surface's size.
    #[must_use]
    pub fn number(self, px_max: f32) -> SliderNumber {
        let scale = match self {
            Self::Pixels => px_max,
            Self::Surface { .. } => px_max / SPATIAL_PX_MAX * SURFACE_RADIUS_MAX * 100.0,
        };
        SliderNumber::Affine {
            scale,
            offset: 0.0,
            integer: false,
        }
    }
}

/// The slider slots of `params` that are spatial extents a SURFACE re-expresses (the
/// neighbourhood radii; Motion's distance and Halftone's dot read the image plane and never
/// reach a surface). Gate: these are exactly the slots whose thumb changes between units.
#[must_use]
pub fn spatial_extent_slots(params: &AdjustmentParams) -> &'static [usize] {
    match params {
        AdjustmentParams::GaussianBlur(_) => &[0],
        AdjustmentParams::Sharpen(_) => &[1],
        AdjustmentParams::Bloom(_) => &[2],
        AdjustmentParams::ShadowsHighlights(_) => &[2, 5],
        _ => &[],
    }
}

/// ⭐ **Re-express the spatial extents of `params` from `from` to `to`, each slider where it
/// was** — the seed of a surface's new adjustment (the grid defaults, at the same thumb).
pub fn rescale_spatial_params(params: &mut AdjustmentParams, from: SpatialUnits, to: SpatialUnits) {
    let thumbs = adjustment_slider_params_in(params, from);
    for &slot in spatial_extent_slots(params) {
        if let Some(&(_, v)) = thumbs.get(slot) {
            set_adjustment_slider_param_in(params, slot, v, to);
        }
    }
}
