//! **The default Motion document: the SNOW, falling into the SEA** — a whole particle system, on
//! one canvas (Motion Nodes O4 — docs 48 · 49 · 50 · 51 · 52 · 60).
//!
//! ```text
//!                     ┌──────────┐ out ──┬──→ scale ──→ move ──→ output
//!                     │ sim.zone │       │
//!            state ←──┴──────────┘       ⊙  (the state entry: last tick's population)
//!              ↑                         │
//!   cull ← falloff ← drive(α) ← drive(size) ← ramp ← lifetime ← collide ← step ← buoyancy ← wind ← combine
//!                        ↑          ↑         ↑                                                     ↑ in1
//!                        └───── map_range ← attribute("life")             poisson → move → sim.spawn
//! ```
//!
//! Every line of a particle system is on that canvas, and every one of them is a node you can cut:
//!
//! - **BIRTH** — `sim.spawn` emits only *this tick's* newborns; `motion.combine` merges them into
//!   the live state. The zone's `init` is left UNWIRED on purpose: the population starts at
//!   NOTHING and is entirely born, so every flake has a unique identity from birth (doc 49). The
//!   birth sites come from `motion.distribute_poisson` — a *guaranteed* minimum spacing across the
//!   sky, so they never double up and never read as the comb a `motion.grid` row was (doc 60).
//! - **LIFE** — `force.wind` + `sim.step`: the velocity lives in the STATE, so the flakes
//!   *accelerate* instead of drifting (doc 48).
//! - **THE WORLD** — `force.buoyancy` and `sim.collide`. The sea is **shallow**: a flake arrives
//!   with a second and a half of gravity behind it, punches *through* the surface, taps the bed,
//!   and the water lifts it back to float and bob on the swell while it melts (doc 60). Both nodes
//!   are load-bearing — take the sea away and the flakes just land, take the bed away and a heavy
//!   splash falls out of the world.
//! - **AGE** — `sim.step` grows an `age`; `sim.lifetime` writes how far through life each flake is
//!   and `value.attribute` reads that back out as a value field (doc 50), which drives THREE things
//!   at once: the colour (`motion.color_ramp`), the size and the opacity (`motion.drive`, doc 51).
//!   The flake melts: it whitens, it shrinks, it fades.
//! - **DEATH** — twice, and a particle system has both: **old age** (`sim.lifetime`) and **place**
//!   (`motion.falloff` + `motion.cull` — inside a zone that pair is not a filter but a KILL).
//!
//! Birth and death balance, so the snowfall reaches a **steady state** and stays there.
//!
//! The rig scenes that used to boot with the editor (a rubber hose, flesh skinned to a FABRIK
//! tentacle, their shared breathing goal) and the deliberately-orphaned card that demoed the inert
//! veil are **gone** (Enio, 2026-07-12: *"deixe só o grafo da chuva"*). They live in git history;
//! every node they used is still registered and still tested — drop them from the add-menu and
//! they work. A boot document is a demo, not an archive.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

const COL_W: f32 = 190.0;
/// The zone's row. The interior sits one card below it (a state loop reads right-to-left, which is
/// the shape it has on a canvas that reads left-to-right); the birth chain sits below that.
const RAIN_ROW: f32 = 0.0;
const RAIN_INTERIOR_DY: f32 = 140.0;

/// The snow: flakes born along a band at the top of the kill disc, falling under gravity, splashing
/// into the sea, and melting of old age while they bob.
///
/// The birth sites are a **Poisson-disc** distribution, not a grid row: `motion.distribute_poisson`
/// names the *spacing* and lets the count fall out (doc 60), so no two sites ever coincide and the
/// line of them is irregular the way falling snow is. The band is wide and barely taller than the
/// spacing, so it fills as one wobbly row — a comb with the teeth knocked crooked.
const SNOW_SKY_W: f32 = 3.2;
const SNOW_SKY_H: f32 = 0.35;
/// The minimum distance between two birth sites (the Poisson radius). ~14 sites fit the band.
const SNOW_SITE_GAP: f32 = 0.33;
const SNOW_SKY_Y: f32 = 2.6;
const SNOW_RATE: f32 = 16.0;
const SNOW_GUST: f32 = 0.35;
/// A flake lives ~3.4 s (± the hashed variance) — ~1.5 s of falling, and the rest afloat.
const SNOW_LIFE: f32 = 4.5;
const SNOW_LIFE_VARIANCE: f32 = 0.25;

/// **The sea** (`force.buoyancy`, doc 60). The still-water line sits 0.6 above the floor, so the
/// water is SHALLOW — that is deliberate, and it is what keeps both nodes in the demo alive: a
/// flake hits the surface with ~1.5 s of gravity behind it, punches through, taps the bed
/// (`sim.collide`), and the water pushes it back up to float.
///
/// `density` (the lift at full submersion) beats `RAIN_GRAVITY` by 3.5x, so a flake settles about
/// a third of its draft under the surface and stays there. Make it *smaller* than gravity and the
/// flake sinks to the bed instead — a stone, not a bug.
pub(crate) const SEA_LEVEL: f32 = -0.5;
const SEA_DENSITY: f32 = 14.0;
/// A flake's draft: how far under it must be to be fully submerged. It is a snowflake, so: not far.
pub(crate) const SEA_DRAFT: f32 = 0.3;
/// Water is thick. Without this the flake would pogo on the surface forever instead of settling.
const SEA_DRAG: f32 = 5.0;
/// The swell: crests one every 2.4 units, travelling right at half a unit a second.
pub(crate) const SEA_WAVE_AMP: f32 = 0.14;
const SEA_WAVE_LEN: f32 = 2.4;
const SEA_WAVE_SPEED: f32 = 0.5;
/// The column the fade is driven by (`sim.lifetime` writes it: 0 at birth, 1 at the end).
const SNOW_ATTR: &str = "life";
/// `motion.color_ramp`'s presets: 0 Rainbow · 1 Heat · **2 Ice** · 3 Grayscale · 4 Custom.
const SNOW_RAMP_ICE: f32 = 2.0;
/// What a flake is worth at the very end of its life: a tenth of its size and a tenth of its
/// opacity. Not zero — a flake that shrank to nothing a frame before dying reads as a pop.
const SNOW_FADE_FLOOR: f32 = 0.1;
/// The ground the snow settles on (doc 52). `sim.collide`'s shapes: **0 Floor** · 1 Disc · 2 Bowl.
/// It sits well inside the kill disc, so a flake that lands dies of OLD AGE where it lies rather
/// than being culled at the rim.
const SNOW_FLOOR_SHAPE: f32 = 0.0;
pub(crate) const SNOW_FLOOR_Y: f32 = -1.1;
const SNOW_FLOOR_BOUNCE: f32 = 0.25;
const SNOW_FLOOR_FRICTION: f32 = 0.35;
/// `motion.drive`'s channels: 0 X · 1 Y · 2 Rotation · 3 Size · 4 Opacity (doc 51). Modes:
/// 0 Add · 1 Set · 2 Multiply.
const DRIVE_CH_SIZE: f32 = 3.0;
const DRIVE_CH_OPACITY: f32 = 4.0;
const DRIVE_MODE_SET: f32 = 1.0;
/// The quad the lowering draws per flake. (The fallback for a stream with no `size` column is the
/// IDENTITY — doc 39 — so a document that wants small quads says so.)
const RAIN_QUAD: f32 = 0.18;
const RAIN_GRAVITY: f32 = 4.0;
/// The kill disc, around the birth line's own centre: a LINEAR falloff of radius 3.4, culled at
/// 0.05, so a flake dies 0.95 x 3.4 = 3.2 world units out. The birth sites sit well inside it, so
/// every flake is born alive and dies only by living.
const RAIN_KILL_R: f32 = 3.4;
const RAIN_KILL_KEEP: f32 = 0.05;
/// Where the scene sits in the world.
const RAIN_X: f32 = 0.0;
pub(crate) const RAIN_Y: f32 = 2.4;

/// What the boot document is made of: its sinks, and the sub-chain the editor folds
/// into a subgraph card (doc 57).
pub(crate) struct Demo {
    pub sinks: Vec<NodeId>,
    /// The AGE chain — `sim.lifetime` and the five nodes that read the age it writes
    /// and turn it into colour, size and opacity. One input crosses into it (the
    /// collided population) and one output crosses out (the faded one), so the card
    /// it folds into has exactly one socket on each side: the smallest complete
    /// demonstration of a derived interface there is.
    pub aging: Vec<NodeId>,
}

/// Author the document into `g`.
pub(crate) fn build(g: &mut Graph) -> Option<Demo> {
    build_sim_zone(g)
}

/// Connect `from` → `to` on the given ports, an immediate (non-delayed) edge.
fn wire(g: &mut Graph, from: (NodeId, u16), to: (NodeId, u16)) -> Option<()> {
    g.connect(Edge {
        from,
        to,
        delayed: false,
    })
    .ok()
}

/// Wire a straight chain left-to-right (port 0 to port 0), one card per column from `col`.
fn chain(g: &mut Graph, row: f32, col: usize, nodes: &[NodeId]) -> Option<()> {
    for (i, n) in nodes.iter().enumerate() {
        g.set_pos(
            *n,
            Pos {
                x: (col + i) as f32 * COL_W,
                y: row,
            },
        );
    }
    for pair in nodes.windows(2) {
        wire(g, (pair[0], 0), (pair[1], 0))?;
    }
    Some(())
}

/// **The Simulation Zone** (O4, docs 48 + 49) — SNOW: it is born, it falls, it dies.
///
/// ```text
///                    ┌──────────┐ out ──┬──→ scale ──→ move ──→ output
///                    │ sim.zone │       │
///           state ←──┴──────────┘       ⊙  (the state entry: last tick's population)
///             ↑                         │
///   cull ← falloff ← sim.step ← wind ← combine(in0)
///                                          ↑ in1
///                     grid(1x14) → move → sim.spawn        (this tick's NEWBORNS)
/// ```
///
/// **The zone's `init` is deliberately left UNWIRED**: the population starts at nothing and is
/// entirely BORN. Every flake therefore has a unique identity from birth (`sim.spawn` stamps the
/// birth ordinal), and the whole triangle of a particle system is on the canvas:
///
/// - **BIRTH** — `sim.spawn` emits only *this tick's* newborns, and `motion.combine` merges them
///   into the live state. (A `motion.emitter` cannot do this: it is stateless, so merging it
///   would merge the same particles again, every frame, forever.)
/// - **LIFE** — `force.wind` + `sim.step`: velocity ACCUMULATES in the state, so the flakes
///   accelerate instead of drifting at a constant rate.
/// - **AGE** — `sim.step` grows an `age` on every flake, `sim.lifetime` kills what has outlived
///   its own (hashed) lifetime and writes how far through life the survivors are; `value.attribute`
///   reads that number back out as a value field, and `motion.color_ramp` COLOURS by it. The flake
///   fades along the Ice ramp as it ages — the most ordinary sentence in motion graphics, which
///   this library could not say until the attribute node existed (doc 50).
/// - **DEATH** — two of them, and a particle system has both: **old age** (`sim.lifetime`) and
///   **place** (`motion.falloff` + `motion.cull` — the stray that leaves the circle is gone, and
///   inside a zone that pair is not a filter but a KILL).
///
/// Birth and death balance, so the snow reaches a **steady state** and stays there. Every node in
/// the interior but the `sim.*` ones already existed; the zone is what made them a simulation.
fn build_sim_zone(g: &mut Graph) -> Option<Demo> {
    let zone = g.add_node("sim.zone");
    let combine = g.add_node("motion.combine");
    let wind = g.add_node("force.wind");
    let sea = g.add_node("force.buoyancy");
    let step = g.add_node("sim.step");
    let ground = g.add_node("sim.collide");
    let life = g.add_node("sim.lifetime");
    let attr = g.add_node("value.attribute");
    let fade = g.add_node("value.map_range");
    let ramp = g.add_node("motion.color_ramp");
    let shrink = g.add_node("motion.drive");
    let dim = g.add_node("motion.drive");
    let falloff = g.add_node("motion.falloff");
    let cull = g.add_node("motion.cull");
    let sky = g.add_node("motion.distribute_poisson");
    let lift = g.add_node("motion.move");
    let spawn = g.add_node("sim.spawn");
    let scale = g.add_node("motion.scale");
    let mv = g.add_node("motion.move");
    let output = g.add_node("motion.output");

    // Three rows: the zone and its render chain · the interior (which reads right-to-left, the
    // shape a state loop has on a canvas that reads left-to-right) · the birth chain under it.
    chain(g, RAIN_ROW, 1, &[zone])?;
    chain(g, RAIN_ROW, 2, &[scale, mv, output])?;
    wire(g, (zone, 0), (scale, 0))?;
    for (i, n) in [
        combine, wind, sea, step, ground, life, ramp, shrink, dim, falloff, cull,
    ]
    .iter()
    .enumerate()
    {
        g.set_pos(
            *n,
            Pos {
                x: i as f32 * COL_W,
                y: RAIN_ROW + RAIN_INTERIOR_DY,
            },
        );
    }
    wire(g, (combine, 0), (wind, 0))?;
    // Two forces, one integrator (the Houdini rule): gravity and the sea both ADD into the same
    // transient `accel` column, and `sim.step` is the only thing that turns it into motion.
    wire(g, (wind, 0), (sea, 0))?;
    wire(g, (sea, 0), (step, 0))?;
    wire(g, (step, 0), (ground, 0))?; // …and the world pushes BACK
    wire(g, (ground, 0), (life, 0))?;
    wire(g, (life, 0), (ramp, 0))?;
    wire(g, (ramp, 0), (shrink, 0))?;
    wire(g, (shrink, 0), (dim, 0))?;
    wire(g, (dim, 0), (falloff, 0))?;
    wire(g, (falloff, 0), (cull, 0))?;
    wire(g, (cull, 0), (zone, 1))?; // the interior's END lands on `state`

    // The colour is DRIVEN BY THE AGE: `value.attribute` reads the `life` column that
    // `sim.lifetime` just wrote (0 at birth, 1 at the end) and hands it to the ramp's `t`. A
    // diamond — the ramp reads the same stream twice, once as geometry and once as a value — and
    // the cook memoizes the shared branch, so it costs one evaluation, not two.
    for (i, n) in [attr, fade].iter().enumerate() {
        g.set_pos(
            *n,
            Pos {
                x: (3 + i) as f32 * COL_W,
                y: RAIN_ROW + 2.0 * RAIN_INTERIOR_DY,
            },
        );
    }
    wire(g, (life, 0), (attr, 0))?;
    wire(g, (attr, 0), (ramp, 1))?; // colour by age…
    wire(g, (attr, 0), (fade, 0))?;
    wire(g, (fade, 0), (shrink, 1))?; // …and SHRINK by it…
    wire(g, (fade, 0), (dim, 1))?; // …and FADE OUT by it
    g.set_text_param(attr, "attr", SNOW_ATTR);
    g.set_param(life, "life", SNOW_LIFE);
    g.set_param(life, "variance", SNOW_LIFE_VARIANCE);
    g.set_param(ramp, "preset", SNOW_RAMP_ICE);
    // The ground: the flakes land on it, hop once, slide, and melt where they lie. A collision
    // is a BOUNCE and not a shove, and only a zone could have one — outside it there is no
    // velocity to reflect (doc 52).
    g.set_param(ground, "shape", SNOW_FLOOR_SHAPE);
    g.set_param(ground, "height", SNOW_FLOOR_Y);
    g.set_param(ground, "restitution", SNOW_FLOOR_BOUNCE);
    g.set_param(ground, "friction", SNOW_FLOOR_FRICTION);
    // `life` runs 0 → 1 over a flake's life; the fade runs the other way, and never all the way
    // to nothing (a flake that shrank to a true zero would pop out of existence a frame before
    // it died, which reads as a glitch rather than a melt).
    g.set_param(fade, "in_lo", 0.0);
    g.set_param(fade, "in_hi", 1.0);
    g.set_param(fade, "out_lo", 1.0);
    g.set_param(fade, "out_hi", SNOW_FADE_FLOOR);
    // **`Set`, not `Multiply`.** Both columns RIDE THE STATE, so a multiply would compound every
    // tick — the flakes would collapse to nothing in a second and the fade would be a function of
    // how many frames had passed, not of age. `Set` is idempotent: the channel is a pure function
    // of the flake's life, re-derived each tick.
    g.set_param(shrink, "channel", DRIVE_CH_SIZE);
    g.set_param(shrink, "mode", DRIVE_MODE_SET);
    g.set_param(dim, "channel", DRIVE_CH_OPACITY);
    g.set_param(dim, "mode", DRIVE_MODE_SET);

    chain(g, RAIN_ROW + 3.0 * RAIN_INTERIOR_DY, 0, &[sky, lift, spawn])?;
    wire(g, (spawn, 0), (combine, 1))?; // …the newborns join the population

    // **The state entry.** The zone's PREVIOUS output enters the interior at its head — here the
    // free `in0` of the merge, which is exactly where the editor's plumbing would put it (it is
    // the one `pre` edge in the scene, and it draws as a portal badge, never as a spline). The
    // demo writes the topology the plumbing would, so the boot document is already reconciled.
    g.connect(Edge {
        from: (zone, 0),
        to: (combine, 0),
        delayed: true,
    })
    .ok()?;

    // The sky: a wide, barely-tall band of birth sites at a guaranteed spacing, lifted to the top
    // of the disc. The band is thinner than one site is wide, so it fills as a single crooked row.
    g.set_param(sky, "width", SNOW_SKY_W);
    g.set_param(sky, "height", SNOW_SKY_H);
    g.set_param(sky, "radius", SNOW_SITE_GAP);
    g.set_param(lift, "dy", SNOW_SKY_Y);
    g.set_param(spawn, "rate", SNOW_RATE);
    // Gravity: `force.wind`'s angle is degrees, 270 = straight down (y-up world). A little gust
    // so the flakes do not fall in lockstep columns.
    g.set_param(wind, "angle", 270.0);
    g.set_param(wind, "strength", RAIN_GRAVITY);
    g.set_param(wind, "gust", SNOW_GUST);
    // The sea, into which the snow falls.
    g.set_param(sea, "level", SEA_LEVEL);
    g.set_param(sea, "density", SEA_DENSITY);
    g.set_param(sea, "depth", SEA_DRAFT);
    g.set_param(sea, "drag", SEA_DRAG);
    g.set_param(sea, "wave_amplitude", SEA_WAVE_AMP);
    g.set_param(sea, "wave_length", SEA_WAVE_LEN);
    g.set_param(sea, "wave_speed", SEA_WAVE_SPEED);
    // The kill: `motion.falloff` writes a per-element mask from a disc around the origin, and
    // `motion.cull` keeps only what is still inside it (Falloff mode).
    g.set_param(falloff, "radius", RAIN_KILL_R);
    g.set_param(falloff, "curve", 0.0); // 0 = Linear: the mask is 1 - d/radius
    g.set_param(cull, "mode", 1.0); // Falloff
    g.set_param(cull, "amount", RAIN_KILL_KEEP); // …keep what the mask still calls "inside"
    g.set_param(scale, "amount", RAIN_QUAD);
    g.set_param(mv, "dx", RAIN_X);
    g.set_param(mv, "dy", RAIN_Y);
    Some(Demo {
        sinks: vec![output],
        aging: vec![life, attr, fade, ramp, shrink, dim],
    })
}
