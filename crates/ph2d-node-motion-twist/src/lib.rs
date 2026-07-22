//! `motion.twist` — a **twist deformer**: rotate each element about a pivot by an
//! angle that grows with its distance from the pivot (Motion Nodes M3, deformers —
//! doc 01 §3 / doc 18). This is the classic Maya/Blender/Houdini Twist: the centre
//! stays put and the rim spins the full `angle`, so a straight row curls into a
//! spiral and a spiral coils tighter. It is the first NON-rigid transform in the
//! module — unlike `motion.rotate` (a uniform turn), the rotation here is a
//! function of position, so it DEFORMS rather than moves.
//!
//! **The strength is a value input** (`amount`, the value domain — doc 12), so it
//! can be ANIMATED: wire a `value.lfo` to `amount` and the whole layout coils and
//! uncoils in time (the boot scene does exactly this). `amount` is the `0..1`
//! multiplier on `angle`; **unconnected it reads as `1.0`** (full static twist, so
//! a bare Twist is visible), length-1 broadcasts, length-N is per-element. The rim
//! is found from the actual max radius each frame, so the twist is scale-independent
//! (the outermost element always gets `angle · amount`).
//!
//! Falloff-masked like every behaviour (a focus field limits which elements
//! deform). `Pure` — no clock, no state; the animation comes from the `amount`
//! input, not the playhead. Transcendental-free (HR-5): the corrected-parabolic
//! `cos/sin` (`trig`, in cycles) and `√` (IEEE-deterministic).

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, ReduceOp, ReduceSpec};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `amount` input — the per-instance scalar field on the
/// `v` column (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this
/// stays a leaf drop-crate).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value stream's column.
const VALUE_COL: &str = "v";
/// Degrees per full turn — the exact divisor into the cycle-based trig's unit
/// (IEEE division is correctly rounded → deterministic, HR-5).
const DEG_PER_TURN: f32 = 360.0;
/// Below this the layout is treated as a single point (no rim to normalise
/// against), so `motion.twist` never divides by (near-)zero.
const MIN_RADIUS: f32 = 1e-6;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.twist"),
    name: "motion.twist",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The 0..1 strength multiplier — a value, so it can be animated. Optional:
        // unconnected reads as 1.0 (full static twist).
        PortSpec {
            name: "amount",
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
        // Max twist at the rim, in degrees (the centre gets 0).
        ParamSpec {
            name: "angle",
            default: 90.0,
        },
        ParamSpec {
            name: "pivot_x",
            default: 0.0,
        },
        ParamSpec {
            name: "pivot_y",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The whole-stream reduction this deformer needs: the **rim radius** about the
/// pivot (GPU/M5, the deformer channel — `ph2d_nodegraph::reduce_meta`).
///
/// This is the `r_max` fold at the top of [`twist`], declared so the sequencer
/// can run it on the device before the per-element pass. ⚠️ The `.max(MIN_RADIUS)`
/// floor is **NOT** part of the reduction — it is applied by the consumer, on
/// both paths, because it is a guard about *what the answer is used for* (a
/// division), not about what was measured.
///
/// ⚠️ **An ε, not bit-exact, and the reason is one multiply.** `Max` itself is
/// exact in any order (its sibling `motion.bend` folds `|x − pivot|` and IS
/// bit-exact), but the value folded here is `√(dx² + dy²)`: a device may contract
/// `dx*dx + dy*dy` into an FMA where the host does not, so the two can differ in
/// the last ulps before the fold ever sees them.
static REDUCES: &[ReduceSpec] = &[ReduceSpec {
    name: "r_max",
    column: "P",
    dim: Dim::Vec2,
    port: 0,
    op: ReduceOp::Max,
    value: "sqrt((v.x - params.pivot_x) * (v.x - params.pivot_x) \
            + (v.y - params.pivot_y) * (v.y - params.pivot_y))",
    params: &["pivot_x", "pivot_y"],
    // The same identity the `P` binding declares (see `motion.bend`'s note).
    identity: [0.0; 4],
}];

/// The device form of [`twist`] (GPU/M5). One invocation per element, reading
/// the rim radius from the reduction above.
///
/// ⚠️ **The trig is the CPU's polynomial, ported operation for operation** — see
/// the same note on `motion.bend`: HR-5 makes the canonical path
/// transcendental-free, so the device's real `sin` would be a *different curve*,
/// not a tighter ε.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let tw_p = read_in_P(i);\n\
        let tw_dx = tw_p.x - params.pivot_x;\n\
        let tw_dy = tw_p.y - params.pivot_y;\n\
        let tw_r = sqrt(tw_dx * tw_dx + tw_dy * tw_dy);\n\
        // The MIN_RADIUS floor belongs to the CONSUMER, not the reduction.\n\
        let tw_rmax = max(reduce_r_max(), 1e-6);\n\
        let tw_deg = params.angle * read_amount_v(i) * (tw_r / tw_rmax);\n\
        let tw_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let tw_ph = tw_deg / 360.0;\n\
        let tw_c = twist_sin_cycles(tw_ph + 0.25);\n\
        let tw_s = twist_sin_cycles(tw_ph);\n\
        let tw_rx = params.pivot_x + (tw_c * tw_dx - tw_s * tw_dy);\n\
        let tw_ry = params.pivot_y + (tw_s * tw_dx + tw_c * tw_dy);\n\
        write_P(i, vec2<f32>(\n\
        \x20   tw_p.x + (tw_rx - tw_p.x) * tw_f,\n\
        \x20   tw_p.y + (tw_ry - tw_p.y) * tw_f));\n",
    wgsl_lib: "\
        // The corrected parabolic sine at `phase` CYCLES — the port of `trig.rs`.\n\
        fn twist_sin_cycles(phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
            var p: f32;\n\
            if (f < 0.5) {\n\
                let u = f * 2.0;\n\
                p = 4.0 * u * (1.0 - u);\n\
            } else {\n\
                let u = (f - 0.5) * 2.0;\n\
                p = -4.0 * u * (1.0 - u);\n\
            }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 1,
        },
    ],
    params: &["angle", "pivot_x", "pivot_y"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The `amount` multiplier for instance `i`: **unconnected (empty) → 1.0** (full
/// static twist); length-1 broadcasts; length-N is element-wise.
fn amount_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => 1.0,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(0.0),
    }
}

/// Rotate `p` about `pivot` by `deg` degrees, then blend from the original toward
/// the rotated position by `f` (the falloff): `f = 0` keeps `p`, `f = 1` takes the
/// full twist.
fn twist_point(p: [f32; 2], pivot: [f32; 2], deg: f32, f: f32) -> [f32; 2] {
    let (c, s) = cos_sin_cycles(deg / DEG_PER_TURN);
    let dx = p[0] - pivot[0];
    let dy = p[1] - pivot[1];
    let rx = pivot[0] + (c * dx - s * dy);
    let ry = pivot[1] + (s * dx + c * dy);
    [p[0] + (rx - p[0]) * f, p[1] + (ry - p[1]) * f]
}

/// Twist `base` about `pivot`: element `i` rotates by `angle · amount_i · (r_i /
/// r_max)`, so the rim turns the full `angle` and the centre not at all.
fn twist(
    base: &[[f32; 2]],
    pivot: [f32; 2],
    angle: f32,
    amount: &[f32],
    falloff: &[f32],
) -> Vec<[f32; 2]> {
    // The rim radius this frame — the twist is normalised against it, so it is
    // scale-independent (outermost always gets the full `angle`).
    let r_max = base
        .iter()
        .map(|p| {
            let (dx, dy) = (p[0] - pivot[0], p[1] - pivot[1]);
            (dx * dx + dy * dy).sqrt()
        })
        .fold(0.0_f32, f32::max)
        .max(MIN_RADIUS);
    // `r_max` is a max-reduction across all instances (kept serial above); given
    // it, output element `i` is a pure per-instance map → parallel above the
    // threshold (bit-identical, no reduction). GPU/M5 Fase 0.
    par_build(base.len(), |i| {
        let p = base[i];
        let (dx, dy) = (p[0] - pivot[0], p[1] - pivot[1]);
        let r = (dx * dx + dy * dy).sqrt();
        let deg = angle * amount_at(amount, i) * (r / r_max);
        let f = falloff.get(i).copied().unwrap_or(1.0).clamp(0.0, 1.0);
        twist_point(p, pivot, deg, f)
    })
}

struct MotionTwist;

impl NodeOp for MotionTwist {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let angle = ctx.param("angle");
        let pivot = [ctx.param("pivot_x"), ctx.param("pivot_y")];
        let amount: Vec<f32> = match ctx.input(1).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let input = ctx.input(0);
        let n = input.count();
        let base: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        // Pure per-instance map → parallel above the threshold
        // (bit-identical, no reduction). GPU/M5 Fase 0.
        let falloff: Vec<f32> = par_build(n, |i| falloff_at(input, i));
        let moved = twist(&base, pivot, angle, &amount, &falloff);
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            if name != "P" {
                out.set(name.clone(), col.clone());
            }
        }
        out.set("P", Column::Vec2(moved));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTwist))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Twist",
            // Transform blue: a spatial deformer.
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // GPU/M5: the kernel and the whole-stream reduction it reads. Side metadata
    // on the registry (ADR-0126) — the frozen node contract is untouched.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_reduces(MANIFEST.id, REDUCES);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "angle",
        label: "Angle",
        min: -720.0,
        max: 720.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pivot_x",
        label: "Pivot X",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pivot_y",
        label: "Pivot Y",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn radius(p: [f32; 2]) -> f32 {
        (p[0] * p[0] + p[1] * p[1]).sqrt()
    }

    /// The twist PRESERVES each element's radius (it is a rotation about the
    /// pivot) — only the angle changes. This is the deformer's defining property.
    #[test]
    fn it_preserves_the_radius_of_every_element() {
        // A row of points at increasing radius on the +x axis.
        let base = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];
        let f = [1.0; 4];
        let out = twist(&base, [0.0, 0.0], 180.0, &[], &f); // amount empty → 1.0
        for (b, o) in base.iter().zip(out.iter()) {
            // Tolerance covers the ~0.09% parabolic-trig wobble (never accumulates —
            // each frame rotates the pristine P by an absolute angle).
            assert!(
                (radius(*b) - radius(*o)).abs() < 5e-3,
                "radius kept: {b:?}→{o:?}"
            );
        }
    }

    /// The SIGNATURE of a twist (vs a uniform rotate): the rim turns MORE than the
    /// interior. The outermost point rotates by the full `angle`; a mid point by
    /// `angle·(r/r_max)`. FALSIFICATION of "it's just a rotate" (which would turn
    /// every point equally).
    #[test]
    fn the_rim_turns_more_than_the_interior() {
        // Points at r = 1 and r = 4 on +x. angle 90°, amount 1.
        let base = [[1.0, 0.0], [4.0, 0.0]];
        let out = twist(&base, [0.0, 0.0], 90.0, &[], &[1.0; 2]);
        // Outer (r=4=r_max) rotates the full 90° → lands on +y: (0, 4).
        assert!(
            out[1][0].abs() < 0.05 && (out[1][1] - 4.0).abs() < 0.05,
            "rim at 90°: {:?}",
            out[1]
        );
        // Inner (r=1) rotates 90°·(1/4) = 22.5° → still mostly +x, small +y.
        assert!(out[0][0] > out[0][1], "interior turns less: {:?}", out[0]);
        assert!(out[0][1] > 0.0, "but it does turn: {:?}", out[0]);
    }

    /// `amount` scales the twist: `0` is identity (no deform), `0.5` is half. It is
    /// the animatable strength (a `value.lfo` drives it in the scene).
    #[test]
    fn amount_scales_the_twist_and_zero_is_identity() {
        let base = [[2.0, 0.0]];
        let none = twist(&base, [0.0, 0.0], 90.0, &[0.0], &[1.0]);
        assert!(
            (none[0][0] - 2.0).abs() < 1e-4 && none[0][1].abs() < 1e-4,
            "amount 0 = identity"
        );
        let half = twist(&base, [0.0, 0.0], 90.0, &[0.5], &[1.0]);
        // 90°·0.5 = 45° → (2·cos45, 2·sin45) ≈ (1.414, 1.414).
        assert!(
            (half[0][0] - 1.414).abs() < 0.03 && (half[0][1] - 1.414).abs() < 0.03,
            "half: {:?}",
            half[0]
        );
    }

    /// A `falloff = 0` element is left untouched (the mask gates the deform), while
    /// its neighbour twists — the family-wide focus behaviour.
    #[test]
    fn falloff_zero_leaves_an_element_untouched() {
        let base = [[2.0, 0.0], [2.0, 0.0]];
        let out = twist(&base, [0.0, 0.0], 90.0, &[], &[1.0, 0.0]);
        assert!(
            (out[1][0] - 2.0).abs() < 1e-4 && out[1][1].abs() < 1e-4,
            "masked stays put"
        );
        assert!(out[0][1] > 1.0, "focused twists");
    }

    #[test]
    fn registers_and_resolves() {
        use ph2d_nodegraph::cook::OpResolver;
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionTwist as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
        assert!(Ops.resolve(MANIFEST.id).is_some());
    }
}
