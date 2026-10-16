//! ⛔ **Uma leitura da placa REPETIDA não é uma leitura nova** (auditoria do fecho, 2026-09-24).
//!
//! Com a leitura dos cartões sem espera (`GpuCook::tap_sem_espera`), o quadro em que a placa ainda
//! não respondeu recebe a leitura ANTERIOR. Os fios «marcham» quando o digest MUDA e a sonda junta
//! uma amostra por leitura — os dois comparam leituras entre quadros, logo uma repetida fazia os
//! fios piscar e a sonda duplicar valores, exactamente nas cenas pesadas.
//!
//! Sem placa de propósito: a decisão é DADO (`MotionState::tap_fresco`), e a leitura é montada à mão.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::{Graph, NodeId};
use std::collections::BTreeMap;

fn uma_nuvem(x: f32) -> Stream {
    Stream::new(4).with("P", Column::Vec2(vec![[x, 0.0]; 4]))
}

/// Um nó que a placa encenou (nada na memória da CPU) e a leitura dele, montada à mão.
fn na_placa() -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    let mut g = Graph::new();
    let n = g.add_node("motion.grid");
    m.doc.graph = g;
    m.gpu_live = true;
    (m, n)
}

fn quente(m: &mut MotionState, n: NodeId, x: f32, fresca: bool) -> bool {
    m.tap_fresco = fresca;
    let leitura: BTreeMap<NodeId, Stream> = [(n, uma_nuvem(x))].into();
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    stamp(m, Some(&leitura), &mut snap);
    snap.nodes
        .iter()
        .find(|c| c.id == n.0)
        .expect("o cartao existe")
        .hot
}

/// ⭐ **Os fios mantêm o estado num quadro de leitura repetida.** O CONTROLO é a mesma leitura
/// declarada NOVA: aí «nada mudou» é verdade e o fio pára.
#[test]
fn uma_leitura_repetida_nao_para_os_fios() {
    let (mut m, n) = na_placa();
    assert!(
        !quente(&mut m, n, 1.0, true),
        "a 1.a leitura nao tem com que comparar"
    );
    assert!(quente(&mut m, n, 2.0, true), "o valor mudou: o fio marcha");
    assert!(
        quente(&mut m, n, 2.0, false),
        "a placa ainda nao respondeu: a leitura e' a MESMA de antes, e o fio continua a marchar"
    );
    assert!(
        !quente(&mut m, n, 2.0, true),
        "CONTROLO: a mesma leitura declarada NOVA diz que nada mudou, e o fio para"
    );
}

/// ⭐ **A sonda não junta uma amostra repetida.** O CONTROLO é a leitura nova, que junta.
#[test]
fn uma_leitura_repetida_nao_entra_no_anel_da_sonda() {
    let (mut m, n) = na_placa();
    m.probe = Some(n);
    let leitura: BTreeMap<NodeId, Stream> = [(n, uma_nuvem(1.0))].into();
    let sonda = |m: &mut MotionState, fresca: bool| {
        m.tap_fresco = fresca;
        super::super::edit::sample_probe(m, 0.0, Some(&leitura)).expect("a sonda le");
        m.probe_ring.len()
    };
    assert_eq!(sonda(&mut m, true), 1, "uma leitura nova, uma amostra");
    assert_eq!(
        sonda(&mut m, false),
        1,
        "uma leitura REPETIDA nao e' amostra nova"
    );
    assert_eq!(sonda(&mut m, true), 2, "CONTROLO: a seguinte, nova, junta");
}

/// O cartão do nó `n` depois de um `stamp` com a leitura `tap`.
fn o_cartao(m: &mut MotionState, n: NodeId, tap: Option<&BTreeMap<NodeId, Stream>>) -> String {
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    stamp(m, tap, &mut snap);
    let c = snap.nodes.iter().find(|c| c.id == n.0).expect("o cartao");
    format!("{:?} · {:?}", c.readout, c.count)
}

/// ⛔ **Bug #20: num quadro da placa o memo da CPU NÃO está vazio** — guarda o último quadro da CPU
/// (uma troca de rota), e lido primeiro ele congelava o cartão e as colunas nesse quadro. ⭐ Com a
/// leitura da placa, o cartão e as colunas são os MESMOS que com o memo vazio. CONTROLO: o memo
/// velho existe e, sem leitura da placa (um quadro de CPU), é ele que o cartão mostra.
#[test]
fn num_quadro_da_placa_o_cartao_le_a_placa_e_nao_o_memo_velho() {
    let (mut m, n) = na_placa();
    let leitura: BTreeMap<NodeId, Stream> = [(
        n,
        uma_nuvem(7.0).with("marca", Column::Scalar(vec![1.0; 4])),
    )]
    .into();
    let limpo = o_cartao(&mut m, n, Some(&leitura));
    let (g, r) = (&m.doc.graph, &m.registry);
    m.pump.cook.cook(g, r, n, 0.0).expect("a CPU coze a grelha");
    let do_memo = o_cartao(&mut m, n, None);
    assert_ne!(
        do_memo, limpo,
        "CONTROLO: o memo velho diz outra coisa que a placa"
    );
    assert_eq!(
        o_cartao(&mut m, n, Some(&leitura)),
        limpo,
        "num quadro da placa o cartão mostra a placa, não o memo velho da CPU"
    );
    m.gpu_tap = Some(leitura);
    let cols = super::super::columns::at(&m, n, 0).expect("há colunas");
    assert!(
        cols.iter().any(|c| c.name == "marca"),
        "as colunas são as da placa, não as do memo velho"
    );
}
