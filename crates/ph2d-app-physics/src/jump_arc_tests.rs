//! A curva do pulo no canvas: o arrasto da alça e o cartão que escreve no componente.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::Camera2d;

fn cena(show: bool) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            PlatformPlayer {
                show_jump_arc: show,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(1.0, 2.0)),
        ))
        .id();
    (sim, e)
}

fn s() -> PhysicsSettings {
    PhysicsSettings::default()
}

#[test]
fn arrastar_a_alca_poe_o_apice_sob_o_cursor() {
    let (mut sim, e) = cena(true);
    let mut st = PhysicsState::default();
    let d = desenho(&sim, &s(), Some(e.to_bits()), true).unwrap();
    let a = d.apice();
    assert!(open(&mut st, &sim, &s(), (e, PointHandleKind::JumpApex), a));
    assert!(!open(
        &mut PhysicsState::default(),
        &sim,
        &s(),
        (e, PointHandleKind::AnchorA),
        a
    ));
    // O alvo: 1 m mais alto e 1 m mais longe.
    let alvo = [a[0] + 1.0, a[1] + 1.0];
    let cam = Camera2d::default();
    let win = WindowSize {
        width: 800,
        height: 600,
    };
    let px = cam.world_to_screen(alvo, win);
    let mut present = ph2d_ecs::PresentWorld::default();
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let bridge = ph2d_physics_ecs::PhysicsBridge::new();
    let mut ctx = CanvasCtx {
        camera: &cam,
        window: win,
        sim: &mut sim,
        bridge: &bridge,
        present: &mut present,
        toasts: &mut toasts,
        hero: None,
    };
    advance(&st, &mut ctx, px);
    let d2 = desenho(&sim, &s(), Some(e.to_bits()), true).unwrap();
    let b = d2.apice();
    // Uma tolerância de um pixel da câmara de teste.
    let tol = 2.0 * cam.height_world / 600.0;
    assert!(
        (b[0] - alvo[0]).abs() < tol && (b[1] - alvo[1]).abs() < tol,
        "{b:?} ≠ {alvo:?}"
    );
}

#[test]
fn a_caixa_e_o_feel_do_cartao_chegam_ao_componente_e_ao_canvas() {
    use crate::inspector::player::apply_player_edit_in;
    use ph2d_editor_core::screens::hero::PlayerFieldEdit as E;
    let (mut sim, e) = cena(false);
    let bits = e.to_bits();
    apply_player_edit_in(&mut sim, bits, E::ShowJumpArc(true), &s());
    assert!(
        desenho(&sim, &s(), Some(bits), true).is_some(),
        "marcar liga a curva"
    );
    apply_player_edit_in(&mut sim, bits, E::ShowJumpArc(false), &s());
    assert!(
        desenho(&sim, &s(), Some(bits), true).is_none(),
        "desmarcar desliga"
    );
    apply_player_edit_in(&mut sim, bits, E::ShowJumpArc(true), &s());
    let antes = desenho(&sim, &s(), Some(bits), true).unwrap().apice();
    apply_player_edit_in(&mut sim, bits, E::Feel(2), &s());
    let p = sim.world().get::<PlatformPlayer>(e).unwrap();
    let heavy = ph2d_physics_ecs::jump_feel::Feel::Heavy.gravity();
    assert_eq!(
        p.fall_gravity, heavy.fall_gravity,
        "Heavy escreve as gravidades"
    );
    let depois = desenho(&sim, &s(), Some(bits), true).unwrap().apice();
    assert!(
        (depois[1] - antes[1]).abs() < 0.01,
        "o topo fica: {antes:?} → {depois:?}"
    );
}
