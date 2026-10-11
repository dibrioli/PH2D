//! Testes de [`super`] — as alças no MUNDO: seguem a pose, o press agarra só sobre elas, o arrasto
//! escreve pela porta única e re-cozinha no lugar, o `Esc` repõe e o duplo-clique volta à fábrica.

use super::*;
use crate::shape_live::recook_shape;
use ph2d_ecs::Transform;

const PX: f64 = 0.01;

/// Uma forma viva no mundo com POSE (translação + rotação) — a mesma montagem dos testes de
/// `shape_params`.
fn live(
    kind: ShapeKind,
    values: ShapeValues,
    pose: [f32; 2],
    rot: f32,
) -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let shape = VecShape::Param {
        kind: kind.as_u16(),
        w: 4.0,
        h: 2.0,
        values,
    };
    let id = scene.push_path(recook_shape(&shape).expect("cozinha"));
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let entity = Entity::from_bits(*map.get(&id).expect("entidade"));
    if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
        if let Some(mut t) = e.get_mut::<Transform>() {
            t.translation = ph2d_core::Vec2::new(pose[0], pose[1]);
            t.rotation = rot;
        }
        e.insert(shape);
    }
    (sim, scene, map, id)
}

fn values_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> ShapeValues {
    panel_shape_target(sim, map, &[id]).expect("alvo").3
}

/// As alças vivem em MUNDO e seguem a pose: rodar e mover a forma leva-as junto, à mesma
/// distância do centro.
#[test]
fn the_handles_follow_the_pose_of_the_shape() {
    let k = ShapeKind::Star;
    let (sim0, _, map0, id0) = live(k, k.defaults(), [0.0, 0.0], 0.0);
    let (sim1, _, map1, id1) = live(k, k.defaults(), [5.0, -3.0], 0.7);
    let a = view(&sim0, &map0, &[id0], PX, None, None).expect("alças");
    let b = view(&sim1, &map1, &[id1], PX, None, None).expect("alças");
    assert_eq!(a.points.len(), law::handles_of(k).len());
    let (c, s) = (0.7_f64.cos(), 0.7_f64.sin());
    for (p, q) in a.points.iter().zip(&b.points) {
        let want = [5.0 + p[0] * c - p[1] * s, -3.0 + p[0] * s + p[1] * c];
        assert!(
            (q[0] - want[0]).hypot(q[1] - want[1]) < 1e-5,
            "{q:?} contra {want:?}"
        );
    }
}

/// O press só agarra SOBRE uma alça; ao lado dela, o clique segue para a caixa.
#[test]
fn a_press_grabs_only_on_a_handle() {
    let k = ShapeKind::Polygon;
    let (sim, _, map, id) = live(k, k.defaults(), [1.0, 1.0], 0.3);
    let v = view(&sim, &map, &[id], PX, None, None).unwrap();
    for (n, p) in v.points.iter().enumerate() {
        let g = press(&sim, &map, &[id], *p, PX).expect("agarra");
        assert_eq!(g.handle(), n);
        assert!(press(&sim, &map, &[id], [p[0] + 3.0 * HIT_PX * PX, p[1]], PX).is_none());
    }
    assert!(
        press(&sim, &map, &[], v.points[0], PX).is_none(),
        "sem seleção não há alças"
    );
}

/// ⭐ O arrasto escreve pela porta única: o raio interior da estrela muda, a forma re-cozinha no
/// lugar (o mesmo id), o `Esc` repõe o do press e o duplo-clique repõe o de fábrica.
#[test]
fn a_drag_writes_through_the_one_door_and_esc_and_double_click_restore() {
    let k = ShapeKind::Star;
    let mut v0 = k.defaults();
    v0[1] = 0.3;
    let (mut sim, mut scene, map, id) = live(k, v0, [2.0, 0.0], 0.0);
    let inner = law::handles_of(k)
        .iter()
        .position(|h| h.law == law::Law::Radial)
        .unwrap();
    let p = view(&sim, &map, &[id], PX, None, None).unwrap().points[inner];
    let mut g = press(&sim, &map, &[id], p, PX).unwrap();
    assert_eq!(g.handle(), inner);
    // Para fora, ao longo do raio: o raio normalizado sobe.
    let c = [2.0, 0.0];
    let out = [c[0] + (p[0] - c[0]) * 2.0, c[1] + (p[1] - c[1]) * 2.0];
    let before = scene
        .paths()
        .iter()
        .find(|q| q.id == id)
        .unwrap()
        .verts
        .clone();
    assert!(drag(
        &mut sim,
        &mut scene,
        &map,
        &[id],
        &mut g,
        out,
        Mods::default(),
        PX
    ));
    let after = values_of(&sim, &map, id);
    assert!(after[1] > v0[1] + 0.05, "o vale não andou: {}", after[1]);
    let now = scene
        .paths()
        .iter()
        .find(|q| q.id == id)
        .unwrap()
        .verts
        .clone();
    assert_ne!(before, now, "a geometria não re-cozinhou");
    assert!(cancel(&mut sim, &mut scene, &map, &[id], &g));
    assert_eq!(values_of(&sim, &map, id)[1], v0[1], "o Esc não repôs");
    assert!(reset_at(&mut sim, &mut scene, &map, &[id], p, PX));
    assert_eq!(
        values_of(&sim, &map, id)[1],
        k.defaults()[1],
        "o duplo-clique não repôs"
    );
}

/// Um arrasto cuja forma deixou de ser o alvo (outra seleção) não escreve noutra.
#[test]
fn a_drag_whose_shape_left_the_selection_writes_nothing() {
    let k = ShapeKind::Star;
    let (mut sim, mut scene, map, id) = live(k, k.defaults(), [0.0, 0.0], 0.0);
    let p = view(&sim, &map, &[id], PX, None, None).unwrap().points[0];
    let mut g = press(&sim, &map, &[id], p, PX).unwrap();
    assert!(!drag(
        &mut sim,
        &mut scene,
        &map,
        &[],
        &mut g,
        [0.0, 0.0],
        Mods::default(),
        PX
    ));
}

/// A guia: com o cursor sobre uma alça de trilho, a vista traz o trilho dela em MUNDO.
#[test]
fn hovering_a_handle_shows_its_track() {
    let k = ShapeKind::RoundRect;
    let (sim, _, map, id) = live(k, k.defaults(), [0.0, 0.0], 0.0);
    let v = view(&sim, &map, &[id], PX, None, None).unwrap();
    assert!(v.hot.is_none() && v.track.is_empty());
    let h = view(&sim, &map, &[id], PX, None, Some(v.points[0])).unwrap();
    assert_eq!(h.hot, Some(0));
    assert!(h.track.len() >= 2, "{:?}", h.track);
}

/// Agarrar uma alça ESTACIONADA (raio zero, a `PARK_PX` do canto) e não mexer não arredonda nada;
/// mexer 5 px para dentro dá 5 px de raio — a pega mede-se ao ponto verdadeiro.
#[test]
fn a_parked_handle_grabbed_and_held_still_changes_nothing() {
    let k = ShapeKind::RoundRect;
    let mut v0 = k.defaults();
    v0[0] = 0.0;
    let (mut sim, mut scene, map, id) = live(k, v0, [0.0, 0.0], 0.0);
    let p = view(&sim, &map, &[id], PX, None, None).unwrap().points[0];
    // A alça do canto de cima à esquerda (−2, 1) estacionou PARK_PX para a direita.
    assert!(
        (p[0] - (-2.0 + PARK_PX * PX)).abs() < 1e-9 && (p[1] - 1.0).abs() < 1e-9,
        "{p:?}"
    );
    let mut g = press(&sim, &map, &[id], p, PX).unwrap();
    assert!(drag(
        &mut sim,
        &mut scene,
        &map,
        &[id],
        &mut g,
        p,
        Mods::default(),
        PX
    ));
    assert!(
        values_of(&sim, &map, id)[0].abs() < 1e-9,
        "segurar parado arredondou"
    );
    let q = [p[0] + 5.0 * PX, p[1]];
    assert!(drag(
        &mut sim,
        &mut scene,
        &map,
        &[id],
        &mut g,
        q,
        Mods::default(),
        PX
    ));
    assert!((values_of(&sim, &map, id)[0] - 5.0 * PX).abs() < 1e-6);
}

/// ⭐ O DUPLO-CLIQUE é o 2.º press na MESMA alça dentro da janela do despacho: repõe a fábrica e
/// não deixa arrasto vivo. Fora da janela — ou noutra alça — é só outro agarrar.
#[test]
fn a_second_press_on_the_same_handle_inside_the_window_resets_it() {
    use ph2d_editor_core::interaction::DOUBLE_CLICK_WINDOW_NS as W;
    let k = ShapeKind::Star;
    let mut v0 = k.defaults();
    v0[1] = 0.3;
    let (mut sim, mut scene, map, id) = live(k, v0, [0.0, 0.0], 0.0);
    let inner = law::handles_of(k)
        .iter()
        .position(|h| h.law == law::Law::Radial)
        .unwrap();
    let p = view(&sim, &map, &[id], PX, None, None).unwrap().points[inner];
    let mut input = ShapeHandleInput::default();
    let sel = [id];
    // Dois presses LONGE no tempo: dois agarrares, nada reposto.
    assert!(input.press(w(&mut sim, &mut scene, &map, &sel), p, PX, 0));
    assert!(input.release());
    assert!(input.press(w(&mut sim, &mut scene, &map, &sel), p, PX, W + 1));
    assert!(input.grab.is_some());
    assert_eq!(values_of(&sim, &map, id)[1], 0.3);
    assert!(input.release());
    // O 2.º DENTRO da janela: repõe, e o arrasto não fica vivo.
    assert!(input.press(w(&mut sim, &mut scene, &map, &sel), p, PX, 2 * W + 10));
    assert!(input.release());
    assert!(input.press(
        w(&mut sim, &mut scene, &map, &sel),
        p,
        PX,
        2 * W + 10 + W / 2
    ));
    assert!(
        input.grab.is_none(),
        "o duplo-clique deixou um arrasto vivo"
    );
    assert_eq!(
        values_of(&sim, &map, id)[1],
        k.defaults()[1],
        "o duplo-clique não repôs"
    );
}

fn w<'a>(
    sim: &'a mut SimWorld,
    scene: &'a mut VecScene,
    map: &'a VecEntityMap,
    selection: &'a [VecPathId],
) -> World<'a> {
    World {
        sim,
        scene,
        map,
        selection,
    }
}
