//! `motion.look_at` — orient each element toward a **target** point (Motion Nodes
//! M3, deformers — doc 01 §3 / doc 20). This is one of the most-used nodes in all
//! of motion graphics: arrows that point at the cursor, petals that face the sun,
//! a shoal of fish turned toward their prey. It writes the `rot` channel so each
//! element's local +x axis aims at the target — the AE/Cavalry "orient toward
//! point", Houdini `lookat`.
//!
//! **The target is a value input** (`target_x`/`target_y`, the value domain —
//! doc 12), so it can be ANIMATED: wire `value.lfo`s and the whole field turns to
//! track a moving point (the boot scene does exactly this). Unconnected, the
//! target reads as the origin `(0, 0)` (everything faces centre). A per-element
//! target field gives each element its own aim. An `offset` param (degrees) rotates
//! the result — `+90` makes them point *across* the target, `180` *away*.
//!
//! **How much of the aim lands is the family's weight** — the multiplicative
//! `falloff` column times a `strength` param, exactly as `move`/`rotate`/`scale`/
//! `noise`/`wiggle`/`stagger`/`drive` read it (MOPs: *every* effect is modulated
//! by `f@mops_falloff`; C4D: every effector carries Strength + Fields). At weight 1
//! — no field, default strength — the answer is the aim verbatim, so nothing
//! already authored moves. Below it each element turns **part of the way** toward
//! the target along the SHORT arc: a heading is not a coordinate, and a naive lerp
//! sends an element two degrees from its target the long way round.
//!
//! Transcendental-free (HR-5): the heading is `atan2` via a **Rajan rational
//! approximation** (`atan(a) ≈ ¼π·a − a·(a−1)·(0.2447 + 0.0663·a)` for `a ∈ [0,1]`,
//! quadrant-folded), ~0.0015 rad (0.09°) off true `atan2` using only multiply/add —
//! well under a pixel of orientation error. `Pure` (no clock, no state; the target
//! input carries the animation).

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, ParamGate, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `target_*` inputs — the per-instance scalar field on the
/// `v` column (mirror of `ph2d_node_pulse_counter::VALUE`; kept local, leaf crate).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

const VALUE_COL: &str = "v";
/// `180/π` — radians to the `rot` channel's degrees (a constant, not a call; HR-5).
const RAD_TO_DEG: f32 = 57.295_78;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.look_at"),
    name: "motion.look_at",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The target point, as two value fields (so it can be animated). Optional:
        // unconnected reads as 0 → the origin.
        PortSpec {
            name: "target_x",
            ty: VALUE,
        },
        PortSpec {
            name: "target_y",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // Degrees added to the aim — 0 points AT the target, 180 points away.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // ⚠️ **The point, as NUMBERS.** `Point` mode shipped reading only the value
        // INPUTS, which meant the one way to aim at a coordinate was to wire two
        // `value.*` nodes — a mode named after a point, with no point in it (Enio:
        // *"Point serve para que se não há coordenadas do ponto?"*). A wire still
        // wins when one is connected, which is the driven-param convention this
        // panel already paints; these are what the artist types when it is not.
        ParamSpec {
            name: "target_x",
            default: 0.0,
        },
        ParamSpec {
            name: "target_y",
            default: 0.0,
        },
        // WHERE the target comes from: 0 the value inputs, 1 a named object,
        // 2 the cursor. Default 0 ⇒ every document written before this reads
        // exactly what it read before.
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // How much of the aim lands — the family's `Strength`, multiplied into the
        // same weight the `falloff` column carries (they scale the SAME turn, so
        // they are one number by the time the blend sees them). `1` is the whole
        // aim, which is what this node did before the weight existed.
        ParamSpec {
            name: "strength",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **Where a `motion.look_at` gets the point it aims at.**
///
/// The node could always aim anywhere — the target is a value INPUT, so a pair of
/// `value.*` nodes drives it. What it could not do is aim at something the artist
/// can NAME or at the cursor, which is the whole reason the node exists in every
/// other motion-graphics tool (Enio: the node shipped *"sem alvo por nome/mouse"*).
/// Wiring two LFOs to make arrows follow the mouse is not a workaround; it is a
/// thing an artist cannot do at all, because the cursor is not in the graph.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TargetMode {
    /// The `target_x` / `target_y` value inputs — the original behaviour, and the
    /// default, so no document moves.
    Point,
    /// The centroid of a NAMED object the app published (the `target` text param,
    /// picked from the live list — the same channel `motion.path` walks).
    Object,
    /// The cursor, published by the editor each frame under
    /// [`ph2d_nodegraph::external::CURSOR`] — the reserved namespace lives beside
    /// the table, not inside whichever node reads one of its values first.
    Cursor,
}

impl TargetMode {
    /// From the `mode` param. An out-of-range index falls back to `Point` — a
    /// visible no-op beats aiming somewhere nobody asked for.
    #[must_use]
    pub fn of(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Object,
            2 => Self::Cursor,
            _ => Self::Point,
        }
    }
}

/// The mean of an external's `P` column — the point an object "is at".
///
/// `None` for an external that is absent or carries no positions, and that is the
/// honest answer: the node then falls back to its value inputs rather than aiming
/// at the origin, which would look like a deliberate choice the artist did not make.
fn centroid(s: &Stream) -> Option<[f32; 2]> {
    let Some(Column::Vec2(p)) = s.get("P") else {
        return None;
    };
    if p.is_empty() {
        return None;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "a count of published points; the mean is a screen position"
    )]
    let n = p.len() as f32;
    let sum = p
        .iter()
        .fold([0.0f32, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]]);
    Some([sum[0] / n, sum[1] / n])
}

/// `atan2(y, x)` in radians, transcendental-free (Rajan rational approximation of
/// `atan` on `[0,1]`, folded across the eight octants). ~0.0015 rad error, only
/// multiply/add/compare (HR-5). Returns 0 at the origin.
fn atan2_approx(y: f32, x: f32) -> f32 {
    let (ax, ay) = (x.abs(), y.abs());
    let hi = ax.max(ay);
    if hi == 0.0 {
        return 0.0; // target coincides with the element — leave it at 0°.
    }
    // a = min/max ∈ [0,1]; atan(a) by the Rajan polynomial.
    let a = ax.min(ay) / hi;
    let mut r = FRAC_PI_4 * a - a * (a - 1.0) * (0.2447 + 0.0663 * a);
    if ay > ax {
        r = FRAC_PI_2 - r; // reflect about the 45° line when y dominates
    }
    if x < 0.0 {
        r = PI - r; // left half-plane
    }
    if y < 0.0 {
        r = -r; // lower half-plane
    }
    r
}

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`) — the
/// family's channel, read exactly as `motion.rotate` reads it (doc 89 folha 08).
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// Fold a degree difference into `(-180, 180]` — **the shortest way round**.
///
/// ⚠️ Without this the weight is worse than useless on the exact elements it is
/// meant to spare: an element already pointing at `-179°` that should aim at
/// `179°` is **2° away**, and a plain lerp at half weight sends it to `0°` — the
/// long way, through pointing at nothing. An angle is not a position.
///
/// ⚠️ **`floor`, never `round`.** The closed form wants a nearest-integer, and
/// Rust's `round` breaks ties away from zero while WGSL's breaks them to even —
/// so at exactly a half-turn (`d = ±180`, the one input a reader would call a
/// corner case) the two languages would disagree by a whole 360, and the blend at
/// half weight would turn the element to opposite sides on CPU and GPU. `floor`
/// has no ties, so the two agree by construction, and the half-turn resolves
/// clockwise on both.
fn wrap180(d: f32) -> f32 {
    d - 360.0 * (d / 360.0 + 0.5).floor()
}

/// **The one place the weight is applied**, and the reason it has three arms.
///
/// * `w >= 1` returns the aim **VERBATIM**. Not "the lerp happens to land there":
///   full weight reproducing today byte for byte is what keeps every document ever
///   written unmoved, and `orig + (aimed − orig)` is not `aimed` in `f32` for every
///   pair. It also keeps `offset` free to push the aim past ±180 the way it does
///   today — [`wrap180`] would fold that number, same angle, different number, and
///   anything reading `rot` as a value would see the change.
/// * `w <= 0` returns the original **VERBATIM** — the promise a falloff makes is
///   that outside it nothing happens, and `orig ± 360` is not nothing.
/// * Between them, turn along the short arc.
///
/// The two ends are also the clamp: a `strength` above 1 would mean *turn past the
/// thing you are looking at*, and because an angle wraps, extrapolating it is
/// unbounded and unreadable — unlike a position, where overshoot has a picture.
fn blend_aim(orig: f32, aimed: f32, w: f32) -> f32 {
    if w >= 1.0 {
        return aimed;
    }
    if w <= 0.0 {
        return orig;
    }
    orig + wrap180(aimed - orig) * w
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`atan2_approx`] plus the
/// broadcast rule, element for element.
///
/// **The first kernel with more than one CONNECTED stream, and the reason
/// [`ColumnAccess::ReadBroadcast`] exists.** The target is two VALUE fields, and
/// the whole point of the node is that they may be either: one global point the
/// entire field turns toward (a bare `value.lfo`, length 1), or a per-element
/// aim (length N). The CPU says that in `target_at`; the engine says it by
/// binding `v` on ports 1 and 2 as broadcast reads, and an ABSENT port still
/// falls back to the declared identity `0.0` — the `0 =>` arm, unchanged.
///
/// **Every** read is port-qualified here (`read_in_P`, not `read_P`) — the rule
/// is per NODE, not per column: a manifest with more than one input port names
/// all of its readers, because `v` is bound on two ports and a bare `read_v`
/// would silently resolve to one of them. The write stays bare (one output).
///
/// `LOOK_AT_RAD_TO_DEG` is the same literal constant the CPU uses — a `180/π`
/// recomputed in WGSL would be a second answer to a fixed number.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let la_p = read_in_P(i);\n\
        let la_dx = read_target_x_v(i) - la_p.x;\n\
        let la_dy = read_target_y_v(i) - la_p.y;\n\
        let la_aim = la_atan2(la_dy, la_dx) * 57.29578 + params.offset;\n\
        let la_w = read_in_falloff(i) * params.strength;\n\
        write_rot(i, la_blend(read_in_rot(i), la_aim, la_w));\n",
    wgsl_lib: "\
        // The three arms of the CPU `blend_aim`, in the same order and for the\n\
        // same reasons: full weight is the aim verbatim, no weight is the original\n\
        // verbatim, and between them the turn takes the SHORT arc.\n\
        fn la_blend(orig: f32, aimed: f32, w: f32) -> f32 {\n\
            if (w >= 1.0) { return aimed; }\n\
            if (w <= 0.0) { return orig; }\n\
            let d = aimed - orig;\n\
            // `floor`, never `round`: WGSL rounds ties to even and Rust away from\n\
            // zero, which would split the exact half-turn between the two.\n\
            return orig + (d - 360.0 * floor(d / 360.0 + 0.5)) * w;\n\
        }\n\
        fn la_atan2(y: f32, x: f32) -> f32 {\n\
            let ax = abs(x);\n\
            let ay = abs(y);\n\
            let hi = max(ax, ay);\n\
            // Target coincides with the element — leave it at 0 degrees.\n\
            if (hi == 0.0) { return 0.0; }\n\
            // a = min/max in [0,1]; atan(a) by the Rajan polynomial (HR-5).\n\
            let a = min(ax, ay) / hi;\n\
            var r = 0.7853982 * a - a * (a - 1.0) * (0.2447 + 0.0663 * a);\n\
            if (ay > ax) { r = 1.5707964 - r; }\n\
            if (x < 0.0) { r = 3.1415927 - r; }\n\
            if (y < 0.0) { r = -r; }\n\
            return r;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 2,
        },
        // The family's weight. Identity `1.0` is the whole point: a stream that
        // never met a field is aimed in full, which is what this node did before.
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        // `ReadWrite`, and now the READ side carries weight too: at partial weight
        // the answer is a turn FROM where the element already points, so the prior
        // `rot` is an input, not just a slot. Absent ⇒ the `0.0` identity, which is
        // the same starting angle the CPU uses.
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["offset", "strength"],
    count_law: None,
    variant_by_param: None,
    // ⚠️ The kernel reads the target from the two value PORTS, and the Object /
    // Cursor modes resolve theirs from the external table, which the device does
    // not see. Refusing is the honest answer (the ADR-0155 precedent: a document
    // the lowering cannot express recuses to the CPU) — the named cost is that a
    // graph aiming at an object or the cursor loses GPU residency for this node.
    applicable: Some(|p| {
        // ...and only while the target is the PORTS. The kernel reads the two value
        // ports and knows nothing about a typed coordinate, so with a point authored
        // it would aim at the port identity — the origin — while the CPU aims where
        // the artist typed. Two producers disagreeing, and the one nobody reads a
        // number from is the one on screen. The zero default is today's behaviour, so
        // a graph that drives the target by wire keeps its residency; typing a point
        // is what costs it, and that is the named trade.
        TargetMode::of(p("mode")) == TargetMode::Point
            && p("target_x") == 0.0
            && p("target_y") == 0.0
    }),
};

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// The target coordinate for element `i`: **unconnected (empty) → 0.0** (origin);
/// length-1 broadcasts; length-N is per-element.
fn target_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => 0.0,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(0.0),
    }
}

struct MotionLookAt;

impl NodeOp for MotionLookAt {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let offset = ctx.param("offset");
        let strength = ctx.param("strength");
        let mode = TargetMode::of(ctx.param("mode"));
        // ⚠️ The named target is read BEFORE `ctx.input(0)`: `external` takes `&mut
        // self` and `input` hands out a borrow that outlives the call, so resolving
        // afterwards would not compile. Reading it first also means the aim is the
        // object's place THIS frame, not a value stashed at spawn.
        let aim = match mode {
            TargetMode::Point => None,
            // ⚠️ The POSITION channel, never the object's own external: that one is an
            // APPEARANCE stream whose `P` is `[0, 0]` by design, so reading it aimed
            // every target at the origin — the exact failure this node's fallback
            // exists to prevent, arriving through a column with the right name and
            // another question's answer in it (Enio: *"Look At Object não funciona"*).
            TargetMode::Object => ctx
                .text_param("target")
                .map(str::to_owned)
                .filter(|n| !n.trim().is_empty())
                .and_then(|n| centroid(ctx.external(&ph2d_nodegraph::external::position_of(&n)))),
            TargetMode::Cursor => centroid(ctx.external(ph2d_nodegraph::external::CURSOR)),
        };
        // The authored point, used when no wire drives the target ports. A connected
        // port wins — the input is the animated answer and the param is the typed one,
        // and "the wire wins" is the same order the panel shows a driven row in.
        let (px_, py_) = (ctx.param("target_x"), ctx.param("target_y"));
        let (tx, ty) = match aim {
            // Broadcast: one point for the whole field. A length-1 column is what
            // `target_at` already treats as "the same target for everybody", so the
            // three modes meet in ONE aiming loop instead of three.
            Some([x, y]) => (vec![x], vec![y]),
            None => {
                let (wx, wy) = (
                    scalar_col(ctx.input(1), VALUE_COL),
                    scalar_col(ctx.input(2), VALUE_COL),
                );
                // Empty ⇒ nothing is wired ⇒ the typed coordinate. Per axis, because
                // wiring only `y` and typing `x` is a thing an artist does.
                (
                    if wx.is_empty() { vec![px_] } else { wx },
                    if wy.is_empty() { vec![py_] } else { wy },
                )
            }
        };
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        // Where each element ALREADY points — the far end of the blend. Absent is
        // `0.0`, the same identity the kernel declares for the column.
        let base: Vec<f32> = match input.get("rot") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        // Pure per-instance map → parallel above the threshold
        // (bit-identical, no reduction). GPU/M5 Fase 0.
        let rot: Vec<f32> = par_build(n, |i| {
            let dx = target_at(&tx, i) - p[i][0];
            let dy = target_at(&ty, i) - p[i][1];
            let aimed = atan2_approx(dy, dx) * RAD_TO_DEG + offset;
            let w = falloff_at(input, i) * strength;
            blend_aim(base.get(i).copied().unwrap_or(0.0), aimed, w)
        });
        // Copy every column through, then set the freshly-aimed rotation.
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            if name != "rot" {
                out.set(name.clone(), col.clone());
            }
        }
        out.set("rot", Column::Scalar(rot));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionLookAt))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Look At",
            // Transform blue: it writes the rotation channel.
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Aim At",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Point", "Object", "Cursor"],
        },
    },
    // The named target. A TEXT param, so it never touches the frozen manifest
    // (doc 33) — the same channel and the same picker `motion.path` uses.
    ParamUiHint {
        param: "target",
        label: "Object",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Source,
    },
    ParamUiHint {
        param: "target_x",
        label: "Target X",
        min: -20.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "target_y",
        label: "Target Y",
        min: -20.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // ⚠️ The track starts at `0` because `0` is the OFF: at zero weight the node
    // is a pass-through, and a floor here would hide the neutral.
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        // ⚠️ `Angle`, not `Slider`: this is degrees, and the widget answering the
        // unit question first is what makes a `deg` suffix impossible to get wrong
        // (doc 88 — `unit_of` lets no table entry contradict an `Angle`).
        widget: ParamWidget::Angle,
    },
];

/// The picker is offered only where it means something: in `Point` mode the target
/// is the value inputs and in `Cursor` mode it is the mouse, so an object name there
/// is a control the cook will never read.
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: "target",
        when: "mode",
        values: &[1],
    },
    // The coordinates belong to the mode that uses them. In Object/Cursor mode the
    // target comes from elsewhere, so a pair of number rows there would be two knobs
    // the cook never reads.
    ParamGate {
        param: "target_x",
        when: "mode",
        values: &[0],
    },
    ParamGate {
        param: "target_y",
        when: "mode",
        values: &[0],
    },
];

/// They are world coordinates, so the panel reads them in the artist's unit (doc 88).
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "target_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "target_y",
        unit: ParamUnit::Length,
    },
];

#[cfg(test)]
#[path = "aim_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "target_mode_tests.rs"]
mod target_mode_tests;

#[cfg(test)]
#[path = "falloff_tests.rs"]
mod falloff_tests;
