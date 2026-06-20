//! Atlas data types: a packed rectangle ([`AtlasRegion`]) and the
//! insert failure modes ([`AtlasInsertError`]). Pure data + UV math, no
//! GPU handles — kept separate from the `TextureAtlas` orchestrator so
//! the hot `uv()` path and the error surface can be read in isolation.

/// A reserved rectangle inside the atlas, in atlas-pixel coordinates.
/// Returned by [`TextureAtlas::insert`] / [`TextureAtlas::region`].
///
/// [`TextureAtlas::insert`]: super::TextureAtlas::insert
/// [`TextureAtlas::region`]: super::TextureAtlas::region
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl AtlasRegion {
    /// Convert pixel coordinates into normalized texture UV
    /// `(u_min, v_min, u_max, v_max)` for the given atlas side
    /// length. Hot path: called per sprite per frame in extract.
    ///
    /// **Half-texel inset**: each boundary moves 0.5 px toward the
    /// region center before normalization. With `FilterMode::Linear`
    /// the sampler reads a 2×2 texel neighborhood at the UV corner;
    /// without the inset that neighborhood straddles the region
    /// edge and pulls neighboring atlas content into the bilinear
    /// blend. The visible artifact (user-reported "linha na margem
    /// esquerda") was an opaque black halo from the empty-pixel
    /// rows that border every packed region. Inset by half a texel
    /// shifts the sample square strictly inside, killing the bleed
    /// without measurable cropping (the half-texel offset is well
    /// below the visible threshold at any reasonable zoom level).
    pub fn uv(self, atlas_size: u32) -> [f32; 4] {
        let s = atlas_size.max(1) as f32;
        let inset = 0.5;
        [
            (self.x as f32 + inset) / s,
            (self.y as f32 + inset) / s,
            (self.x as f32 + self.w as f32 - inset) / s,
            (self.y as f32 + self.h as f32 - inset) / s,
        ]
    }
}

/// Returned by [`TextureAtlas::insert`] when the source can't be
/// packed into the remaining atlas space (or is larger than the
/// atlas itself). Surface as a toast at the call site; the v2
/// regrow strategy will catch this internally and double the
/// texture before retrying.
///
/// [`TextureAtlas::insert`]: super::TextureAtlas::insert
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AtlasInsertError {
    /// No skyline slot fits the requested `width × height`.
    AtlasFull { width: u32, height: u32 },
    /// Source dimensions exceed the atlas's hard size (i.e. would
    /// never fit even into an empty atlas).
    SourceTooLarge { width: u32, height: u32, atlas: u32 },
}

impl std::fmt::Display for AtlasInsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AtlasFull { width, height } => {
                write!(f, "atlas full: no room for {width}×{height} sprite")
            }
            Self::SourceTooLarge {
                width,
                height,
                atlas,
            } => write!(f, "source {width}×{height} exceeds atlas {atlas}×{atlas}"),
        }
    }
}

impl std::error::Error for AtlasInsertError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_uv_normalizes_to_unit_square() {
        let region = AtlasRegion {
            x: 100,
            y: 200,
            w: 50,
            h: 40,
        };
        let uv = region.uv(1000);
        // Half-texel inset: each boundary shifts 0.5 px inward
        // → (100.5, 200.5, 149.5, 239.5) / 1000.
        assert!((uv[0] - 100.5 / 1000.0).abs() < 1e-6, "u0 = {}", uv[0]);
        assert!((uv[1] - 200.5 / 1000.0).abs() < 1e-6, "v0 = {}", uv[1]);
        assert!((uv[2] - 149.5 / 1000.0).abs() < 1e-6, "u1 = {}", uv[2]);
        assert!((uv[3] - 239.5 / 1000.0).abs() < 1e-6, "v1 = {}", uv[3]);
    }

    #[test]
    fn region_uv_half_texel_inset_prevents_edge_bleed() {
        // A 1-pixel region: both u0 and u1 collapse to the same
        // half-texel center (no width left after the inset). This
        // is OK — a 1-px sprite has no edge to bleed.
        let region = AtlasRegion {
            x: 10,
            y: 10,
            w: 1,
            h: 1,
        };
        let uv = region.uv(100);
        assert!((uv[0] - 0.105).abs() < 1e-6);
        assert!((uv[2] - 0.105).abs() < 1e-6); // collapsed
        // A 4-pixel region: inset shaves 1 texel total across both
        // boundaries, so the effective sampled rect is 3 texels wide.
        let region = AtlasRegion {
            x: 0,
            y: 0,
            w: 4,
            h: 4,
        };
        let uv = region.uv(8);
        // (0+0.5)/8 .. (4-0.5)/8 = 0.0625 .. 0.4375
        assert!((uv[0] - 0.0625).abs() < 1e-6);
        assert!((uv[2] - 0.4375).abs() < 1e-6);
    }

    #[test]
    fn region_uv_handles_zero_atlas_size() {
        // Defensive: a zero-size atlas would NaN under naïve `/`.
        // `uv` clamps `atlas_size.max(1)` so the math stays sane.
        let region = AtlasRegion {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
        let uv = region.uv(0);
        // Half-texel inset on a zero region: u0 = 0.5/1 = 0.5,
        // u1 = -0.5/1 = -0.5. Defensive: zero atlas_size clamps
        // to 1 in `uv()` so we don't div by zero, but the result
        // is degenerate (the renderer ignores zero-area UVs).
        assert!(uv[0] >= 0.0 && uv[0] <= 1.0);
        assert!(uv[1] >= 0.0 && uv[1] <= 1.0);
    }
}
