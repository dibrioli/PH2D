//! Os gates da VISTA do gizmo vetorial (`anchor_half` / `view` / `container_view`) — módulo
//! irmão de [`super`], teto de LOC (HR-18). Segue o idioma que o arquivo já usava para os testes
//! de hit (`vec_gizmo_view_hit_tests.rs`): o produto num arquivo, os gates noutro.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_vec_scene::{line, rectangle};

/// REGRESSÃO (Enio 2026-07-09: "line e arc não podem ser transformadas com o
/// gizmo"). Uma forma ABERTA não tem interior — sem raio de traço ela nunca é
/// pega, e o gizmo de Select nunca a agarra. Com raio, o clique no traço pega.
#[test]
fn an_open_line_is_picked_by_stroke_proximity_not_interior() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(line([0.0, 0.0], [10.0, 0.0]));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    let vs = VecViewState::default();
    // Um clique 0.4 ACIMA da linha (fora do traço): sem raio não pega — uma linha
    // aberta não tem interior.
    assert_eq!(
        pick_at_world(&sim, &scene, &vs, &map, [5.0, 0.4], 0.0),
        None
    );
    // Com raio 1.0 (> 0.4): pega pela proximidade do traço.
    assert_eq!(
        pick_at_world(&sim, &scene, &vs, &map, [5.0, 0.4], 1.0),
        Some(e.to_bits())
    );
    // Longe do traço, mesmo com raio: não pega.
    assert_eq!(
        pick_at_world(&sim, &scene, &vs, &map, [5.0, 5.0], 1.0),
        None
    );
}

fn scene_with_square() -> (SimWorld, VecScene, VecEntityMap, Entity) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(rectangle([-1.0, -1.0], [1.0, 1.0]));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    (sim, scene, map, e)
}

/// O gizmo lê a forma como um sprite: `anchor` = centro da bbox local,
/// `half` = meia-extensão dela. Um quadrado [-1,1]² centrado na origem.
#[test]
fn a_path_reports_its_local_bbox_as_a_sprite_anchor_and_half() {
    let (sim, scene, _, e) = scene_with_square();
    let (anchor, half) = anchor_half(&sim, &scene, e).unwrap();
    assert_eq!(anchor, [0.0, 0.0]);
    assert_eq!(half, [1.0, 1.0]);
}

/// A `GizmoView` acompanha o `Transform`: transladar move a caixa, escalar a
/// cresce, e o pivô continua sendo a origem da entidade.
#[test]
fn the_gizmo_box_follows_the_transform_of_the_entity() {
    let (mut sim, scene, _, e) = scene_with_square();
    let cam = Camera2d::default();
    let ws = WindowSize {
        width: 800,
        height: 600,
    };
    sim.world_mut().entity_mut(e).insert(Transform {
        translation: Vec2::new(10.0, 5.0),
        scale: Vec2::new(3.0, 3.0),
        ..Transform::IDENTITY
    });
    let v = view(&sim, &scene, e, &cam, ws, (0.0, 0.0), false).unwrap();
    assert_eq!(v.pivot_world, [10.0, 5.0]);
    assert_eq!(v.bbox_min_world, [7.0, 2.0], "10±3, 5±3");
    assert_eq!(v.bbox_max_world, [13.0, 8.0]);
}

/// **O SPINE de um Blend não publica gizmo** (ADR-0128, Enio 2026-07-15) — como o conector. A
/// linha é editável só no modo Node; no Select o que se move são as formas-fonte. Uma forma
/// normal publica; a MESMA forma com um `VecBlend` (é um spine) não.
#[test]
fn a_blend_spine_publishes_no_gizmo() {
    let (mut sim, scene, _, e) = scene_with_square();
    let cam = Camera2d::default();
    let ws = WindowSize {
        width: 800,
        height: 600,
    };
    // Sem o componente, a forma tem gizmo.
    assert!(view(&sim, &scene, e, &cam, ws, (0.0, 0.0), false).is_some());
    // Com o `VecBlend` (a entidade é um spine de blend), NÃO tem.
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_ecs::VecBlend::new(vec![1, 2], 3));
    assert!(
        view(&sim, &scene, e, &cam, ws, (0.0, 0.0), false).is_none(),
        "o spine do blend não tem gizmo (a linha é Node-only)"
    );
}

/// **O container de um Envelope publica um gizmo = UNIÃO dos filhos** (ADR-0129 Fatia 3) — a
/// caixa que o gizmo de sprite arrasta para mover o envelope inteiro (Fatia 2). Um grupo comum
/// (sem `VecEnvelope`) não publica caixa por esta porta.
#[test]
fn an_envelope_container_publishes_a_union_gizmo_box() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let a = scene.push_path(rectangle([-4.0, -1.0], [-2.0, 1.0]));
    let b = scene.push_path(rectangle([2.0, -1.0], [4.0, 1.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let container = crate::envelope_live::create(&mut sim, &mut scene, &map, &[a, b]).unwrap();
    let ce = Entity::from_bits(container);
    let cam = Camera2d::default();
    let ws = WindowSize {
        width: 800,
        height: 600,
    };

    let v = container_view(&sim, &scene, ce, &cam, ws, (0.0, 0.0), false)
        .expect("o container devia publicar um gizmo");
    // A caixa abrange de x≈-4 a x≈4 — a UNIÃO das duas formas, não uma só.
    assert!(
        v.bbox_min_world[0] <= -3.9 && v.bbox_max_world[0] >= 3.9,
        "a caixa devia unir as duas formas: {:?}..{:?}",
        v.bbox_min_world,
        v.bbox_max_world
    );

    // Um grupo COMUM (sem VecEnvelope) NÃO publica caixa por esta porta.
    let plain = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::Name::new("G")))
        .id();
    assert!(container_view(&sim, &scene, plain, &cam, ws, (0.0, 0.0), false).is_none());
}

/// O picking respeita o `Transform`: o interior está onde a forma é DESENHADA,
/// não onde ela é guardada.
#[test]
fn picking_finds_the_shape_where_the_transform_puts_it() {
    let (mut sim, scene, map, e) = scene_with_square();
    let vs = VecViewState::default();
    assert!(pick_at_world(&sim, &scene, &vs, &map, [0.0, 0.0], 0.0).is_some());

    sim.world_mut().entity_mut(e).insert(Transform {
        translation: Vec2::new(50.0, 0.0),
        ..Transform::IDENTITY
    });
    assert_eq!(
        pick_at_world(&sim, &scene, &vs, &map, [0.0, 0.0], 0.0),
        None,
        "a origem ficou vazia"
    );
    assert_eq!(
        pick_at_world(&sim, &scene, &vs, &map, [50.0, 0.0], 0.0),
        Some(e.to_bits()),
        "a forma está onde o transform a pôs"
    );
}

/// Travada ou escondida não é selecionável no canvas — como um sprite.
#[test]
fn a_hidden_or_locked_shape_is_not_pickable() {
    let (sim, scene, map, _) = scene_with_square();
    let id = scene.paths()[0].id;
    let hidden = VecViewState {
        hidden: vec![id],
        locked: Vec::new(),
    };
    assert_eq!(
        pick_at_world(&sim, &scene, &hidden, &map, [0.0, 0.0], 0.0),
        None
    );
    let locked = VecViewState {
        hidden: Vec::new(),
        locked: vec![id],
    };
    assert_eq!(
        pick_at_world(&sim, &scene, &locked, &map, [0.0, 0.0], 0.0),
        None
    );
}

/// O marquee pega a forma pela bbox de MUNDO.
#[test]
fn the_marquee_selects_a_translated_shape_by_its_world_bbox() {
    let (mut sim, scene, map, e) = scene_with_square();
    let vs = VecViewState::default();
    sim.world_mut().entity_mut(e).insert(Transform {
        translation: Vec2::new(20.0, 20.0),
        ..Transform::IDENTITY
    });
    assert!(pick_in_world_rect(&sim, &scene, &vs, &map, [-5.0, -5.0], [5.0, 5.0]).is_empty());
    assert_eq!(
        pick_in_world_rect(&sim, &scene, &vs, &map, [15.0, 15.0], [25.0, 25.0]),
        vec![e.to_bits()]
    );
}
