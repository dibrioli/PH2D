//! As FAIXAS do Airbrush **Rate** e do watercolor **Flow** (filho de [`super`], cortado pelo tecto de LOC):
//! os `pub const` voltam pelo `pub use` do pai, no mesmo endereço `crate::spec::*`.

/// Airbrush **Rate** (timer period, seconds) soft-range floor — the fastest spray the UI slider
/// reaches (~100 Hz). Blender's `rate` `ui_range` is `0.01..1.0` s (default `0.1`); the hard RNA
/// range is wider, but the reference's property declarations are not in the vendored checkout, so
/// this is the well-known Blender soft range, not an in-tree-verified value. The default `0.1` IS
/// verified in-tree. See [`crate::Stroke::tick`].
pub const AIRBRUSH_RATE_MIN_S: f32 = 0.01;
/// Airbrush **Rate** soft-range ceiling (slowest spray, ~1 Hz). See [`AIRBRUSH_RATE_MIN_S`].
pub const AIRBRUSH_RATE_MAX_S: f32 = 1.0;

/// **Flow Size** range — a multiple of the Classic's scale of detail (`1` = the Classic's; the pattern
/// is normalized to it). `4×` either way spans a wash's edge from one slow meander to a fine fray; past
/// it a feature is either wider than any stroke or finer than the AA's sub-texel taps.
pub const FLOW_SIZE_MIN: f32 = 0.25;
/// See [`FLOW_SIZE_MIN`].
pub const FLOW_SIZE_MAX: f32 = 4.0;
/// The patterns the watercolor **Flow** dropdown offers, in menu order. `None` is **Classic**. The
/// organic procedurals only: a lattice pattern (Bricks, Grid, Dots…) is a drawing, not a way water runs.
pub const FLOW_KINDS: [crate::texture::TextureKind; 9] = {
    use crate::texture::TextureKind as K;
    [
        K::None,
        K::Clouds,
        K::Noise,
        K::Marble,
        K::Wood,
        K::Voronoi,
        K::DistortedNoise,
        K::Musgrave,
        K::Stucci,
    ]
};
