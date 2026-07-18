//! `PH2D_PHYSICS_SMOKE=1` — a READY-TO-SEE scene for the global rigid
//! physics (ADR-0130 W1): one **dynamic** sprite dropped above a **static**
//! floor. On play it falls and settles on the floor.
//!
//! The sprites are plain ECS entities carrying `RigidBody` + `Collider`.
//! **Nothing here touches the rapier world** — the bridge
//! (`render_loop::physics_bridge`) builds the bodies from the components,
//! steps at the `Playhead` tick, and reads the pose back into `Transform`.
//! That is deliberate ([[feedback_ready_to_smoke_example]] + the impasto
//! smoke scar): if the bridge were dead, the sprite would hang in the air
//! instead of falling — the honest failure, not a hidden pre-step.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::Sprite;

impl crate::App {
    /// Frame prologue, once. No-op without the env.
    pub(crate) fn physics_smoke(&mut self) {
        if self.physics_smoke_done || std::env::var_os("PH2D_PHYSICS_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // world not up yet; retry next frame
        }
        self.physics_smoke_done = true;

        let gfx = self.gfx.as_mut().expect("gfx");

        // Static floor: wide thin cuboid centered at y = -1 (top at y = -0.8).
        // The sprite quad (full size) matches the collider (half-extents).
        gfx.sim.world_mut().spawn((
            Transform::from_translation(Vec2::new(0.0, -1.0)),
            Sprite::atlas(0, [8.0, 0.4], [0.40, 0.42, 0.48, 1.0]),
            Name::new("Floor"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 4.0,
                    half_y: 0.2,
                },
                density: 1.0,
            },
        ));

        // Dynamic ball dropped from y = 4 → should settle at y ≈ -0.5
        // (floor_top -0.8 + radius 0.3).
        gfx.sim.world_mut().spawn((
            Transform::from_translation(Vec2::new(0.0, 4.0)),
            Sprite::atlas(0, [0.6, 0.6], [1.0, 0.5, 0.2, 1.0]),
            Name::new("FallingSprite"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                density: 1.0,
            },
        ));

        // Play so the bridge steps the world forward.
        self.playhead.rewind();
        self.playhead.play();

        eprintln!(
            "[physics-smoke] FallingSprite (dynamic ball) dropped above Floor (static). \
             It should fall and settle on the floor. A dead bridge leaves it hanging in the air."
        );
    }
}
