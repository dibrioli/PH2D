//! Testes de [`super`] — as alças no MUNDO: seguem a pose, o press agarra só sobre elas, o arrasto
//! escreve pela porta única e re-cozinha no lugar, o `Esc` repõe e o duplo-clique volta à fábrica.

use super::*;
use crate::shape_live::recook_shape;
use crate::shape_params::panel_shape_target;
use ph2d_ecs::Transform;

const PX: f64 = 0.01;
/// O portão das alças: o modo Forma, desarmado (F10).
const G: Gate = Gate::SHAPE;

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
    let a = view(&sim0, &map0, &[id0], G, PX, None, None).expect("alças");
    let b = view(&sim1, &map1, &[id1], G, PX, None, None).expect("alças");
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
    let v = view(&sim, &map, &[id], G, PX, None, None).unwrap();
    for (n, p) in v.points.iter().enumerate() {
        let g = press(&sim, &map, &[id], G, *p, PX).expect("agarra");
        assert_eq!(g.handle(), n);
        assert!(press(&sim, &map, &[id], G, [p[0] + 3.0 * HIT_PX * PX, p[1]], PX).is_none());
    }
    assert!(
        press(&sim, &map, &[], G, v.points[0], PX).is_none(),
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
        .position(|h| h.fields.first() == Some(&1))
        .unwrap();
    let p = view(&sim, &map, &[id], G, PX, None, None).unwrap().points[inner];
    let mut g = press(&sim, &map, &[id], G, p, PX).unwrap();
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
        w(&mut sim, &mut scene, &map, &[id]),
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
    assert!(cancel(&mut sim, &mut scene, &map, &[id], G, &g));
    assert_eq!(values_of(&sim, &map, id)[1], v0[1], "o Esc não repôs");
    assert!(reset_at(w(&mut sim, &mut scene, &map, &[id]), p, PX));
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
    let p = view(&sim, &map, &[id], G, PX, None, None).unwrap().points[0];
    let mut g = press(&sim, &map, &[id], G, p, PX).unwrap();
    assert!(!drag(
        w(&mut sim, &mut scene, &map, &[]),
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
    let v = view(&sim, &map, &[id], G, PX, None, None).unwrap();
    assert!(v.hot.is_none() && v.track.is_empty());
    let h = view(&sim, &map, &[id], G, PX, None, Some(v.points[0])).unwrap();
    assert_eq!(h.hot, Some(0));
    assert!(h.track.len() >= 2, "{:?}", h.track);
}

/// Agarrar o ⊙ ESTACIONADO (raio zero, `PARK_PX` para dentro do canto em cada eixo) e não mexer
/// não arredonda nada; mexer 5 px para dentro — na horizontal ou na DIAGONAL, como o centro do arco
/// anda — dá 5 px de raio: a pega mede-se ao ponto verdadeiro.
#[test]
fn a_parked_handle_grabbed_and_held_still_changes_nothing() {
    let k = ShapeKind::RoundRect;
    let mut v0 = k.defaults();
    v0[0] = 0.0;
    let (mut sim, mut scene, map, id) = live(k, v0, [0.0, 0.0], 0.0);
    let p = view(&sim, &map, &[id], G, PX, None, None).unwrap().points[0];
    // O ⊙ do canto de cima à esquerda (−2, 1) estacionou PARK_PX para dentro, na diagonal.
    assert!(
        (p[0] - (-2.0 + PARK_PX * PX)).abs() < 1e-9 && (p[1] - (1.0 - PARK_PX * PX)).abs() < 1e-9,
        "{p:?}"
    );
    let mut d = press(&sim, &map, &[id], G, p, PX).unwrap();
    assert!(drag(
        w(&mut sim, &mut scene, &map, &[id]),
        &mut d,
        [p[0] + 5.0 * PX, p[1] - 5.0 * PX],
        Mods::default(),
        PX
    ));
    assert!(
        (values_of(&sim, &map, id)[0] - 5.0 * PX).abs() < 1e-6,
        "a mão na diagonal"
    );
    assert!(drag(
        w(&mut sim, &mut scene, &map, &[id]),
        &mut d,
        p,
        Mods::default(),
        PX
    ));
    let mut g = press(&sim, &map, &[id], G, p, PX).unwrap();
    assert!(drag(
        w(&mut sim, &mut scene, &map, &[id]),
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
        w(&mut sim, &mut scene, &map, &[id]),
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
        .position(|h| h.fields.first() == Some(&1))
        .unwrap();
    let p = view(&sim, &map, &[id], G, PX, None, None).unwrap().points[inner];
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
        gate: G,
    }
}

/// O `Esc` pela PORTA de entrada (a que a shell chama): a meio de um arrasto, os campos voltam aos
/// do press e o arrasto acaba; sem arrasto, o `Esc` não é consumido.
#[test]
fn esc_through_the_input_door_restores_the_press_and_ends_the_drag() {
    let k = ShapeKind::Star;
    let mut v0 = k.defaults();
    v0[1] = 0.3;
    let (mut sim, mut scene, map, id) = live(k, v0, [0.0, 0.0], 0.0);
    let inner = law::handles_of(k)
        .iter()
        .position(|h| h.fields.first() == Some(&1))
        .unwrap();
    let p = view(&sim, &map, &[id], G, PX, None, None).unwrap().points[inner];
    let sel = [id];
    let mut input = ShapeHandleInput::default();
    assert!(
        !input.escape(w(&mut sim, &mut scene, &map, &sel)),
        "Esc sem arrasto foi consumido"
    );
    assert!(input.press(w(&mut sim, &mut scene, &map, &sel), p, PX, 0));
    let out = [p[0] * 2.0, p[1] * 2.0];
    assert!(input.drag(
        w(&mut sim, &mut scene, &map, &sel),
        out,
        Mods::default(),
        PX
    ));
    assert!(
        values_of(&sim, &map, id)[1] > 0.35,
        "o arrasto não escreveu"
    );
    assert!(input.escape(w(&mut sim, &mut scene, &map, &sel)));
    assert_eq!(values_of(&sim, &map, id)[1], 0.3, "o Esc não repôs o press");
    assert!(input.grab.is_none(), "o Esc deixou o arrasto vivo");
}

/// ⭐ (F11) **As caras do Illustrator e o círculo do raio** — a estrela mostra ⇕ ⊙ ○ ⊙ (o R2 e os
/// dois cantos, sem a envergadura), e o círculo tracejado só existe DURANTE o arrasto do raio,
/// pelo ponto agarrado (a elipse da caixa, normalizada: `w = 4`, `h = 2`).
#[test]
fn a_star_wears_the_illustrator_faces_and_its_radius_drag_draws_the_ring() {
    use ph2d_vec_render::HandleFace::{Corner, Count, Radius};
    let k = ShapeKind::Star;
    let (sim, _, map, id) = live(k, k.defaults(), [0.0, 0.0], 0.0);
    let v = view(&sim, &map, &[id], G, PX, None, None).unwrap();
    assert_eq!(v.faces, [Count, Corner, Radius, Corner]);
    assert!(v.ring.is_empty(), "sem arrasto não há círculo");
    assert!(
        v.stems.iter().all(Option::is_none),
        "a estrela não tem haste"
    );
    let inner = 2;
    let p = v.points[inner];
    let k0 = (p[0] / 2.0).hypot(p[1] / 1.0);
    let g = view(&sim, &map, &[id], G, PX, Some(inner), None).unwrap();
    assert!(g.ring.len() > 32);
    for q in &g.ring {
        assert!(((q[0] / 2.0).hypot(q[1] / 1.0) - k0).abs() < 1e-9, "{q:?}");
    }
}

/// ⭐ (F11) **A pizza: os ângulos na HASTE, por fora do aro** — a raiz da haste é o ponto do aro e
/// a alça fica `PARK_PX` além dele.
#[test]
fn the_pie_angles_ride_a_stem_outside_the_rim() {
    let k = ShapeKind::Pie;
    let (sim, _, map, id) = live(k, k.defaults(), [0.0, 0.0], 0.0);
    let v = view(&sim, &map, &[id], G, PX, None, None).unwrap();
    for (p, s) in v.points.iter().zip(&v.stems) {
        let root = s.expect("haste");
        let (rp, rr) = ((p[0] / 2.0).hypot(p[1]), (root[0] / 2.0).hypot(root[1]));
        assert!((rr - 1.0).abs() < 1e-6, "a raiz no aro: {root:?}");
        assert!(rp > rr, "a alça por fora do aro: {p:?}");
    }
}

/// ⭐ (F11) **A etiqueta segue a alça agarrada** — nenhuma sem arrasto; com o ⇕ agarrado, o nome
/// traduzido das pontas e o número.
#[test]
fn the_readout_follows_the_grabbed_handle() {
    let k = ShapeKind::Star;
    let (sim, _, map, id) = live(k, k.defaults(), [0.0, 0.0], 0.0);
    let mut vec = crate::state::VecState {
        entities: map.clone(),
        ..Default::default()
    };
    vec.pen.select_many(&[id]);
    vec.draw_config.mode = DrawMode::Shape;
    let display = ph2d_editor_core::LengthDisplay::of(&Default::default());
    assert_eq!(
        readout(&vec, &sim, PX, display),
        None,
        "sem arrasto, sem etiqueta"
    );
    let p = view(&sim, &map, &[id], Gate::of(&vec), PX, None, None)
        .unwrap()
        .points[0];
    vec.shape_handles.grab = press(&sim, &map, &[id], Gate::of(&vec), p, PX);
    let want = format!("{}: 5", ph2d_panel_vector::nomes_do_motor::campo("Points"));
    assert_eq!(readout(&vec, &sim, PX, display), Some(want));
}
