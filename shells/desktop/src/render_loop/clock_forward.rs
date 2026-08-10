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

#[cfg(test)]
mod tests {
    use super::clock_is_playing_forward;
    use ph2d_core::Playhead;

    fn playing() -> Playhead {
        let mut p = Playhead::new(1.0 / 60.0);
        p.play();
        p
    }

    /// O caso que DISPARA — e é o único.
    #[test]
    fn tocando_para_a_frente_dispara() {
        assert!(clock_is_playing_forward(&playing(), false));
    }

    /// Parado não dispara: um sinal descreve uma TRAVESSIA, e um relógio parado não atravessa
    /// nada por mais tempo que se olhe para ele.
    #[test]
    fn parado_nao_dispara() {
        let mut p = playing();
        p.pause();
        assert!(!clock_is_playing_forward(&p, false));
    }

    /// Marcha ré não dispara — a travessia existe, mas ao contrário; publicá-la faria um evento
    /// de *chegar* soar ao *sair*.
    #[test]
    fn marcha_re_nao_dispara() {
        let mut p = playing();
        p.set_rate(-1.0);
        assert!(!clock_is_playing_forward(&p, false));
    }

    /// **A metade que só o `jumped` sabe, e é a que carrega a lei.** Um seek para a frente COM o
    /// play ligado é indistinguível de um avanço enorme para quem só olha o relógio — e é uma
    /// descontinuidade, não uma travessia. Sem esta metade, arrastar a régua vira uma
    /// metralhadora de eventos (ADR-0143 §3).
    #[test]
    fn um_salto_para_a_frente_no_play_nao_dispara() {
        assert!(!clock_is_playing_forward(&playing(), true));
    }
}
