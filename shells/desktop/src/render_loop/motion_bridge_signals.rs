//! **O que o grafo GRITA** — as duas metades do lado de Motion da fronteira `pulse.* ↔
//! ph2d-runtime` (folha 12 do doc 89), irmãs do `motion_bridge` pelo teto de LOC do shell.
//!
//! ⚠️ Nada aqui conhece a `ph2d-runtime`: a ponte deixa um FATO em `MotionState.signals_out` e
//! quem o transforma num `Signal` é o shell, que já é dono da outbox e já drena as outras duas
//! fontes. O produtor não chama ninguém (ADR-0075).

use crate::motion_state::MotionState;

/// Os nós `pulse.signal` do grafo, em ordem de id — as TOMADAS da marcha.
///
/// Irmã do `output_nodes`, e pela mesma razão: o shell endereça tipos de nó pelo NOME
/// canônico, como faz com o id de ferramenta.
pub(super) fn signal_nodes(
    graph: &ph2d_nodegraph::graph::Graph,
) -> Vec<ph2d_nodegraph::graph::NodeId> {
    let mut ids: Vec<_> = graph
        .nodes()
        .iter()
        .filter(|inst| inst.type_name == "pulse.signal")
        .map(|inst| inst.id)
        .collect();
    ids.sort();
    ids
}

/// Lê as tomadas do tique recém-cozido e acumula o que gritou.
///
/// ⚠️ **Um nó SEM nome não grita**, e isso não é uma validação: um `pulse.signal` acabado de
/// soltar no grafo tem o campo vazio, e um sinal anônimo não é endereçável por consumidor
/// nenhum — publicá-lo seria um evento que ninguém pode escutar de propósito.
///
/// ⚠️ **`any` colapsa a LINHA no QUADRO**, e a contagem viaja junto: uma grade de 576 pontos
/// que dispara ao mesmo tempo é UM evento com `rows = 576`, não 576 eventos.
pub(super) fn collect_signals(motion: &mut MotionState, tick: u64) {
    if motion.signal_taps.is_empty() {
        return;
    }
    for (node, stream) in motion.pump.tap_streams() {
        let rows = ph2d_node_pulse_signal::fired_rows(stream);
        if rows == 0 {
            continue;
        }
        let Some(name) = motion
            .doc
            .graph
            .node_text_params()
            .get(node)
            .and_then(|m| m.get(ph2d_node_pulse_signal::NAME_KEY))
            .filter(|n| !n.trim().is_empty())
        else {
            continue;
        };
        motion
            .signals_out
            .push(crate::motion_state::MotionSignalOut {
                name: name.to_string(),
                tick,
                rows,
            });
    }
}

#[cfg(test)]
#[path = "motion_bridge_signals_tests.rs"]
mod tests;
