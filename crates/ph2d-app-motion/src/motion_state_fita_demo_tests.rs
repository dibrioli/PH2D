//! O gate da cena `=139` (doc 132 §2, P2): monta, e com o tecto do roteiro a fita dobra em segundos.

use super::*;
use crate::motion_state::MotionState;
use ph2d_eval_motion::{Historia, Razao};

/// O tecto do gate: a dobra tem de chegar ≥ 1 s antes de o relógio parar ([`PARA_EM_S`]), para a
/// faixa parada ter largura na régua.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "segundos inteiros"
)]
const MAX_TIQUES: u64 = ((PARA_EM_S - 1.0) * 60.0) as u64;

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

/// ⭐ **O RELÓGIO PÁRA AOS [`PARA_EM_S`] E A FAIXA FICA NA RÉGUA** — a ferramenta toca o relógio do
/// editor ao entrar, e sem a paragem a faixa seguia-o para fora do ecrã (foto do coordenador, 13/10:
/// 25 s depois de abrir, a faixa em `[22, 25]` s numa régua de 0 a 6). Pela porta do `dispatch`
/// ([`crate::motion_demo_legend::encena_a_paragem`]), quadro a quadro como o produto; e o CONTROLO:
/// sem a cena montada, o relógio não pára.
#[test]
fn o_relogio_da_cena_para_aos_cinco_segundos() {
    let corre = |monta_a_cena: bool| {
        if monta_a_cena {
            let mut doc = MotionDoc::new();
            let mut reg = NodeRegistry::new();
            ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry");
            assert!(build(&mut doc, &reg).is_some(), "a cena monta");
        }
        let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
        ph.play();
        for _ in 0..1_500 {
            ph.advance();
            crate::motion_demo_legend::encena_a_paragem(&mut ph);
        }
        ph
    };
    let ph = corre(true);
    assert!(!ph.is_playing(), "a cena pára o relógio");
    assert!(
        (ph.time() - PARA_EM_S).abs() < 1e-9,
        "parou em {}",
        ph.time()
    );
    // Uma vez só: tocar de novo é do artista.
    let mut de_novo = ph;
    de_novo.play();
    for _ in 0..60 {
        de_novo.advance();
        crate::motion_demo_legend::encena_a_paragem(&mut de_novo);
    }
    assert!(de_novo.is_playing() && de_novo.time() > PARA_EM_S + 0.9);
    let sem = corre(false);
    assert!(
        sem.is_playing() && sem.time() > 20.0,
        "CONTROLO: sem a cena, nada pára"
    );
}
