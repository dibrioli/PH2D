//! **O vocabulário de canais das REGRAS de desenho técnico do vetor** ([`crate::canal::familia::CAD`]) — a
//! medida animada da W4.5 (`docs/BlenderConstraints/04_W4_cad.md` §3): o valor de uma regra de distância, de
//! ângulo ou de raio. Quem lê e escreve é `ph2d_vec_entities::medida`, chamado pelo `ph2d-app-vec`.
//!
//! ⛔ **Os números são gravados no documento da timeline: só APPEND** (gate `o_vocabulario_do_cad_nao_renumera`).
//!
//! A entidade é a do path DONO da regra (o do primeiro ponto citado); **`i:j` = o `RegraId`** (32 bits: `i` os
//! BAIXOS, `j` os altos) — apagar outra regra não muda de quem é a curva. Família própria e não a `CONSTRAINT`:
//! essa é drenada e esquecida inteira pelo dono da pilha de constraints do objecto, e cada família drena o seu.

use crate::canal::{CampoInfo, Especie, FieldKey, familia::CAD};

/// A medida de uma distância ou de um raio, em METROS (o mundo); a timeline e o Inspector mostram-na no px/metros
/// do projecto ([`crate::canal::Unidade::Comprimento`]).
pub const MEDIDA: u16 = 1;
/// A medida de um ângulo, em GRAUS — campo próprio para que quem lê o canal saiba que número é
/// ([`crate::canal::Unidade::Angulo`]).
pub const ANGULO: u16 = 2;

/// Todo campo declarado.
pub const CAMPOS: [u16; 2] = [MEDIDA, ANGULO];

/// A chave do campo `campo` da regra de id `regra`.
#[must_use]
pub const fn chave(campo: u16, regra: u32) -> FieldKey {
    #[expect(clippy::cast_possible_truncation, reason = "as duas metades do id")]
    FieldKey::new(CAD, campo, regra as u16, (regra >> 16) as u16)
}

/// O id da regra que a chave endereça.
#[must_use]
pub const fn regra_de(k: FieldKey) -> u32 {
    (k.j as u32) << 16 | k.i as u32
}

/// O que o campo é.
#[must_use]
pub fn info(campo: u16) -> Option<CampoInfo> {
    let rotulo = match campo {
        MEDIDA => "canal.cad.medida",
        ANGULO => "canal.cad.angulo",
        _ => return None,
    };
    Some(CampoInfo {
        rotulo,
        especie: Especie::Continuo,
        grupo: campo,
        por_i: true,
    })
}

/// O nome do membro (cada campo é o seu grupo).
#[must_use]
pub fn membro(campo: u16) -> Option<&'static str> {
    Some(info(campo)?.rotulo)
}

#[cfg(test)]
#[path = "canal_cad_tests.rs"]
mod tests;
