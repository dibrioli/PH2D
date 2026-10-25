//! **O vocabulário de canais da pilha de CONSTRAINTS** ([`crate::canal::familia::CONSTRAINT`]) — o
//! que se anima num item da pilha (plano `docs/BlenderConstraints/01_plano.md` §2.2): a força, o
//! liga/desliga e os ajustes de cada tipo, nas unidades que o cartão mostra (graus, metros, índice). Quem lê e escreve é `ph2d-app-constraints` (`canais.rs`).
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

/// Limit Rotation: o lado de baixo do arco, em graus.
pub const LIMITE_MIN: u16 = 3;
/// Limit Rotation: o lado de cima do arco, em graus.
pub const LIMITE_MAX: u16 = 4;
/// Limit Distance: a distância ao alvo, em metros (a unidade do mundo).
pub const DISTANCIA: u16 = 5;
/// Look At: o desvio do eixo, em graus.
pub const DESVIO: u16 = 6;
/// Limit Distance: Inside · Outside · Surface ([`MODOS`]).
pub const MODO: u16 = 7;
/// Copy e Limit: World · Local ([`ESPACOS`]).
pub const ESPACO: u16 = 8;
/// Copy: Replace · Add ([`MISTURAS`]).
pub const MISTURA: u16 = 9;

/// As palavras de [`MODO`], pelo índice.
pub const MODOS: &[&str] = &[
    "panel.inspector.constraint.mode.inside",
    "panel.inspector.constraint.mode.outside",
    "panel.inspector.constraint.mode.surface",
];
/// As palavras de [`ESPACO`], pelo índice.
pub const ESPACOS: &[&str] = &[
    "panel.inspector.constraint.space.world",
    "panel.inspector.constraint.space.local",
];
/// As palavras de [`MISTURA`], pelo índice.
pub const MISTURAS: &[&str] = &[
    "panel.inspector.constraint.mix.replace",
    "panel.inspector.constraint.mix.add",
];

/// Os campos de um item, na ordem do cartão (o tipo diz quais tem).
pub const CAMPOS: [u16; 9] = [
    FORCA, LIGADA, LIMITE_MIN, LIMITE_MAX, DISTANCIA, DESVIO, MODO, ESPACO, MISTURA,
];

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
        LIMITE_MIN => c("canal.constraint.limite_min", campo, Especie::Continuo),
        LIMITE_MAX => c("canal.constraint.limite_max", campo, Especie::Continuo),
        DISTANCIA => c("canal.constraint.distancia", campo, Especie::Continuo),
        DESVIO => c("canal.constraint.desvio", campo, Especie::Continuo),
        MODO => c("canal.constraint.modo", campo, Especie::Opcoes(MODOS)),
        ESPACO => c("canal.constraint.espaco", campo, Especie::Opcoes(ESPACOS)),
        MISTURA => c("canal.constraint.mistura", campo, Especie::Opcoes(MISTURAS)),
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
