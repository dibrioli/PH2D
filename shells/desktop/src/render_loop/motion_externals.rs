//! **A porta ÚNICA das MEMBRANAS** — tudo o que o shell publica no canal externo
//! do grafo, no instante em que tem de ser publicado.
//!
//! Um nó recebe params, entradas e o playhead — nada mais. Tudo o que ele precisa
//! e não pode alcançar (a biblioteca de vetor, a fonte tipográfica, um arquivo de
//! som) chega por aqui, sob uma **chave de conteúdo** que os dois lados derivam da
//! mesma função.
//!
//! ⚠️ **O INSTANTE é a metade que não se pode mover, e cada membrana já pagou por
//! ele:** publicar **antes** do dreno de intents mintaria a chave PRÉ-edição — a
//! forma escolhida no quadro anterior, o texto sem a tecla que o artista acabou de
//! digitar, o espectro de um quadro que o cook não vai cozinhar. É por isso que as
//! três moram numa chamada só: *post-drain, pre-cook* é uma propriedade do GRUPO, e
//! três sítios espalhados são três oportunidades de a quarta membrana nascer no
//! lugar errado.

use crate::motion_state::MotionState;

/// Publica as três fontes externas do quadro. `seconds` é o relógio do playhead —
/// só a de áudio o consome (a geometria de forma e texto é atemporal).
pub(super) fn publish_all(motion: &mut MotionState, seconds: f64) {
    // ADR-0154: a forma vetorial VIVA.
    super::motion_shape_gen::publish(motion);
    // O texto, uma instância por GLIFO.
    super::motion_text_gen::publish(motion);
    // As bandas de áudio — função do ARQUIVO e do PLAYHEAD (doc 63 §6), e a única
    // das três que muda com o relógio.
    super::motion_audio_gen::publish(motion, seconds);
}
