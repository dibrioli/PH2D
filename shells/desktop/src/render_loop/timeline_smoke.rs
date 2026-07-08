//! General-timeline visual smoke — the **animatable engine** on screen, behind a
//! flag (`PH2D_TIMELINE_SMOKE=1`).
//!
//! This is **debug scaffolding**, not a production path. It proves the one thing
//! the headless tests cannot show a human: that a `ph2d-anim` `Clip`, sampled at
//! the live engine [`Playhead`](ph2d_core::Playhead), drives real sprites frame
//! by frame — the whole point of the general timeline.
//!
//! It reuses the Motion vertical's 27-instance grid (see `super::motion_bridge`) as
//! a **known-good, guaranteed-visible base** (building a valid `RenderInstance`
//! by hand is error-prone; an invisible sprite would be a false-green smoke),
//! then **modulates every instance** from a two-track `Clip` sampled at the
//! Playhead: an x-sweep `Track` slides the grid left↔right and an opacity `Track`
//! pulses it. As the playhead advances the grid animates; `pause` freezes it;
//! `seek` jumps it. When on, it **replaces** the M5 demo scene for the frame
//! (owns the canvas), exactly like `motion_smoke`. Off by default.

use std::sync::OnceLock;

use ph2d_anim::{
    AnimTarget, AnimValue, Clip, EasingFamily, EasingMode, Interp, Key, RationalTime, Track,
};
use ph2d_ecs::PresentWorld;
use ph2d_eval_motion::evaluate_motion;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_render::SpriteRenderer;

/// Re-center the vertical's `x∈[0,6], y∈[0,2]` span on the origin (matches
/// `motion_smoke`). Presentation only.
const VIEW_CENTER_OFFSET: [f32; 2] = [-3.0, -1.0];
/// Shrink unit sprites below the grid spacing so they read as discrete cells.
const DISPLAY_SIZE: [f32; 2] = [0.6, 0.6];
/// Demo atlas cell count (matches `sim_populate`'s `i % 16`).
const ATLAS_CELLS: usize = 16;

/// Opaque target for the x-sweep track (meters added to every instance's x).
const SWEEP: AnimTarget = AnimTarget::new(0);
/// Opaque target for the opacity-pulse track (`0..1`).
const PULSE: AnimTarget = AnimTarget::new(1);
/// The demo animation loops every this-many seconds.
const LOOP_SECS: f64 = 3.0;

/// Is the timeline smoke enabled? Read once from `PH2D_TIMELINE_SMOKE=1`.
pub(super) fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_TIMELINE_SMOKE").is_some_and(|v| v == "1"))
}

/// The demo `Clip`, built once: a symmetric x-sweep (`0 → +2 → 0 → −2 → 0`) plus
/// an opacity pulse (`0.35 → 1.0 → 0.35`), both eased, over [`LOOP_SECS`].
fn demo_clip() -> &'static Clip {
    static CLIP: OnceLock<Clip> = OnceLock::new();
    CLIP.get_or_init(|| {
        let ease = Interp::Eased(ph2d_anim::Easing::new(
            EasingFamily::Cubic,
            EasingMode::InOut,
        ));
        let sweep = Track::new(vec![
            Key {
                t: RationalTime::from_seconds(0.0),
                value: AnimValue::Float(0.0),
                interp: ease,
            },
            Key {
                t: RationalTime::from_seconds(0.75),
                value: AnimValue::Float(2.0),
                interp: ease,
            },
            Key {
                t: RationalTime::from_seconds(1.5),
                value: AnimValue::Float(0.0),
                interp: ease,
            },
            Key {
                t: RationalTime::from_seconds(2.25),
                value: AnimValue::Float(-2.0),
                interp: ease,
            },
            Key {
                t: RationalTime::from_seconds(3.0),
                value: AnimValue::Float(0.0),
                interp: Interp::Hold,
            },
        ]);
        let pulse = Track::new(vec![
            Key {
                t: RationalTime::from_seconds(0.0),
                value: AnimValue::Float(0.35),
                interp: ease,
            },
            Key {
                t: RationalTime::from_seconds(1.5),
                value: AnimValue::Float(1.0),
                interp: ease,
            },
            Key {
                t: RationalTime::from_seconds(3.0),
                value: AnimValue::Float(0.35),
                interp: Interp::Hold,
            },
        ]);
        Clip::new(RationalTime::from_seconds(LOOP_SECS))
            .with_track(SWEEP, sweep)
            .with_track(PULSE, pulse)
    })
}

/// Sample the demo clip at an absolute playhead time, looped to [`LOOP_SECS`].
/// Returns `(x_offset_meters, opacity)`. Presentation glue mapping the sampled
/// values onto the smoke instances; the live-seam proof itself (advancing the
/// Playhead changes the sample) is headless in `ph2d-anim/tests/playhead_drive.rs`.
fn sample_demo(clip: &Clip, playhead_time: f64) -> (f32, f32) {
    let t = playhead_time.rem_euclid(LOOP_SECS);
    let x = match clip.sample(SWEEP, t) {
        Some(AnimValue::Float(v)) => v,
        _ => 0.0,
    };
    let opacity = match clip.sample(PULSE, t) {
        Some(AnimValue::Float(v)) => v,
        _ => 1.0,
    };
    (x, opacity)
}

/// Cook the base grid and publish playhead-animated instances into `present`.
pub(super) fn run(present: &mut PresentWorld, renderer: &SpriteRenderer, playhead_time: f64) {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).expect("motion.grid registers");
    ph2d_node_motion_transform::register(&mut reg).expect("motion.transform registers");
    ph2d_node_motion_clone::register(&mut reg).expect("motion.clone registers");

    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let xf = g.add_node("motion.transform");
    let clone = g.add_node("motion.clone");
    g.connect(Edge {
        from: (grid, 0),
        to: (xf, 0),
        delayed: false,
    })
    .expect("grid -> transform");
    g.connect(Edge {
        from: (xf, 0),
        to: (clone, 0),
        delayed: false,
    })
    .expect("transform -> clone");
    g.validate(&reg).expect("motion vertical is well-typed");

    let mut cook = Cook::new();
    let instances =
        evaluate_motion(&mut cook, &g, &reg, clone, 0.0).expect("motion vertical cooks");

    // The animation: the whole grid slides in x and pulses in opacity, driven
    // entirely by the ph2d-anim Clip sampled at the engine Playhead.
    let (dx, opacity) = sample_demo(demo_clip(), playhead_time);

    let atlas = renderer.atlas();
    let world = present.world_mut();
    world.clear_entities();
    for (idx, mut ri) in instances.into_iter().enumerate() {
        ri.world_pos[0] += VIEW_CENTER_OFFSET[0] + dx;
        ri.world_pos[1] += VIEW_CENTER_OFFSET[1];
        ri.size = DISPLAY_SIZE;
        ri.opacity = opacity;
        ri.atlas_uv = atlas.region_uv((idx % ATLAS_CELLS) as u32);
        world.spawn(ri);
    }
}
