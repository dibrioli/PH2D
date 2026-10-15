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
/// organizar qual é o primeiro, com Blend entre eles»*; e no smoke: *«o ajuste de opacidade deveria ter
/// para os dois»*):
/// - **Texture Color** = `rampa[valor da textura do Shape]` (o padrão procedural, ou o valor da
///   imagem). Como no Grain, um padrão procedural deixa de RECORTAR: vira cor, e a cobertura é a queda.
///   Uma imagem continua a ser a ponta. **Sem textura no Shape ela não pinta** (não há o que colorir —
///   o chamador desliga-a, como o Grain sem grão).
/// - **Falloff Color** = `rampa_da_queda[cobertura do carimbo]` — a lei de antes do prompt 38; sozinha e
///   opaca repete-a ao byte (o padrão recorta e a cor segue padrão × queda).
///
/// As camadas empilham-se sobre a **cor do pincel**: a de baixo em Normal com a sua opacidade, a de cima
/// (`falloff_first` ⇒ a Falloff) com o modo `blend` e a sua. O ALFA da rampa não é transparência de
/// camada: ele segue o `RampAlphaMode`, empilhado pelas mesmas opacidades.
#[derive(Clone, Copy, Debug)]
pub struct ShapeRampLayers<'a> {
    /// Texture Color ligada E com uma textura a colorir (a LUT dela é a [`RampPaint::lut`]).
    pub texture: bool,
    /// Falloff Color ligada: a LUT dela.
    pub falloff: Option<&'a [[f32; 4]]>,
    /// A Falloff Color é a camada de CIMA.
    pub falloff_first: bool,
    /// O modo da camada de cima sobre a de baixo.
    pub blend: crate::blend::BrushBlend,
    /// As opacidades da Texture Color e da Falloff Color, `[0, 1]`.
    pub texture_opacity: f32,
    pub falloff_opacity: f32,
}

impl Default for ShapeRampLayers<'_> {
    fn default() -> Self {
        Self {
            texture: true,
            falloff: None,
            falloff_first: false,
            blend: crate::blend::BrushBlend::Mix,
            texture_opacity: 1.0,
            falloff_opacity: 1.0,
        }
    }
}

impl ShapeRampLayers<'_> {
    /// Alguma camada pinta.
    #[must_use]
    pub fn any(&self) -> bool {
        self.texture || self.falloff.is_some()
    }

    /// A cor do pincel aparece por baixo das camadas: a de baixo (a única, se só uma pinta) não é opaca.
    /// Então a cor depende do dab, e só as rotas que a têm por dab servem.
    #[must_use]
    pub fn shows_brush_color(&self) -> bool {
        let baixo = match (self.texture, self.falloff.is_some()) {
            (true, true) if self.falloff_first => self.texture_opacity,
            (_, true) => self.falloff_opacity,
            (true, false) => self.texture_opacity,
            (false, false) => return false,
        };
        baixo < 1.0
    }

    /// A cor de um texel: `v_textura` indexa a Texture Color, `v_cobertura` a Falloff Color, `pincel` é
    /// a cor do pincel por baixo.
    #[inline]
    #[must_use]
    pub fn color(
        &self,
        lut_textura: &[[f32; 4]],
        v_textura: f32,
        v_cobertura: f32,
        pincel: [f32; 3],
    ) -> [f32; 4] {
        let t = self.texture.then(|| {
            (
                crate::dab::ramp_sample(lut_textura, v_textura),
                self.texture_opacity,
            )
        });
        let q = self.falloff.map(|l| {
            (
                crate::dab::ramp_sample(l, v_cobertura),
                self.falloff_opacity,
            )
        });
        let (baixo, cima) = if self.falloff_first { (t, q) } else { (q, t) };
        compose_shape_ramp_layers(pincel, baixo, cima, self.blend)
    }
}

/// `a → b` por `t`, EXACTO nas pontas (uma camada opaca devolve a sua cor ao bit).
#[inline]
fn mistura(a: f32, b: f32, t: f32) -> f32 {
    if t >= 1.0 {
        b
    } else if t <= 0.0 {
        a
    } else {
        a + (b - a) * t
    }
}

/// ⭐ **A porta única da pilha** das duas camadas (os núcleos, a LUT combinada do carimbo em cache e a
/// aguada passam por aqui): sobre a cor do `pincel` (alfa 1), a de `baixo` em Normal com a sua
/// opacidade, a de `cima` com o modo `blend` e a sua. Cada camada é `(rgba da rampa, opacidade)`.
#[must_use]
pub fn compose_shape_ramp_layers(
    pincel: [f32; 3],
    baixo: Option<([f32; 4], f32)>,
    cima: Option<([f32; 4], f32)>,
    blend: crate::blend::BrushBlend,
) -> [f32; 4] {
    let mut c = [pincel[0], pincel[1], pincel[2], 1.0];
    if let Some((b, op)) = baixo {
        for (ck, bk) in c.iter_mut().zip(b) {
            *ck = mistura(*ck, bk, op);
        }
    }
    if let Some((t, op)) = cima {
        let op = op.clamp(0.0, 1.0);
        let rgb = if baixo.is_none() {
            [t[0], t[1], t[2]]
        } else {
            let m =
                crate::blend::blend_over(blend, [c[0], c[1], c[2], 1.0], [t[0], t[1], t[2]], 1.0);
            [m[0], m[1], m[2]]
        };
        for (ck, rk) in c.iter_mut().zip(rgb) {
            *ck = mistura(*ck, rk, op);
        }
        c[3] = mistura(c[3], t[3], op);
    }
    c
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
/// aqui. `pincel` = a cor do pincel (por baixo das camadas do Shape), `w` = a silhueta (a cobertura),
/// `shape` = o valor da textura do Shape (Texture Color), `grain` = o valor amostrado do Grain.
#[inline]
pub(crate) fn ramp_texel(
    ramp: Option<&RampPaint>,
    pincel: [f32; 3],
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
            RampIndex::Shape => r.shape.color(r.lut, shape.unwrap_or(w), w, pincel),
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
