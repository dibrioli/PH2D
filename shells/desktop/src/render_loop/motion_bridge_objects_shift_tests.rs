//! Gates do **canal da aparência DESLOCADA** (doc 89, folha 14) — irmão de
//! `motion_bridge_objects_tests`, cortado pelo mesmo assunto que o módulo de produto:
//! o pai mede *o que a cena tem*, este *quando olhar para isso*.
//!
//! FILHO de `objects` via `#[path]`, então `use super::*` alcança o `appearance_tile`
//! privado e o `publish_shifted`/`wanted_shifts` do irmão de produto.

use super::*;

// ── o canal da aparência DESLOCADA (doc 89, folha 14) ────────────────────────

/// Um grafo com UM `source.object` nomeando `named`, deslocado por `off`.
fn graph_with_shifted_source(named: &str, off: f32) -> ph2d_nodegraph::graph::Graph {
    let mut g = ph2d_nodegraph::graph::Graph::new();
    let n = g.add_node("source.object");
    g.set_text_param(n, "object", named);
    g.set_param(n, ph2d_node_source_object::TIME_OFFSET_PARAM, off);
    g
}

/// **Um offset num meio SEM animação própria é transparente, não um sumiço.**
///
/// Este é o gate que decide se o param é seguro de shipar: um sprite não tem um
/// desenho por quadro, então "meio segundo à frente" tem a mesma resposta que
/// "agora". Sem a cópia, o nó leria um external que ninguém publicou — stream vazio,
/// e o objeto **desaparece da cena** com o param parecendo funcionar noutro objeto.
#[test]
fn a_shift_on_a_still_medium_publishes_the_same_appearance() {
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.set_external(
        "Ball".to_string(),
        appearance_tile([2.0, 3.0], [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], 7),
    );
    let g = graph_with_shifted_source("Ball", 0.25);
    let fbake = crate::motion_flip_bake::FlipObjectBake::default();
    publish_shifted(&mut cook, &g, &fbake);

    let key = ph2d_nodegraph::external::appearance_of("Ball", 0.25);
    let e = cook
        .externals()
        .get(&key)
        .expect("o canal deslocado tem de existir, senao o objeto some");
    assert_eq!(
        e.value.get("texture_id"),
        Some(&Column::Scalar(vec![7.0])),
        "um meio sem animacao mostra a MESMA coisa deslocado"
    );
}

/// **Um documento que não desloca ninguém publica exatamente o que sempre publicou.**
/// A neutralidade da wave, vista da membrana: nenhuma chave nova, nenhuma tile a mais.
#[test]
fn an_unshifted_document_publishes_no_extra_channel() {
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.set_external(
        "Ball".to_string(),
        appearance_tile([2.0, 3.0], [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], 7),
    );
    let before = cook.externals().len();
    // O param escrito EXPLICITAMENTE em zero — um teste que chega ao estado por
    // omissão inverte de sentido no dia em que o default se mover.
    let g = graph_with_shifted_source("Ball", 0.0);
    let fbake = crate::motion_flip_bake::FlipObjectBake::default();
    publish_shifted(&mut cook, &g, &fbake);
    assert_eq!(
        cook.externals().len(),
        before,
        "offset zero nao pode cunhar canal nenhum"
    );
}

/// **A tile do FLIP no quadro deslocado VENCE a cópia transparente** — o P0 desta
/// wave. O par com o gate acima: um deles sozinho seria satisfeito por uma membrana
/// que copia sempre (e o offset seria um controle morto no único meio que o tem).
#[test]
fn a_flips_shifted_tile_beats_the_transparent_copy() {
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.set_external(
        "Walk".to_string(),
        appearance_tile([2.0, 3.0], [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], 7),
    );
    let mut fbake = crate::motion_flip_bake::FlipObjectBake::default();
    // O bake respondeu o pedido deslocado com OUTRA tile (texture_id 90) — que é o
    // que um desenho diferente é, do lado do render.
    fbake.seed_named_shift_for_test(1, "Walk", 0.25, 90, [3.0, 4.0]);
    let g = graph_with_shifted_source("Walk", 0.25);
    publish_shifted(&mut cook, &g, &fbake);

    let key = ph2d_nodegraph::external::appearance_of("Walk", 0.25);
    let e = cook
        .externals()
        .get(&key)
        .expect("o canal deslocado existe");
    assert_eq!(
        e.value.get("texture_id"),
        Some(&Column::Scalar(vec![90.0])),
        "a tile do quadro deslocado tem de vencer a copia"
    );
}

/// **Os offsets que o bake assa são os que o DOCUMENTO pede, com o zero sempre
/// dentro** — e distintos, para que dois nós pedindo o mesmo deslocamento não
/// custem duas tiles.
#[test]
fn the_wanted_shifts_are_the_documents_own_plus_zero() {
    let mut g = ph2d_nodegraph::graph::Graph::new();
    for off in [0.25_f32, 0.25, -0.5, 0.0] {
        let n = g.add_node("source.object");
        g.set_text_param(n, "object", "Walk");
        g.set_param(n, ph2d_node_source_object::TIME_OFFSET_PARAM, off);
    }
    let mut got = wanted_shifts(&g);
    got.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(
        got,
        vec![-0.5, 0.0, 0.25],
        "zero + os distintos, sem repetir"
    );
}
