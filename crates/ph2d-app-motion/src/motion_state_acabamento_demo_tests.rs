//! Os gates da cena `=135`: o filtro e a lente estão NO fio, a lente nasce só com a vinheta, o
//! filtro é a sombra projectada, e o fio passa intacto — medido na SAÍDA, pela bomba do produto.

use super::*;
use crate::motion_state::MotionState;

fn cozida(tiques: u64) -> (MotionState, Cena) {
    let mut m = MotionState::new();
    let cena = monta(&mut m.doc, &m.registry).expect("a cena monta");
    m.sinks = vec![cena.saida];
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    for t in 0..tiques {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
        let sinks = m.sinks.clone();
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
    }
    (m, cena)
}

/// ⭐ **A cena ensina o que acontece** (`CLAUDE.md` §5.0): ao abrir, a lente CORRE (a vinheta
/// ligada) e os outros três efeitos estão neutros — o passo (1) do roteiro.
#[test]
fn the_scene_opens_with_only_the_vignette_on() {
    let (mut m, cena) = cozida(2);
    let lido = ph2d_node_fx_lens::from_graph(&m.doc.graph).expect("a cena tem a lente");
    assert_eq!(lido.no, cena.lente);
    let l = lido.lente;
    assert!(!l.vinheta.neutra());
    assert!(l.aberracao.neutra() && l.grade.neutra(false) && l.grao.neutra());
    m.lente.prepara(&m.doc.graph, 2);
    assert!(m.lente.liga(true), "a lente corre com o Motion na cena");
}

/// O filtro e a lente estão NO fio, entre a cor e a saída — e o fio passa INTACTO (passthrough).
#[test]
fn the_filter_and_the_lens_sit_on_the_wire_and_pass_it_through() {
    let (m, cena) = cozida(30);
    let g = &m.doc.graph;
    let liga = |de, para| g.edges().iter().any(|e| e.from.0 == de && e.to.0 == para);
    assert!(liga(cena.tint, cena.forma) && liga(cena.forma, cena.camada));
    assert!(liga(cena.camada, cena.lente));
    assert!(
        g.edges()
            .iter()
            .any(|e| e.from.0 == cena.lente && e.to.0 == cena.saida)
    );
    let n = |id| m.pump.cook.peek(id).expect("cozeu")[0].as_stream().count();
    assert!(
        n(cena.saida) > 100,
        "a galáxia tem estrelas: {}",
        n(cena.saida)
    );
    assert_eq!(n(cena.saida), n(cena.tint));
}

/// ⭐ Os passos dos filtros: a cena abre com o RELEVO em cada disco e a SOMBRA da camada inteira,
/// pela ordem do fio — a pilha parte-se entre os dois — e os filtros correm.
#[test]
fn the_scene_opens_with_a_bevel_on_each_shape_and_a_shadow_on_the_layer() {
    let (mut m, cena) = cozida(2);
    m.filtro.prepara(&m.doc.graph);
    let pilha = m.filtro.pilha();
    assert_eq!(
        pilha.iter().map(|d| d.no).collect::<Vec<_>>(),
        vec![cena.forma, cena.camada]
    );
    assert_eq!(pilha[0].op, ph2d_ecs::FxOp::new(ph2d_ecs::FxOp::BEVEL));
    assert_eq!(pilha[1].op.kind, ph2d_ecs::FxOp::DROP_SHADOW);
    let (forma, camada) = ph2d_node_fx_filter::partition(pilha);
    assert_eq!((forma.len(), camada.len()), (1, 1));
    assert!(
        m.filtro.liga(true),
        "os filtros correm com o Motion na cena"
    );
    assert!(m.filtro.queixa(cena.forma).is_none() && m.filtro.queixa(cena.camada).is_none());
}

/// O passo (4): o 1.º na camada inteira e o 2.º em cada forma ⇒ o cartão do 2.º di-lo.
#[test]
fn an_each_shape_after_a_whole_layer_says_so_on_its_card() {
    let (mut m, cena) = cozida(2);
    m.doc
        .graph
        .set_param(cena.forma, ph2d_node_fx_filter::param::SCOPE, 1.0);
    m.doc
        .graph
        .set_param(cena.camada, ph2d_node_fx_filter::param::SCOPE, 0.0);
    m.filtro.prepara(&m.doc.graph);
    assert!(m.filtro.queixa(cena.forma).is_none());
    let q = m.filtro.queixa(cena.camada).expect("o cartão diz");
    assert!(q.contains("Whole Layer"), "{q}");
}

/// O roteiro só nomeia rows que o cartão do `Lens` pinta (o nome que aparece NA TELA).
#[test]
fn every_row_the_script_names_is_on_the_lens_card() {
    let m = MotionState::new();
    let id = ph2d_nodegraph::node::NodeTypeId::of(ph2d_node_fx_lens::TYPE_NAME);
    let rotulos: Vec<&str> = m
        .registry
        .param_ui(id)
        .unwrap_or(&[])
        .iter()
        .map(|h| ph2d_i18n::tr(h.label))
        .collect();
    for nome in [
        "Vignette",
        "Aberration",
        "Temperature",
        "Saturation",
        "Exposure",
        "LUT File",
        "LUT Mix",
        "Grain",
    ] {
        assert!(rotulos.contains(&nome), "{nome} ∉ {rotulos:?}");
    }
    for secao in [
        "node.group.vignette",
        "node.group.aberration",
        "node.group.grade",
        "node.group.grain",
    ] {
        assert_ne!(ph2d_i18n::tr(secao), secao, "secção sem nome: {secao}");
    }
}

/// O roteiro só nomeia rows e opções que o cartão do `Filter` pinta.
#[test]
fn every_row_the_script_names_is_on_the_filter_card() {
    let m = MotionState::new();
    let id = ph2d_nodegraph::node::NodeTypeId::of(ph2d_node_fx_filter::TYPE_NAME);
    let hints = m.registry.param_ui(id).unwrap_or(&[]);
    let rotulos: Vec<&str> = hints.iter().map(|h| ph2d_i18n::tr(h.label)).collect();
    for nome in [
        "Effect", "Apply To", "Radius", "Offset X", "Offset Y", "Opacity",
    ] {
        assert!(rotulos.contains(&nome), "{nome} ∉ {rotulos:?}");
    }
    let tipos: Vec<&str> = ph2d_node_fx_filter::KIND_LABELS
        .iter()
        .map(|c| ph2d_i18n::tr(c))
        .collect();
    for tipo in ["Bevel", "Inner Shadow", "Drop Shadow"] {
        assert!(tipos.contains(&tipo), "{tipo} ∉ {tipos:?}");
    }
    let opcoes: Vec<&str> = ph2d_node_fx_filter::SCOPE_LABELS
        .iter()
        .map(|c| ph2d_i18n::tr(c))
        .collect();
    assert_eq!(opcoes, ["Each Shape", "Whole Layer"]);
}
