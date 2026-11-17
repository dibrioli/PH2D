//! ⭐ GEF 08 MI-10: o filho «Motion Input» é uma entrada do grafo, sem lugar no espaço — não é um vazio:
//! sem anel, sem caixa do gizmo e sem clique no canvas (o pai, ao lado, continua a ter os dele).
//!
//! **Mutação que deve sangrar:** tirar o `MotionInput` de `is_empty_object`.

use ph2d_app_vec::group_gizmo_view::{is_empty_object, pick_empty_at_world, view};
use ph2d_ecs::{MotionInput, Name, SimWorld, Transform};

#[test]
fn um_filho_motion_input_nao_tem_gizmo_nem_se_arrasta() {
    let mut sim = SimWorld::new();
    let heroi = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Hero")))
        .id();
    let filho = ph2d_physics_ecs::pendura_entrada(sim.world_mut(), None, Some(heroi), "Input");
    let cam = ph2d_render::Camera2d::default();
    let janela = ph2d_host::WindowSize {
        width: 800,
        height: 600,
    };
    let caixa = |sim: &SimWorld, e| view(sim, e, &cam, janela, (0.0, 0.0), false, 100.0).is_some();
    // Controlo: antes de levar a entrada, o filho é um vazio como outro qualquer.
    assert!(is_empty_object(&sim, filho) && caixa(&sim, filho));
    sim.world_mut()
        .entity_mut(filho)
        .insert(MotionInput::default());
    assert!(
        !is_empty_object(&sim, filho),
        "o filho Motion Input é um vazio"
    );
    assert!(!caixa(&sim, filho), "o filho Motion Input publica gizmo");
    assert_eq!(
        pick_empty_at_world(&sim, [0.0, 0.0], 100.0),
        vec![heroi.to_bits()],
        "o clique no lugar do herói vai ao herói, nunca à entrada"
    );
}
