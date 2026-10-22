//! ⭐ **A ponta de N IMAGENS** (L19, CAP-17 — o «image hose» do Krita, o «sampled brush» com variantes;
//! `docs/Painter/Pinceis/03_MOTORES.md` §4): a Shape leva até [`HOSE_MAX`] imagens e cada dab carimba UMA,
//! escolhida por [`HoseMode`]. As imagens (pesadas) moram no tool; o pincel leva só a contagem, o modo e os pesos,
//! e cada dab leva o índice na pegada ([`crate::DabTilt::imagem`]).
//!
//! A lei medida (06 §10.6, Krita `Stamp Herbals` a 1,5·D): **sorteio com PESO, não ciclo** — 3 imagens, pressão 1:
//! `AABCAAAAABBABAA` (A em 10/15) —, o tamanho pela pressão (a dinâmica de sempre) e a cor da frente (a tinta; a cor
//! da imagem é a «Use Texture Colors» de sempre). Os outros modos são os do Krita/Photoshop: em sequência, pela
//! pressão, pelo ângulo do traço.
//!
//! [`HOSE_MAX`] `= 8` é o recurso do PAINEL (uma linha de peso por imagem) e do `BrushSpec` `Copy` (os pesos vão
//! dentro dele, copiados por dab em `at_dab`); os oráculos medidos usam 3 (Herbals) e 1 por toque (Leaves).

/// Quantas imagens a ponta leva no máximo (ver o módulo).
pub const HOSE_MAX: usize = 8;

/// Como cada dab escolhe a sua imagem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HoseMode {
    /// Sorteio com PESO (o Krita `Random`, a lei medida no Herbals).
    #[default]
    Random,
    /// Em sequência, dab a dab (o Krita `Incremental`), recomeçando em cada traço.
    Sequence,
    /// Pela pressão: a leve a primeira, a cheia a última (o Krita `Pressure`).
    Pressure,
    /// Pelo ângulo do traço: a volta inteira dividida pelas imagens (o Krita `Angular`).
    Angle,
}

impl HoseMode {
    /// Quantos modos.
    pub const COUNT: u8 = 4;

    /// O índice estável (preset e painel).
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Random => 0,
            Self::Sequence => 1,
            Self::Pressure => 2,
            Self::Angle => 3,
        }
    }

    /// O inverso de [`Self::to_u8`]; fora da faixa = [`Self::Random`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Sequence,
            2 => Self::Pressure,
            3 => Self::Angle,
            _ => Self::Random,
        }
    }
}

/// A ponta de N imagens (o campo [`crate::BrushSpec::hose`]); a [`Self::NEUTRA`] é a imagem única de sempre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageHose {
    /// Quantas imagens (`1` = a Shape de sempre); escrito pelo tool, que as guarda.
    pub n: u8,
    /// Como cada dab escolhe.
    pub mode: HoseMode,
    /// O peso de cada imagem no sorteio (`0` = nunca); só o [`HoseMode::Random`] o lê.
    pub pesos: [u8; HOSE_MAX],
}

impl Default for ImageHose {
    fn default() -> Self {
        Self::NEUTRA
    }
}

impl ImageHose {
    /// Uma imagem, pesos iguais.
    pub const NEUTRA: Self = Self {
        n: 1,
        mode: HoseMode::Random,
        pesos: [1; HOSE_MAX],
    };

    /// Há mais de uma imagem a escolher.
    #[must_use]
    pub fn ativa(&self) -> bool {
        self.n > 1
    }

    /// ⭐ **A imagem deste dab** (`0..n`): `u ∈ [0, 1)` o sorteio, `seq` o contador do traço (anda 1 por dab),
    /// `dir` o rumo do traço (`[0, 0]` = ainda sem rumo ⇒ a primeira).
    #[must_use]
    pub fn escolhe(&self, pressure: f32, dir: [f32; 2], u: f32, seq: &mut u32) -> u8 {
        let n = self.n.clamp(1, HOSE_MAX as u8);
        let k = match self.mode {
            HoseMode::Random => {
                let pesos = &self.pesos[..usize::from(n)];
                let total: u32 = pesos.iter().map(|&p| u32::from(p)).sum();
                if total == 0 {
                    return 0;
                }
                let mut r = (u.clamp(0.0, 1.0) * total as f32) as u32;
                let mut k = 0;
                for (i, &p) in pesos.iter().enumerate() {
                    if r < u32::from(p) {
                        k = i as u8;
                        break;
                    }
                    r -= u32::from(p);
                    k = i as u8;
                }
                k
            }
            HoseMode::Sequence => {
                let k = (*seq % u32::from(n)) as u8;
                *seq = seq.wrapping_add(1);
                k
            }
            HoseMode::Pressure => ((pressure.clamp(0.0, 1.0) * f32::from(n)) as u8).min(n - 1),
            HoseMode::Angle => {
                if dir == [0.0, 0.0] {
                    return 0;
                }
                // O «ângulo de losango» (`0..4`, monótono no ângulo, sem transcendentais: HR-5).
                let [x, y] = dir;
                let d = if y >= 0.0 {
                    if x >= 0.0 {
                        y / (x + y)
                    } else {
                        1.0 - x / (-x + y)
                    }
                } else if x < 0.0 {
                    2.0 - y / (-x - y)
                } else {
                    3.0 + x / (x - y)
                };
                ((d / 4.0 * f32::from(n)) as u8).min(n - 1)
            }
        };
        k.min(n - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **O sorteio respeita o PESO**: 30 000 dabs, pesos 4·2·1 — a frequência de cada imagem é o peso ÷ a soma,
    /// dentro de 4 desvios-padrão binomiais.
    #[test]
    fn o_sorteio_respeita_o_peso() {
        let h = ImageHose {
            n: 3,
            mode: HoseMode::Random,
            pesos: [4, 2, 1, 1, 1, 1, 1, 1],
        };
        let mut rng = 0x1234_5678_u64;
        let mut seq = 0;
        let mut conta = [0u32; 3];
        let n = 30_000;
        for _ in 0..n {
            let u = crate::jitter::next_f32(&mut rng);
            conta[usize::from(h.escolhe(1.0, [1.0, 0.0], u, &mut seq))] += 1;
        }
        for (k, peso) in [4.0, 2.0, 1.0].into_iter().enumerate() {
            let p = peso / 7.0;
            let sd = (f64::from(n) * p * (1.0 - p)).sqrt();
            let esperado = f64::from(n) * p;
            assert!(
                (f64::from(conta[k]) - esperado).abs() < 4.0 * sd,
                "imagem {k}: {} × {esperado:.0} ± {sd:.0}",
                conta[k]
            );
        }
        // Peso 0 = nunca.
        let h0 = ImageHose {
            pesos: [1, 0, 1, 1, 1, 1, 1, 1],
            ..h
        };
        assert!((0..2000).all(|i| h0.escolhe(1.0, [1.0, 0.0], i as f32 / 2000.0, &mut seq) != 1));
    }

    #[test]
    fn a_sequencia_a_pressao_e_o_angulo() {
        let h = ImageHose {
            n: 4,
            ..ImageHose::NEUTRA
        };
        let mut seq = 0;
        let s = ImageHose {
            mode: HoseMode::Sequence,
            ..h
        };
        let v: Vec<u8> = (0..6)
            .map(|_| s.escolhe(1.0, [0.0; 2], 0.0, &mut seq))
            .collect();
        assert_eq!(v, [0, 1, 2, 3, 0, 1]);
        let p = ImageHose {
            mode: HoseMode::Pressure,
            ..h
        };
        assert_eq!(
            [0.0, 0.3, 0.6, 1.0].map(|q| p.escolhe(q, [0.0; 2], 0.0, &mut seq)),
            [0, 1, 2, 3]
        );
        let a = ImageHose {
            mode: HoseMode::Angle,
            ..h
        };
        let dirs = [[1.0, 0.1], [-0.1, 1.0], [-1.0, -0.1], [0.1, -1.0]];
        assert_eq!(dirs.map(|d| a.escolhe(1.0, d, 0.0, &mut seq)), [0, 1, 2, 3]);
    }
}
