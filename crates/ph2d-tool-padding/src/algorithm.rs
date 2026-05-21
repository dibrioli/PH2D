//! Padding — pure canvas-resize logic.
//!
//! `std`-only, no editor/ECS coupling: consumes straight-alpha RGBA8 +
//! a signed [`PaddingSpec`] and returns a fresh RGBA8 buffer plus the
//! pixel offset of the original content inside the new canvas. Port of
//! the legacy `addPadding` (Game-Engine-Legada/.../image/padding.ts),
//! which also subsumes `directionalExpand` (single-edge case).

/// Signed per-edge padding, in pixels. Positive = expand that edge with
/// fully-transparent pixels; negative = crop that many pixels off the
/// edge. A directional-expand drag edits exactly one field.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PaddingSpec {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl PaddingSpec {
    /// Uniform padding on all four edges.
    pub fn uniform(all: i32) -> Self {
        Self {
            top: all,
            right: all,
            bottom: all,
            left: all,
        }
    }

    /// True when every edge is zero (caller can skip the bake + undo entry).
    pub fn is_noop(self) -> bool {
        self.top == 0 && self.right == 0 && self.bottom == 0 && self.left == 0
    }
}

/// Output of [`add_padding`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaddingResult {
    /// RGBA8 buffer of the resized canvas. `pixels.len() == width * height * 4`.
    pub pixels: Vec<u8>,
    /// New canvas dimensions (each clamped to ≥ 1).
    pub width: u32,
    pub height: u32,
    /// Pixel shift of the original content's top-left inside the new
    /// canvas (`padLeft - cropLeft`, `padTop - cropTop`). The caller uses
    /// this to reproject the sprite pivot so the world position holds.
    pub pivot_delta_x: i32,
    pub pivot_delta_y: i32,
    /// `false` when the spec was a no-op / produced an identical canvas
    /// (caller skips the asset swap + undo entry).
    pub changed: bool,
}

/// Resize `rgba` (straight-alpha RGBA8, length `w*h*4`) by a signed
/// per-edge `spec`. Positive edges add transparent pixels; negative edges
/// crop. Crops are clamped so the content stays ≥ 1px on each axis.
///
/// SCAFFOLD STUB — the Implementer fills the body (faithful port of the
/// legacy `addPadding`) and the unit tests. Returns the input unchanged
/// for now so the (future) shell bake wiring compiles and is inert.
///
/// # Contract the Implementer fills (from `padding.ts`)
/// 1. Split each signed edge into pad (≥0) vs crop (the negated negative).
/// 2. Clamp crops so `contentW/H = max(1, src − cropNear − cropFar)`.
/// 3. `newW = max(1, contentW + padLeft + padRight)` (same for H).
/// 4. Allocate `newW*newH*4` transparent (all-zero) RGBA8; copy the
///    cropped content rect into it at `(padLeft, padTop)`.
/// 5. `pivot_delta = (padLeft − cropLeft, padTop − cropTop)`.
/// 6. `changed = spec is not a no-op AND dims/content actually moved`.
///
/// Pin with tests: pure expand (transparent border), pure crop, mixed
/// expand+crop on opposite edges, over-crop clamp to 1px, no-op spec.
pub fn add_padding(rgba: &[u8], w: u32, h: u32, spec: PaddingSpec) -> PaddingResult {
    // TODO(impl): port `addPadding`; build the resized buffer + pivot
    // delta; add the unit tests listed above.
    let _ = spec;
    PaddingResult {
        pixels: rgba.to_vec(),
        width: w,
        height: h,
        pivot_delta_x: 0,
        pivot_delta_y: 0,
        changed: false,
    }
}
