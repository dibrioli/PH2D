//! **A receita e a quantidade** (E4, [ADR-0197 §1.1](../../../../docs/architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md)):
//! Pigmento + Água dão SÓ a receita (a concentração `φ`), o Volume (a Opacity) dá SÓ a quantidade `h` por dab, e
//! a pressão só toca a quantidade (a força do carimbo). É a ÚNICA função que lê os controlos da tinta — o que um
//! dab pousa por unidade de depósito é `φ·h` de pigmento e `(1−φ)(1−m)·h ÷ K_W` de água. Porquê e números:
//! `docs/Painter/Arquitetura/E4_receita_e_quantidade.md`.

use crate::sim::Params;
use crate::tuning::Knob;

/// Massa de pigmento que vale UMA unidade de água no volume (`φ = P ÷ (P + K_W·veículo)`).
pub const K_W: f64 = 1000.0;
/// A pressão do slider em que a lei velha foi afinada: o produto nunca a move (`Sliders::default().pressure`).
pub const PRESSAO_DE_REFERENCIA: f64 = 0.7;
/// O `PigmentPerDab` de arranque do motor (`tuning/defs.rs`).
pub const PIGMENTO_DO_MOTOR: f64 = 600.0;
/// O `h` por unidade de depósito com a carga `1` e o Volume `1`: o volume que o motor de arranque pousava.
pub const H_REF: f64 = PIGMENTO_DO_MOTOR + K_W * veiculo(1.0, 1.0);

/// O veículo da receita: a água do slider × `WaterPerDab`, na escala da lei velha à pressão de referência
/// (sem a pressão invertida — ela MORRE: era a receita a mudar com a mão).
#[must_use]
pub const fn veiculo(agua: f64, agua_por_dab: f64) -> f64 {
    let u = agua * 0.35;
    2.0 * u * u * agua_por_dab / (0.2 + 0.7 * PRESSAO_DE_REFERENCIA + 0.01)
}

/// Os controlos que a receita lê — um controlo, uma grandeza.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Controlos {
    /// Pigment (`Knob::PigmentPerDab`): a parte de pigmento da receita.
    pub pigmento: f64,
    /// Water (slider): o veículo.
    pub agua: f64,
    /// `Knob::WaterPerDab`: escala do veículo (funde-se ao Water).
    pub agua_por_dab: f64,
    /// Volume (a Opacity, `Sliders::pigmento`): só a quantidade.
    pub volume: f64,
    /// A carga do pincel (`Knob::Intensity`, fundido ao Volume): a quantidade de base do preset.
    pub carga: f64,
    /// O meio `m` (`0` = sem meio; nenhum preset o tem antes da E10).
    pub meio: f64,
}

impl Controlos {
    #[must_use]
    pub fn do_motor(s: &super::Sliders, p: &Params) -> Self {
        Self {
            pigmento: p.k(Knob::PigmentPerDab),
            agua: s.water,
            agua_por_dab: p.k(Knob::WaterPerDab),
            volume: s.pigmento,
            carga: p.k(Knob::Intensity),
            meio: 0.0,
        }
    }
}

/// `{ φ, m, h }` de um dab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Receita {
    pub phi: f64,
    pub m: f64,
    pub h: f64,
}

impl Receita {
    /// Pigmento por unidade de depósito: `φ·h`.
    #[must_use]
    pub fn pigmento(&self) -> f64 {
        self.phi * self.h
    }

    /// Água por unidade de depósito: `(1−φ)(1−m)·h ÷ K_W`.
    #[must_use]
    pub fn agua(&self) -> f64 {
        (1.0 - self.phi) * (1.0 - self.m) * self.h / K_W
    }
}

/// **A receita**: Pigmento + Água → `φ`; Volume × carga → `h`. Nenhum dos dois lados lê o outro.
#[must_use]
pub fn receita(c: &Controlos) -> Receita {
    let p = c.pigmento.max(0.0);
    let v = K_W * veiculo(c.agua, c.agua_por_dab).max(0.0);
    let phi = if p + v > 0.0 { p / (p + v) } else { 0.0 };
    Receita {
        phi,
        m: c.meio.clamp(0.0, 1.0),
        h: c.volume.max(0.0) * c.carga.max(0.0) * H_REF,
    }
}

/// **A força do carimbo** (a forma × a pressão): só a QUANTIDADE e a área — `b` é a pressão da mão, `pressao`
/// o slider do motor. Já não lê `Knob::Intensity` (fundido ao Volume na [`receita`]).
#[must_use]
pub fn forca_do_carimbo(b: f64, pressao: f64) -> f64 {
    (b * 0.5 * (0.2 + 0.7 * pressao)).clamp(0.0, 3.0)
}

/// A água de um dab do traço fino (o hospedeiro repõe o pigmento por área e a água fica a do dab inteiro,
/// `03_MOTORES.md` §7 b): `fracao` = `agua_do_dab` ou `1 ÷ peso`. Não é um controlo do painel — fica em aberto
/// para o filme (E6a).
#[must_use]
pub fn agua_do_traco_fino(agua: f64, fracao: f64) -> f64 {
    agua * fracao
}

/// **A migração de um preset da lei velha** (`PigmentPerDab`, `WaterPerDab`, Water e Opacity do preset) →
/// (`PigmentPerDab`, `Intensity`) da lei nova: o mesmo pigmento e a mesma água por dab, à pressão de referência,
/// com a Opacity do preset. A Opacity velha mexia SÓ no pigmento, por isso entra na receita do preset.
#[must_use]
pub fn migra_da_lei_velha(
    pigmento: f64,
    agua_por_dab: f64,
    agua: f64,
    opacidade: f64,
) -> (f64, f64) {
    if opacidade <= 0.0 || !opacidade.is_finite() {
        return (pigmento, 1.0);
    }
    let p = pigmento * opacidade;
    let carga = (p + K_W * veiculo(agua, agua_por_dab)) / (opacidade * H_REF);
    (p, carga)
}

impl super::Engine {
    /// A receita deste dab com os controlos de agora.
    #[must_use]
    pub fn receita_do_dab(&self, p: &Params) -> Receita {
        receita(&Controlos::do_motor(&self.sliders, p))
    }

    /// Os parâmetros de uma porta que POUSA tinta: o depósito lê o pigmento da receita.
    pub(crate) fn params_do_deposito(&self) -> Params {
        let p = self.sim.gather_params(&self.tuning);
        let r = self.receita_do_dab(&p);
        p.com_receita(&r)
    }
}

#[cfg(test)]
#[path = "receita_tests.rs"]
mod tests;
