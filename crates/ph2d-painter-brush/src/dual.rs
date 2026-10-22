//! ⭐ **O DUAL BRUSH** (L6 / CAP-06, `docs/Painter/Pinceis/03_MOTORES.md` §4): uma 2.ª ponta — forma, tamanho, passo,
//! espalhar, através, contagem e rotação PRÓPRIOS — carimbada ao longo do MESMO caminho num buffer do traço, que
//! RECORTA a cobertura da 1.ª por um modo (o Dual Brush do Photoshop, o dual do Procreate, o `dual_brush_*` do
//! Rebelle, o Masked Brush do Krita).
//!
//! ## A lei
//!
//! A 2.ª ponta pinta a máscara `m` do traço (a união das marcas dela); a 1.ª pinta o traço como uma camada (o plano
//! da L8) e cada texel compõe-se com a cobertura `a` RECORTADA, `r = modo(a, m)`. Os modos são os do grão (L9,
//! [`crate::texture::GrainLaw::factor`] à profundidade cheia — a mesma conta, com a 2.ª ponta no lugar do papel) e
//! dois que o grão não tem: **Subtract** (a 2.ª ESCAVA: `a − m`) e **Overlay** (W3C, `a` como fundo).
//! **Height** e **Linear Height** afundam a 1.ª ponta `a` na 2.ª (a ponta mais densa chega mais fundo — o «Height»
//! do Krita): pinta onde a marca sobe acima de `1 − a`.
//!
//! ⚠️ **`None` no pincel = o carimbo de sempre, ao bit** (o plano não arma por ele).

use crate::marca::DabJitter;
use crate::spec::{BrushSpec, MAX_BRUSH_RADIUS_PX};
use crate::stroke_method::JitterUnit;
use crate::texture::{GrainLaw, GrainMode, TextureSettings};

/// Como a 2.ª ponta recorta a 1.ª.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DualMode {
    /// `a · m` — a 1.ª só onde a 2.ª pinta.
    #[default]
    Multiply,
    /// `min(a, m)`.
    Darken,
    /// `a − m` — a 2.ª ESCAVA a 1.ª.
    Subtract,
    /// W3C Overlay com `a` como fundo — o único que pode ACRESCENTAR (onde `a > ½` e `m > ½`).
    Overlay,
    /// `1 − (1 − a)/m` — a 2.ª morde a orla, o miolo cheio fica cheio.
    ColorBurn,
    /// A 1.ª afunda `a`: pinta `a` onde a marca sobe acima de `1 − a`.
    Height,
    /// Como o Height, proporcional ao quanto afunda: `max(0, a + m − 1)`.
    LinearHeight,
}

impl DualMode {
    /// Quantos modos.
    pub const COUNT: u8 = 7;

    /// O índice estável (preset e painel).
    #[must_use]
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// O inverso de [`Self::to_u8`]; fora da faixa = [`Self::Multiply`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Darken,
            2 => Self::Subtract,
            3 => Self::Overlay,
            4 => Self::ColorBurn,
            5 => Self::Height,
            6 => Self::LinearHeight,
            _ => Self::Multiply,
        }
    }

    /// ⭐ **A cobertura `a` da 1.ª recortada pela máscara `m` da 2.ª** (`0..1` os dois).
    #[must_use]
    #[inline]
    pub fn recorta(self, a: f32, m: f32) -> f32 {
        let (a, m) = (a.clamp(0.0, 1.0), m.clamp(0.0, 1.0));
        if a <= 0.0 {
            return 0.0;
        }
        let grao = |mode, depth: f32| {
            GrainLaw {
                mode,
                ..GrainLaw::NEUTRA
            }
            .factor(a, m, depth, 0.0)
                * a
        };
        match self {
            Self::Multiply => grao(GrainMode::Multiply, 1.0),
            Self::Darken => grao(GrainMode::Darken, 1.0),
            Self::Subtract => (a - m).max(0.0),
            Self::Overlay => {
                if a <= 0.5 {
                    2.0 * a * m
                } else {
                    1.0 - 2.0 * (1.0 - a) * (1.0 - m)
                }
            }
            Self::ColorBurn => grao(GrainMode::ColorBurn, 1.0),
            Self::Height => grao(GrainMode::Height, a),
            Self::LinearHeight => grao(GrainMode::LinearHeight, a),
        }
    }
}

/// A 2.ª ponta (o campo [`BrushSpec::dual`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DualTip {
    /// Como recorta a 1.ª.
    pub modo: DualMode,
    /// A FORMA: o slot Shape dela (qualquer padrão, ou `Image` — a imagem é do tool; `None` = o disco).
    pub forma: TextureSettings,
    /// O diâmetro dela em diâmetros da 1.ª ([`Self::ESCALA_MIN`]..[`Self::ESCALA_MAX`]).
    pub escala: f32,
    /// O passo, em diâmetros DELA (o Spacing).
    pub passo: f32,
    /// Espalhar em toda a direcção, em diâmetros dela (o Scatter, `0..1`).
    pub espalhar: f32,
    /// Espalhar só ATRAVÉS do traço (`0..`[`DabJitter::ACROSS_MAX`] diâmetros; a lei da L4).
    pub atraves: f32,
    /// Quantas marcas por ponto do caminho (o Count, `1..`[`crate::stroke::spray::SPRAY_COUNT_MAX`]).
    pub contagem: u32,
    /// O ângulo fixo da ponta, em graus.
    pub angulo: u16,
    /// A rotação sorteada por marca (`0..1`, o Jitter Rotate).
    pub rotacao: f32,
    /// A dureza do disco (sem forma) — a orla dela.
    pub dureza: f32,
}

impl DualTip {
    /// O menor diâmetro relativo.
    pub const ESCALA_MIN: f32 = 0.02;
    /// O maior diâmetro relativo.
    pub const ESCALA_MAX: f32 = 4.0;
    /// O menor passo.
    pub const PASSO_MIN: f32 = 0.01;
    /// O maior passo (10 diâmetros: as marcas soltas do Photoshop chegam aos 1000 %).
    pub const PASSO_MAX: f32 = 10.0;

    /// A 2.ª ponta de fábrica: um disco a metade do diâmetro, passo de ¼, em Multiply.
    pub const DE_FABRICA: Self = Self {
        modo: DualMode::Multiply,
        forma: TextureSettings {
            kind: crate::TextureKind::None,
            mapping: crate::TextureMapping::ViewPlane,
            angle_deg: 0,
            rake: false,
            flow: false,
            offset: [0.0, 0.0],
            size: [1.0, 1.0],
            stencil_offset: [0.0, 0.0],
            stencil_size: [0.5, 0.5],
            stencil_angle_deg: 0,
            params: [0.5; crate::MAX_TEX_PARAMS],
            lei: GrainLaw::NEUTRA,
        },
        escala: 0.5,
        passo: 0.25,
        espalhar: 0.0,
        atraves: 0.0,
        contagem: 1,
        angulo: 0,
        rotacao: 0.0,
        dureza: 0.5,
    };

    /// A forma passa a `k`, com os parâmetros do padrão no neutro dele (a lei da Shape ao mudar de padrão).
    pub fn muda_a_forma(&mut self, k: crate::TextureKind) {
        if self.forma.kind == k {
            return;
        }
        self.forma.kind = k;
        self.forma.params = [0.5; crate::MAX_TEX_PARAMS];
        for (i, s) in crate::param_specs(k).iter().enumerate() {
            self.forma.params[i] = s.default;
        }
    }

    /// ⭐ **O pincel da 2.ª ponta**: o caminho é o da 1.ª (método, estabilizador, simetria, a dobra da arte), a ponta é
    /// a dela, e a tinta é BRANCA a cobertura cheia em `Mix` (o que se pinta é a máscara `m`, no alfa).
    #[must_use]
    pub fn pincel(&self, p: &BrushSpec) -> BrushSpec {
        let mut s = *p;
        s.radius_px = (p.radius_px * self.escala.clamp(Self::ESCALA_MIN, Self::ESCALA_MAX))
            .clamp(0.5, MAX_BRUSH_RADIUS_PX);
        s.hardness = self.dureza.clamp(0.0, 1.0);
        s.spacing = self.passo.clamp(Self::PASSO_MIN, Self::PASSO_MAX);
        s.jitter = self.espalhar.clamp(0.0, 1.0);
        s.jitter_unit = JitterUnit::Brush;
        s.jitter_scale = 0.0;
        s.jitter_rotate = self.rotacao.clamp(0.0, 1.0);
        s.jitter_spacing = 0.0;
        s.spray_count = self
            .contagem
            .clamp(1, crate::stroke::spray::SPRAY_COUNT_MAX);
        s.dab_jitter = DabJitter {
            across: self.atraves.clamp(0.0, DabJitter::ACROSS_MAX),
            ..DabJitter::NEUTRO
        };
        s.shape = self.forma;
        // O ângulo roda a ponta; com o Flow a pegada já é a do traço e o ângulo roda o PADRÃO nela (riscas ao
        // longo × através).
        if self.forma.flow {
            s.shape.angle_deg = self.angulo % 360;
            s.dab_angle_deg = 0;
        } else {
            s.dab_angle_deg = self.angulo % 360;
        }
        s.dab_flatten = 0.0;
        s.dab_quina = 0.0;
        // A caneta DEITADA é da mão, não da ponta (J3, `Pinceis/exames/grafite.md`): a 2.ª ponta deita-se com a 1.ª —
        // de pé, o Multiply recortava a pegada deitada à largura de pé (o 8B com a orla: através 1,11×, Rebelle 3,45×).
        s.dab_tilt = None;
        // A tinta da máscara: branca, cheia, sem nada que mude a cor nem a cobertura do dab.
        s.strength = 1.0;
        s.flow = 1.0;
        s.accumulate = true;
        s.space_attenuation = false;
        s.continuous_spacing = false;
        s.blend = crate::BrushBlend::Mix;
        s.blend_sobre_vazio = false;
        s.color = [1.0; 3];
        s.color_jitter_enabled = false;
        s.color_jitter_hue = 0.0;
        s.color_jitter_sat = 0.0;
        s.color_jitter_val = 0.0;
        s.cor_mao = crate::CorDaMao::NEUTRA;
        s.pigment = false;
        s.pigment_mix = 0.0;
        s.texture = TextureSettings::default();
        s.taper = crate::taper::Taper::NEUTRAL;
        s.traco = crate::TracoComposto::NEUTRO;
        s.dual = None;
        s.pixel = crate::PixelTip::Off;
        s.hose = crate::ImageHose::NEUTRA;
        s.style_solid = false;
        s.watercolor = false;
        s.smooth_edges = false;
        s.impasto = false;
        s.impasto_smooth_edges = false;
        s
    }
}

impl Default for DualTip {
    fn default() -> Self {
        Self::DE_FABRICA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo modo dá `0..1`; todos menos o Overlay só TIRAM (`r ≤ a`); a máscara cheia em Multiply é a 1.ª.
    #[test]
    fn os_modos_so_tiram_menos_o_overlay() {
        for k in 0..DualMode::COUNT {
            let modo = DualMode::from_u8(k);
            assert_eq!(modo.to_u8(), k);
            for i in 0..=20 {
                for j in 0..=20 {
                    let (a, m) = (i as f32 / 20.0, j as f32 / 20.0);
                    let r = modo.recorta(a, m);
                    assert!((0.0..=1.0).contains(&r), "{modo:?} {a} {m}: {r}");
                    if modo != DualMode::Overlay {
                        assert!(r <= a + 1e-6, "{modo:?} {a} {m}: {r}");
                    }
                }
            }
        }
        assert!((DualMode::Multiply.recorta(0.7, 1.0) - 0.7).abs() < 1e-6);
        assert!((DualMode::Multiply.recorta(0.7, 0.5) - 0.35).abs() < 1e-6);
    }

    /// ⭐ **Cada modo é outra lei**: sobre a grelha `a × m`, cada par de modos difere em muitos pontos.
    #[test]
    fn cada_modo_e_outra_lei() {
        let grelha: Vec<Vec<f32>> = (0..DualMode::COUNT)
            .map(|k| {
                let modo = DualMode::from_u8(k);
                (1..20)
                    .flat_map(|i| {
                        (1..20).map(move |j| modo.recorta(i as f32 / 20.0, j as f32 / 20.0))
                    })
                    .collect()
            })
            .collect();
        for x in 0..grelha.len() {
            for y in x + 1..grelha.len() {
                let difere = grelha[x]
                    .iter()
                    .zip(&grelha[y])
                    .filter(|(p, q)| (*p - *q).abs() > 1e-3)
                    .count();
                assert!(
                    difere > 30,
                    "{:?} × {:?}: {difere}",
                    DualMode::from_u8(x as u8),
                    DualMode::from_u8(y as u8)
                );
            }
        }
    }

    /// O pincel da 2.ª ponta pinta BRANCO a cobertura cheia, com a ponta dela e o caminho da 1.ª.
    #[test]
    fn o_pincel_da_segunda_e_a_ponta_dela_no_caminho_da_primeira() {
        let mut p = BrushSpec {
            radius_px: 40.0,
            stabilizer: 0.4,
            color: [0.2, 0.1, 0.0],
            strength: 0.3,
            ..BrushSpec::default()
        };
        let d = DualTip {
            escala: 0.25,
            contagem: 3,
            ..DualTip::DE_FABRICA
        };
        p.dual = Some(d);
        let s = d.pincel(&p);
        assert_eq!(s.radius_px, 10.0);
        assert_eq!(s.spray_count, 3);
        assert_eq!(s.stabilizer, 0.4);
        assert_eq!(s.color, [1.0; 3]);
        assert_eq!(s.strength, 1.0);
        assert!(s.dual.is_none());
    }

    /// ⭐ **A 2.ª ponta deita-se com a 1.ª** (J3): a lei da caneta deitada é da mão — o pincel dela leva-a.
    #[test]
    fn a_segunda_ponta_deita_se_com_a_primeira() {
        let mut p = BrushSpec::default();
        p.pen_tilt.start_deg = 39.0;
        p.pen_tilt.width = 3.3;
        let d = DualTip::DE_FABRICA;
        p.dual = Some(d);
        assert_eq!(d.pincel(&p).pen_tilt, p.pen_tilt);
    }
}
