//! **A história da fita, dita ao artista** (doc 132 §2): pela MESMA porta da fita
//! ([`ph2d_eval_motion::Fita::historia_em`]), a Timeline pinta a faixa gravada e o cartão das
//! entradas diz quando o scrub mostra a base no lugar do que se publicou.
//!
//! ⚠️ **Uma faixa só, e é a UNIÃO dos objectos Motion por construção:** todos cozem no MESMO
//! grafo, com UMA bomba e UMA fita — o que se grava (e o que se dobra) é a história de todos.

use crate::motion_state::MotionState;
use ph2d_eval_motion::Historia;

/// A chave da queixa no cartão do `value.input`.
pub const CHAVE_DA_QUEIXA: &str = "app.motion.motion_historia.history_folded";

/// **Depois do cozimento do quadro**: a faixa `[base, último]` (segundos da Timeline) vai ao
/// painel, e o segundo que o replay mostrou no lugar da história dobrada fica para o cartão.
pub(crate) fn publica(motion: &mut MotionState, fixed_dt: f64) {
    #[expect(clippy::cast_precision_loss, reason = "tiques cabem num f64")]
    let s = |t: u64| t as f64 * fixed_dt;
    let faixa = if motion.sinks.is_empty() {
        None
    } else {
        motion.pump.fita().faixa().map(|(a, b)| (s(a), s(b)))
    };
    ph2d_panel_timeline::set_recorded_history(faixa);
    motion.historia_dobrada = match motion.pump.historia_mostrada() {
        Historia::Dobrada { desde } => Some(s(desde)),
        Historia::Verdade => None,
    };
}

/// Nada cozeu neste quadro (o Motion fora de cena): sem faixa.
pub(crate) fn limpa() {
    ph2d_panel_timeline::set_recorded_history(None);
}

/// **O que o cartão do nó `tipo` diz**: um `value.input` — o nó cujo valor vem da fita — com o
/// scrub antes da base.
#[must_use]
pub(crate) fn queixa(motion: &MotionState, tipo: &str) -> Option<String> {
    let s = motion.historia_dobrada?;
    (tipo == ph2d_node_value_input::MANIFEST.name)
        .then(|| ph2d_i18n::tr_with(CHAVE_DA_QUEIXA, &[("s", &format!("{s:.2}"))]))
}
