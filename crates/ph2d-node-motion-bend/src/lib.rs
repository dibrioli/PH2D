//! `motion.bend` — a **bend deformer**: wrap the layout around an arc along the X
//! axis, so a straight row curls into a rainbow and the centre stays put (Motion
//! Nodes M3, deformers — doc 01 §3 / doc 20). This is the classic Maya/Blender/
//! Cinema4D Bend: the object's X extent maps onto a circular arc of total angle
//! `angle`, each element's along-axis distance becoming an arc angle, its
//! perpendicular offset kept as the radial distance. The rim curls up, the pivot
//! column holds — the second deformer of a different family from `motion.twist`
//! (which rotates about a point).
//!
//! **The strength is a value input** (`amount`, the value domain — doc 12), so it
//! can be ANIMATED: wire a `value.lfo` and the layout curls and uncurls (the boot
//! scene does this). `amount` scales `angle`; **unconnected it reads as `1.0`**
//! (full static bend). `amount = 0` (or `angle = 0`) is the identity. Positive
//! curls up, negative curls down. Falloff-masked like every behaviour. `Pure`.
//!
//! Transcendental-free (HR-5): the arc uses the corrected-parabolic `cos/sin`
//! (`trig`, in cycles) and the constant `π` (a literal, not a call); `√` is
//! IEEE-deterministic. Arc length is preserved: an element at X distance `d` from
//! the pivot travels an arc of length `d`, so the bend never stretches the layout.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{
    ColumnAccess, ColumnBinding, GpuKernel, ReduceOp, ReduceSpec,
};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::f32::consts::{PI, TAU};

mod trig;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `amount` input (mirror of `ph2d_node_pulse_counter::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

const VALUE_COL: &str = "v";
/// Below this total rim angle (radians) the bend is the identity (no arc to wrap
/// onto), so `motion.bend` never divides by (near-)zero curvature.
const MIN_ANGLE_RAD: f32 = 1e-4;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.bend"),
    name: "motion.bend",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The 0..1 (or ±) strength multiplier — a value, so it can be animated.
        // Optional: unconnected reads as 1.0 (full static bend).
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
        // Total arc angle over the layout's X extent, in degrees (the rim's turn).
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

/// The whole-stream reduction this deformer needs: the layout's **X extent**
/// about the pivot (GPU/M5, the deformer channel — `ph2d_nodegraph::reduce_meta`).
///
/// This is the `x_extent` fold at the top of [`bend`], declared so the sequencer
/// can run it on the device before the per-element pass. ⚠️ **`Max` over an
/// expression built only from subtraction and `abs` — so this one is BIT-EXACT
/// against the CPU**: `Max` is associative and exact in any evaluation order, and
/// there is no multiply-add here for a device to contract into an FMA. (Its
/// sibling `motion.twist` folds a radius, which has a product, and is an ε.)
static REDUCES: &[ReduceSpec] = &[ReduceSpec {
    name: "x_extent",
    column: "P",
    dim: Dim::Vec2,
    port: 0,
    op: ReduceOp::Max,
    value: "abs(v.x - params.pivot_x)",
    params: &["pivot_x"],
    // The same identity the `P` binding declares — an absent `P` is materialised
    // as the origin by BOTH paths, so both measure the extent of a layout of
    // origins (which is `|pivot_x|`, not "no extent").
    identity: [0.0; 4],
}];

/// The device form of [`bend`] (GPU/M5). One invocation per element, reading the
/// layout's X extent from the reduction above.
///
/// ⚠️ **The trig is the CPU's polynomial, ported operation for operation** — not
/// WGSL's `sin`/`cos`. The CPU is transcendental-free by HR-5 (the corrected
/// parabolic sine, ~0.09% off true trig), so calling the device's real `sin`
/// here would not be a tighter ε, it would be a *different curve*: the arc would
/// visibly differ from the canonical one wherever the approximation does.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let bd_p = read_in_P(i);\n\
        let bd_dx = bd_p.x - params.pivot_x;\n\
        let bd_dy = bd_p.y - params.pivot_y;\n\
        let bd_theta = params.angle * read_amount_v(i) * 3.1415927 / 180.0;\n\
        let bd_ext = reduce_x_extent();\n\
        var bd_bent = vec2<f32>(bd_dx, bd_dy);\n\
        if (bd_ext >= 1e-4 && abs(bd_theta) >= 1e-4) {\n\
        \x20   let bd_k = bd_theta / bd_ext;\n\
        \x20   let bd_r = 1.0 / bd_k;\n\
        \x20   let bd_ph = (bd_k * bd_dx) / 6.2831855;\n\
        \x20   let bd_c = bend_sin_cycles(bd_ph + 0.25);\n\
        \x20   let bd_s = bend_sin_cycles(bd_ph);\n\
        \x20   bd_bent = vec2<f32>((bd_r - bd_dy) * bd_s, bd_r * (1.0 - bd_c) + bd_dy * bd_c);\n\
        }\n\
        let bd_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        write_P(i, vec2<f32>(\n\
        \x20   bd_p.x + (params.pivot_x + bd_bent.x - bd_p.x) * bd_f,\n\
        \x20   bd_p.y + (params.pivot_y + bd_bent.y - bd_p.y) * bd_f));\n",
    wgsl_lib: "\
        // The corrected parabolic sine at `phase` CYCLES — the port of `trig.rs`.\n\
        fn bend_sin_cycles(phase: f32) -> f32 {\n\
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
            // ReadWrite, not ReadWriteExisting: the CPU materialises an absent
            // `P` from the origin and always emits one (`out.set("P", …)`).
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
            // The `amount_at` rule, declared: absent reads 1.0 (full static
            // bend), length-1 broadcasts, length-N is per element.
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

fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The `amount` multiplier for element `i`: **unconnected (empty) → 1.0**;
/// length-1 broadcasts; length-N is per-element.
fn amount_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => 1.0,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(0.0),
    }
}

/// Bend `base` about `pivot`. The X extent maps onto an arc of total angle
/// `angle_deg · amount`; element `i` at `(dx, dy)` (relative to the pivot) wraps to
/// `((R − dy)·sinθ, R·(1 − cosθ) + dy·cosθ)` where `θ = angle · dx / x_extent` and
/// `R = x_extent / angle` — a rotation-free arc-wrap that preserves arc length.
fn bend(
    base: &[[f32; 2]],
    pivot: [f32; 2],
    angle_deg: f32,
    amount: &[f32],
    falloff: &[f32],
) -> Vec<[f32; 2]> {
    // The layout's half-extent along X (the rim distance) — the arc is scaled to it.
    let x_extent = base
        .iter()
        .map(|p| (p[0] - pivot[0]).abs())
        .fold(0.0_f32, f32::max);
    // `x_extent` is a max-reduction across all instances (kept serial above);
    // given it, output element `i` is a pure per-instance map → parallel above
    // the threshold (bit-identical, no reduction). GPU/M5 Fase 0.
    par_build(base.len(), |i| {
        let p = base[i];
        let dx = p[0] - pivot[0];
        let dy = p[1] - pivot[1];
        let theta_max = angle_deg * amount_at(amount, i) * PI / 180.0;
        let bent = if x_extent < MIN_ANGLE_RAD || theta_max.abs() < MIN_ANGLE_RAD {
            [dx, dy] // no extent / no angle → identity
        } else {
            let k = theta_max / x_extent; // curvature (rad per world unit)
            let r = 1.0 / k; // radius of the spine arc
            let (c, s) = cos_sin_cycles((k * dx) / TAU); // cos/sin of the arc angle
            [(r - dy) * s, r * (1.0 - c) + dy * c]
        };
        let f = falloff.get(i).copied().unwrap_or(1.0).clamp(0.0, 1.0);
        // Blend from the original toward the bent position by the falloff.
        [
            p[0] + (pivot[0] + bent[0] - p[0]) * f,
            p[1] + (pivot[1] + bent[1] - p[1]) * f,
        ]
    })
}

struct MotionBend;

impl NodeOp for MotionBend {
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
        let moved = bend(&base, pivot, angle, &amount, &falloff);
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
    reg.register(Box::new(MotionBend))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Bend",
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
        min: -270.0,
        max: 270.0,
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

    /// A straight horizontal row bends into an arc: the pivot column (dx=0) stays
    /// put, and the rim (max dx) curls UP (+y for a positive angle) and IN (its x
    /// shrinks). FALSIFICATION of "it does nothing" — a dead bend keeps the row flat.
    #[test]
    fn a_straight_row_bends_into_an_arc() {
        let row = [[-2.0, 0.0], [-1.0, 0.0], [0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
        let out = bend(&row, [0.0, 0.0], 90.0, &[], &[1.0; 5]); // amount empty → 1
        // Centre (dx=0) is unmoved.
        assert!(
            out[2][0].abs() < 1e-4 && out[2][1].abs() < 1e-4,
            "centre holds: {:?}",
            out[2]
        );
        // Rim (+x) curls up and in.
        assert!(out[4][1] > 0.3, "rim curls up (+y): {:?}", out[4]);
        assert!(out[4][0] < 2.0, "rim pulls in (x shrinks): {:?}", out[4]);
        // Symmetry: the -x rim curls up too, mirrored in x.
        assert!(out[0][1] > 0.3, "-x rim curls up: {:?}", out[0]);
        assert!(
            (out[0][0] + out[4][0]).abs() < 1e-3,
            "left/right mirror in x"
        );
    }

    /// Arc length is PRESERVED: the bent rim sits at radius `R` from the arc centre
    /// and its arc length from the centre equals its original X distance. We check a
    /// 180° bend of a unit-extent row folds the two rims onto the SAME point (a
    /// half-circle closes the ends together above the pivot).
    #[test]
    fn the_bend_preserves_arc_length() {
        // A 180° bend wraps the row into a half-circle; the two ±rims meet at the top.
        let row = [[-1.0, 0.0], [1.0, 0.0]];
        let out = bend(&row, [0.0, 0.0], 180.0, &[], &[1.0; 2]);
        // Both rims land at the same x (0) and the same height (the arc diameter).
        assert!(
            (out[0][0] - out[1][0]).abs() < 1e-3,
            "rims meet in x: {:?} {:?}",
            out[0],
            out[1]
        );
        assert!((out[0][1] - out[1][1]).abs() < 1e-3, "rims meet in y");
        assert!(
            out[0][1] > 0.5,
            "and they are lifted above the pivot: {}",
            out[0][1]
        );
    }

    /// `amount` scales the bend and `0` is the identity; a negative amount curls the
    /// row DOWN instead of up.
    #[test]
    fn amount_scales_and_signs_the_bend() {
        let row = [[2.0, 0.0]];
        let flat = bend(&row, [0.0, 0.0], 90.0, &[0.0], &[1.0]);
        assert!(
            (flat[0][0] - 2.0).abs() < 1e-4 && flat[0][1].abs() < 1e-4,
            "amount 0 = identity"
        );
        let up = bend(&row, [0.0, 0.0], 90.0, &[1.0], &[1.0]);
        let down = bend(&row, [0.0, 0.0], 90.0, &[-1.0], &[1.0]);
        assert!(up[0][1] > 0.3, "positive curls up: {:?}", up[0]);
        assert!(down[0][1] < -0.3, "negative curls down: {:?}", down[0]);
    }

    /// A `falloff = 0` element is left flat while its neighbour bends — the mask
    /// gates the deform.
    #[test]
    fn falloff_zero_leaves_an_element_flat() {
        let row = [[2.0, 0.0], [2.0, 0.0]];
        let out = bend(&row, [0.0, 0.0], 90.0, &[], &[1.0, 0.0]);
        assert!(out[1][1].abs() < 1e-4, "masked stays flat: {:?}", out[1]);
        assert!(out[0][1] > 0.3, "focused bends: {:?}", out[0]);
    }

    #[test]
    fn registers_and_resolves() {
        use ph2d_nodegraph::cook::OpResolver;
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionBend as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
        assert!(Ops.resolve(MANIFEST.id).is_some());
    }
}
