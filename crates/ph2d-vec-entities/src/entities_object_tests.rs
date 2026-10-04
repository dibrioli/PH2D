//! Gates do OBJECTO VETORIAL (spec/06 F3 ▸ Vector): a regra das soltas e a fronteira do objecto.

use super::super::{bits, group_entities, setup, sync, ungroup_entities};
use super::*;
use crate::transform::world_transform;
use ph2d_vec_scene::rectangle;

fn parent(sim: &SimWorld, e: Entity) -> Option<Entity> {
    sim.world().get::<ChildOf>(e).map(ChildOf::parent)
}

fn is_object(sim: &SimWorld, e: Entity) -> bool {
    sim.world().get::<VecObject>(e).is_some()
}

/// ⭐⭐ GATE — **cada forma solta ganha o SEU objecto** (escolha do dono, 04/10), no sítio dela: o
/// mundo não se mexe e o objecto toma-lhe a ordem de raiz.
#[test]
fn every_loose_shape_gets_its_own_object_in_its_place() {
    let (mut sim, mut scene, mut map) = setup();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([5.0, 0.0], [6.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    let (ea, eb) = (bits(&map, a), bits(&map, b));
    sim.world_mut()
        .get_mut::<Transform>(ea)
        .unwrap()
        .translation = ph2d_core::Vec2::new(3.0, 4.0);
    let before = (world_transform(&sim, ea), world_transform(&sim, eb));
    let order_a = *sim.world().get::<RootOrder>(ea).unwrap();
    assert_eq!(adopt_loose(&mut sim, &map, None, &[], "Vector"), 2);
    let (oa, ob) = (parent(&sim, ea).unwrap(), parent(&sim, eb).unwrap());
    assert!(is_object(&sim, oa) && is_object(&sim, ob) && oa != ob);
    assert_eq!(sim.world().get::<RootOrder>(oa), Some(&order_a));
    assert!(sim.world().get::<RootOrder>(ea).is_none());
    assert_eq!(
        (world_transform(&sim, ea), world_transform(&sim, eb)),
        before
    );
    assert_eq!(
        adopt_loose(&mut sim, &map, None, &[], "Vector"),
        0,
        "idempotente"
    );
}

/// ⭐⭐ GATE — **com um Edit aberto, a forma nova entra no objecto do Edit**, sem se mexer.
#[test]
fn with_an_edit_open_the_new_shape_enters_its_object() {
    let (mut sim, mut scene, mut map) = setup();
    let o = spawn_object(&mut sim, "Vector", ph2d_core::Vec2::new(10.0, -2.0));
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    let ea = bits(&map, a);
    let before = world_transform(&sim, ea);
    assert_eq!(adopt_loose(&mut sim, &map, Some(o), &[], "Vector"), 1);
    assert_eq!(parent(&sim, ea), Some(o));
    assert!(sim.world().get::<RootOrder>(ea).is_none());
    assert_eq!(world_transform(&sim, ea), before);
    assert_eq!(object_of(&sim, ea), Some(o));
}

/// ⭐ GATE — **a forma que nasceu ATRÁS do objecto entra no FUNDO dele** (o balde vai para o fundo
/// do que preenche); as outras entram à frente.
#[test]
fn a_shape_born_behind_the_object_enters_at_its_back() {
    let (mut sim, mut scene, mut map) = setup();
    let o = spawn_object(&mut sim, "Vector", ph2d_core::Vec2::ZERO);
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    adopt_loose(&mut sim, &map, Some(o), &[], "Vector");
    ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
    let fill = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    let ef = bits(&map, fill);
    let below = sim.world().get::<RootOrder>(o).unwrap().0;
    sim.world_mut()
        .entity_mut(ef)
        .insert(RootOrder(below.saturating_sub(1)));
    sim.world_mut()
        .entity_mut(o)
        .insert(RootOrder(below.max(1)));
    adopt_loose(&mut sim, &map, Some(o), &[], "Vector");
    ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
    let ea = bits(&map, a);
    let order = |e| {
        sim.world()
            .get::<SiblingOrder>(e)
            .expect("ordem entre irmãs")
            .0
    };
    assert!(
        order(ef) < order(ea),
        "o preenchimento não foi para o fundo"
    );
}

/// ⭐⭐ GATE — **a forma EM GESTO espera**: a mão escreve-a em MUNDO a cada quadro, e prendê-la a um
/// pai no meio do traço deslocá-la-ia de baixo do cursor.
#[test]
fn a_shape_in_gesture_waits() {
    let (mut sim, mut scene, mut map) = setup();
    let o = spawn_object(&mut sim, "Vector", ph2d_core::Vec2::new(4.0, 4.0));
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    assert_eq!(adopt_loose(&mut sim, &map, Some(o), &[a], "Vector"), 0);
    assert_eq!(parent(&sim, bits(&map, a)), None);
    assert_eq!(adopt_loose(&mut sim, &map, Some(o), &[], "Vector"), 1);
}

/// ⭐ GATE — **uma moldura leva os filhos**: a cabeça é a forma mais alta da cadeia, e os filhos
/// continuam filhos dela dentro do objecto.
#[test]
fn a_frame_and_its_children_are_one_head() {
    let (mut sim, mut scene, mut map) = setup();
    let frame = scene.push_path(rectangle([0.0, 0.0], [9.0, 9.0]));
    let kid = scene.push_path(rectangle([1.0, 1.0], [2.0, 2.0]));
    sync(&mut sim, &mut scene, &mut map);
    let (ef, ek) = (bits(&map, frame), bits(&map, kid));
    sim.world_mut()
        .entity_mut(ek)
        .remove::<RootOrder>()
        .insert(ChildOf(ef));
    assert_eq!(adopt_loose(&mut sim, &map, None, &[], "Vector"), 1);
    assert_eq!(parent(&sim, ek), Some(ef));
    assert!(is_object(&sim, parent(&sim, ef).unwrap()));
}

/// ⭐ GATE — **um grupo SÓ de formas entra inteiro num objecto** (um envelope, uma booleana viva, o
/// grupo de um projecto antigo): a cabeça sobe por todo pai que só tem vetor, e o grupo fica.
#[test]
fn a_group_of_shapes_enters_one_object_whole() {
    let (mut sim, mut scene, mut map) = setup();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([3.0, 0.0], [4.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    let (ea, eb) = (bits(&map, a), bits(&map, b));
    let g = Entity::from_bits(
        group_entities(&mut sim, &[ea.to_bits(), eb.to_bits()], "G".into()).unwrap(),
    );
    let before = world_transform(&sim, ea);
    assert_eq!(adopt_loose(&mut sim, &map, None, &[], "Vector"), 1);
    assert_eq!((parent(&sim, ea), parent(&sim, eb)), (Some(g), Some(g)));
    assert!(is_object(&sim, parent(&sim, g).unwrap()));
    assert_eq!(world_transform(&sim, ea), before);
}

/// ⭐ GATE — **uma forma debaixo de um pai de OUTRA família é embrulhada NO LUGAR**: o objecto novo
/// fica no pai, no sítio dela entre as irmãs — o sprite não entra num objecto vetorial.
#[test]
fn a_shape_under_a_foreign_parent_is_wrapped_in_place() {
    let (mut sim, mut scene, mut map) = setup();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    let ea = bits(&map, a);
    let sprite = sim
        .world_mut()
        .spawn((
            Transform::default(),
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ))
        .id();
    sim.world_mut()
        .entity_mut(ea)
        .remove::<RootOrder>()
        .insert((ChildOf(sprite), SiblingOrder(3)));
    let before = world_transform(&sim, ea);
    adopt_loose(&mut sim, &map, None, &[], "Vector");
    let oa = parent(&sim, ea).unwrap();
    assert!(is_object(&sim, oa));
    assert_eq!(parent(&sim, oa), Some(sprite));
    assert_eq!(sim.world().get::<SiblingOrder>(oa), Some(&SiblingOrder(3)));
    assert_eq!(world_transform(&sim, ea), before);
}

/// Um objecto com duas formas dentro. Devolve `(sim, map, objecto, [a, b])`.
fn object_with_two() -> (SimWorld, VecEntityMap, Entity, [Entity; 2]) {
    let (mut sim, mut scene, mut map) = setup();
    let o = spawn_object(&mut sim, "Vector", ph2d_core::Vec2::new(1.0, 1.0));
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([3.0, 0.0], [4.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    adopt_loose(&mut sim, &map, Some(o), &[], "Vector");
    let (ea, eb) = (bits(&map, a), bits(&map, b));
    (sim, map, o, [ea, eb])
}

/// ⭐⭐ GATE — **agrupar DENTRO do objecto fica dentro** (é o que a booleana viva faz): o topo de
/// uma forma nunca é o próprio objecto — senão os dois membros davam um topo só e o verbo recusava.
#[test]
fn grouping_inside_an_object_stays_inside_it() {
    let (mut sim, _map, o, [ea, eb]) = object_with_two();
    let before = (world_transform(&sim, ea), world_transform(&sim, eb));
    let g = group_entities(&mut sim, &[ea.to_bits(), eb.to_bits()], "Boolean".into())
        .map(Entity::from_bits)
        .expect("o grupo dentro do objecto recusou");
    assert_eq!(parent(&sim, g), Some(o));
    assert_eq!((parent(&sim, ea), parent(&sim, eb)), (Some(g), Some(g)));
    assert_eq!(
        (world_transform(&sim, ea), world_transform(&sim, eb)),
        before
    );
    assert_eq!(object_of(&sim, ea), Some(o));
}

/// ⭐⭐ GATE — **desagrupar nunca dissolve o objecto**, e um grupo dentro dele devolve as formas
/// ao objecto.
#[test]
fn ungrouping_never_dissolves_the_object() {
    let (mut sim, _map, o, [ea, eb]) = object_with_two();
    assert_eq!(ungroup_entities(&mut sim, &[o.to_bits()]), 0);
    assert_eq!(ungroup_entities(&mut sim, &[ea.to_bits()]), 0);
    assert!(sim.world().get_entity(o).is_ok());
    let g = group_entities(&mut sim, &[ea.to_bits(), eb.to_bits()], "G".into()).unwrap();
    assert_eq!(ungroup_entities(&mut sim, &[ea.to_bits()]), 1);
    assert!(sim.world().get_entity(Entity::from_bits(g)).is_err());
    assert_eq!((parent(&sim, ea), parent(&sim, eb)), (Some(o), Some(o)));
}

/// ⭐ GATE — **o clique numa forma do objecto nomeia a FORMA**, não o objecto (as formas
/// escolhem-se uma a uma no Edit); e fora de objectos o grupo continua a subir.
#[test]
fn the_click_stops_at_the_object_boundary() {
    let (sim, _map, o, [ea, _]) = object_with_two();
    assert_eq!(super::super::selection::selection_root(&sim, ea), ea);
    assert_eq!(top_within_object(&sim, ea), ea);
    assert_eq!(top_within_object(&sim, o), o);
}

/// ⭐⭐ GATE — **a moldura com um filho EM GESTO espera inteira**: a cabeça sobe do filho à
/// moldura, e levá-la para o objecto a meio do traço mudaria o pai de quem a mão está a escrever.
#[test]
fn a_head_with_a_shape_in_gesture_under_it_waits() {
    let (mut sim, mut scene, mut map) = setup();
    let o = spawn_object(&mut sim, "Vector", ph2d_core::Vec2::new(4.0, 4.0));
    let frame = scene.push_path(rectangle([0.0, 0.0], [9.0, 9.0]));
    let kid = scene.push_path(rectangle([1.0, 1.0], [2.0, 2.0]));
    sync(&mut sim, &mut scene, &mut map);
    let (ef, ek) = (bits(&map, frame), bits(&map, kid));
    sim.world_mut()
        .entity_mut(ek)
        .remove::<RootOrder>()
        .insert(ChildOf(ef));
    assert_eq!(adopt_loose(&mut sim, &map, Some(o), &[kid], "Vector"), 0);
    assert_eq!(
        parent(&sim, ef),
        None,
        "a moldura saiu a meio do traço do filho"
    );
    assert_eq!(adopt_loose(&mut sim, &map, Some(o), &[], "Vector"), 1);
}
