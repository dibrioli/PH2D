//! **What the artist chose** — the two enums the Body card cycles: where a dab's height comes from
//! ([`DepthSource`]) and which channels it writes ([`DrawTo`]).
//!
//! Sibling of [`crate::height`] (the kernel) for the workspace file-LOC cap, and along a seam that was
//! already there: this file answers *what the brush is set to*, that one answers *what it lays down*.
//! Re-exported from `height`, so callers still see one surface.

/// Where a dab's height comes from — which part of the dab mask the colour path already built
/// (`silhouette × grain`) sculpts the relief.
///
/// **Two sources, not three.** The design doc listed a third, `Shape` ("the silhouette sample
/// alone"), but for every brush an artist actually builds it is a *silent duplicate* of `Uniform`:
/// with no Shape slot the silhouette IS the falloff, and with an Image Shape the image already
/// replaces the falloff, so "silhouette alone" and "grain neutral" are the same number. Shipping it
/// would have been a knob that does nothing — the exact species of bug the 2026-07-12 sweep spent
/// its length exterminating. Cut before it was written.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DepthSource {
    /// **Uniform** (default) — the Grain does *not* bite: the relief is the dab's **body** — the
    /// silhouette pushed through [`body_profile`], a level film with its wall at the paint's edge.
    /// Corel Painter documents its Uniform the same way: *"applies brushstrokes with even depth and
    /// little texture"*. The Grain still textures the *pigment*; it just doesn't carve the *body*.
    #[default]
    Uniform,
    /// **Grain** — the full dab mask (`w × g`): the Grain's striations become bristle marks in the
    /// relief, so the height varies *inside* the dab. This is the real impasto brush (Corel Painter's
    /// bristle depth, ArtRage's loaded brush): the grain's valleys are where the tuft left less paint.
    Grain,
}

impl DepthSource {
    /// Number of variants (the panel cycler iterates `0..COUNT`).
    pub const COUNT: u8 = 2;

    /// Stable wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Uniform => 0,
            Self::Grain => 1,
        }
    }

    /// Inverse of [`Self::to_u8`]; unknown values fall back to [`Self::Uniform`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Grain,
            _ => Self::Uniform,
        }
    }

    /// Short label for the panel cycler (English; HR-15).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Uniform => "Uniform",
            Self::Grain => "Grain",
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
