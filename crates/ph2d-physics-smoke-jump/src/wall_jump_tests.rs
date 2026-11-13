//! **Bug #14 — pular encostado na parede sobe a Jump Height inteira.**
//!
//! O corpo do player não raspa em nada (`lei_da_forma_do_player`, na ponte): com atrito, a força do andar
//! contra a face × μ comia a subida (0,774 m de 2,0, μ 0,5 Average). Medido nesta fixture, antes da
//! cura: player μ 0 sozinho 1,105 (a `Average` deixa 0,25), parede μ 0 sozinha 1,105, os dois 0 ⇒
//! 2,000; `Min` com o player a 0 ⇒ 2,000; o Snap (cinemático) já subia 2,000.
//!
//! **Mutação que deve sangrar:** tirar o `lei_da_forma_do_player` de `desc_do_corpo`.

use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, PhysicsBridge, PlatformPlayer, PlayerInput, PlayerMode, RigidBody,
};
use ph2d_physics_scene::{slab, spawn_player};

/// A parede: face esquerda em `x = 3`, alta o bastante para nenhum pulo passar dela.
const WALL_FACE: f32 = 3.0;

/// Como o corpo está ligado às paredes.
#[derive(Clone, Copy, Debug)]
enum Walls {
    Off,
    Slide,
    Grab,
}

/// A subida do centro num pulo SEGURADO, com a seta `drive` premida desde antes do pulo.
fn rise(jump_height: f32, mode: PlayerMode, walls: Walls, drive: f32) -> f32 {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    slab(w, "Floor", Vec2::new(0.0, -0.5), [10.0, 0.5], 0.0, [1.0; 4]);
    slab(
        w,
        "Wall",
        Vec2::new(WALL_FACE + 1.0, 4.0),
        [1.0, 4.5],
        0.0,
        [1.0; 4],
    );
    let p = spawn_player(w, Vec2::new(WALL_FACE - 1.0, 1.2));
    {
        let mut e = w.entity_mut(p);
        let mut cfg = e.get_mut::<PlatformPlayer>().expect("player");
        cfg.jump_height = jump_height;
        cfg.walls_on = !matches!(walls, Walls::Off);
        if matches!(walls, Walls::Grab) {
            cfg.wall_grab_stamina = 1.0;
        }
        if mode.drives_itself() {
            e.insert(mode);
            e.get_mut::<RigidBody>().expect("body").kind = BodyKind::Kinematic;
        }
    }
    let mut bridge = PhysicsBridge::new();
    let mut tick = 0_u64;
    let y = |sim: &SimWorld| sim.world().get::<Transform>(p).expect("t").translation.y;
    let mut step = |sim: &mut SimWorld, jump: bool| {
        bridge.set_player_input(
            p,
            PlayerInput {
                drive,
                jump,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(sim, true, tick);
    };
    // Assenta e, com a seta premida, ENCOSTA na face (1 m a 6 m/s cabe em 60 tiques).
    for _ in 0..60 {
        step(&mut sim, false);
    }
    if drive > 0.0 {
        let x = sim.world().get::<Transform>(p).expect("t").translation.x;
        assert!(
            x > WALL_FACE - 0.25,
            "{mode:?} {walls:?}: não encostou na face (x {x:.3})"
        );
    }
    let y0 = y(&sim);
    let mut hi = y0;
    for _ in 0..150 {
        step(&mut sim, true);
        hi = hi.max(y(&sim));
    }
    hi - y0
}

/// **A varredura**: alturas 1–2,5, Dynamic e Snap, paredes desligadas, com slide e com grab — a
/// subida encostado com a seta contra a parede é a do pulo livre, ±1 mm.
#[test]
fn a_jump_against_a_wall_rises_the_full_jump_height() {
    for h in [1.0_f32, 1.5, 2.0, 2.5] {
        for mode in [PlayerMode::Dynamic, PlayerMode::Kinematic] {
            for walls in [Walls::Off, Walls::Slide, Walls::Grab] {
                let free = rise(h, mode, walls, 0.0);
                let hug = rise(h, mode, walls, 1.0);
                assert!(
                    (hug - free).abs() < 1.0e-3,
                    "altura {h} {mode:?} {walls:?}: encostado sobe {hug:.4}, livre {free:.4}"
                );
                assert!(
                    (free - h).abs() < 0.02,
                    "altura {h} {mode:?} {walls:?}: o livre sobe {free:.4}"
                );
            }
        }
    }
}
