//! **O vocabulário de canais da CAMADA do Flip** ([`crate::canal::familia::FLIP`]) — o que a
//! secção Layers do painel Flip edita numa camada (plano 35 §A.8): o olho, a opacidade, a mistura
//! e a profundidade multiplano. Quem lê e escreve é `ph2d-app-flip` (`canais_camada.rs`), pelo
//! MESMO funil que o painel usa
//! ([plano 35 §3.0](../../../docs/UI_New_and_Simple/vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md)).
//!
//! ⛔ **Os números são gravados no documento da timeline: só APPEND, nunca renumerar** (gate
//! `o_vocabulario_do_flip_nao_renumera`).
//!
//! A entidade é a do objecto Flip (`FlipObjectRef`); **`i` = o `LayerId` da camada** — o endereço
//! durável (reordenar as camadas não muda de quem é a curva). Cada linha do painel é um grupo, e o
//! grupo parte-se por camada.
//!
//! ⛔ Fora (plano 35 §8): o cadeado (trava de EDIÇÃO), o nome (identidade), a ordem (estrutura) e
//! os traços (a lei do Grease Pencil, `ph2d-flip/src/autokey.rs`).

use crate::canal::CampoInfo;

/// `FlipLayer::visible` (0/1).
pub const VISIVEL: u16 = 1;
/// `FlipLayer::opacity` (`0..=1`).
pub const OPACIDADE: u16 = 2;
/// `FlipLayer::blend` (a `tag` do `BlendMode`, `0..MAX_BLEND_MODES`).
pub const MISTURA: u16 = 3;
/// `FlipLayer::depth` — a fração de paralaxe multiplano (`0..=1`).
pub const PROFUNDIDADE: u16 = 4;

/// Os campos de uma camada, na ordem do painel.
pub const CAMPOS: [u16; 4] = [VISIVEL, MISTURA, OPACIDADE, PROFUNDIDADE];

const fn c(rotulo: &'static str, grupo: u16, discreto: bool) -> CampoInfo {
    CampoInfo {
        rotulo,
        discreto,
        grupo,
        por_i: true,
        texto: false,
    }
}

/// O que o campo `campo` da camada é.
#[must_use]
pub fn info(campo: u16) -> Option<CampoInfo> {
    Some(match campo {
        VISIVEL => c("canal.flip.visivel", campo, true),
        OPACIDADE => c("canal.flip.opacidade", campo, false),
        MISTURA => c("canal.flip.mistura", campo, true),
        PROFUNDIDADE => c("canal.flip.profundidade", campo, false),
        _ => return None,
    })
}

/// O nome do campo dentro do grupo: cada grupo é um campo só, o nome é o do grupo.
#[must_use]
pub fn membro(campo: u16) -> Option<&'static str> {
    Some(info(campo)?.rotulo)
}

#[cfg(test)]
#[path = "canal_flip_tests.rs"]
mod tests;
