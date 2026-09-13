use super::*;
use crate::draw_bands::Family;
use ph2d_ecs::{Name, RootOrder, SimWorld, Transform, VecPathRef};

/// ⭐ Uma forma vetorial ocupa um rank.
#[test]
fn a_vector_path_entity_participates_in_the_order() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Shape"),
            VecPathRef(42),
            RootOrder(0),
        ))
        .id();
    assert_eq!(vector_participant(sim.world(), e), Some(42));
}

/// ⛔ **Uma SPRITE não é uma forma**, mesmo que carregue um `VecPathRef` — aquele campo é
/// proveniência de autoria. Contá-la nas duas famílias dar-lhe-ia dois ranks.
#[test]
fn a_sprite_is_never_counted_as_a_shape_even_carrying_a_path_ref() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Image"),
            Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            VecPathRef(42),
        ))
        .id();
    assert_eq!(vector_participant(sim.world(), e), None);
}

/// Um objecto vazio não ocupa rank nenhum.
#[test]
fn a_plain_object_participates_as_neither() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Empty")))
        .id();
    assert_eq!(vector_participant(sim.world(), e), None);
}

/// ⭐⭐ **A conversão regista TODOS os ranks, não só os das formas.**
///
/// **Mutação que deve sangrar:** saltar o `record` quando `path.is_none()` — o buraco encheria
/// com `Sprite` por omissão e o gate abaixo passaria por acidente; por isso a fixtura põe uma
/// FORMA no rank alto, que o preenchimento por omissão erraria.
#[test]
fn the_conversion_records_every_rank_not_only_the_shapes() {
    let mut sim = SimWorld::new();
    let img = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Image"), RootOrder(0)))
        .id();
    let shape = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Shape"),
            VecPathRef(7),
            RootOrder(1),
        ))
        .id();
    let inputs: Vec<SortInput> = [img, shape]
        .into_iter()
        .map(|entity| SortInput {
            entity,
            world_pos: ph2d_core::Vec2::ZERO,
        })
        .collect();
    let mut scratch = SortScratch::new();
    ph2d_ecs::sort_key::compute_sort_ranks_into(&mut scratch, sim.world(), &inputs);
    let mut order = crate::draw_bands::FrameOrder::default();
    build_frame_order(&inputs, &[(shape, 7)], &scratch, &mut order);

    // ⛔ **`complete()`, e não `families` cru** — é ele que distingue «não registado» de
    // «sprite», e foi a ausência dessa distinção que deixou a 1.ª prova de mutação SOBREVIVER.
    assert_eq!(
        order.complete(),
        Some(vec![Family::Sprite, Family::Vector]),
        "a conversao perdeu um rank ou trocou uma familia: {:?}",
        order.families
    );
    assert!(order.has_vectors());
}

/// ⚠️ **Ela é IDEMPOTENTE** — o `clear` é dela, e sem ele um segundo quadro empilharia as
/// formas do primeiro e abriria faixas que não desenham nada.
#[test]
fn running_the_conversion_twice_gives_the_same_order() {
    let mut sim = SimWorld::new();
    let shape = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Shape"), VecPathRef(7)))
        .id();
    let inputs = vec![SortInput {
        entity: shape,
        world_pos: ph2d_core::Vec2::ZERO,
    }];
    let mut scratch = SortScratch::new();
    ph2d_ecs::sort_key::compute_sort_ranks_into(&mut scratch, sim.world(), &inputs);
    let mut order = crate::draw_bands::FrameOrder::default();
    build_frame_order(&inputs, &[(shape, 7)], &scratch, &mut order);
    let first = order.complete();
    build_frame_order(&inputs, &[(shape, 7)], &scratch, &mut order);
    assert_eq!(order.complete(), first);
    assert_eq!(order.vector_ranks.len(), 1, "as formas empilharam");
}
