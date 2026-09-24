//! Os gates do gizmo do PIVÔ (ordem do dono, 2026-09-19).
//!
//! ⚠️⚠️ **O que estes gates NÃO podem medir, e porquê:** o `resolve` lê as TOMADAS, que só existem
//! depois de o `motion_bridge::dispatch` cozinhar — e um arnês headless não tem shell. *É por isso
//! que a lei foi extraída para [`super::pontos_de`], que é pura:* ela mede-se com streams à mão, e
//! o que fica por gatear é só a FIAÇÃO (o `resolve` chamá-la), que vive na shell.

use ph2d_nodegraph::attr::{Column, Stream};

/// Um sink com `n` linhas, cada uma com a sua posição e a geometria dada.
fn sink(pos: &[[f32; 2]], geo: &[f32]) -> Stream {
    Stream::new(pos.len())
        .with("P", Column::Vec2(pos.to_vec()))
        .with("geometry_id", Column::Scalar(geo.to_vec()))
}

/// A saída de uma forma: uma linha, com o handle dela.
fn forma(geo: f32) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("geometry_id", Column::Scalar(vec![geo]))
}

/// ⭐⭐⭐ **O ALVO CAI NA POSIÇÃO DE CADA PEÇA — e só nas peças DESTA forma.**
///
/// ⚠️ **A metade que separa é o CONTROLO:** um sink com duas formas misturadas. Sem ele, um
/// `pontos_de` que devolvesse o sink inteiro passava — e o artista via alvos sobre peças de uma
/// forma cujo knob ele não está a tocar.
#[test]
fn o_alvo_cai_na_posicao_de_cada_peca_desta_forma() {
    let s = sink(
        &[[1.0, 2.0], [9.0, 9.0], [3.0, 4.0]],
        // a do meio é de OUTRA forma
        &[7.0, 8.0, 7.0],
    );
    assert_eq!(
        super::pontos_de(&forma(7.0), &s),
        vec![[1.0, 2.0], [3.0, 4.0]],
        "so' as peças com o `geometry_id` desta forma"
    );
    // ⛔ E a outra forma vê as DELA — o controlo na direcção oposta, que é o que prova que a
    // régua está a filtrar e não a devolver as duas primeiras.
    assert_eq!(super::pontos_de(&forma(8.0), &s), vec![[9.0, 9.0]]);
}

/// ⚠️ **Um sink SEM `geometry_id` devolve VAZIO** — uma corrente sem geometria não carimbou forma
/// nenhuma. ⛔ A cura preguiçosa («sem coluna, aceita tudo») poria o alvo sobre posições que esta
/// forma nunca tocou.
#[test]
fn sem_coluna_de_geometria_nao_ha_alvo() {
    let sem = Stream::new(2).with("P", Column::Vec2(vec![[1.0, 1.0], [2.0, 2.0]]));
    assert!(super::pontos_de(&forma(7.0), &sem).is_empty());
    // E o outro lado: uma forma que ainda não internou geometria também não acende.
    let s = sink(&[[1.0, 1.0]], &[7.0]);
    assert!(super::pontos_de(&Stream::new(1), &s).is_empty());
}

/// ⚠️ **O TECTO é honrado** — o mesmo do contorno do colisor, porque é o mesmo recurso.
#[test]
fn o_tecto_de_alvos_e_honrado() {
    let n = super::MAX_ALVOS + 37;
    let s = sink(&vec![[0.0, 0.0]; n], &vec![7.0; n]);
    assert_eq!(super::pontos_de(&forma(7.0), &s).len(), super::MAX_ALVOS);
}

/// ⭐⭐ **O GIZMO SÓ ACENDE NO ARRASTO DE UM PIVÔ** — as quatro metades, e cada uma mata uma cura
/// barata: sem arrasto · noutro param · noutro tipo de nó · e o caso positivo.
///
/// ⚠️ **A régua entra pelo canal PUBLICADO** (`set_graph_param_scrub`), que é a porta real — um
/// arnês que chamasse um predicado interno afirmaria sobre código que o produto não percorre.
#[test]
fn o_alvo_so_acende_no_arrasto_de_um_pivot() {
    let mut m = crate::motion_state::MotionState::new();
    let forma = m.doc.graph.add_node("source.shape".to_string());
    let outro = m.doc.graph.add_node("motion.grid".to_string());

    let ph = |v| ph2d_panel_motion_graph::set_graph_param_scrub(v);
    ph(None);
    assert!(
        super::forma_com_pivot_em_arrasto(&m).is_none(),
        "sem arrasto nao ha' alvo"
    );
    ph(Some((forma.0, ph2d_node_motion_shape::param::SIZE)));
    assert!(
        super::forma_com_pivot_em_arrasto(&m).is_none(),
        "arrastar OUTRO param da forma nao acende"
    );
    ph(Some((outro.0, ph2d_node_motion_shape::param::PIVOT_X)));
    assert!(
        super::forma_com_pivot_em_arrasto(&m).is_none(),
        "um `pivot_x` num no' que nao e' uma forma nao acende"
    );
    for p in [
        ph2d_node_motion_shape::param::PIVOT_X,
        ph2d_node_motion_shape::param::PIVOT_Y,
    ] {
        ph(Some((forma.0, p)));
        assert_eq!(
            super::forma_com_pivot_em_arrasto(&m),
            Some(forma),
            "arrastar o {p} da forma ACENDE"
        );
        // E a tomada que ele pede é exactamente essa forma.
        assert_eq!(super::taps_for(&m), vec![forma]);
    }
    // ⛔⛔ **E com a forma a chegar a um SINK, ele pede o sink TAMBÉM** (ciclo 12): o
    // `ponto_gizmo::taps_for` deixou de pedir o sink que a placa já desenhou, e uma forma com
    // pivô é exactamente esse sink — sem esta metade o alvo do pivô sumia em silêncio.
    let out = m.doc.graph.add_node("motion.output".to_string());
    m.doc
        .graph
        .connect(ph2d_nodegraph::graph::Edge {
            from: (forma, 0),
            to: (out, 0),
            delayed: false,
        })
        .expect("forma -> output");
    ph(Some((forma.0, ph2d_node_motion_shape::param::PIVOT_X)));
    assert_eq!(super::taps_for(&m), vec![forma, out]);
    ph(None);
}
