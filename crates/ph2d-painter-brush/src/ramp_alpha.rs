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

/// **Quem INDEXA a rampa de cor** — sempre a dona dela (prompt 36): a rampa do **Grain** pelo valor do
/// grão; a do **Shape** pelas suas duas camadas ([`ShapeRampLayers`], prompt 38).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RampIndex {
    /// O valor do Grain (a rampa do Grain); sem Grain cai na silhueta.
    #[default]
    Grain,
    /// As camadas da rampa do Shape; o Grain, se houver, morde a cobertura como sem rampa.
    Shape,
}

/// **As duas camadas da cor do Shape** (prompt 38, dono 15/10: *«em shape parece atuar sobre o falloff e
/// não sobre a textura… o modo como está pode ser preservado como falloff color… ambos na mesma seção,
/// organizar qual é o primeiro, com Blend entre eles»*):
/// - **Texture Color** = `rampa[valor da textura do Shape]` (o padrão procedural, ou a luminância da
///   imagem). Como no Grain, um padrão procedural deixa de RECORTAR: vira cor, e a cobertura é a queda.
///   Uma imagem continua a ser a ponta (a forma dela fica). Sem Shape activo o valor é a silhueta.
/// - **Falloff Color** = `rampa_da_queda[cobertura do carimbo]` — a lei de antes do prompt 38; sozinha
///   repete-a ao byte (o padrão recorta e a cor segue padrão × queda).
///
/// Com as duas, a de cima (`falloff_first` ⇒ a Falloff) mistura-se sobre a de baixo como uma camada:
/// [`crate::blend::blend_over`] com o modo `blend` e o alfa `alfa_de_cima × mix`.
#[derive(Clone, Copy, Debug)]
pub struct ShapeRampLayers<'a> {
    /// Texture Color ligada (a LUT dela é a [`RampPaint::lut`]).
    pub texture: bool,
    /// Falloff Color ligada: a LUT dela.
    pub falloff: Option<&'a [[f32; 4]]>,
    /// A Falloff Color é a camada de CIMA.
    pub falloff_first: bool,
    pub blend: crate::blend::BrushBlend,
    /// A opacidade da camada de cima, `[0, 1]`.
    pub mix: f32,
}

impl Default for ShapeRampLayers<'_> {
    fn default() -> Self {
        Self {
            texture: true,
            falloff: None,
            falloff_first: false,
            blend: crate::blend::BrushBlend::Mix,
            mix: 1.0,
        }
    }
}

impl ShapeRampLayers<'_> {
    /// A cor de um texel: `v_textura` indexa a Texture Color, `v_cobertura` a Falloff Color.
    #[inline]
    #[must_use]
    pub fn color(&self, lut_textura: &[[f32; 4]], v_textura: f32, v_cobertura: f32) -> [f32; 4] {
        compose_shape_ramp_layers(
            self.texture
                .then(|| crate::dab::ramp_sample(lut_textura, v_textura)),
            self.falloff
                .map(|l| crate::dab::ramp_sample(l, v_cobertura)),
            self.falloff_first,
            self.blend,
            self.mix,
        )
    }
}

/// ⭐ **A porta única da mistura** das duas camadas (os núcleos, a LUT combinada do carimbo em cache e
/// a aguada passam por aqui): uma só ⇒ ela; as duas ⇒ a de cima sobre a de baixo. RGBA reto.
#[must_use]
pub fn compose_shape_ramp_layers(
    textura: Option<[f32; 4]>,
    queda: Option<[f32; 4]>,
    falloff_first: bool,
    blend: crate::blend::BrushBlend,
    mix: f32,
) -> [f32; 4] {
    match (textura, queda) {
        (Some(c), None) | (None, Some(c)) => c,
        (None, None) => [1.0, 1.0, 1.0, 1.0],
        (Some(t), Some(q)) => {
            let (cima, baixo) = if falloff_first { (q, t) } else { (t, q) };
            crate::blend::blend_over(
                blend,
                baixo,
                [cima[0], cima[1], cima[2]],
                cima[3] * mix.clamp(0.0, 1.0),
            )
        }
    }
}

/// Uma rampa de cor como os núcleos do carimbo a leem: a LUT assada (a do Grain, ou a Texture Color do
/// Shape), o que o alfa faz, quem a indexa e — dona Shape — as camadas.
#[derive(Clone, Copy, Debug)]
pub struct RampPaint<'a> {
    pub lut: &'a [[f32; 4]],
    pub alpha_mode: RampAlphaMode,
    pub index: RampIndex,
    pub shape: ShapeRampLayers<'a>,
}

impl RampPaint<'_> {
    /// A cor lê o VALOR da textura do Shape (Texture Color ligada) — então um padrão procedural deixa de
    /// recortar e o núcleo amostra-o à parte ([`crate::dab::shape_value_at`]).
    #[inline]
    #[must_use]
    pub fn reads_shape_value(&self) -> bool {
        self.index == RampIndex::Shape && self.shape.texture
    }
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
/// aqui. `w` = a silhueta (a cobertura), `shape` = o valor da textura do Shape (Texture Color), `grain`
/// = o valor amostrado do Grain (se houver).
#[inline]
pub(crate) fn ramp_texel(
    ramp: Option<&RampPaint>,
    w: f32,
    shape: Option<f32>,
    grain: Option<f32>,
    depth: f32,
    granulation: f32,
) -> RampTexel {
    let mut t = RampTexel {
        color: None,
        g: 1.0,
        stamp_alpha: 1.0,
    };
    let dono_grao = matches!(ramp, Some(r) if r.index == RampIndex::Grain);
    if let Some(s) = grain.filter(|_| !dono_grao) {
        t.g = crate::texture::grain_coverage(s, depth, granulation);
    }
    if let Some(r) = ramp {
        let c = match r.index {
            RampIndex::Grain => crate::dab::ramp_sample(r.lut, grain.unwrap_or(w)),
            RampIndex::Shape => r.shape.color(r.lut, shape.unwrap_or(w), w),
        };
        t.color = Some([c[0], c[1], c[2]]);
        match r.alpha_mode {
            RampAlphaMode::None => {}
            RampAlphaMode::Strength => t.g *= c[3],
            RampAlphaMode::TextureAlpha => t.stamp_alpha = c[3],
        }
    }
    t
}
