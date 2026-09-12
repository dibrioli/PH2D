//! **AS FIXTURAS DO GIZMO DE WARP — montadas, e NÃO marchadas.**
//!
//! ⛔⛔ **A separação é o achado de 2026-09-08, e ela é load-bearing:** a sonda
//! [`crate::render_loop::warp_gizmo_probe`] montava a cena **e marchava a bomba na CPU** na mesma função, e
//! por isso deu `resolve = Some` na cena exacta do report enquanto o app dava `None`. A marcha
//! da CPU cozinha as tomadas de passagem — logo qualquer fixtura que marche **esconde** o
//! defeito da rota totalmente-na-GPU, que é precisamente não haver marcha nenhuma.
//!
//! ⇒ aqui monta-se e arma-se a tomada, e **quem chama escolhe a rota**: a sonda marcha na CPU,
//! o portão [`crate::render_loop::motion_bridge_gpu_taps_tests`] entrega o quadro ao device.
//!
//! ⚠️ **A selecção do grafo é GLOBAL** (`ph2d_panel_motion_graph::set_graph_selection`), então
//! duas fixturas em paralelo trocam de nó seleccionado uma à outra. [`trava`] é a porta única —
//! todo teste que monte uma destas cenas a toma antes.

use crate::motion::motion_state::MotionState;
use crate::render_loop::warp_gizmo;
use ph2d_nodegraph::graph::{Edge, NodeId};
use std::sync::{Mutex, MutexGuard, OnceLock};

/// A trava da SELECÇÃO — ver o cabeçalho. Envenenada não interessa: o estado que ela protege é
/// reescrito por inteiro na montagem seguinte.
pub(super) fn trava() -> MutexGuard<'static, ()> {
    static M: OnceLock<Mutex<()>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Arma as tomadas do gizmo pela MESMA porta e na MESMA ordem do `motion_bridge`.
fn arma(m: &mut MotionState) {
    let taps = warp_gizmo::taps_for(m);
    m.pump.set_taps(&taps);
    m.pump.clear_tap_fires();
}

/// `motion.grid → <nó> → motion.output`, com o nó seleccionado. Montada, não marchada.
pub(super) fn cadeia(tipo: &str) -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    let grid = m.doc.graph.add_node("motion.grid".to_string());
    let no = m.doc.graph.add_node(tipo.to_string());
    let out = m.doc.graph.add_node("motion.output".to_string());
    for (a, b) in [(grid, no), (no, out)] {
        m.doc
            .graph
            .connect(Edge {
                from: (a, 0),
                to: (b, 0),
                delayed: false,
            })
            .expect("as portas encaixam");
    }
    m.sinks = vec![out];
    ph2d_panel_motion_graph::set_graph_selection(vec![no.0]);
    arma(&mut m);
    (m, no)
}

/// A CENA DO DONO: a `=111`, com um `motion.bezier_warp` enfiado nela — que é o que ele tinha no
/// ecrã quando reportou. ⚠️ Uma cadeia sintética limpa **não reproduz**, e é por isso que esta
/// existe: *a fixtura tem de ser a do report*. Montada, não marchada.
pub(super) fn cena_do_dono(depois_do_mirror: bool) -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    let sinks = crate::motion::motion_demo_legend::monta("111", &mut m.doc, &m.registry).0;
    m.sinks = sinks;
    let acha = |t: &str| -> Option<NodeId> {
        m.doc
            .graph
            .nodes()
            .iter()
            .find(|n| n.type_name == t)
            .map(|n| n.id)
    };
    let mirror = acha("motion.mirror").expect("a cena 111 tem espelho");
    let twist = acha("motion.twist").expect("a cena 111 tem torcao");
    let (de, para) = if depois_do_mirror {
        (mirror, twist)
    } else {
        (
            warp_gizmo::upstream_of(&m.doc.graph, mirror).expect("a montante"),
            mirror,
        )
    };
    let bw = m.doc.graph.add_node("motion.bezier_warp".to_string());
    m.doc.graph.disconnect(para, 0);
    for (a, b) in [(de, bw), (bw, para)] {
        m.doc
            .graph
            .connect(Edge {
                from: (a, 0),
                to: (b, 0),
                delayed: false,
            })
            .expect("encaixa");
    }
    ph2d_panel_motion_graph::set_graph_selection(vec![bw.0]);
    arma(&mut m);
    (m, bw)
}

/// A marcha da CPU sobre uma cena já montada — a rota que a SONDA escolhe, dita em voz alta
/// para que ninguém a confunda com o quadro do produto (ver o cabeçalho).
pub(super) fn marcha_na_cpu(m: &mut MotionState) {
    let sinks = m.sinks.clone();
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let _ = m.pump.advance_or_scrub_scoped(
        &m.doc.graph,
        &m.registry,
        &sinks,
        0,
        |t| t as f64 / 60.0,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &scopes,
    );
}
