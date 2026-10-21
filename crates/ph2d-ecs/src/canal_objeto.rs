//! **O vocabulário de canais do OBJECTO** ([`crate::canal::familia::OBJETO`]) — o que o Inspector
//! edita num objecto e não é um dos `PropKind` de sempre: o skew do `Transform` e a aparência da
//! sprite (tint, self tint, cantos, silhueta, opacidade final, emissão). Quem lê e escreve é
//! `ph2d-app-components` (`canais_objeto.rs`); aqui só mora o vocabulário
//! ([plano 35 §3.0](../../../docs/UI_New_and_Simple/vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md)).
//!
//! ⛔ **Os números são gravados no documento da timeline: só APPEND, nunca renumerar** (gate
//! `o_vocabulario_do_objeto_nao_renumera`).
//!
//! O valor é o do DOCUMENTO: o skew em radianos; as cores em `0..=1` (o `Sprite` guarda `f32`);
//! a emissão é o multiplicador (`0` = o componente ausente, que é o que o render lê).
//!
//! ⚠️ O alfa do `tint` é o `PropKind::Opacity` de sempre (`Sprite::tint[3]`): o grupo do tint tem
//! só o RGB, e quem pinta a swatch junta-lhe a `Opacity` (`marcas_inspector`).

use crate::canal::CampoInfo;

/// `Transform::skew_x` / `skew_y` (radianos) — um grupo: a linha do Inspector é uma.
pub const SKEW_X: u16 = 1;
pub const SKEW_Y: u16 = 2;
/// `SpriteEmissive` (o multiplicador; `0` = ausente).
pub const EMISSAO: u16 = 10;
/// `Sprite::opacity` — a opacidade FINAL (o slider), ortogonal ao `tint[3]`.
pub const OPACIDADE: u16 = 11;
/// `Sprite::tint` r, g, b (`TINTA + 0..3`).
pub const TINTA: u16 = 20;
/// `Sprite::self_tint` r, g, b, a (`AUTO_TINTA + 0..4`).
pub const AUTO_TINTA: u16 = 30;
/// `SpriteCornerTint` r, g, b, a (`CANTOS + 0..4`), `j` = o canto (TL, TR, BL, BR) — o degradê
/// inteiro é UM grupo (a linha dos cantos tem um losango). Ausente = branco.
pub const CANTOS: u16 = 40;
/// `Sprite::tint_fill` — a silhueta (0 ou 1).
pub const SILHUETA: u16 = 50;

/// O fim (exclusivo) de cada bloco de cor.
pub const TINTA_FIM: u16 = TINTA + 3;
pub const AUTO_TINTA_FIM: u16 = AUTO_TINTA + 4;
pub const CANTOS_FIM: u16 = CANTOS + 4;

/// Quantos cantos o degradê tem (o `j` de [`CANTOS`]).
pub const N_CANTOS: u16 = 4;

const fn c(rotulo: &'static str, grupo: u16) -> CampoInfo {
    CampoInfo {
        rotulo,
        discreto: false,
        grupo,
        por_i: false,
        texto: false,
    }
}

/// O que o campo `campo` do objecto é.
#[must_use]
pub fn info(campo: u16) -> Option<CampoInfo> {
    Some(match campo {
        SKEW_X | SKEW_Y => c("canal.objeto.skew", SKEW_X),
        EMISSAO => c("canal.objeto.emissao", campo),
        OPACIDADE => c("canal.objeto.opacidade", campo),
        TINTA..TINTA_FIM => c("canal.objeto.tinta", TINTA),
        AUTO_TINTA..AUTO_TINTA_FIM => c("canal.objeto.auto_tinta", AUTO_TINTA),
        CANTOS..CANTOS_FIM => c("canal.objeto.cantos", CANTOS),
        SILHUETA => CampoInfo {
            discreto: true,
            ..c("canal.objeto.silhueta", campo)
        },
        _ => return None,
    })
}

/// O nome do campo dentro do grupo (a linha do membro no dope-sheet).
#[must_use]
pub fn membro(campo: u16) -> Option<&'static str> {
    const RGBA: [&str; 4] = ["canal.m.r", "canal.m.g", "canal.m.b", "canal.m.a"];
    let em = |base: u16| RGBA.get(usize::from(campo - base)).copied();
    match campo {
        SKEW_X => Some("canal.m.x"),
        SKEW_Y => Some("canal.m.y"),
        TINTA..TINTA_FIM => em(TINTA),
        AUTO_TINTA..AUTO_TINTA_FIM => em(AUTO_TINTA),
        CANTOS..CANTOS_FIM => em(CANTOS),
        _ => Some(info(campo)?.rotulo),
    }
}

#[cfg(test)]
#[path = "canal_objeto_tests.rs"]
mod tests;
