//! **O vocabulário de canais da pilha de CONSTRAINTS** ([`crate::canal::familia::CONSTRAINT`]) — o
//! que se anima num item da pilha (plano `docs/BlenderConstraints/01_plano.md` §2.2): a força e o
//! liga/desliga. Quem lê e escreve é `ph2d-app-constraints` (`canais.rs`).
//!
//! ⛔ **Os números são gravados no documento da timeline: só APPEND, nunca renumerar** (gate
//! `o_vocabulario_das_constraints_nao_renumera`).
//!
//! A entidade é a dona da pilha; **`i:j` = o `Item::id`** (32 bits: `i` os BAIXOS, `j` os altos) — o
//! endereço durável: reordenar a pilha não muda de quem é a curva. Cada campo é um grupo, partido por
//! item: o grupo parte-se por `i` ([`FieldKey::grupo`]), daí os bits baixos em `i`.

use crate::canal::{CampoInfo, Especie, FieldKey, familia::CONSTRAINT};

/// `Item::influence` (`0..=1`).
pub const FORCA: u16 = 1;
/// `Item::enabled` (0/1).
pub const LIGADA: u16 = 2;

/// Os campos de um item, na ordem do cartão.
pub const CAMPOS: [u16; 2] = [FORCA, LIGADA];

/// A chave do campo `campo` do item `item`.
#[must_use]
pub const fn chave(campo: u16, item: u32) -> FieldKey {
    #[expect(clippy::cast_possible_truncation, reason = "as duas metades do id")]
    FieldKey::new(CONSTRAINT, campo, item as u16, (item >> 16) as u16)
}

/// O `Item::id` que a chave endereça.
#[must_use]
pub const fn item_de(k: FieldKey) -> u32 {
    (k.j as u32) << 16 | k.i as u32
}

const fn c(rotulo: &'static str, grupo: u16, especie: Especie) -> CampoInfo {
    CampoInfo {
        rotulo,
        especie,
        grupo,
        por_i: true,
    }
}

/// O que o campo é.
#[must_use]
pub fn info(campo: u16) -> Option<CampoInfo> {
    Some(match campo {
        FORCA => c("canal.constraint.forca", campo, Especie::Continuo),
        LIGADA => c("canal.constraint.ligada", campo, Especie::Ligado),
        _ => return None,
    })
}

/// O nome do campo dentro do grupo: cada grupo é um campo só.
#[must_use]
pub fn membro(campo: u16) -> Option<&'static str> {
    Some(info(campo)?.rotulo)
}

#[cfg(test)]
#[path = "canal_constraint_tests.rs"]
mod tests;
