//! **A receita e a quantidade** (E4, [ADR-0197 §1.1](../../../../docs/architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md)):
//! a Dilution dá SÓ a receita (a concentração `φ = 1 − Dilution`), o Volume (µm) dá SÓ a quantidade `h` por dab, e
//! a pressão só toca a quantidade (a força do carimbo). É a ÚNICA função que lê os controlos da tinta — o que um
//! dab pousa por unidade de depósito é `φ·h` de pigmento e `(1−φ)·h ÷ K_W` de líquido. Porquê e números:
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

/// Os controlos que a receita lê — um controlo, uma grandeza (E10, `F3_controlos.md` §9).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Controlos {
    /// Dilution (`Knob::Dilution`): `1 − φ`.
    pub diluicao: f64,
    /// Volume (`Knob::Volume`): a quantidade por dab, em µm de tinta (`h = Volume · M_UM`).
    pub volume_um: f64,
    /// Medium (`Knob::Medium`): o meio `m`.
    pub meio: f64,
}

impl Controlos {
    /// Os knobs da E10; já não lê `PigmentPerDab`, `WaterPerDab`, `Intensity`, o Water nem a Opacity.
    #[must_use]
    pub fn do_motor(_s: &super::Sliders, p: &Params) -> Self {
        Self {
            diluicao: p.k(Knob::Dilution),
            volume_um: p.k(Knob::Volume),
            meio: p.k(Knob::Medium),
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

    /// Líquido LIVRE por unidade de depósito: `[(1−φ) − m·L·φ]₊·h ÷ K_W` — o diluente é líquido com qualquer meio
    /// (dono 01/11); o meio só muda as propriedades dele e o que o pigmento prende ([`crate::meio`]).
    #[must_use]
    pub fn agua(&self) -> f64 {
        crate::meio::liquido_livre(self.phi, self.m) * self.h / K_W
    }
}

/// **A receita**: Dilution → `φ`; Volume → `h`. Nenhum dos dois lados lê o outro.
#[must_use]
pub fn receita(c: &Controlos) -> Receita {
    Receita {
        phi: (1.0 - c.diluicao).clamp(0.0, 1.0),
        m: c.meio.clamp(0.0, 1.0),
        h: c.volume_um.max(0.0) * f64::from(crate::grid::tinta::M_UM),
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

/// **A migração para os controlos da E10** (marca `controlosE10`): (`PigmentPerDab` já com a Opacity dobrada pela
/// E4, `WaterPerDab`, Water, carga = `Intensity`, Opacity) → (Dilution, Volume µm), exacta — a mesma `φ` e o mesmo
/// `h` que a receita da E4 dava.
#[must_use]
pub fn migra_para_controlos(
    pigmento: f64,
    agua_por_dab: f64,
    agua: f64,
    carga: f64,
    opacidade: f64,
) -> (f64, f64) {
    let p = pigmento.max(0.0);
    let v = K_W * veiculo(agua, agua_por_dab).max(0.0);
    let diluicao = if p + v > 0.0 { 1.0 - p / (p + v) } else { 1.0 };
    let volume = opacidade.max(0.0) * carga.max(0.0) * H_REF / f64::from(crate::grid::tinta::M_UM);
    (diluicao, volume)
}

/// **O meio por família** (F3 §3.2, dono 01/11 Q5): `familia` = a pasta do meio (`Oil`, `Pastel`…), `grupo` = a
/// subpasta. Só dois pontos (o seletor Paint type, dono 01/11): óleo, Express Oil e os pastéis de óleo e cera
/// (`Pastel/Oil`, `Pastel/Scrape`) `1`; o resto `0` — a Acrylic é Water com Re-wet 0. Um só sítio: a migração dos
/// presets (`ph2d-tool-painter`) e as sondas leem-no.
#[must_use]
pub fn meio_da_familia(familia: &str, grupo: &str) -> f64 {
    match (familia, grupo) {
        ("Oil" | "ExpressOil", _) | ("Pastel", "Oil" | "Scrape") => 1.0,
        _ => 0.0,
    }
}

/// O Body que dá o MESMO traço isolado que a parede `levelingOwnStroke = 1` dava a um pincel SEM meio (Wet Liner,
/// Wet Wash, Gouache Opaque); no óleo o meio já dá a cedência (Body 0). Medido: `E10a2_meio_e_corpo.md`.
pub const BODY_DA_PAREDE: f64 = 0.16;

/// **Os controlos da E10 de um preset gravado** (`pigmentPerDab`, `waterPerDab`, `intensity` do mapa, Water e
/// Opacity dos volumes): a E4 primeiro se o preset não a traz (`receita_e4 = false`), depois a E10 — o mesmo
/// caminho do carregador de presets. Devolve (Dilution, Volume µm).
#[must_use]
pub fn controlos_do_preset(
    pigmento: f64,
    agua_por_dab: f64,
    carga: f64,
    agua: f64,
    opacidade: f64,
    receita_e4: bool,
) -> (f64, f64) {
    let (pig, carga) = if receita_e4 {
        (pigmento, carga)
    } else {
        migra_da_lei_velha(pigmento, agua_por_dab, agua, opacidade)
    };
    migra_para_controlos(pig, agua_por_dab, agua, carga, opacidade)
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
