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
//! Transcendental-free (HR-5): the heading is `atan2` via a **Rajan rational
//! approximation** (`atan(a) ≈ ¼π·a − a·(a−1)·(0.2447 + 0.0663·a)` for `a ∈ [0,1]`,
//! quadrant-folded), ~0.0015 rad (0.09°) off true `atan2` using only multiply/add —
//! well under a pixel of orientation error. `Pure` (no clock, no state; the target
//! input carries the animation).

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
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
    ],
    lowerings: &[LoweringKind::Cpu],
};

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
        write_rot(i, la_atan2(la_dy, la_dx) * 57.29578 + params.offset);\n",
    wgsl_lib: "\
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
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["offset"],
    count_law: None,
    applicable: None,
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
        let tx = scalar_col(ctx.input(1), VALUE_COL);
        let ty = scalar_col(ctx.input(2), VALUE_COL);
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        // Pure per-instance map → parallel above the threshold
        // (bit-identical, no reduction). GPU/M5 Fase 0.
        let rot: Vec<f32> = par_build(n, |i| {
            let dx = target_at(&tx, i) - p[i][0];
            let dy = target_at(&ty, i) - p[i][1];
            atan2_approx(dy, dx) * RAD_TO_DEG + offset
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
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "offset",
    label: "Offset",
    min: -180.0,
    max: 180.0,
    step: 1.0,
    widget: ParamWidget::Slider,
}];

#[cfg(test)]
mod tests {
    use super::*;

    /// Aim `input`'s `rot` at a target field `(tx, ty)` with `offset`, directly via
    /// the op (no cook needed — a target field is just two scalar columns).
    fn aim(input: Stream, tx: &[f32], ty: &[f32], offset: f32) -> Vec<f32> {
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        (0..n)
            .map(|i| {
                let dx = target_at(tx, i) - p[i][0];
                let dy = target_at(ty, i) - p[i][1];
                atan2_approx(dy, dx) * RAD_TO_DEG + offset
            })
            .collect()
    }

    /// The `atan2` approximation matches true `atan2` at the cardinals and a few
    /// obliques, well within 0.003 rad — checked against KNOWN constants (no std
    /// `atan2` call, so the test stays transcendental-free too).
    #[test]
    fn atan2_approx_matches_true_atan2() {
        let cases = [
            (0.0, 1.0, 0.0),                // +x → 0
            (1.0, 1.0, FRAC_PI_4),          // 45°
            (1.0, 0.0, FRAC_PI_2),          // +y → 90°
            (1.0, -1.0, 3.0 * FRAC_PI_4),   // 135°
            (0.0, -1.0, PI),                // -x → 180°
            (-1.0, -1.0, -3.0 * FRAC_PI_4), // -135°
            (-1.0, 0.0, -FRAC_PI_2),        // -90°
            (-1.0, 1.0, -FRAC_PI_4),        // -45°
            (1.0, 2.0, 0.4636476),          // atan(0.5)
            (2.0, 1.0, 1.1071488),          // atan(2)
        ];
        for (y, x, want) in cases {
            let got = atan2_approx(y, x);
            assert!(
                (got - want).abs() < 0.003,
                "atan2({y},{x}) = {got}, want {want}"
            );
        }
    }

    /// The origin (target on the element) is safe — no NaN, aim stays 0.
    #[test]
    fn a_coincident_target_is_zero_not_nan() {
        assert_eq!(atan2_approx(0.0, 0.0), 0.0);
    }

    /// Each element aims its `rot` (degrees) at the target. Two elements either
    /// side of a target at the origin (unconnected) point in opposite ±x directions.
    #[test]
    fn each_element_aims_its_rotation_at_the_target() {
        let two = Stream::new(2).with("P", Column::Vec2(vec![[-1.0, 0.0], [1.0, 0.0]]));
        let rot = aim(two, &[], &[], 0.0); // empty target → origin
        assert!(rot[0].abs() < 0.2, "left (at -1) aims +x (0°): {}", rot[0]);
        assert!(
            (rot[1].abs() - 180.0).abs() < 0.2,
            "right (at +1) aims -x (180°): {}",
            rot[1]
        );
    }

    /// `offset` rotates the aim: +90 makes an element face across the target.
    #[test]
    fn offset_rotates_the_aim() {
        let one = Stream::new(1).with("P", Column::Vec2(vec![[-1.0, 0.0]])); // faces +x (0°)
        let rot = aim(one, &[], &[], 90.0);
        assert!(
            (rot[0] - 90.0).abs() < 0.2,
            "0° + offset 90 = 90°: {}",
            rot[0]
        );
    }

    /// An animated target (a value field) turns the aim: the same element faces
    /// +x, then +y, as the target moves from the right to above it.
    #[test]
    fn a_moving_target_turns_the_aim() {
        let p = || Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
        let right = aim(p(), &[5.0], &[0.0], 0.0); // target to the right → 0°
        let up = aim(p(), &[0.0], &[5.0], 0.0); // target above → 90°
        assert!(right[0].abs() < 0.2, "target right → 0°: {}", right[0]);
        assert!((up[0] - 90.0).abs() < 0.2, "target up → 90°: {}", up[0]);
    }

    /// End to end through the cook: the op copies P through and writes `rot`.
    #[test]
    fn registers_and_writes_the_rotation_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.look_at.test.src"),
            name: "motion.look_at.test.src",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                // One element at (-2, 0): aims +x (0°) at the unconnected origin.
                ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[-2.0, 0.0]])));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionLookAt),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.look_at.test.src");
        let la = g.add_node("motion.look_at");
        g.connect(Edge {
            from: (src, 0),
            to: (la, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, la, 0.0).unwrap();
        let s = out[0].as_stream();
        match s.get("rot").unwrap() {
            Column::Scalar(v) => assert!(v[0].abs() < 0.2, "aims +x at the origin: {}", v[0]),
            _ => panic!("rot"),
        }
        assert!(s.get("P").is_some(), "P passes through");
    }
}
