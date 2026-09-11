//! **As cinco cenas BASE da física** — as que vivem na shell porque o roteador
//! as tinha em casa, e não porque precisem dela.
//!
//! ⚠️ **Por que estão separadas do roteador (W2/L2):** o `physics_smoke.rs` passou
//! dos 600 LOC do HR-18 quando cada braço do `match` cresceu de
//! `self.cena()` para `self.run_physics_scene(crate::…::cena)`. O corte é **por
//! responsabilidade**, nunca uma entrada nova no `FILE_OVERAGE_OK`: um ficheiro é
//! o ROTEADOR (que nível corre o quê, e o que se aplica depois) e o outro são
//! CENAS. ⭐ As cinco são também as próximas candidatas a sair para a crate — elas
//! só continuam aqui porque o `physics_smoke_author` mexe no Inspector.

use ph2d_app_physics::common::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

impl crate::App {
    /// **Scene 1 (W1).** One falling body, one floor. Approved 2026-07-18.
    pub(crate) fn physics_smoke_drop(&mut self) {
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
            Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], [1.0, 0.5, 0.2, 1.0]),
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
    pub(crate) fn physics_smoke_pile(&mut self) {
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
                    Sprite::atlas(WHITE_TILE_KEY, [0.56, 0.56], [1.0, hue + 0.2, hue, 1.0]),
                    ColliderShape::Cuboid {
                        half_x: 0.28,
                        half_y: 0.28,
                    },
                )
            } else {
                (
                    Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], [hue, 0.62, 0.95, 1.0]),
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

    /// **Scene 4 (W2b).** The world panel, with a scene built so every knob on
    /// it changes something you can SEE.
    ///
    /// Opens the panel itself — asking the artist to find `W` first is exactly
    /// the assembly a ready-to-smoke scene removes
    /// ([[feedback_ready_to_smoke_example]]).
    ///
    /// The scene is chosen per knob, not for looks:
    /// - **bodies of three sizes**, so *Air Drag* can separate the heavy from
    ///   the light. ⚠️ *Damping* deliberately will NOT: it is a uniform decay,
    ///   and the first smoke failed precisely because that knob was labelled
    ///   "Air Drag" (Enio: *"todos os objetos grandes e pequenos caem na mesma
    ///   velocidade"*). The scene proves both behaviours;
    /// - **a tall stack**, because *Sub-steps* and *Iterations* are visible in
    ///   how a stack settles and in how deep a fast body sinks on impact — a
    ///   single body shows neither;
    /// - **bodies that come to rest**, because *Sleep* is only observable on
    ///   something that stops.
    pub(crate) fn physics_smoke_world(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());

        // Three sizes × four columns: a stack that settles unevenly (so it
        // topples a little, like scene 2) and has small bodies that a drag
        // value slows down long before it touches the big ones.
        let sizes = [0.7f32, 0.45, 0.28];
        for i in 0..12u32 {
            let col = (i % 4) as f32;
            let row = (i / 4) as f32;
            let s = sizes[(i % 3) as usize];
            let x = col * 0.9 - 1.35;
            let y = 2.0 + row * 1.1;
            let hue = 0.25 + 0.18 * (i % 3) as f32;
            gfx.sim.world_mut().spawn((
                Transform::from_translation(Vec2::new(x, y)),
                Sprite::atlas(WHITE_TILE_KEY, [s, s], [hue, 0.55, 0.85, 1.0]),
                Name::new(format!("Body{i:02}")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: s * 0.5,
                        half_y: s * 0.5,
                    },
                    density: 1.0,
                    ..Collider::default()
                },
            ));
        }

        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 4] 12 bodies of three sizes falling onto Floor, with the PHYSICS \
             panel open (`W` toggles it).\n\
             Try, in order:\n\
               · Gravity Y -> 0      : everything stops falling, mid-air.\n\
               · Gravity X           : the pile slides sideways.\n\
               · Air Drag / Density  : the BIG bodies fall fastest (drag scales\n\
                                       with cross-section, mass resists it).\n\
               · Damping / Linear    : everything slows EQUALLY (uniform decay;\n\
                                       mass cannot enter it -- this is the knob\n\
                                       whose old 'Air Drag' label was the bug).\n\
               · Sub-steps           : less sink on impact (watch a body land).\n\
               · Sleep / Delay -> 0  : the settled pile freezes sooner.\n\
               · Show Colliders      : must agree with the `B` key, always.\n\
               · Reset to Defaults   : everything back, in one click.\n\
             Then Ctrl+S, Ctrl+O: the settings must come back with the project."
        );
    }

    /// **Scene 5 (W2c).** Two groups, two layers, one floor.
    ///
    /// The scene exists to make ONE cell of the matrix visible: the left group
    /// is on layer 0 with the floor, the right group on layer 1. Turning off the
    /// `(1, 0)` cell drops the right group through the floor while the left
    /// group sits — and the right group still stacks on ITSELF, which is what
    /// separates "layers work" from "collisions broke".
    ///
    /// Colour follows LAYER, not position: same-coloured groups would make the
    /// artist read positions to infer a rule.
    pub(crate) fn physics_smoke_layers(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());

        for (layer, x0, hue) in [(0u8, -2.2f32, 0.30f32), (1, 0.8, 0.85)] {
            for i in 0..6u32 {
                let col = (i % 2) as f32;
                let row = (i / 2) as f32;
                gfx.sim.world_mut().spawn((
                    Transform::from_translation(Vec2::new(x0 + col * 0.7, 1.5 + row * 0.8)),
                    Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], [hue, 0.55, 0.9, 1.0]),
                    Name::new(format!("L{layer}_Body{i}")),
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: 0.25,
                            half_y: 0.25,
                        },
                        density: 1.0,
                        layer,
                        ..Collider::default()
                    },
                ));
            }
        }

        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 5] LEFT group on layer 0 (with the floor), RIGHT group on layer 1.\n\
             The physics panel is open (`W`); expand 'Collision Layers'.\n\
             Try:\n\
               · click cell (1,0) : the RIGHT group falls THROUGH the floor, the left\n\
                                    group stays, and the right group still stacks on itself.\n\
               · click it again   : they land again -- and this must work on the bodies\n\
                                    ALREADY in the scene, not only on new ones.\n\
               · click cell (1,1) : the right group stops colliding with ITSELF.\n\
               · Inspector > Layer: select one body and move it to another layer.\n\
               · Ctrl+S, Ctrl+O   : the matrix comes back with the project."
        );
    }

    /// **Scene 3 (W2).** A floor and three plain sprites with NO physics.
    ///
    /// The whole point is the empty state: select a sprite, open **Physics
    /// Body** in the Inspector, click **Add Physics Body**, press Play. Before
    /// W2 there was no gesture anywhere in the editor that could do that.
    pub(crate) fn physics_smoke_author(&mut self) {
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
                Sprite::atlas(WHITE_TILE_KEY, [w, h], [1.0, hue, 0.35, 1.0]),
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
