//! **Impasto**: the brush's height channel — the paint's own thickness.
//!
//! The engine paints two things per dab, from **one** kernel: colour and, when
//! [`crate::BrushSpec::impasto`] is on, a height `h`. The height is a *second output* of the dab
//! pipeline that already exists — it consumes the same dab list (already mirrored by Symmetry,
//! already replicated by Tiling) and the same [`crate::StampMask`] (silhouette × grain) that the
//! colour consumes. That is what makes Shape / Shape-Tone / Grain / Falloff / Stroke / Jitter /
//! Mirror / Tiling work under impasto **for free** — see `docs/Painter/16_impasto_plano_implementacao.md` §0.
//!
//! `h` is a signed `f32`: positive lifts paint off the canvas, negative carves into it.
//!
//! Not a lighting module — the light pass is the compositor's (`impasto_pass`). This is only the
//! *material*: what the brush deposits.

/// Where a dab's height comes from — which part of the already-composed dab mask sculpts the relief.
///
/// The mask the colour path builds is `silhouette × grain`. All three sources read *that* mask
/// (or a part of it), never a second geometry pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DepthSource {
    /// **Uniform** — a flat plateau: the dab lays down its full [`crate::BrushSpec::impasto_depth`]
    /// wherever it has coverage, ignoring the Grain's texture. The silhouette still shapes the
    /// *outline* of the relief (a Shape tip leaves that tip's profile), but the interior is smooth.
    /// The default: a plain round brush lays a smooth ridge of paint, like a loaded palette knife.
    #[default]
    Uniform,
    /// **Grain** — the Grain sample modulates the height, so the relief carries the texture's
    /// striations. This is the bristle-brush look: the grain's valleys are where the tuft laid down
    /// less paint. The natural source for a real impasto brush (Corel Painter's bristle depth,
    /// ArtRage's loaded-brush grain).
    Grain,
    /// **Shape** — only the silhouette's own profile sculpts the height (the Grain does not bite).
    /// A soft falloff gives a rounded dome; a hard Shape image gives that image's alpha as relief —
    /// the way a stamp or an impasto texture brush deposits a fixed 3-D tip.
    Shape,
}

impl DepthSource {
    /// Number of variants (the panel cycler iterates `0..COUNT`).
    pub const COUNT: u8 = 3;

    /// Stable wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Uniform => 0,
            Self::Grain => 1,
            Self::Shape => 2,
        }
    }

    /// Inverse of [`Self::to_u8`]; unknown values fall back to [`Self::Uniform`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Grain,
            2 => Self::Shape,
            _ => Self::Uniform,
        }
    }

    /// Short label for the panel cycler (English; HR-15).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Uniform => "Uniform",
            Self::Grain => "Grain",
            Self::Shape => "Shape",
        }
    }
}

/// Which channels a dab writes — colour, height, or both.
///
/// Lets one brush be a pure *sculpting* tool (relief with no pigment: the palette knife that
/// spreads clear medium) or a pure *painting* tool over existing relief.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawTo {
    /// **Color + Depth** (default): the dab paints pigment and lays down thickness — the ordinary
    /// loaded brush.
    #[default]
    ColorAndDepth,
    /// **Color** only: pigment with no thickness. Equivalent to impasto off *for this brush*, but
    /// keeps the impasto settings around so the artist can flip back without re-dialling them.
    Color,
    /// **Depth** only: thickness with no pigment — the canvas RGBA is left byte-identical and only
    /// the height field changes. Sculpt clear medium, or carve (negative depth) into paint that is
    /// already down.
    Depth,
}

impl DrawTo {
    /// Number of variants (the panel cycler iterates `0..COUNT`).
    pub const COUNT: u8 = 3;

    /// Stable wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::ColorAndDepth => 0,
            Self::Color => 1,
            Self::Depth => 2,
        }
    }

    /// Inverse of [`Self::to_u8`]; unknown values fall back to [`Self::ColorAndDepth`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Color,
            2 => Self::Depth,
            _ => Self::ColorAndDepth,
        }
    }

    /// Whether a dab with this setting deposits **pigment**. `false` ⇒ the colour path must leave the
    /// canvas RGBA untouched.
    #[must_use]
    pub fn writes_color(self) -> bool {
        matches!(self, Self::ColorAndDepth | Self::Color)
    }

    /// Whether a dab with this setting deposits **height**. `false` ⇒ the height field is untouched.
    #[must_use]
    pub fn writes_depth(self) -> bool {
        matches!(self, Self::ColorAndDepth | Self::Depth)
    }

    /// Short label for the panel cycler (English; HR-15).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::ColorAndDepth => "Color + Depth",
            Self::Color => "Color",
            Self::Depth => "Depth",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_discriminants_round_trip() {
        for v in 0..DepthSource::COUNT {
            assert_eq!(DepthSource::from_u8(v).to_u8(), v);
        }
        for v in 0..DrawTo::COUNT {
            assert_eq!(DrawTo::from_u8(v).to_u8(), v);
        }
        // Unknown wire values fall back to the default, never panic.
        assert_eq!(DepthSource::from_u8(200), DepthSource::default());
        assert_eq!(DrawTo::from_u8(200), DrawTo::default());
    }

    #[test]
    fn default_draw_to_writes_both_channels() {
        let d = DrawTo::default();
        assert!(d.writes_color() && d.writes_depth());
        assert!(DrawTo::Color.writes_color() && !DrawTo::Color.writes_depth());
        assert!(!DrawTo::Depth.writes_color() && DrawTo::Depth.writes_depth());
    }
}
