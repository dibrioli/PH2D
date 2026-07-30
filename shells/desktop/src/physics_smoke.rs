//! `PH2D_PHYSICS_SMOKE=<n>` — READY-TO-SEE scenes for the global rigid
//! physics (ADR-0131).
//!
//! The scene index (`n` -> wave -> what you see) is the wave table in
//! [`docs/Physics/00_plano_waves.md`], the maintained map. It is NOT copied here:
//! a duplicate only rots — the inline table stopped at 28 while the dispatch grew
//! to 40 — so the `match` below is the source of truth for which `n` runs what,
//! and the plan doc is the source of truth for what each shows.
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
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// **As cenas que abrem PARADAS.** Uma cena que espera um gesto do artista
/// (adicionar um corpo, assar, arrastar um rig) não pode ter meio caído antes de
/// ele chegar ao mouse.
///
/// Uma TABELA e não uma cadeia de `|`: com vinte e poucas entradas o `matches!`
/// gastava uma linha por cena e comia o teto de LOC deste arquivo — e a lista é
/// exatamente o tipo de coisa que só cresce.
/// ⚠️ **Uma cena que pede um GESTO DE ALÇA tem de estar aqui.** As alças de ponto
/// (âncora de joint, centro/aro de roldana) são publicadas **rest-only** — durante
/// o play o overlay desenha a geometria do SOLVER, e elas autoram a AUTORADA —,
/// então uma cena que nasce tocando simplesmente **não tem alça nenhuma**, e o
/// artista relata a feature como quebrada (foi o que aconteceu com a 63).
///
/// ⚠️ **E isto é uma ENUMERAÇÃO escrita à mão**, ou seja exatamente a forma que a
/// próxima cena nasce fora. O gate `handle_scenes_start_paused` varre as mensagens
/// das cenas e exige que quem manda arrastar uma alça esteja nesta lista.
const PAUSED_SCENES: &[&str] = &[
    "3", "7", "14", "15", "16", "17", "21", "22", "23", "24", "37", "38", "39", "40", "41", "43",
    "44", "45", "46", "47", "51", "54", "58", "63", "64",
];

/// Static floor, centered at `y = -1` (top at `y = -0.8`). The sprite quad
/// (full size) matches the collider (half-extents).
pub(crate) fn spawn_floor(world: &mut bevy_ecs::world::World) {
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, -1.0)),
        Sprite::atlas(WHITE_TILE_KEY, [8.0, 0.4], [0.40, 0.42, 0.48, 1.0]),
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
            "4" => self.physics_smoke_world(),
            "5" => self.physics_smoke_layers(),
            "6" => self.physics_smoke_joints(),
            "7" => self.physics_smoke_bake(),
            "8" => self.physics_smoke_parented(),
            "9" => self.physics_smoke_scale(),
            "10" => self.physics_smoke_sensor(),
            "11" => self.physics_smoke_weld(),
            "12" => self.physics_smoke_gravity(),
            "13" => self.physics_smoke_capsule(),
            "14" => self.physics_smoke_launch(),
            "15" => self.physics_smoke_ccd(),
            "16" => self.physics_smoke_lock_rotation(),
            "17" => self.physics_smoke_offset(),
            "18" => self.physics_smoke_freeze_position(),
            "19" => self.physics_smoke_mass(),
            "20" => self.physics_smoke_dominance(),
            "21" => self.physics_smoke_material(),
            "22" => self.physics_smoke_damping(),
            "23" => self.physics_smoke_one_way(),
            "24" => self.physics_smoke_area(),
            "25" => self.physics_smoke_contacts(),
            "26" => self.physics_smoke_area_drag(),
            "27" => self.physics_smoke_buoyancy(),
            "28" => self.physics_smoke_form_drag(),
            "29" => self.physics_smoke_events(),
            "30" => self.physics_smoke_impact_demolition(),
            "31" => self.physics_smoke_fast_impact(),
            "32" => self.physics_smoke_spin_zone(),
            "33" => self.physics_smoke_author_spin(),
            "34" => self.physics_smoke_force_frame(),
            "35" => self.physics_smoke_falloff(),
            "36" => self.physics_smoke_mirror(),
            "37" => self.physics_smoke_bake_range(),
            "38" => self.physics_smoke_joint_anchor(),
            "39" => self.physics_smoke_bake_joint(),
            "40" => self.physics_smoke_author_joint(),
            "41" => self.physics_smoke_anchor_follows(),
            "42" => self.physics_smoke_live_tune(),
            "43" => self.physics_smoke_joint_glyphs(),
            "44" => self.physics_smoke_joint_handles(),
            "45" => self.physics_smoke_joint_pose(),
            "46" => self.physics_smoke_joint_draw(),
            "47" => self.physics_smoke_joint_slider(),
            "48" => self.physics_smoke_joint_motor(),
            "49" => self.physics_smoke_joint_break(),
            "50" => self.physics_smoke_joint_pair(),
            "51" => self.physics_smoke_joint_rig(),
            "52" => self.physics_smoke_grab(),
            "53" => self.physics_smoke_interact(),
            "54" => self.physics_smoke_ik(),
            "55" => self.physics_smoke_fk(),
            "56" => self.physics_smoke_rod(),
            "57" => self.physics_smoke_wheel(),
            "58" => self.physics_smoke_pulley(),
            "59" => self.physics_smoke_winch(),
            "60" => self.physics_smoke_break(),
            "61" => self.physics_smoke_tackle(),
            "62" => self.physics_smoke_differential(),
            "63" => self.physics_smoke_composition(),
            "64" => self.physics_smoke_weston(),
            _ => self.physics_smoke_drop(),
        }

        // Play so the bridge steps the world forward — except in the scenes
        // that must sit STILL until the artist has done something. Scene 3
        // waits for a body to be added; scene 7 waits for a Bake, and a bake
        // taken while the clock runs would be a bake of a scene that has
        // already half-fallen. (A `match`, not another `!=`: with two of them
        // the next scene to want a paused clock would have to notice both.)
        // ⚠️ ARM the transport's Physics toggle. It is off by default — Play
        // means "play my animation" until an artist opts in — and every scene
        // here exists to show the solver working, so shipping them disarmed
        // would demo a frozen world and read as "physics is broken". Scene 7
        // then asks the artist to turn it back OFF, which is the point of Bake.
        self.timeline.flags.simulate_physics = true;

        self.playhead.rewind();
        if PAUSED_SCENES.contains(&which.trim()) {
            self.playhead.pause();
        } else {
            self.playhead.play();
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
    fn physics_smoke_world(&mut self) {
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
    fn physics_smoke_layers(&mut self) {
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
