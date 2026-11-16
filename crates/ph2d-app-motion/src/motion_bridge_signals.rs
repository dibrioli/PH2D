//! **O que o grafo GRITA** — o lado de Motion da fronteira `pulse.* ↔ ph2d-runtime` (doc 89
//! folha 12). Nada aqui conhece a `ph2d-runtime`: a ponte deixa um FATO em
//! `MotionState.signals_out` e o shell o publica (ADR-0075).

use crate::motion_state::MotionState;

const SEND: &str = ph2d_node_send_signal::MANIFEST.name;

/// Os nós `pulse.signal` do grafo, em ordem de id — as TOMADAS da marcha.
///
/// Irmã do `output_nodes`, e pela mesma razão: o shell endereça tipos de nó pelo NOME
/// canônico, como faz com o id de ferramenta.
pub fn signal_nodes(graph: &ph2d_nodegraph::graph::Graph) -> Vec<ph2d_nodegraph::graph::NodeId> {
    let mut ids: Vec<_> = graph
        .nodes()
        .iter()
        .filter(|i| i.type_name == "pulse.signal" || i.type_name == SEND)
        .map(|inst| inst.id)
        .collect();
    ids.sort();
    ids
}

/// Lê o LIVRO-RAZÃO do quadro — o que cada tomada disse em cada tique marchado — e acumula o
/// que gritou.
///
/// ⚠️ Lê `tap_fires` (não `tap_streams`): independe da ROTA (CPU, híbrida, placa). Um nó SEM
/// nome não grita. As linhas COLAPSAM: 576 pontos a disparar juntos = UM evento, `rows = 576`.
pub fn collect_signals(motion: &mut MotionState) {
    motion.envios.retem(&motion.signal_taps);
    if motion.signal_taps.is_empty() {
        return;
    }
    for (tick, node, stream) in motion.pump.tap_fires() {
        let tick = *tick;
        // MI-8: o Send signal decide a borda aqui, com o valor do tique anterior.
        let rows = match ph2d_node_send_signal::ler(stream) {
            Some(l) => motion.envios.passo(*node, tick, &l),
            None => ph2d_node_pulse_signal::fired_rows(stream),
        };
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
        let source = crate::motion_objeto::origem::dono_do_no(motion, *node).map(|d| d.entity);
        let name = name.to_string();
        let out = crate::motion_state::MotionSignalOut {
            name,
            source,
            tick,
            rows,
        };
        motion.signals_out.push(out);
    }
}

#[cfg(test)]
#[path = "motion_bridge_signals_tests.rs"]
mod tests;
