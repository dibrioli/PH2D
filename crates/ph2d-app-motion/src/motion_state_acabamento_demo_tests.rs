//! Os gates da cena `=135`: a lente está NO fio, nasce só com a vinheta, e o fio passa intacto —
//! medido na SAÍDA, pela bomba do produto.

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

/// A lente está NO fio, entre a cor e a saída — e o fio passa INTACTO (passthrough).
#[test]
fn the_lens_sits_on_the_wire_and_passes_it_through() {
    let (m, cena) = cozida(30);
    let g = &m.doc.graph;
    assert!(
        g.edges()
            .iter()
            .any(|e| e.from.0 == cena.tint && e.to.0 == cena.lente)
    );
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
