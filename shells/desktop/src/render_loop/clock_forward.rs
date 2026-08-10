//! **A lei de *"o relógio está TOCANDO para a frente?"*** — uma pergunta, dois emissores.
//!
//! Ela não mora em nenhum dos dois porque não é de nenhum dos dois: a timeline pergunta antes
//! de gritar um marker, e a ponte de Motion antes de publicar um `pulse.signal`. Escrever a
//! condição duas vezes é como uma delas ganha um caso especial e a outra não — e o modo de
//! falha é uma metralhadora de eventos ao arrastar a régua.

use ph2d_core::Playhead;

/// Um scrub, uma marcha ré e uma pausa não disparam nada.
///
/// `jumped` é o que separa *tocar* de *saltar*: um seek para a frente COM o play ligado parece
/// um avanço enorme, e é uma descontinuidade, não uma travessia (ADR-0143 §3).
pub(crate) fn clock_is_playing_forward(playhead: &Playhead, jumped: bool) -> bool {
    !jumped && playhead.is_advancing_forward()
}
