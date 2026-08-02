//! Os gates da metade que MOSTRA e EDITA a moldura selecionada.

use super::*;
use ph2d_ecs::{Transform, VecPathRef};
use std::collections::BTreeMap;

/// Um mundo com um caminho `id` que é (ou não) moldura, e o mapa `VecPathId → entidade`.
fn world(frame: Option<bool>) -> (SimWorld, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), VecPathRef(7)))
        .id();
    if let Some(clip) = frame {
        sim.world_mut().entity_mut(e).insert(VecFrame { clip });
    }
    let mut map: VecEntityMap = BTreeMap::new();
    map.insert(7, e.to_bits());
    (sim, map, 7)
}

/// A seção só existe sobre uma moldura — e é este `None` que a esconde.
#[test]
fn a_plain_shape_is_not_a_frame() {
    let (sim, map, id) = world(None);
    assert_eq!(selected_frame_clip(&sim, &map, &[id]), None);
}

/// Sobre uma moldura, o chip mostra o que o componente diz.
#[test]
fn a_frame_reports_its_clip() {
    for clip in [false, true] {
        let (sim, map, id) = world(Some(clip));
        assert_eq!(selected_frame_clip(&sim, &map, &[id]), Some(clip));
    }
}

/// **Com dois selecionados a pergunta não tem UMA resposta**, e mostrar a do primeiro editaria o
/// que o artista não está a olhar.
#[test]
fn two_selected_paths_report_no_frame() {
    let (mut sim, mut map, id) = world(Some(true));
    let other = sim
        .world_mut()
        .spawn((Transform::default(), VecPathRef(8)))
        .id();
    map.insert(8, other.to_bits());
    assert_eq!(selected_frame_clip(&sim, &map, &[id, 8]), None);
    assert_eq!(selected_frame_clip(&sim, &map, &[]), None);
}

/// O chip escreve — e escrever o MESMO valor não muda nada (o undo é por diff; um passo por
/// clique repetido encheria a fila com estados idênticos).
#[test]
fn the_chip_writes_the_clip_and_a_no_op_changes_nothing() {
    let (mut sim, map, id) = world(Some(true));
    assert!(
        !set_selected_frame_clip(&mut sim, &map, &[id], true),
        "no-op"
    );
    assert!(set_selected_frame_clip(&mut sim, &map, &[id], false));
    assert_eq!(selected_frame_clip(&sim, &map, &[id]), Some(false));
}

/// ⚠️ **Escrever numa forma que não é moldura CRIARIA uma** — um chip de opção viraria um gesto
/// de criação, e o artista ganharia um contêiner que nunca desenhou.
#[test]
fn the_chip_never_turns_a_plain_shape_into_a_frame() {
    let (mut sim, map, id) = world(None);
    assert!(!set_selected_frame_clip(&mut sim, &map, &[id], true));
    assert_eq!(selected_frame_clip(&sim, &map, &[id]), None);
}
