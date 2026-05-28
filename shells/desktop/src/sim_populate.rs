//! Initial sim population helpers for the desktop shell demo.
//!
//! Wave 3.1 stage B — extracted from `main.rs::App::{populate_sim,
//! populate_sim_live}` as free functions. Behavior-preserving lift.
//!
//! Two fixtures:
//! - [`populate_sim`] — 1000-sprite Vogel spiral with deterministic
//!   per-index velocity (M5 baseline demo).
//! - [`populate_sim_live`] — 8 named entities in a depth-2 tree
//!   (M14.6 hierarchy panel demo).

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_render::Sprite;

use crate::{SPRITE_COUNT, Velocity, WORLD_HALF};

/// Spawn `SPRITE_COUNT` sprites on a Vogel (golden-angle) spiral
/// with pseudo-random velocities derived from index — fully
/// deterministic, no PRNG dep.
pub(crate) fn populate_sim(sim: &mut SimWorld) {
    for i in 0..SPRITE_COUNT {
        let f = i as f32;
        let angle = f * 2.399_963_2; // golden angle (rad)
        let r = (f / SPRITE_COUNT as f32).sqrt() * (WORLD_HALF - 0.5);
        // T1.3.5 cross-OS bit-identical: any sprite Transform.translation
        // feeds the propagation hash gate. Demo populator uses libm too.
        let (sin_a, cos_a) = libm::sincosf(angle);
        let pos = Vec2::new(r * cos_a, r * sin_a);
        // Velocity in m/s; both axes seeded by independent index hashes
        // so motion isn't correlated with the spiral pattern.
        let vx = ((f * 12.9898).sin() * 43758.547).fract() * 3.0 - 1.5;
        let vy = ((f * 78.233).sin() * 12345.678).fract() * 3.0 - 1.5;
        sim.world_mut().spawn((
            Transform::from_translation(pos),
            Velocity(Vec2::new(vx, vy)),
            Sprite::atlas(i % 16, [0.18, 0.18], [1.0, 1.0, 1.0, 1.0]),
        ));
    }
}

/// Spawn 8 named entities in a depth-2 tree — used in the default
/// editor mode. `Name` components let the hierarchy panel display
/// readable rows ("sprite_001" through "sprite_008") instead of a
/// wall of `Entity_…` hex strings.
///
/// 2 roots × 3 children = 8 entities arranged as a depth-2 tree.
/// Lets the user exercise the M14.6C hierarchy expand/collapse
/// chevron (which only appears on parents) and M14.6A hide/show
/// propagation as soon as the demo boots — without that, every row
/// is a root and there's nothing to collapse. Layout: roots on
/// Y=+0.5/-0.5 with their 3 children fanned out on the same row,
/// 0.5 m apart.
pub(crate) fn populate_sim_live(sim: &mut SimWorld) {
    let mut sprite_idx = 0u32;
    let world = sim.world_mut();
    for group in 0..2 {
        let y_root = if group == 0 { 0.5 } else { -0.5 };
        let root_entity = world
            .spawn((
                Transform::from_translation(Vec2::new(-1.5, y_root)),
                Sprite::atlas(sprite_idx % 16, [0.4, 0.4], [1.0, 1.0, 1.0, 1.0]),
                Name::new(format!("group_{:02}", group + 1)),
            ))
            .id();
        sprite_idx += 1;
        for child in 0..3 {
            let f = child as f32;
            let vx = ((sprite_idx as f32 * 12.9898).sin() * 43758.547).fract() * 1.0 - 0.5;
            world.spawn((
                Transform::from_translation(Vec2::new(-0.5 + f, y_root)),
                Velocity(Vec2::new(vx, 0.0)),
                Sprite::atlas(sprite_idx % 16, [0.4, 0.4], [1.0, 1.0, 1.0, 1.0]),
                Name::new(format!("sprite_{:03}", sprite_idx)),
                ph2d_ecs::ChildOf(root_entity),
            ));
            sprite_idx += 1;
        }
    }
}
