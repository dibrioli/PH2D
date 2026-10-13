//! O gate da cena `=139` (doc 132 §2, P2): monta, e com o tecto do roteiro a fita dobra em segundos.

use super::*;
use crate::motion_state::MotionState;
use ph2d_eval_motion::{Historia, Razao};

/// O tecto do gate: `4 s` a 60 Hz — o roteiro diz «espere ~5 s».
const MAX_TIQUES: u64 = 240;

/// ⭐ **A CENA MONTA E, COM O TECTO PEQUENO, DOBRA EM MENOS DE [`MAX_TIQUES`]** — pelo caminho da
/// CPU do produto (publica, escopos, bomba). Os gatilhos `sinal_*` são o que a enche: sem eles
/// (só a `batida`) a mesma fita não dobra no mesmo tempo (o CONTROLO).
#[test]
fn a_cena_monta_e_com_o_tecto_pequeno_dobra_em_segundos() {
    let corre = |com_sinais: bool| -> Option<u64> {
        let mut m = MotionState::new();
        m.doc = MotionDoc::new();
        let c = monta(&mut m.doc, &m.registry).expect("a cena monta");
        if !com_sinais {
            let sinais: Vec<NodeId> = m
                .doc
                .graph
                .nodes()
                .iter()
                .filter(|n| {
                    m.doc
                        .graph
                        .node_text_param_overrides(n.id)
                        .and_then(|t| t.get(ph2d_node_value_input::NAME_KEY))
                        .is_some_and(|s| s.starts_with("sinal_"))
                })
                .map(|n| n.id)
                .collect();
            assert_eq!(sinais.len(), GATILHOS);
            for n in sinais {
                assert!(m.doc.graph.remove_node(n));
            }
        }
        m.sinks = vec![c.saida];
        m.pump.define_a_lei(true);
        let tecto = ph2d_eval_motion::razao::tecto_da_fita(Some(TECTO_DO_ROTEIRO), None);
        m.pump.define_o_razao_da_fita(Razao::new(tecto));
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        for t in 0..MAX_TIQUES {
            #[expect(clippy::cast_precision_loss, reason = "um tique")]
            let s = t as f64 / 60.0;
            crate::motion_externals::publish_all(&mut m, s);
            let sinks = m.sinks.clone();
            m.pump.advance_or_scrub_scoped(
                &m.doc.graph,
                &m.registry,
                &sinks,
                t,
                |t| t as f64 / 60.0,
                m.default_uv_rect,
                m.default_size,
                &scopes,
            );
            assert!(m.pump.last_error().is_none(), "{:?}", m.pump.last_error());
            if m.pump.fita().historia_em(0) != Historia::Verdade {
                return Some(t);
            }
        }
        None
    };
    let t = corre(true).expect("com o tecto do roteiro a fita tem de dobrar em 4 s");
    eprintln!("cena 139: a fita dobrou no tique {t}");
    assert!(
        corre(false).is_none_or(|sem| sem > t),
        "CONTROLO: os gatilhos `sinal_*` são o que enche a fita"
    );
}
