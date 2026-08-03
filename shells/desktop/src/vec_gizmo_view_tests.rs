//! Os gates da VISTA do gizmo vetorial (`anchor_half` / `view` / `container_view`) — módulo
//! irmão de [`super`], teto de LOC (HR-18). Segue o idioma que o arquivo já usava para os testes
//! de hit (`vec_gizmo_view_hit_tests.rs`): o produto num arquivo, os gates noutro.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_vec_scene::rectangle;

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
    let v = view(
        &sim,
        &scene,
        &Default::default(),
        e,
        &cam,
        ws,
        (0.0, 0.0),
        false,
    )
    .unwrap();
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
    assert!(
        view(
            &sim,
            &scene,
            &Default::default(),
            e,
            &cam,
            ws,
            (0.0, 0.0),
            false
        )
        .is_some()
    );
    // Com o `VecBlend` (a entidade é um spine de blend), NÃO tem.
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_ecs::VecBlend::new(vec![1, 2], 3));
    assert!(
        view(
            &sim,
            &scene,
            &Default::default(),
            e,
            &cam,
            ws,
            (0.0, 0.0),
            false
        )
        .is_none(),
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

/// **A CAIXA DO GIZMO segue a pose do layout** — o terceiro consumidor da mesma lei (os outros são
/// as âncoras e o hit-test).
///
/// Sem ela o artista seleciona um filho colocado e a caixa de transformação aparece onde a forma
/// foi AUTORADA: as alças ficam longe da arte que elas manipulam.
///
/// ⚠️ O oráculo é o CENTRO em mundo, não "mudou": a pose entra depois do transform, então uma
/// composição na ordem errada desloca pela pose ESCALADA e passaria num gate de desigualdade.
#[test]
fn the_gizmo_box_follows_the_pose_the_layout_gave() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let id = scene.push_path(rectangle([-1.0, -1.0], [1.0, 1.0]));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(id)))
        .id();
    let cam = Camera2d::default();
    let ws = WindowSize {
        width: 800,
        height: 600,
    };

    let bare = view(
        &sim,
        &scene,
        &Default::default(),
        e,
        &cam,
        ws,
        (0.0, 0.0),
        false,
    )
    .expect("gizmo");
    let c0 = [
        (bare.bbox_min_world[0] + bare.bbox_max_world[0]) * 0.5,
        (bare.bbox_min_world[1] + bare.bbox_max_world[1]) * 0.5,
    ];
    assert!(c0[0].abs() < 1e-5 && c0[1].abs() < 1e-5, "sem pose: {c0:?}");

    // A moldura empurrou-a 10 para a direita e dobrou-a.
    let placed = VecViewState {
        poses: vec![(id, ph2d_vec_scene::Xform([2.0, 0.0, 0.0, 2.0, 10.0, 0.0]))],
        ..Default::default()
    };
    let v = view(&sim, &scene, &placed, e, &cam, ws, (0.0, 0.0), false).expect("gizmo");
    let c = [
        (v.bbox_min_world[0] + v.bbox_max_world[0]) * 0.5,
        (v.bbox_min_world[1] + v.bbox_max_world[1]) * 0.5,
    ];
    let half = (v.bbox_max_world[0] - v.bbox_min_world[0]) * 0.5;
    assert!(
        (c[0] - 10.0).abs() < 1e-4 && c[1].abs() < 1e-4,
        "a caixa devia centrar-se em (10, 0): {c:?}"
    );
    assert!(
        (half - 2.0).abs() < 1e-4,
        "e medir 2 de meia-largura (1 x 2): {half}"
    );
}
