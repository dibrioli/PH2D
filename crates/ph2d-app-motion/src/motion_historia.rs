//! **A história da fita, dita ao artista** (doc 132 §2): pela MESMA porta da fita
//! ([`ph2d_eval_motion::Fita::historia_em`]), a Timeline pinta a faixa gravada e o cartão das
//! entradas diz quando o scrub mostra outra coisa que o que se publicou (a base de uma história
//! dobrada, ou o publicado agora antes do primeiro tique gravado).
//!
//! ⚠️ **Uma faixa só, e é a UNIÃO dos objectos Motion por construção:** todos cozem no MESMO
//! grafo, com UMA bomba e UMA fita — o que se grava (e o que se dobra) é a história de todos.

use crate::motion_state::MotionState;
use ph2d_eval_motion::Historia;

/// A chave da queixa no cartão do `value.input`: o replay mostra a BASE de uma história dobrada.
pub const CHAVE_DA_QUEIXA: &str = "app.motion.motion_historia.history_folded";

/// A chave da queixa quando o tique não está na fita: o replay mostra o publicado AGORA.
pub const CHAVE_NAO_GRAVADA: &str = "app.motion.motion_historia.history_not_recorded";

/// **Depois do cozimento do quadro**: a faixa `[base, último]` (segundos da Timeline) vai ao
/// painel, e o que o replay mostrou no lugar do publicado fica para o cartão.
pub(crate) fn publica(motion: &mut MotionState, fixed_dt: f64) {
    #[expect(clippy::cast_precision_loss, reason = "tiques cabem num f64")]
    let s = |t: u64| t as f64 * fixed_dt;
    let faixa = if motion.sinks.is_empty() {
        None
    } else {
        motion.pump.fita().faixa().map(|(a, b)| (s(a), s(b)))
    };
    ph2d_panel_timeline::set_recorded_history(faixa);
    motion.queixa_da_historia = match motion.pump.historia_mostrada() {
        Historia::Verdade => None,
        Historia::Dobrada { desde } => Some(frase_dobrada(s(desde))),
        Historia::NaoGravada => Some(ph2d_i18n::tr(CHAVE_NAO_GRAVADA).to_owned()),
    };
}

/// A queixa da história dobrada no segundo `segundos` — a frase que o cartão pinta, e a que o gate
/// de largura mede com o pior segundo.
#[must_use]
pub fn frase_dobrada(segundos: f64) -> String {
    ph2d_i18n::tr_with(CHAVE_DA_QUEIXA, &[("s", &format!("{segundos:.2}"))])
}

/// Nada cozeu neste quadro (o Motion fora de cena): sem faixa.
pub(crate) fn limpa() {
    ph2d_panel_timeline::set_recorded_history(None);
}

/// **O que o cartão do nó `tipo` diz**: um `value.input` — o nó cujo valor vem da fita — quando o
/// replay não mostra o que se publicou nesse tique.
#[must_use]
pub(crate) fn queixa(motion: &MotionState, tipo: &str) -> Option<String> {
    let q = motion.queixa_da_historia.as_ref()?;
    (tipo == ph2d_node_value_input::MANIFEST.name).then(|| q.clone())
}
