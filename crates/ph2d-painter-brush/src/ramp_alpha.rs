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
/// As camadas empilham-se como camadas de imagem sobre o TRANSPARENTE (dono 15/10: *«que o alpha
/// obedeça o alfa de cada cor do color ramp»*): a de baixo em Normal, a de cima (`falloff_first` ⇒ a
/// Falloff) com o modo `blend`; o alfa de cada uma é `alfa da cor × Opacity`. O alfa do resultado é o
/// da TINTA — a cobertura do texel multiplica-se por ele (uma cor transparente pinta menos, como o mapa
/// de gradiente do Krita na ponta).
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

    /// A cor de um texel (RGBA reto; o alfa é o da tinta): `v_textura` indexa a Texture Color,
    /// `v_cobertura` a Falloff Color.
    #[inline]
    #[must_use]
    pub fn color(&self, lut_textura: &[[f32; 4]], v_textura: f32, v_cobertura: f32) -> [f32; 4] {
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
        compose_shape_ramp_layers(baixo, cima, self.blend)
    }
}

/// O alfa de uma camada: `alfa × opacidade`, EXACTO com a opacidade a 1.
#[inline]
fn alfa_da_camada(a: f32, opacidade: f32) -> f32 {
    if opacidade >= 1.0 {
        a
    } else {
        a * opacidade.max(0.0)
    }
}

/// ⭐ **A porta única da pilha** das duas camadas (os núcleos, a LUT combinada do carimbo em cache e a
/// aguada passam por aqui), sobre o transparente: a de `baixo` em Normal, a de `cima` com o modo
/// `blend` (composição W3C: `B(cb, cs)` onde há fundo, a própria cor onde não há). Cada camada é
/// `(rgba da rampa, opacidade)`; devolve RGBA reto, o alfa da tinta. Uma camada opaca devolve a sua
/// cor ao bit.
#[must_use]
pub fn compose_shape_ramp_layers(
    baixo: Option<([f32; 4], f32)>,
    cima: Option<([f32; 4], f32)>,
    blend: crate::blend::BrushBlend,
) -> [f32; 4] {
    let mut c = [0.0f32; 4];
    if let Some((b, op)) = baixo {
        c = [b[0], b[1], b[2], alfa_da_camada(b[3], op)];
    }
    if let Some((t, op)) = cima {
        let a_s = alfa_da_camada(t[3], op);
        let a_b = c[3];
        if a_b <= 0.0 {
            return [t[0], t[1], t[2], a_s];
        }
        let m = crate::blend::blend_over(blend, [c[0], c[1], c[2], 1.0], [t[0], t[1], t[2]], 1.0);
        let a_o = a_s + a_b * (1.0 - a_s);
        if a_o <= 0.0 {
            return [0.0; 4];
        }
        for k in 0..3 {
            c[k] = (a_s * (1.0 - a_b) * t[k] + a_s * a_b * m[k] + (1.0 - a_s) * a_b * c[k]) / a_o;
        }
        c[3] = a_o;
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
    /// O desvio do Randomize Color DESTE carimbo ([`crate::Dab::color_shift`]): a cor que a rampa dá
    /// desloca-se por ele ([`crate::apply_color_shift`]). Zero ⇒ a cor da rampa ao bit.
    pub shift: [f32; 3],
}

/// A LUT de uma rampa com o desvio `shift` do carimbo em cada entrada (o alfa fica) — ponto a ponto a
/// mesma lei de [`ramp_texel`], para quem lê a LUT direto ([`crate::blit_stamp_ramped`]). Reusa `out`.
pub fn shift_ramp_lut(lut: &[[f32; 4]], shift: [f32; 3], out: &mut Vec<[f32; 4]>) {
    out.clear();
    out.extend(lut.iter().map(|c| {
        let s = crate::jitter::apply_color_shift([c[0], c[1], c[2]], shift);
        [s[0], s[1], s[2], c[3]]
    }));
}

impl RampPaint<'_> {
    /// A cor lê o VALOR da textura do Shape (Texture Color ligada) — então um padrão procedural deixa de
    /// recortar e o núcleo amostra-o à parte ([`crate::dab::shape_value_at`]).
    #[inline]
    #[must_use]
    pub fn reads_shape_value(&self) -> bool {
        self.index == RampIndex::Shape && self.shape.texture
    }

    /// ⭐ **A cor da rampa num texel** (RGB reto já deslocado pelo `shift`, alfa = o da tinta) — a lei ÚNICA
    /// de quem a indexa: a dona Grain pelo valor do grão (sem grão, a silhueta `w`); a dona Shape pelas
    /// camadas. O núcleo do carimbo ([`ramp_texel`]), a aguada e o Wet Paint passam por aqui.
    #[inline]
    #[must_use]
    pub fn cor(&self, w: f32, shape: Option<f32>, grain: Option<f32>) -> [f32; 4] {
        let c = match self.index {
            RampIndex::Grain => crate::dab::ramp_sample(self.lut, grain.unwrap_or(w)),
            RampIndex::Shape => self.shape.color(self.lut, shape.unwrap_or(w), w),
        };
        let s = crate::jitter::apply_color_shift([c[0], c[1], c[2]], self.shift);
        [s[0], s[1], s[2], c[3]]
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
/// = o valor amostrado do Grain.
#[inline]
pub(crate) fn ramp_texel(
    ramp: Option<&RampPaint>,
    w: f32,
    shape: Option<f32>,
    grain: Option<f32>,
    depth: f32,
    granulation: f32,
    lei: &crate::texture::GrainLaw,
) -> RampTexel {
    let mut t = RampTexel {
        color: None,
        g: 1.0,
        stamp_alpha: 1.0,
    };
    let dono_grao = matches!(ramp, Some(r) if r.index == RampIndex::Grain);
    if let Some(s) = grain.filter(|_| !dono_grao) {
        t.g = lei.factor(w, s, depth, granulation);
    }
    if let Some(r) = ramp {
        let c = r.cor(w, shape, grain);
        t.color = Some([c[0], c[1], c[2]]);
        match r.alpha_mode {
            RampAlphaMode::None => {}
            RampAlphaMode::Strength => t.g *= c[3],
            RampAlphaMode::TextureAlpha => t.stamp_alpha = c[3],
        }
    }
    t
}
