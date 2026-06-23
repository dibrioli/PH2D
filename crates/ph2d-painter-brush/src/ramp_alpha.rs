//! How a Color Ramp stop's **alpha** affects painting when the ramp drives the brush colour.
//!
//! The ramp's RGB always recolours each texel (via [`crate::stamp_dab_ramped`]); this enum selects
//! what the ramp's **alpha** channel means at that texel — nothing, less brush strength, or the
//! painted pixel's own opacity (so the sprite can be made transparent).

/// The action of a Color-Ramp colour's alpha during a ramped stamp. Stable `u8` wire discriminant for
/// the panel dropdown + round-trip tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RampAlphaMode {
    /// Alpha is ignored — the ramp only recolours; every covered texel paints at the full falloff
    /// coverage. The default ("alpha does nothing").
    #[default]
    None,
    /// Alpha scales the brush **Strength** (per-texel coverage): translucent ramp areas paint weaker
    /// so the layer below shows through, but the destination's own opacity is never reduced.
    Strength,
    /// Alpha drives the painted pixel's **own alpha**: the dab stamps the ramp as an RGBA image, so
    /// translucent ramp areas punch the sprite transparent (parts of the sprite become see-through).
    TextureAlpha,
}

impl RampAlphaMode {
    /// Number of variants (the panel dropdown iterates `0..COUNT`).
    pub const COUNT: u8 = 3;

    /// Stable wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Strength => 1,
            Self::TextureAlpha => 2,
        }
    }

    /// Inverse of [`Self::to_u8`]; unknown values fall back to [`Self::None`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Strength,
            2 => Self::TextureAlpha,
            _ => Self::None,
        }
    }

    /// Short label for the panel dropdown (English; HR-15). The row carries an "Alpha" label, so the
    /// option names are just the action.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "Off",
            Self::Strength => "Reduce Strength",
            Self::TextureAlpha => "Sprite Alpha",
        }
    }
}
