//! `PH2D_PHYSICS_SMOKE=<n>` — READY-TO-SEE scenes for the global rigid
//! physics (ADR-0131).
//!
//! | `n` | Wave | Scene |
//! |-----|------|-------|
//! | `1` | W1   | one **dynamic** sprite dropped above a **static** floor |
//! | `2` | W1.5 | a **pile** of bodies, for scrubbing the clock backwards |
//! | `3` | W2   | plain sprites + a floor, for authoring bodies in the Inspector |
//!
//! The sprites are plain ECS entities carrying `RigidBody` + `Collider`.
//! **Nothing here touches the rapier world** — the bridge
//! (`render_loop::physics_bridge`) builds the bodies from the components,
//! steps at the `Playhead` tick, and reads the pose back into `Transform`.
//! That is deliberate ([[feedback_ready_to_smoke_example]] + the impasto
//! smoke scar): if the bridge were dead, the sprites would hang in the air
//! instead of falling — the honest failure, not a hidden pre-step.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::Sprite;

/// Static floor, centered at `y = -1` (top at `y = -0.8`). The sprite quad
/// (full size) matches the collider (half-extents).
fn spawn_floor(world: &mut bevy_ecs::world::World) {
    world.spawn((
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
            ..Collider::default()
        },
    ));
}

impl crate::App {
    /// Frame prologue, once. No-op without the env.
    pub(crate) fn physics_smoke(&mut self) {
        let Some(which) = std::env::var("PH2D_PHYSICS_SMOKE").ok() else {
            return;
        };
        if self.physics_smoke_done {
            return;
        }
        if self.gfx.is_none() {
            return; // world not up yet; retry next frame
        }
        self.physics_smoke_done = true;

        match which.trim() {
            "2" => self.physics_smoke_pile(),
            "3" => self.physics_smoke_author(),
            _ => self.physics_smoke_drop(),
        }

        // Play so the bridge steps the world forward — except in the
        // authoring scene, which must sit STILL until the artist has given
        // something a body (a scene already running is a scene you cannot
        // set up).
        self.playhead.rewind();
        if which.trim() != "3" {
            self.playhead.play();
        } else {
            self.playhead.pause();
        }
    }

    /// **Scene 1 (W1).** One falling body, one floor. Approved 2026-07-18.
    fn physics_smoke_drop(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());

        // Dynamic body dropped from y = 4 → settles at y ≈ -0.5
        // (floor_top -0.8 + half height 0.3).
        //
        // ⚠️ The collider is a CUBOID matching the sprite quad, not a ball.
        // A sprite is a textured square, so a ball collider under it draws as
        // a box and behaves as a circle — the mismatch Enio reported. The
        // outline overlay makes any such mismatch visible; a demo scene
        // should not contain one to begin with.
        gfx.sim.world_mut().spawn((
            Transform::from_translation(Vec2::new(0.0, 4.0)),
            Sprite::atlas(0, [0.6, 0.6], [1.0, 0.5, 0.2, 1.0]),
            Name::new("FallingSprite"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 0.3,
                },
                density: 1.0,
                ..Collider::default()
            },
        ));

        eprintln!(
            "[physics-smoke 1] FallingSprite (dynamic box) dropped above Floor (static). \
             It should fall and settle on the floor. A dead bridge leaves it hanging in the air."
        );
    }

    /// **Scene 2 (W1.5).** A pile, because a pile is the scene where a wrong
    /// scrub is *visible*: mid-fall the bodies are strewn across the air, and
    /// settled they are a heap. A scrub that quietly replayed from the wrong
    /// state would show one when the ruler says the other.
    ///
    /// Opens the timeline panel itself — asking the artist to press `L` before
    /// the smoke can be run is exactly the assembly a ready-to-smoke scene is
    /// supposed to remove ([[feedback_ready_to_smoke_example]]).
    fn physics_smoke_pile(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());

        // 12 bodies on a staggered grid, so they tumble against each other on
        // the way down instead of dropping in tidy columns.
        for i in 0..12u32 {
            let col = (i % 4) as f32;
            let row = (i / 4) as f32;
            let x = col * 0.9 - 1.35
                + if (row as u32).is_multiple_of(2) {
                    0.0
                } else {
                    0.28
                };
            let y = 1.6 + row * 1.1;
            // Every collider matches its sprite quad. A sprite is a
            // textured SQUARE, so a ball collider under one draws as a box
            // and rolls like a circle — the thing Enio caught. Two sizes,
            // both boxes, so the pile still stacks unevenly and tips over.
            let hue = 0.25 + 0.06 * (i % 5) as f32;
            let (sprite, shape) = if i % 3 != 2 {
                (
                    Sprite::atlas(0, [0.56, 0.56], [1.0, hue + 0.2, hue, 1.0]),
                    ColliderShape::Cuboid {
                        half_x: 0.28,
                        half_y: 0.28,
                    },
                )
            } else {
                (
                    Sprite::atlas(0, [0.6, 0.6], [hue, 0.62, 0.95, 1.0]),
                    ColliderShape::Cuboid {
                        half_x: 0.3,
                        half_y: 0.3,
                    },
                )
            };
            gfx.sim.world_mut().spawn((
                Transform::from_translation(Vec2::new(x, y)),
                sprite,
                Name::new(format!("Body{i:02}")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape,
                    density: 1.0,
                    ..Collider::default()
                },
            ));
        }

        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("timeline", true);
        }

        eprintln!(
            "[physics-smoke 2] 12 bodies falling onto Floor. Let them settle, then DRAG THE \
             PLAYHEAD BACKWARDS on the timeline ruler: the pile must rebuild exactly as it fell, \
             with no stall. (Timeline panel opened for you; `L` toggles it.)"
        );
    }

    /// **Scene 3 (W2).** A floor and three plain sprites with NO physics.
    ///
    /// The whole point is the empty state: select a sprite, open **Physics
    /// Body** in the Inspector, click **Add Physics Body**, press Play. Before
    /// W2 there was no gesture anywhere in the editor that could do that.
    fn physics_smoke_author(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());

        // Three different aspect ratios, because the collider Add derives is
        // the sprite's own box — a mistake there is invisible on a square.
        for (i, (w, h)) in [(1.6f32, 0.5f32), (0.5, 1.4), (0.9, 0.9)]
            .into_iter()
            .enumerate()
        {
            let hue = 0.3 + 0.2 * i as f32;
            gfx.sim.world_mut().spawn((
                Transform::from_translation(Vec2::new(i as f32 * 1.8 - 1.8, 2.0 + i as f32 * 0.6)),
                Sprite::atlas(0, [w, h], [1.0, hue, 0.35, 1.0]),
                Name::new(format!("Prop{i}")),
            ));
        }

        eprintln!(
            "[physics-smoke 3] Three plain sprites and a floor, clock PAUSED. Select a sprite, \
             open the Inspector's 'Physics Body' section, click 'Add Physics Body', then press \
             Play: it should fall and land. The collider is boxed to the sprite, so B (collider \
             outlines) should trace each sprite exactly."
        );
    }
}
