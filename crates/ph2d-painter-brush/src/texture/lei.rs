//! ⭐ **A LEI do grão** (L9, `docs/Painter/Pinceis/03_MOTORES.md` §4): o MODO com que o grão morde a ponta e a
//! PROFUNDIDADE por dab (mínima, jitter, pela pressão) — o Texture do Photoshop, o Grain do Procreate.
//!
//! - **O modo de hoje é o MULTIPLY** (medido, `o_modo_de_hoje_e_o_multiply`): `w · (1 − d·(1 − t))` — a textura
//!   puxada para o branco pela profundidade `d` e multiplicada na ponta `w`. É o neutro, pela porta de sempre
//!   ([`super::grain_coverage`]) ao bit.
//! - Os outros são função da ponta `w`, do grão `t` e da profundidade `d` (a cobertura `r ≤ w`; o factor que as
//!   rotas multiplicam é `r / w`): **Subtract** `w − d·(1 − t)` · **Darken** `min(w, T)` · **Color Burn**
//!   `1 − (1 − w)/T` (com `T = 1 − d·(1 − t)`) · **Height** — a ponta afunda `d` no papel e pinta onde o papel
//!   sobe acima dela (`t > 1 − d`) · **Linear Height** — a tinta proporcional ao quanto afunda,
//!   `(t − (1 − d)) / d`.
//! - **A profundidade por dab** (`depth_jitter`, `depth_min`, `depth_pressure`, a lei do «Depth Jitter /
//!   Minimum Depth / Control» do Photoshop): `d · max(min, 1 − jitter·(1 − x))`, `x` = a pressão do dab ou um
//!   sorteio. É o que faz o Height ABRIR o grão com a pressão — medido nos lápis do Rebelle (rampa 0→1: a tinta
//!   do miolo tem cv 0,37–0,46 no 3.º–4.º decil e 0,07–0,14 na pressão cheia; num Multiply o cv é constante).

/// O modo do grão.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GrainMode {
    /// O de sempre: `w · (1 − d·(1 − t))`.
    #[default]
    Multiply,
    /// `w − d·(1 − t)`: o vale do papel tira tinta por igual, a orla macia desaparece primeiro.
    Subtract,
    /// `min(w, T)`.
    Darken,
    /// `1 − (1 − w)/T`: o grão morde a orla; o miolo cheio fica cheio.
    ColorBurn,
    /// A ponta afunda `d`: pinta só onde o papel sobe acima dela (`t > 1 − d`).
    Height,
    /// Como o Height, a tinta proporcional ao quanto afunda: `(t − (1 − d)) / d`.
    LinearHeight,
}

impl GrainMode {
    /// Quantos modos (a faixa da linha do painel).
    pub const COUNT: u8 = 6;

    /// O índice estável (preset e painel).
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Multiply => 0,
            Self::Subtract => 1,
            Self::Darken => 2,
            Self::ColorBurn => 3,
            Self::Height => 4,
            Self::LinearHeight => 5,
        }
    }

    /// O modo é função da PONTA `w` (Subtract, Darken, Color Burn): quem pinta com a ponta cheia tem de lha dar
    /// (o Wet, `wetpaint/dab_route.rs`) — com `w = 1` os três eram o Multiply (o controlo mentia, J2).
    #[must_use]
    pub fn le_a_ponta(self) -> bool {
        matches!(self, Self::Subtract | Self::Darken | Self::ColorBurn)
    }

    /// O inverso de [`Self::to_u8`]; fora da faixa = [`Self::Multiply`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Subtract,
            2 => Self::Darken,
            3 => Self::ColorBurn,
            4 => Self::Height,
            5 => Self::LinearHeight,
            _ => Self::Multiply,
        }
    }
}

/// A lei do grão do slot ([`super::TextureSettings::lei`]); a [`Self::NEUTRA`] é a de sempre ao bit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrainLaw {
    /// O modo.
    pub mode: GrainMode,
    /// A profundidade MÍNIMA de um dab, em fracção da Grain Depth (`0..1`).
    pub depth_min: f32,
    /// Quanto a profundidade varia por dab (`0..1`; `0` = a mesma em todos, o neutro).
    pub depth_jitter: f32,
    /// A variação segue a PRESSÃO do dab (o «Control: Pen Pressure»); desligado = sorteada por dab.
    pub depth_pressure: bool,
    /// ⭐ A ponta de IMAGEM lê-se pela ÁREA que o píxel cobre (o mipmap do Krita; J4 `texturas`,
    /// `docs/Painter/Pinceis/exames/texturas.md` «Motor J4»): reduzida a 5 px, a imagem amostrada num ponto por
    /// píxel parte-se em pontos; pela área dá a média (a linha contínua). Só na Shape `Image` e sem o Flow;
    /// desligado = a amostra no centro do píxel, ao bit. Ver [`super::shape::sample_shape`].
    pub area: bool,
}

impl Default for GrainLaw {
    fn default() -> Self {
        Self::NEUTRA
    }
}

impl GrainLaw {
    /// O grão de sempre.
    pub const NEUTRA: Self = Self {
        mode: GrainMode::Multiply,
        depth_min: 0.0,
        depth_jitter: 0.0,
        depth_pressure: false,
        area: false,
    };

    /// A profundidade varia por dab (o dab leva o multiplicador na pegada, [`crate::DabTilt::grao`]).
    #[must_use]
    pub fn varia_por_dab(&self) -> bool {
        self.depth_jitter > 0.0
    }

    /// O multiplicador da Grain Depth deste dab: `max(min, 1 − jitter·(1 − x))`, `x` = a pressão (com
    /// [`Self::depth_pressure`]) ou o sorteio `u ∈ [0, 1)`.
    #[must_use]
    pub fn profundidade_do_dab(&self, pressure: f32, u: f32) -> f32 {
        let x = if self.depth_pressure { pressure } else { u }.clamp(0.0, 1.0);
        (1.0 - self.depth_jitter.clamp(0.0, 1.0) * (1.0 - x)).max(self.depth_min.clamp(0.0, 1.0))
    }

    /// ⭐ **O factor do grão** sobre a ponta `w` (o que as rotas multiplicam, `r / w`), para o grão `t` à
    /// profundidade `depth` — o Multiply pela porta de sempre ([`super::grain_coverage`]), ao bit.
    #[must_use]
    pub fn factor(&self, w: f32, t: f32, depth: f32, granulation: f32) -> f32 {
        if self.mode == GrainMode::Multiply {
            return super::grain_coverage(t, depth, granulation);
        }
        let (t, d) = (t.clamp(0.0, 1.0), depth.clamp(0.0, 1.0));
        let r = match self.mode {
            GrainMode::Height => {
                return f32::from(u8::from(t > 1.0 - d)) * super::granulation_gate(t, granulation);
            }
            GrainMode::LinearHeight => {
                let f = if d > 0.0 {
                    ((t - (1.0 - d)) / d).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                return f * super::granulation_gate(t, granulation);
            }
            _ if w <= 0.0 => return 0.0,
            GrainMode::Subtract => (w - d * (1.0 - t)).max(0.0),
            GrainMode::Darken => w.min(1.0 - d * (1.0 - t)),
            GrainMode::ColorBurn => {
                let tt = 1.0 - d * (1.0 - t);
                if tt <= 0.0 {
                    0.0
                } else {
                    (1.0 - (1.0 - w) / tt).max(0.0)
                }
            }
            GrainMode::Multiply => unreachable!("o Multiply sai pela porta de sempre"),
        };
        (r / w).clamp(0.0, 1.0) * super::granulation_gate(t, granulation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **O modo de hoje é o Multiply** (medido): a porta de sempre ([`super::super::grain_coverage`]) é
    /// `1 − d·(1 − t)` para toda ponta, à precisão do `f32`, e NÃO é o Subtract (difere onde a ponta é macia).
    #[test]
    fn o_modo_de_hoje_e_o_multiply() {
        let mut difere_do_subtract = 0;
        for i in 0..=20 {
            for j in 0..=20 {
                let (t, d) = (i as f32 / 20.0, j as f32 / 20.0);
                let hoje = super::super::grain_coverage(t, d, 0.0);
                assert!((hoje - (1.0 - d * (1.0 - t))).abs() < 1e-6, "t {t} d {d}");
                for w in [0.25, 0.5, 1.0] {
                    let sub = GrainLaw {
                        mode: GrainMode::Subtract,
                        ..GrainLaw::NEUTRA
                    };
                    if (sub.factor(w, t, d, 0.0) - hoje).abs() > 1e-3 {
                        difere_do_subtract += 1;
                    }
                }
            }
        }
        assert!(difere_do_subtract > 100);
    }

    /// Cada modo dá cobertura `r ≤ w` (o grão só TIRA), e o Height/Linear Height abrem com a profundidade.
    #[test]
    fn os_modos_so_tiram_e_o_height_abre_com_a_profundidade() {
        for m in 0..GrainMode::COUNT {
            let l = GrainLaw {
                mode: GrainMode::from_u8(m),
                ..GrainLaw::NEUTRA
            };
            for t in [0.0, 0.3, 0.7, 1.0] {
                for d in [0.0, 0.4, 1.0] {
                    let f = l.factor(0.6, t, d, 0.0);
                    assert!((0.0..=1.0).contains(&f), "{m} {t} {d}: {f}");
                }
            }
        }
        let h = GrainLaw {
            mode: GrainMode::Height,
            ..GrainLaw::NEUTRA
        };
        let area = |d: f32| {
            (0..100)
                .filter(|i| h.factor(1.0, *i as f32 / 99.0, d, 0.0) > 0.0)
                .count()
        };
        assert!(
            area(0.1) < area(0.5) && area(0.5) < area(0.95),
            "{} {} {}",
            area(0.1),
            area(0.5),
            area(0.95)
        );
    }

    #[test]
    fn a_profundidade_do_dab_segue_a_pressao() {
        let l = GrainLaw {
            depth_jitter: 1.0,
            depth_min: 0.2,
            depth_pressure: true,
            ..GrainLaw::NEUTRA
        };
        assert_eq!(l.profundidade_do_dab(1.0, 0.0), 1.0);
        assert_eq!(l.profundidade_do_dab(0.5, 0.0), 0.5);
        assert_eq!(l.profundidade_do_dab(0.0, 0.9), 0.2);
        assert_eq!(GrainLaw::NEUTRA.profundidade_do_dab(0.1, 0.1), 1.0);
    }
}
