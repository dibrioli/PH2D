//! **O «Outline» pelo dreno real** (W2c): o traço na mesma leva do `Shape(3)`, um passo de undo.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue, register_ecs_components};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_render::register_render_components;
use ph2d_vec_scene::VecScene;

use super::preso_tests::captura;

/// ⭐ **«Outline» pelo dreno REAL é um passo só** (W2c): `dispatch` aplica o `Shape(3)` e o
/// `retrace_edited` traça na mesma leva; o Ctrl+Z devolve a caixa sem `ColliderOutline`.
#[test]
fn escolher_outline_pelo_dispatch_e_um_passo_e_ctrl_z_devolve_a_caixa() {
    use crate::undo::{ProjectUndo, SelectionMark};
    use ph2d_editor_core::PhysicsFieldEdit as P;
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderOutline, ColliderShape, RigidBody};
    let (mut sim, vec, mut reg) = (SimWorld::new(), VecScene::new(), ComponentRegistry::new());
    register_ecs_components(&mut reg);
    register_render_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let sprite = ph2d_render::Sprite::atlas(0, [4.0, 4.0], [1.0; 4]);
    let kind = BodyKind::Static;
    let corpo = sim
        .world_mut()
        .spawn((
            Name::new("Ring"),
            Transform::default(),
            RigidBody { kind },
            sprite,
        ))
        .id();
    let caixa = ColliderShape::Cuboid {
        half_x: 2.0,
        half_y: 2.0,
    };
    let _ = ph2d_physics_ecs::nasce_com_forma(sim.world_mut(), corpo, caixa);
    let db = ph2d_asset::AssetDb::new();
    let argola = (0..32u32 * 32).flat_map(|i| {
        let (x, y) = (i % 32, i / 32);
        [
            255,
            255,
            255,
            if (8..24).contains(&x) && (8..24).contains(&y) {
                0
            } else {
                255
            },
        ]
    });
    let atlas =
        std::collections::BTreeMap::from([(0, db.insert_image_rgba8(32, 32, argola.collect()))]);
    let antes = captura(&mut sim, &vec, &reg);
    let mut hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    let (mut q, mut toasts) = (
        EditorCommandQueue::default(),
        ph2d_editor_core::ToastQueue::default(),
    );
    let pick = [(corpo.to_bits(), P::Shape(3))];
    let _ = super::inspector_commits::dispatch(
        None,
        None,
        &[],
        None,
        None,
        None,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &pick,
        &[],
        &mut hero,
        &mut sim,
        &db,
        &atlas,
        &mut toasts,
        &mut q,
        &reg,
        0,
        0,
        0,
        0,
    );
    let formas = |sim: &mut SimWorld| {
        let mut qq = sim
            .world_mut()
            .query::<(&Collider, Option<&ColliderOutline>)>();
        qq.iter(sim.world())
            .map(|(c, o)| (c.shape, o.map(|o| o.rings.len())))
            .collect::<Vec<_>>()
    };
    assert!(
        matches!(
            formas(&mut sim)[..],
            [(ColliderShape::Outline { .. }, Some(2))]
        ),
        "{:?}",
        formas(&mut sim)
    );
    let depois = captura(&mut sim, &vec, &reg);
    let mut fila = ProjectUndo::default();
    fila.push_undo(antes, SelectionMark::default());
    let (volta, _) = fila
        .undo(depois, SelectionMark::default())
        .expect("o passo");
    let _ = volta.restore(&mut sim, &reg);
    assert_eq!(
        formas(&mut sim),
        vec![(caixa, None)],
        "um Ctrl+Z devolve a caixa"
    );
}
