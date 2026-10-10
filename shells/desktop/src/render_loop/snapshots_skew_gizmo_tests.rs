//! ⭐⭐ **A caixa de uma sprite com SKEW é o paralelogramo que o motor desenha** — pelo passe real do
//! gizmo (`publish_gizmo`), com o oráculo na afim do PRESENTE (`GlobalTransform`, o que o render lê).
//! Controlo sem skew: a mesma fixtura dá a caixa de sempre e `model` identidade. E a vista GLOBAL é
//! de mundo: `model` identidade, a envolver os cantos pintados (doc ObjectsAndComponents 01 §15).

use super::*;
use ph2d_ecs::Entity;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::gizmo::{MODEL_IDENTITY, model_apply};

fn pose(t: [f32; 2], skew: [f32; 2]) -> Transform {
    let mut x = Transform::IDENTITY;
    x.translation.x = t[0];
    x.translation.y = t[1];
    x.rotation = 25f32.to_radians();
    x.scale.x = 1.5;
    x.scale.y = 0.8;
    x.skew_x = skew[0];
    x.skew_y = skew[1];
    x
}

/// Sprites `2 × 1` (meia-extensão `(1, 0.5)`) nas poses pedidas, com o espelho no presente.
fn cena(poses: &[Transform]) -> (SimWorld, PresentWorld, Vec<Entity>) {
    let (mut sim, mut present) = (SimWorld::new(), PresentWorld::new());
    let es = poses
        .iter()
        .map(|t| {
            let e = sim
                .world_mut()
                .spawn((*t, ph2d_render::Sprite::atlas(0, [2.0, 1.0], [1.0; 4])))
                .id();
            present
                .world_mut()
                .spawn((SimRef(e), ph2d_ecs::GlobalTransform::from_transform(*t)));
            e
        })
        .collect();
    (sim, present, es)
}

fn passe(sim: &SimWorld, present: &mut PresentWorld, sel: &[Entity]) -> HeroScreen {
    let mut hero = HeroScreen::new(NodeId(1));
    hero.gizmo.selection = Some(sel[0].to_bits());
    hero.gizmo.extra_selection = sel[1..].iter().map(|e| e.to_bits()).collect();
    super::publish_gizmo(
        &mut hero,
        sim,
        present,
        &Camera2d::default(),
        WindowSize {
            width: 800,
            height: 600,
        },
        (0.0, 0.0),
        &[],
        &ph2d_vec_scene::VecScene::default(),
        true,
        &ph2d_vec_scene::VecViewState::default(),
        &FlipDoc::default(),
        Vec::new(),
        None,
        false,
    );
    // O passe real chama a global logo a seguir (`publish`).
    super::gizmo::global_view(&mut hero, sim, (0.0, 0.0), false);
    hero
}

fn desenhados(t: Transform) -> [[f32; 2]; 4] {
    let a = ph2d_ecs::GlobalTransform::from_transform(t).affine();
    [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)].map(|(sx, sy): (f32, f32)| {
        let q = [sx, 0.5 * sy];
        [
            a[0] * q[0] + a[2] * q[1] + a[4],
            a[1] * q[0] + a[3] * q[1] + a[5],
        ]
    })
}

fn da_caixa(v: &ph2d_editor_core::GizmoView) -> [[f32; 2]; 4] {
    let c = [
        (v.bbox_min_world[0] + v.bbox_max_world[0]) * 0.5,
        (v.bbox_min_world[1] + v.bbox_max_world[1]) * 0.5,
    ];
    let h = [
        (v.bbox_max_world[0] - v.bbox_min_world[0]) * 0.5,
        (v.bbox_max_world[1] - v.bbox_min_world[1]) * 0.5,
    ];
    let (s, co) = v.rotation.sin_cos();
    [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)].map(|(sx, sy): (f32, f32)| {
        let (lx, ly) = (sx * h[0], sy * h[1]);
        model_apply(&v.model, [c[0] + lx * co - ly * s, c[1] + lx * s + ly * co])
    })
}

fn iguais(a: [[f32; 2]; 4], b: [[f32; 2]; 4]) -> bool {
    a.iter()
        .zip(b)
        .all(|(p, q)| (p[0] - q[0]).abs() < 1e-4 && (p[1] - q[1]).abs() < 1e-4)
}

#[test]
fn a_skewed_sprite_gizmo_is_the_drawn_parallelogram() {
    let t = pose([2.0, 1.0], [0.35, -0.2]);
    let (sim, mut present, es) = cena(&[t]);
    let hero = passe(&sim, &mut present, &es);
    let v = hero
        .gizmo
        .view
        .expect("a sprite seleccionada publica caixa");
    assert_ne!(v.model, MODEL_IDENTITY, "o skew vai no model");
    assert!(
        iguais(da_caixa(&v), desenhados(t)),
        "{:?} ≠ {:?}",
        da_caixa(&v),
        desenhados(t)
    );
}

/// Controlo: sem skew a fixtura dá a caixa de sempre — prova que a meia-extensão `(1, 0.5)` é a que
/// a porta da folha devolve, e que o caso de cima mede o skew e não a fixtura.
#[test]
fn without_skew_the_same_fixture_gives_the_old_box() {
    let t = pose([2.0, 1.0], [0.0, 0.0]);
    let (sim, mut present, es) = cena(&[t]);
    let v = passe(&sim, &mut present, &es).gizmo.view.unwrap();
    assert_eq!(v.model, MODEL_IDENTITY);
    assert!(iguais(da_caixa(&v), desenhados(t)));
}

/// A vista GLOBAL (duas sprites, uma com skew) é de mundo e envolve os quatro cantos pintados de
/// cada uma — não herda o `model` da primeira.
#[test]
fn the_global_view_is_world_and_wraps_the_drawn_corners() {
    let (a, b) = (pose([2.0, 1.0], [0.5, 0.0]), pose([-3.0, 0.0], [0.0, 0.0]));
    let (sim, mut present, es) = cena(&[a, b]);
    let g = passe(&sim, &mut present, &es)
        .gizmo
        .global_view
        .expect("duas vistas publicam a global");
    assert_eq!(g.model, MODEL_IDENTITY);
    for p in desenhados(a).into_iter().chain(desenhados(b)) {
        assert!(
            (g.bbox_min_world[0]..=g.bbox_max_world[0]).contains(&p[0])
                && (g.bbox_min_world[1]..=g.bbox_max_world[1]).contains(&p[1]),
            "o canto pintado {p:?} fica fora da global {g:?}"
        );
    }
}
