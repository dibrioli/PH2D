//! **O vocabulário de canais da MESA DE SOM** ([`crate::canal::familia::MESA`], F30 §B2) — o que a
//! mesa (`ph2d_audio::Mesa`) guarda, sobre a entidade da [`crate::cena`]. Quem lê e escreve é
//! `ph2d-app-audio` (`canais_mesa.rs`).
//!
//! ⛔ **Os números são gravados no documento da timeline: só APPEND, nunca renumerar** (gate
//! `o_vocabulario_da_mesa_nao_renumera`).
//!
//! O valor é o do documento: ganho linear, Hz, pan `-1..1`, o resto `0..1`; os interruptores 0/1; a
//! chave do ducking é o índice do sub-barramento. Um campo de sub-barramento é
//! [`bus`]`(b, f)` — o barramento mora no NÚMERO (cada um tem o seu rótulo), com `i = j = 0`.

use crate::canal::{CampoInfo, Especie};

pub const GANHO: u16 = 1;
pub const CORTE_ALTO: u16 = 2;
pub const CORTE_BAIXO: u16 = 3;
pub const MUDO: u16 = 4;
pub const PAN: u16 = 5;
pub const LIMITADOR: u16 = 6;
pub const REVERB: u16 = 7;
pub const REVERB_TAMANHO: u16 = 8;
pub const REVERB_RETORNO: u16 = 9;
pub const EQ_GRAVE: u16 = 10;
pub const EQ_MEDIO: u16 = 11;
pub const EQ_AGUDO: u16 = 12;
pub const DELAY: u16 = 13;
pub const DELAY_TEMPO: u16 = 14;
pub const DELAY_FEEDBACK: u16 = 15;
pub const DELAY_RETORNO: u16 = 16;
pub const DUCKING: u16 = 17;
pub const DUCKING_PROFUNDIDADE: u16 = 18;
pub const DUCKING_CHAVE: u16 = 19;

/// Os campos da faixa master, dos efeitos e do ducking, na ordem do vocabulário.
pub const MASTER: [u16; 19] = [
    GANHO,
    CORTE_ALTO,
    CORTE_BAIXO,
    MUDO,
    PAN,
    LIMITADOR,
    REVERB,
    REVERB_TAMANHO,
    REVERB_RETORNO,
    EQ_GRAVE,
    EQ_MEDIO,
    EQ_AGUDO,
    DELAY,
    DELAY_TEMPO,
    DELAY_FEEDBACK,
    DELAY_RETORNO,
    DUCKING,
    DUCKING_PROFUNDIDADE,
    DUCKING_CHAVE,
];

/// Os campos de um sub-barramento (o deslocamento dentro do bloco dele).
pub const B_GANHO: u16 = 0;
pub const B_MUDO: u16 = 1;
pub const B_SOLO: u16 = 2;
pub const B_PAN: u16 = 3;
pub const B_TOM: u16 = 4;
pub const B_CORTE_BAIXO: u16 = 5;
pub const B_ENVIO_REVERB: u16 = 6;
pub const B_ENVIO_DELAY: u16 = 7;
pub const B_COMP: u16 = 8;
/// Os campos de um sub-barramento, na ordem do vocabulário.
pub const DO_BUS: [u16; 9] = [
    B_GANHO,
    B_MUDO,
    B_SOLO,
    B_PAN,
    B_TOM,
    B_CORTE_BAIXO,
    B_ENVIO_REVERB,
    B_ENVIO_DELAY,
    B_COMP,
];

/// Quantos sub-barramentos o vocabulário nomeia (= `ph2d_audio::SUB_BUS_COUNT`, gate na
/// `ph2d-app-audio`).
pub const BUSES: u16 = 4;
/// O primeiro número de sub-barramento, e o tamanho do bloco de cada um.
pub const BUS_BASE: u16 = 32;
pub const BUS_PASSO: u16 = 16;

/// O campo `f` do sub-barramento `b`.
#[must_use]
pub const fn bus(b: u16, f: u16) -> u16 {
    BUS_BASE + b * BUS_PASSO + f
}

/// O inverso de [`bus`]: `(barramento, campo)`, ou `None` fora dos blocos.
#[must_use]
pub fn de_bus(campo: u16) -> Option<(u16, u16)> {
    let r = campo.checked_sub(BUS_BASE)?;
    let (b, f) = (r / BUS_PASSO, r % BUS_PASSO);
    (b < BUSES && DO_BUS.contains(&f)).then_some((b, f))
}

/// Todos os campos: o master e os blocos dos sub-barramentos.
pub fn campos() -> impl Iterator<Item = u16> {
    MASTER
        .into_iter()
        .chain((0..BUSES).flat_map(|b| DO_BUS.into_iter().map(move |f| bus(b, f))))
}

/// As variantes da chave do ducking: os nomes dos sub-barramentos que o painel mostra.
const CHAVES: &[&str] = &[
    "panel.audio_mixer.bus.music",
    "panel.audio_mixer.bus.sfx",
    "panel.audio_mixer.bus.ui",
    "panel.audio_mixer.bus.voice",
];

const ROTULOS_BUS: [[&str; 9]; BUSES as usize] = [
    [
        "canal.mesa.music.ganho",
        "canal.mesa.music.mudo",
        "canal.mesa.music.solo",
        "canal.mesa.music.pan",
        "canal.mesa.music.tom",
        "canal.mesa.music.corte_baixo",
        "canal.mesa.music.envio_reverb",
        "canal.mesa.music.envio_delay",
        "canal.mesa.music.comp",
    ],
    [
        "canal.mesa.sfx.ganho",
        "canal.mesa.sfx.mudo",
        "canal.mesa.sfx.solo",
        "canal.mesa.sfx.pan",
        "canal.mesa.sfx.tom",
        "canal.mesa.sfx.corte_baixo",
        "canal.mesa.sfx.envio_reverb",
        "canal.mesa.sfx.envio_delay",
        "canal.mesa.sfx.comp",
    ],
    [
        "canal.mesa.ui.ganho",
        "canal.mesa.ui.mudo",
        "canal.mesa.ui.solo",
        "canal.mesa.ui.pan",
        "canal.mesa.ui.tom",
        "canal.mesa.ui.corte_baixo",
        "canal.mesa.ui.envio_reverb",
        "canal.mesa.ui.envio_delay",
        "canal.mesa.ui.comp",
    ],
    [
        "canal.mesa.voice.ganho",
        "canal.mesa.voice.mudo",
        "canal.mesa.voice.solo",
        "canal.mesa.voice.pan",
        "canal.mesa.voice.tom",
        "canal.mesa.voice.corte_baixo",
        "canal.mesa.voice.envio_reverb",
        "canal.mesa.voice.envio_delay",
        "canal.mesa.voice.comp",
    ],
];

const fn c(rotulo: &'static str, grupo: u16, especie: Especie) -> CampoInfo {
    CampoInfo {
        rotulo,
        especie,
        grupo,
        por_i: false,
    }
}

/// O que o campo `campo` da mesa é. Cada campo é o seu próprio grupo (uma linha da mesa).
#[must_use]
pub fn info(campo: u16) -> Option<CampoInfo> {
    use Especie::{Continuo, Ligado, Opcoes};
    if let Some((b, f)) = de_bus(campo) {
        let especie = match f {
            B_MUDO | B_SOLO => Ligado,
            _ => Continuo,
        };
        return Some(c(ROTULOS_BUS[b as usize][f as usize], campo, especie));
    }
    Some(match campo {
        GANHO => c("canal.mesa.ganho", campo, Continuo),
        CORTE_ALTO => c("canal.mesa.corte_alto", campo, Continuo),
        CORTE_BAIXO => c("canal.mesa.corte_baixo", campo, Continuo),
        MUDO => c("canal.mesa.mudo", campo, Ligado),
        PAN => c("canal.mesa.pan", campo, Continuo),
        LIMITADOR => c("canal.mesa.limitador", campo, Ligado),
        REVERB => c("canal.mesa.reverb", campo, Ligado),
        REVERB_TAMANHO => c("canal.mesa.reverb_tamanho", campo, Continuo),
        REVERB_RETORNO => c("canal.mesa.reverb_retorno", campo, Continuo),
        EQ_GRAVE => c("canal.mesa.eq_grave", campo, Continuo),
        EQ_MEDIO => c("canal.mesa.eq_medio", campo, Continuo),
        EQ_AGUDO => c("canal.mesa.eq_agudo", campo, Continuo),
        DELAY => c("canal.mesa.delay", campo, Ligado),
        DELAY_TEMPO => c("canal.mesa.delay_tempo", campo, Continuo),
        DELAY_FEEDBACK => c("canal.mesa.delay_feedback", campo, Continuo),
        DELAY_RETORNO => c("canal.mesa.delay_retorno", campo, Continuo),
        DUCKING => c("canal.mesa.ducking", campo, Ligado),
        DUCKING_PROFUNDIDADE => c("canal.mesa.ducking_profundidade", campo, Continuo),
        DUCKING_CHAVE => c("canal.mesa.ducking_chave", campo, Opcoes(CHAVES)),
        _ => return None,
    })
}

/// O nome do campo dentro do grupo: o próprio grupo (um campo por linha).
#[must_use]
pub fn membro(campo: u16) -> Option<&'static str> {
    Some(info(campo)?.rotulo)
}

#[cfg(test)]
#[path = "canal_mesa_tests.rs"]
mod tests;
