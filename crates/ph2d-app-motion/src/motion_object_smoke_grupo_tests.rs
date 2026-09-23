//! Os portões da cena `=14` — o que o roteiro AFIRMA sobre a tela tem de ser o que o grafo monta.

use super::{COLUNAS, build};
use crate::motion_state::MotionState;
use ph2d_eval_motion::{BlendWith, MisturaDoSink, sink_blend_with, sink_style};

/// ⭐⭐ **Quatro colunas × duas médias, e cada célula no alcance que o roteiro lhe dá** — o
/// controlo em `Normal` (sem camada, logo fora do Vello) e as outras três em grupo.
#[test]
fn cada_celula_esta_no_alcance_que_o_roteiro_lhe_da() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc.graph, "Object");
    assert_eq!(sinks.len(), COLUNAS.len() * 2, "uma saida por celula");
    for (i, &sink) in sinks.iter().enumerate() {
        let (alcance, _) = COLUNAS[i / 2];
        let mistura = MisturaDoSink {
            blend: sink_style(&m.doc.graph, sink).blend,
            com: sink_blend_with(&m.doc.graph, sink),
            sink: 0,
        };
        match alcance {
            None => assert!(
                !mistura.tem_camada(),
                "a coluna de controlo nao pode misturar em grupo"
            ),
            Some(com) => {
                assert!(
                    mistura.tem_camada(),
                    "a celula {i} tem de misturar em grupo"
                );
                assert_eq!(mistura.com, com, "a celula {i} esta no alcance errado");
            }
        }
    }
    // O CONTROLO da régua: os três alcances estão TODOS na cena, senão ela não mostra a escolha.
    for com in BlendWith::ALL {
        assert!(
            sinks
                .iter()
                .any(|&s| sink_blend_with(&m.doc.graph, s) == com
                    && sink_style(&m.doc.graph, s).blend == 3),
            "falta a coluna {com:?}"
        );
    }
}

/// ⭐ **Nenhuma célula vai à placa** — um sink em grupo recusa-a pela cerca do doc 118 W4, e o
/// controlo em `Normal` também não: a cena tem OITO saídas (a escada do doc 98).
#[test]
fn as_celulas_em_grupo_recusam_a_placa() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc.graph, "Object");
    let em_grupo = sinks
        .iter()
        .filter(|&&s| crate::motion_bridge::gpu::sink_mistura_em_grupo(&m.doc.graph, s))
        .count();
    assert_eq!(em_grupo, 6, "tres alcances x duas medias");
}
