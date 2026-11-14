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
    /// píxel parte-se em pontos; pela área dá a média (a linha contínua). Só na Shape `Image` e sem o Flow.
    /// ⚠️ Desde 23/10 (o Stars pequeno) a área é a lei de TODA ponta de imagem, nas duas rotas (por píxel e a
    /// máscara em cache, pela pirâmide); ligada, este campo só obriga a rota POR PÍXEL (a área exacta do píxel da
    /// tela em vez do nível da pirâmide) — `dab_mask_cacheable` e o roteador colorido. Ver [`super::shape::sample_shape`].
    pub area: bool,
    /// ⭐ O CONTRASTE do valor amostrado em torno do meio ([`super::contrasta`]; `1` = o de sempre, ao bit). Não se
    /// grava: é o que o «Same as Paper» copia do papel visível (a Roughness do cartão Paper, dono 28/10).
    pub contraste: f32,
    /// ⭐ A MACIEZ do Height (`0..1`, em altura do papel): a tinta sobe de 0 a 1 ao longo de `softness` acima do ponto
    /// onde o papel encontra a ponta, `(t − (1 − d)) / softness` — o grão de borda macia do pastel a meia pressão
    /// (Rudolf, Mould & Neufeld 2003). `0` = o degrau de sempre, ao bit; `= d` é o Linear Height.
    pub softness: f32,
    /// ⭐ A PONTA afunda menos onde é mais fraca: no Height a profundidade é `d·w` e a cobertura é a do papel, não
    /// `w ×` ela — a borda do traço quebra-se pelo papel (o bastão redondo só toca os picos na orla). Desligado = o
    /// Height de sempre, ao bit.
    pub tip_depth: bool,
    /// ⭐ O ENCHIMENTO do dente (`0..1`): no Height o papel sobe a fracção `fill·pó` do que lhe falta onde a camada já tinha
    /// tinta no pen-down ([`super::PoDoPapel`]) — cada repasse cobre mais vales. `0` = o papel de sempre, ao bit.
    pub fill: f32,
    /// ⭐ O papel em DUAS leituras (B33.1: 256 + 241 a 180°, não se repete a 256 px) — só o Wet Paint a liga
    /// (`PainterTool::papel_na_escala`; BUGS_painter #115, dono 07/10). Desligado = a leitura única do main, ao bit.
    /// Não se grava: o meio decide-a a cada leitura.
    pub leitura_dupla: bool,
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
        contraste: 1.0,
        softness: 0.0,
        tip_depth: false,
        fill: 0.0,
        leitura_dupla: false,
    };

    /// O pó pousado conta ([`Self::papel_com_po`] muda o papel): Height com enchimento.
    #[must_use]
    pub fn enche_o_dente(&self) -> bool {
        self.mode == GrainMode::Height && self.fill > 0.0
    }

    /// ⭐ O papel `t` no texel `(px, py)` com o dente cheio do `po` de antes do traço: `t + fill·pó·(1 − t)` — o vale sobe
    /// mais que o pico, e o grão fica macio (somar `fill·pó` por igual chapava os grãos já pintados, medido 10/11); sem pó
    /// ou sem enchimento, `t` ao bit.
    #[must_use]
    pub fn papel_com_po(&self, t: f32, po: Option<&super::PoDoPapel>, px: i64, py: i64) -> f32 {
        match po {
            Some(p) if self.enche_o_dente() => t + self.fill * p.em(px, py) * (1.0 - t).max(0.0),
            _ => t,
        }
    }

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
                let ponta = w.clamp(0.0, 1.0);
                if self.tip_depth && ponta <= 0.0 {
                    return 0.0;
                }
                let d = if self.tip_depth { d * ponta } else { d };
                let alcanca = if self.softness > 0.0 {
                    ((t - (1.0 - d)) / self.softness).clamp(0.0, 1.0)
                } else {
                    f32::from(u8::from(t > 1.0 - d))
                };
                // Com a ponta na profundidade, a cobertura É o papel alcançado: o factor sobre `w` é `alcanca / w`.
                let r = if self.tip_depth {
                    alcanca / ponta
                } else {
                    alcanca
                };
                return r * super::granulation_gate(t, granulation);
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

    /// ⭐ A maciez do Height: `0` é o degrau de sempre ao bit; `softness = d` é o Linear Height; entre os dois a
    /// tinta sobe de 0 a 1 ao longo de `softness` acima do limiar (o grão de borda macia do pastel).
    #[test]
    fn a_maciez_do_height_vai_do_degrau_ao_linear_height() {
        let h = |softness| GrainLaw {
            mode: GrainMode::Height,
            softness,
            ..GrainLaw::NEUTRA
        };
        let lh = GrainLaw {
            mode: GrainMode::LinearHeight,
            ..GrainLaw::NEUTRA
        };
        for i in 0..=40 {
            for j in 1..=10 {
                let (t, d) = (i as f32 / 40.0, j as f32 / 10.0);
                let degrau = f32::from(u8::from(t > 1.0 - d));
                assert_eq!(
                    h(0.0).factor(0.7, t, d, 0.0).to_bits(),
                    degrau.to_bits(),
                    "t {t} d {d}"
                );
                assert!(
                    (h(d).factor(0.7, t, d, 0.0) - lh.factor(0.7, t, d, 0.0)).abs() < 1e-6,
                    "t {t} d {d}"
                );
                let m = h(0.1).factor(0.7, t, d, 0.0);
                let esperado = ((t - (1.0 - d)) / 0.1).clamp(0.0, 1.0);
                assert!((m - esperado).abs() < 1e-6, "t {t} d {d}: {m} × {esperado}");
            }
        }
        // a meio da faixa, a tinta é parcial (o grão de borda macia), nunca 0 ou 1
        assert!((h(0.1).factor(1.0, 0.55, 0.5, 0.0) - 0.5).abs() < 1e-5);
    }

    /// ⭐ A ponta na profundidade: com a ponta cheia é o Height de sempre; na orla (`w` < 1) o bastão afunda `d·w` e a
    /// cobertura `w × factor` é a do PAPEL alcançado (0 ou 1 no degrau), nunca acima de 1 — a borda quebra-se pelo papel.
    #[test]
    fn a_ponta_na_profundidade_quebra_a_borda_pelo_papel() {
        let base = GrainLaw {
            mode: GrainMode::Height,
            ..GrainLaw::NEUTRA
        };
        let ponta = GrainLaw {
            tip_depth: true,
            ..base
        };
        let (mut orla_base, mut orla_ponta) = (0, 0);
        for i in 0..=99 {
            let t = i as f32 / 99.0;
            assert_eq!(
                ponta.factor(1.0, t, 0.8, 0.0).to_bits(),
                base.factor(1.0, t, 0.8, 0.0).to_bits()
            );
            assert_eq!(ponta.factor(0.0, t, 0.8, 0.0), 0.0);
            for w in [0.1_f32, 0.4, 0.7] {
                let r = w * ponta.factor(w, t, 0.8, 0.0);
                assert!(r == 0.0 || (r - 1.0).abs() < 1e-6, "w {w} t {t}: {r}");
                orla_ponta += usize::from(r > 0.0);
                orla_base += usize::from(w * base.factor(w, t, 0.8, 0.0) > 0.0);
            }
        }
        // a orla com a ponta na profundidade toca MENOS píxeis (só os picos), mas os que toca pintam cheio
        assert!(orla_ponta < orla_base, "{orla_ponta} × {orla_base}");
        // e a orla mais fraca toca menos que a mais forte
        let toca = |w: f32| {
            (0..=99)
                .filter(|i| ponta.factor(w, *i as f32 / 99.0, 0.8, 0.0) > 0.0)
                .count()
        };
        assert!(
            toca(0.1) < toca(0.4) && toca(0.4) < toca(0.7),
            "{} {} {}",
            toca(0.1),
            toca(0.4),
            toca(0.7)
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

    /// ⭐ O enchimento do dente: sem enchimento, sem pó ou fora do Height o papel é `t` AO BIT; com ele, o vale sobe a
    /// fracção `fill·pó` do que lhe falta (o pico já no topo não muda).
    #[test]
    fn o_enchimento_sobe_o_vale_e_e_neutro_sem_po() {
        let plano = [255u8, 128, 0, 255];
        let po = super::super::PoDoPapel {
            plano: &plano,
            largura: 2,
        };
        let h = GrainLaw {
            mode: GrainMode::Height,
            fill: 0.5,
            ..GrainLaw::NEUTRA
        };
        for t in [0.0f32, 0.123, 0.5, 0.97] {
            assert_eq!(
                GrainLaw { fill: 0.0, ..h }
                    .papel_com_po(t, Some(&po), 0, 0)
                    .to_bits(),
                t.to_bits()
            );
            assert_eq!(h.papel_com_po(t, None, 0, 0).to_bits(), t.to_bits());
            assert_eq!(
                h.papel_com_po(t, Some(&po), 0, 1).to_bits(),
                t.to_bits(),
                "pó zero"
            );
            let m = GrainLaw {
                mode: GrainMode::Multiply,
                ..h
            };
            assert_eq!(m.papel_com_po(t, Some(&po), 0, 0).to_bits(), t.to_bits());
            assert!((h.papel_com_po(t, Some(&po), 0, 0) - (t + 0.5 * (1.0 - t))).abs() < 1e-6);
        }
        assert_eq!(h.papel_com_po(1.0, Some(&po), 0, 0), 1.0);
        assert!(h.papel_com_po(0.2, Some(&po), 1, 0) < h.papel_com_po(0.2, Some(&po), 0, 0));
    }
}
