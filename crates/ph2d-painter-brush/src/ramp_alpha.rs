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
    pub fn name_key(self) -> &'static str {
        match self {
            Self::None => "paint_brush.ramp_alpha_mode.none",
            Self::Strength => "paint_brush.ramp_alpha_mode.strength",
            Self::TextureAlpha => "paint_brush.ramp_alpha_mode.texture_alpha",
        }
    }
}

/// **Quem INDEXA a rampa de cor** — sempre a dona dela (prompt 36, dono 14/10): a rampa do **Grain**
/// pelo valor do grão; a rampa do **Shape** pela silhueta (o desenho do Shape × falloff). Antes a rampa
/// do Shape era indexada pelo GRÃO sempre que houvesse um: a cor seguia o grão (ancorado à tela no
/// Tiled — «o Mapping virou Tiled») e o grão deixava de morder a cobertura.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RampIndex {
    /// O valor do Grain (a rampa do Grain); sem Grain cai na silhueta.
    #[default]
    Grain,
    /// A silhueta `w` (a rampa do Shape); o Grain, se houver, morde a cobertura como sem rampa.
    Silhouette,
}

/// Uma rampa de cor como os núcleos do carimbo a leem: a LUT assada, o que o alfa dela faz e quem a indexa.
#[derive(Clone, Copy, Debug)]
pub struct RampPaint<'a> {
    pub lut: &'a [[f32; 4]],
    pub alpha_mode: RampAlphaMode,
    pub index: RampIndex,
}

/// A tinta de um texel: a cor (`None` = a do pincel), o factor de cobertura `g` (Grain × alfa da rampa
/// em Strength) e o alfa próprio do texel (`TextureAlpha`).
pub(crate) struct RampTexel {
    pub(crate) color: Option<[f32; 3]>,
    pub(crate) g: f32,
    pub(crate) stamp_alpha: f32,
}

/// ⭐ **A porta única** da cor e da mordida do Grain num texel — os núcleos por pixel
/// ([`crate::dab`]) e o carimbo assado ([`crate::stamp_color::render_ramp_color_stamp`]) passam por
/// aqui. `w` = a silhueta, `grain` = o valor amostrado do Grain (se houver).
#[inline]
pub(crate) fn ramp_texel(
    ramp: Option<&RampPaint>,
    w: f32,
    grain: Option<f32>,
    depth: f32,
    granulation: f32,
) -> RampTexel {
    let mut t = RampTexel {
        color: None,
        g: 1.0,
        stamp_alpha: 1.0,
    };
    let indice = match (ramp, grain) {
        (Some(r), Some(s)) if r.index == RampIndex::Grain => Some((r, s)),
        (Some(r), _) => Some((r, w)),
        (None, _) => None,
    };
    if let Some(s) = grain.filter(|_| !matches!(ramp, Some(r) if r.index == RampIndex::Grain)) {
        t.g = crate::texture::grain_coverage(s, depth, granulation);
    }
    if let Some((r, v)) = indice {
        let c = crate::dab::ramp_sample(r.lut, v);
        t.color = Some([c[0], c[1], c[2]]);
        match r.alpha_mode {
            RampAlphaMode::None => {}
            RampAlphaMode::Strength => t.g *= c[3],
            RampAlphaMode::TextureAlpha => t.stamp_alpha = c[3],
        }
    }
    t
}
