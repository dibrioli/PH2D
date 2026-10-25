//! **O vocabulário de canais do MUNDO da física** ([`crate::canal::familia::MUNDO`], F30 §B) — o
//! que o painel *Physics* edita em `PhysicsSettings`, sobre a entidade da [`crate::cena`]. Quem lê e
//! escreve é `ph2d-app-physics` (`canais_mundo.rs`).
//!
//! ⛔ **Os números são gravados no documento da timeline: só APPEND, nunca renumerar** (gate
//! `o_vocabulario_do_mundo_nao_renumera`).
//!
//! O valor é o do campo: m/s², Hz, s; os sub-passos e as iterações são inteiros; adormecer é 0/1
//! (o sinal do limiar angular). ⛔ Fora: a matriz de camadas (36 bits que o painel liga um a um —
//! sem controlo de campo; plano F30 §B).

use crate::canal::{CampoInfo, Especie};

/// `gravity_x`, `gravity_y` (m/s², um grupo).
pub const GRAVIDADE_X: u16 = 1;
pub const GRAVIDADE_Y: u16 = 2;
/// `substeps`, `solver_iterations`.
pub const SUBPASSOS: u16 = 3;
pub const ITERACOES: u16 = 4;
/// `contact_hz`.
pub const CONTACTO_HZ: u16 = 5;
/// `linear_damping`, `angular_damping`, `air_drag`.
pub const AMORT_LINEAR: u16 = 6;
pub const AMORT_ANGULAR: u16 = 7;
pub const AR: u16 = 8;
/// `sleep_linear_threshold`, `time_until_sleep`, e o interruptor (`sleep_enabled`).
pub const SONO_VELOCIDADE: u16 = 9;
pub const SONO_DEMORA: u16 = 10;
pub const ADORMECE: u16 = 11;

/// Todos os campos, na ordem do vocabulário.
pub const CAMPOS: [u16; 11] = [
    GRAVIDADE_X,
    GRAVIDADE_Y,
    SUBPASSOS,
    ITERACOES,
    CONTACTO_HZ,
    AMORT_LINEAR,
    AMORT_ANGULAR,
    AR,
    SONO_VELOCIDADE,
    SONO_DEMORA,
    ADORMECE,
];

const fn c(rotulo: &'static str, grupo: u16, especie: Especie) -> CampoInfo {
    CampoInfo {
        rotulo,
        especie,
        grupo,
        por_i: false,
    }
}

/// O que o campo `campo` do mundo é.
#[must_use]
pub fn info(campo: u16) -> Option<CampoInfo> {
    use Especie::{Continuo, Ligado, Numero};
    Some(match campo {
        GRAVIDADE_X | GRAVIDADE_Y => c("canal.mundo.gravidade", GRAVIDADE_X, Continuo),
        SUBPASSOS => c("canal.mundo.subpassos", campo, Numero),
        ITERACOES => c("canal.mundo.iteracoes", campo, Numero),
        CONTACTO_HZ => c("canal.mundo.contacto_hz", campo, Continuo),
        AMORT_LINEAR => c("canal.mundo.amort_linear", campo, Continuo),
        AMORT_ANGULAR => c("canal.mundo.amort_angular", campo, Continuo),
        AR => c("canal.mundo.ar", campo, Continuo),
        SONO_VELOCIDADE => c("canal.mundo.sono_velocidade", campo, Continuo),
        SONO_DEMORA => c("canal.mundo.sono_demora", campo, Continuo),
        ADORMECE => c("canal.mundo.adormece", campo, Ligado),
        _ => return None,
    })
}

/// O nome do campo dentro do grupo: os eixos da gravidade; o resto, o grupo.
#[must_use]
pub fn membro(campo: u16) -> Option<&'static str> {
    Some(match campo {
        GRAVIDADE_X => "canal.fisica.x",
        GRAVIDADE_Y => "canal.fisica.y",
        _ => info(campo)?.rotulo,
    })
}

#[cfg(test)]
#[path = "canal_mundo_tests.rs"]
mod tests;
