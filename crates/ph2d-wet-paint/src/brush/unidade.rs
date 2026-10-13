//! As cerdas como QUATRO SLIDERS normalizados — a porta por onde a seção Shape do hospedeiro guarda
//! o tapete de cerdas (plano 27 D1: o tipo *Bristles* do Shape, com Count · Strength · Size · Felt).
//!
//! A faixa e o passo de cada knob são os do Tuning ([`KNOB_DEFS`]) — uma só fonte. O valor é ENCAIXADO
//! no passo e refeito como `inteiro × num ÷ den` (divisão de inteiros, arredondada correctamente), de
//! modo que `knob → unidade → knob` devolve o MESMO `f64` para todo valor no passo: `30 ÷ 1000` é o
//! literal `0.03`. É o que deixa um preset antigo (os knobs no mapa `engine`) passar pelo Shape ao bit.

use super::BristleKnobs;
use crate::tuning::{KNOB_DEFS, Knob};

/// Os knobs das cerdas na ORDEM dos sliders do Shape: Count · Strength · Size · Felt.
pub const BRISTLE_UNIT_KNOBS: [Knob; 4] = [
    Knob::BristleCount,
    Knob::BristleStrength,
    Knob::BristleSize,
    Knob::Felt,
];

/// A grelha de um knob: `valor = (lo + passos) · num ÷ den`, `passos ∈ 0..=n`.
struct Grelha {
    lo: i64,
    n: i64,
    num: f64,
    den: f64,
}

fn grelha(knob: Knob) -> Grelha {
    let d = &KNOB_DEFS[knob as usize];
    let (num, den) = if d.step >= 1.0 {
        (d.step.round(), 1.0)
    } else {
        (1.0, (1.0 / d.step).round())
    };
    let lo = (d.min * den / num).round() as i64;
    let hi = (d.max * den / num).round() as i64;
    Grelha {
        lo,
        n: hi - lo,
        num,
        den,
    }
}

/// O knob a partir do slider `u ∈ [0, 1]`, encaixado no passo do Tuning.
#[must_use]
pub fn knob_from_unit(knob: Knob, u: f32) -> f64 {
    let g = grelha(knob);
    let passos = (f64::from(u.clamp(0.0, 1.0)) * g.n as f64).round() as i64;
    (g.lo + passos) as f64 * g.num / g.den
}

/// O slider `[0, 1]` de um valor do knob (fora da faixa ⇒ a ponta).
#[must_use]
pub fn unit_from_knob(knob: Knob, v: f64) -> f32 {
    let g = grelha(knob);
    let passos = ((v * g.den / g.num).round() as i64 - g.lo).clamp(0, g.n);
    (passos as f64 / g.n as f64) as f32
}

impl BristleKnobs {
    /// As cerdas dos quatro sliders do Shape (Count · Strength · Size · Felt).
    #[must_use]
    pub fn from_unit(u: [f32; 4]) -> Self {
        let [count, strength, size, felt] =
            [0, 1, 2, 3].map(|i| knob_from_unit(BRISTLE_UNIT_KNOBS[i], u[i]));
        BristleKnobs {
            felt,
            bristle_count: count,
            bristle_strength: strength,
            bristle_size: size,
        }
    }

    /// Os quatro sliders do Shape destas cerdas — o inverso exacto de [`Self::from_unit`] no passo.
    #[must_use]
    pub fn to_unit(self) -> [f32; 4] {
        let v = [
            self.bristle_count,
            self.bristle_strength,
            self.bristle_size,
            self.felt,
        ];
        [0, 1, 2, 3].map(|i| unit_from_knob(BRISTLE_UNIT_KNOBS[i], v[i]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo valor no passo de cada knob faz a ida-e-volta AO BIT, e os da fábrica são os literais.
    #[test]
    fn a_ida_e_volta_pelo_slider_e_exacta_em_todo_passo() {
        for knob in BRISTLE_UNIT_KNOBS {
            let g = grelha(knob);
            let d = &KNOB_DEFS[knob as usize];
            assert!(g.n > 0 && g.n <= 1000, "{knob:?}: n = {}", g.n);
            for p in 0..=g.n {
                let v = (g.lo + p) as f64 * g.num / g.den;
                assert!(v >= d.min - 1e-12 && v <= d.max + 1e-12, "{knob:?} {v}");
                let u = unit_from_knob(knob, v);
                assert_eq!(
                    knob_from_unit(knob, u).to_bits(),
                    v.to_bits(),
                    "{knob:?} passo {p}"
                );
            }
        }
        // Os valores dos 12 presets de fábrica (`receitas_de_fabrica.rs`) e o padrão do motor.
        let fabrica = [
            (0.0, 900.0, 2.0, 4.0),
            (0.05, 2000.0, 2.0, 1.0),
            (0.0, 4000.0, 2.0, 0.5),
            (0.03, 1500.0, 1.0, 1.5),
            (0.03, 3000.0, 1.0, 3.0),
            (0.01, 950.0, 1.0, 1.0),
        ];
        for (felt, count, strength, size) in fabrica {
            let k = BristleKnobs {
                felt,
                bristle_count: count,
                bristle_strength: strength,
                bristle_size: size,
            };
            let de_volta = BristleKnobs::from_unit(k.to_unit());
            assert_eq!(
                [
                    de_volta.felt,
                    de_volta.bristle_count,
                    de_volta.bristle_strength,
                    de_volta.bristle_size
                ]
                .map(f64::to_bits),
                [felt, count, strength, size].map(f64::to_bits)
            );
        }
    }
}
