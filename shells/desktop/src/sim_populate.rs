//! Initial sim population helpers for the desktop shell demo.
//!
//! Wave 3.1 stage B — extracted from `main.rs::App::{populate_sim,
//! populate_sim_live}` as free functions. Behavior-preserving lift.
//!
//! Uma fixture:
//! - [`populate_sim`] — 1000-sprite Vogel spiral with deterministic
//!   per-index velocity (M5 baseline demo, sob `PH2D_M5_DEMO=1`).
//!
//! O `populate_sim_live` (8 entidades nomeadas numa árvore de profundidade 2, M14.6) foi
//! **REMOVIDO em 2026-07-17**: ele existia para a Hierarquia ter linhas legíveis quando o
//! app não tinha conteúdo real, e hoje tem. O modo live nasce com a cena VAZIA — a árvore
//! mostra o que o artista criar. Ver `init.rs`.

use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
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
