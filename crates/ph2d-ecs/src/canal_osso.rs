//! **O vocabulário de canais do OSSO** ([`crate::canal::familia::OSSO`], F40) — os números do
//! `ph2d_skeleton_ecs::Bone` que o painel do esqueleto edita e não são `PropKind`: o comprimento, a
//! força e os segmentos. Quem lê e escreve é `ph2d-app-skeleton` (`canais_osso.rs`).
//!
//! O osso é uma ENTIDADE (com o seu `StableId`): a chave mora nela, `i = j = 0` — o endereço
//! durável é o da entidade, como o das quatro alças (`PropKind::BoneBend*`), que ficam onde estão.
//! A âncora de IK é um item da pilha (`canal_constraint`).
//!
//! ⛔ **Os números são gravados no documento da timeline: só APPEND, nunca renumerar** (gate
//! `o_vocabulario_do_osso_nao_renumera`).

use crate::canal::{CampoInfo, Especie};

/// `Bone::length` (unidades locais).
pub const COMPRIMENTO: u16 = 1;
/// `Bone::strength` (comprimentos do osso).
pub const FORCA: u16 = 2;
/// `Bone::segments` (inteiro, `1..=MAX_SEGMENTS`).
pub const SEGMENTOS: u16 = 3;

/// Todos os campos, na ordem do vocabulário.
pub const CAMPOS: [u16; 3] = [COMPRIMENTO, FORCA, SEGMENTOS];

const fn c(rotulo: &'static str, grupo: u16, especie: Especie) -> CampoInfo {
    CampoInfo {
        rotulo,
        especie,
        grupo,
        por_i: false,
    }
}

/// O que o campo `campo` do osso é.
#[must_use]
pub fn info(campo: u16) -> Option<CampoInfo> {
    use Especie::{Continuo, Numero};
    Some(match campo {
        COMPRIMENTO => c("canal.osso.comprimento", campo, Continuo),
        FORCA => c("canal.osso.forca", campo, Continuo),
        SEGMENTOS => c("canal.osso.segmentos", campo, Numero),
        _ => return None,
    })
}

/// O nome do campo: cada um é o seu grupo.
#[must_use]
pub fn membro(campo: u16) -> Option<&'static str> {
    Some(info(campo)?.rotulo)
}

#[cfg(test)]
#[path = "canal_osso_tests.rs"]
mod tests;
