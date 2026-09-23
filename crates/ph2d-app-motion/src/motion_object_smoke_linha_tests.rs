//! Os portões da cena `=15` — o que o roteiro AFIRMA sobre a tela tem de ser o que o grafo monta.

use super::{COLUNAS, SOMBRA_MULTIPLY, build, sombra_de};
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// ⭐⭐ **A sombra da FORMA leva o modo da coluna até à cena vectorial** — a corrente inteira, pela
/// porta do produto: o grafo da cena coze, o lowering vectorial leva o degrau da linha, e a
/// linha da sombra da coluna da direita pede a camada `Multiply` (doc 118 §9).
///
/// ⚠️ O CONTROLO na mesma cena: a coluna da esquerda (`Sink`) não escreve a coluna `blend`, e toda
/// linha dela sai com o degrau `0` — senão «a sombra multiplica» não distinguiria nada.
#[test]
fn a_sombra_da_forma_leva_o_modo_da_coluna_ate_ao_vello() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc.graph, "Object");
    assert_eq!(sinks.len(), COLUNAS.len() * 2, "uma saida por celula");
    // CONTROLO da régua: a cena TEM as duas colunas — sem a `Multiply` o ramo que a mede nunca corria.
    assert!(COLUNAS.iter().any(|c| c.0 == SOMBRA_MULTIPLY) && COLUNAS.iter().any(|c| c.0 == 0.0));
    // ⚠️ A publicação REAL da forma (a do quadro): sem ela o `source.shape` lê o canal externo
    // vazio e a célula não emite nada — o gate mediria o vazio.
    crate::motion_shape_gen::publish(&mut m, 0.0);
    // As saídas das FORMAS são as ímpares (a fileira de baixo de cada coluna).
    for (k, &(modo, _)) in COLUNAS.iter().enumerate() {
        let sink = sinks[2 * k + 1];
        let lido = m
            .doc
            .graph
            .node_param_overrides(sombra_de(&m.doc.graph, sink))
            .and_then(|p| p.get(ph2d_node_fx_drop_shadow::SHADOW_BLEND).copied());
        assert_eq!(lido, Some(modo), "a coluna {k} tem o modo errado");
        let out = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, sink, 0.0)
            .expect("a celula coze");
        let s = out[0].as_stream();
        let mut v = Vec::new();
        ph2d_eval_motion::lower_to_vector_instances_onto(s, ph2d_render::SinkStyle::PLAIN, &mut v);
        assert_eq!(v.len(), 2, "a forma e a sombra dela (coluna {k})");
        let degraus: Vec<u8> = v.iter().map(|i| i.blend_linha).collect();
        if modo == SOMBRA_MULTIPLY {
            assert_eq!(
                degraus,
                [4, 0],
                "a sombra em Multiply, a forma no modo do sink"
            );
            assert!(crate::motion_shape_gen::mistura::camada_da_linha(&v[0]).is_some());
        } else {
            assert!(
                !matches!(s.get("blend"), Some(Column::Scalar(_))) || degraus == [0, 0],
                "CONTROLO: a coluna Sink nao pede modo a linha nenhuma"
            );
            assert_eq!(degraus, [0, 0]);
        }
    }
}
