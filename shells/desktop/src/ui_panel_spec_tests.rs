//! Gates do **plano do painel** (plano UI/UX W8b).

use super::*;
use ph2d_ecs::{ChildOf, Transform, VecPathRef};

/// Uma moldura com quatro filhos: três vestidos e **um que é só desenho**.
fn frame_with_children() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let frame = sim
        .world_mut()
        .spawn((Name("Painel de Cor".into()), Transform::IDENTITY))
        .id();
    let add = |sim: &mut SimWorld, name: &str, kind: Option<u16>| {
        let e = sim
            .world_mut()
            .spawn((Name(name.into()), Transform::IDENTITY, VecPathRef(1)))
            .id();
        sim.world_mut().entity_mut(e).insert(ChildOf(frame));
        if let Some(k) = kind {
            sim.world_mut().entity_mut(e).insert(VecWidget { kind: k });
        }
        e
    };
    add(&mut sim, "Título", Some(WidgetKind::SectionHeader.code()));
    add(&mut sim, "Fundo", None);
    add(&mut sim, "Opacity", Some(WidgetKind::Slider.code()));
    add(&mut sim, "Visível", Some(WidgetKind::Toggle.code()));
    (sim, frame)
}

/// **Só quem VESTE vira row** — e a ordem é a dos filhos.
#[test]
fn the_spec_is_the_dressed_children_in_tree_order() {
    let (sim, frame) = frame_with_children();
    let spec = of(&sim, frame);

    assert_eq!(spec.title, "Painel de Cor");
    assert_eq!(
        spec.id, "painel_de_cor",
        "o slug nao saiu do nome da moldura"
    );
    let got: Vec<(&str, &str)> = spec
        .rows
        .iter()
        .map(|r| (r.kind.as_str(), r.label.as_str()))
        .collect();
    assert_eq!(
        got,
        vec![
            ("SectionHeader", "Título"),
            ("Slider", "Opacity"),
            ("Toggle", "Visível"),
        ],
        "o desenho puro virou row, ou a ordem nao e a dos filhos"
    );
}

/// **A moldura não é a primeira row de si mesma.**
///
/// ⚠️ Vestir a moldura é legítimo (ela É um retângulo desenhado, e a pele dela é o fundo do
/// painel). O que não pode acontecer é ela aparecer também como CONTROLE — seria a árvore lida um
/// nível acima do que ela é, e o painel gerado abriria com uma linha que é ele próprio.
#[test]
fn a_dressed_frame_is_the_panel_not_its_first_row() {
    let (mut sim, frame) = frame_with_children();
    sim.world_mut().entity_mut(frame).insert(VecWidget {
        kind: WidgetKind::Card.code(),
    });
    let spec = of(&sim, frame);
    assert_eq!(spec.rows.len(), 3, "a moldura virou row de si mesma");
}

/// **Um `kind` que este build não conhece não vira row.**
///
/// ⚠️ É o canal que o `from_code` existe para suportar: um documento autorado por um build mais
/// novo. Inventar um tipo aqui seria gerar código para um widget que não existe — e o erro
/// apareceria no `cargo`, longe do desenho que o causou.
#[test]
fn an_unknown_kind_does_not_become_a_row() {
    let (mut sim, frame) = frame_with_children();
    let e = sim
        .world_mut()
        .spawn((Name("Do futuro".into()), Transform::IDENTITY, VecPathRef(9)))
        .id();
    sim.world_mut()
        .entity_mut(e)
        .insert((ChildOf(frame), VecWidget { kind: 60_000 }));
    assert_eq!(
        of(&sim, frame).rows.len(),
        3,
        "um tipo desconhecido virou row"
    );
}

/// **A chave é estável e legível** — é ela que vira `NodeId` por hash.
#[test]
fn the_key_is_a_slug_of_the_label() {
    assert_eq!(key_of("Opacity"), "opacity");
    assert_eq!(key_of("Fill / Stroke"), "fill___stroke");
    assert_eq!(
        key_of(""),
        "_",
        "um rotulo vazio ainda tem de dar uma chave"
    );
}
