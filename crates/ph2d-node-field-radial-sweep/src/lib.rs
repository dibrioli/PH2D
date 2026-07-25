#![forbid(unsafe_code)]
//! `field.radial_sweep` — a Motion **focus field: a bounded angular SECTOR** (a
//! radar / pie wedge) keyed by POSITION. It writes the same multiplicative `falloff`
//! mask `motion.falloff`, `field.index_range` and `field.box` do (the sister-of-`accel`
//! contract, §1.2), so fields compose. Where `field.box` is an axis-aligned rectangle,
//! this is the family's **angular** primitive: a point's weight depends on its ANGLE
//! about `center` (inside `[start_angle, end_angle]`) AND its distance (inside `radius`).
//!
//! **The wedge REPEATS.** `repetitions = N` tiles the sector `N` times evenly around the
//! circle — a fan, a star of beams, a clock (Cavalry Sweep). `repetitions = 1` is a
//! single wedge. Both angular and radial edges get a **soft** ramp (a fraction `[0,1]`
//! of the extent — one dimensionless knob, because the two edges live in different units,
//! degrees vs world), shaped by the same 4-curve (Linear/Quad/Smooth/Smoother) as the box.
//!
//! **Transcendental-free (HR-5).** The angular test never calls `atan2` **per instance**:
//! a monotone **pseudo-angle** (the "diamond angle" — octant-linear, only `abs`/`+`/`/`)
//! maps a direction to `[0, 4)` preserving order, so membership in a sector is exact and
//! the soft ramp is linear in pseudo-space. The sector's pseudo-bounds are computed from
//! `start`/`end` via the SAME parabolic `cos_sin_cycles` the box uses for its rotation
//! (a per-cook constant), so the CPU and GPU kernels share one polynomial and parity
//! holds within ULPs. `sqrt` (radial distance) is IEEE-correctly-rounded on both, so it
//! is bit-safe too. This is a **spatial** field: it has Coordinates (`center`/`rotation`/
//! the gizmo-driven `radius`) and is the second field type the canvas gizmo drives (D9).
//!
//! **Remapping is a DOWNSTREAM node** (D1): inner-offset / contour / clamp / invert-graph
//! belong to `field.remap`, not here — this node only shapes the raw mask, then `min`s the
//! angular and radial ramps (the intersection of the wedge and the disk, exactly as the
//! box `min`s its two axis bands). The neutral is `end − start ≥ 360` (a full disk) with a
//! `radius` larger than the scene and `soft = 0` ⇒ mask `1` everywhere ⇒ the `falloff`
//! column is multiplied by the identity, byte-unchanged (D12).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{par_build, Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.radial_sweep"),
    name: "field.radial_sweep",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "radius",
            default: 10.0,
        },
        ParamSpec {
            name: "start_angle",
            default: 0.0,
        },
        ParamSpec {
            name: "end_angle",
            default: 60.0,
        },
        ParamSpec {
            name: "repetitions",
            default: 1.0,
        },
        ParamSpec {
            name: "soft",
            default: 0.15,
        },
        ParamSpec {
            name: "center_x",
            default: 0.0,
        },
        ParamSpec {
            name: "center_y",
            default: 0.0,
        },
        ParamSpec {
            name: "rotation",
            default: 0.0,
        },
        ParamSpec {
            name: "curve",
            default: 2.0,
        },
        ParamSpec {
            name: "invert",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// An edge curve on a pre-clamped `s ∈ [0,1]` — the SAME set as the other fields
/// (HR-5). `0` Linear · `1` Quad · `2` Smooth · `3` Smoother. Monotone, endpoint-exact.
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        1 => s * s,                                     // Quad
        2 => s * s * (3.0 - 2.0 * s),                   // Smooth (smoothstep)
        3 => s * s * s * (s * (s * 6.0 - 15.0) + 10.0), // Smoother (smootherstep)
        _ => s,                                         // Linear
    }
}

/// The plateau-and-ramp: `1` for `|d| ≤ half − soft`, ramping to `0` at `|d| = half`;
/// `soft = 0` is a hard edge; `soft` is clamped so it can never exceed `half`. `half ≤ 0`
/// degenerates to empty (only `d = 0` is inside). Identical to `field.box`'s `edge_ramp`,
/// used here for BOTH the radial edge (`d = r`, world units) and the angular edge
/// (`d = pseudo-distance`, dimensionless).
fn edge_ramp(d: f32, half: f32, soft: f32) -> f32 {
    let a = d.abs();
    let s = soft.max(0.0).min(half);
    if s > 0.0 {
        ((half - a) / s).clamp(0.0, 1.0)
    } else if a <= half {
        1.0
    } else {
        0.0
    }
}

/// The **diamond pseudo-angle** of `(x, y)` in `[0, 4)`, monotone with the true angle
/// (CCW from +X): `0`→`0`, `+X`→`0`, `+Y`→`1`, `−X`→`2`, `−Y`→`3`. Only `abs`/`+`/`/`
/// and comparisons — no `atan2`, so it is transcendental-free (HR-5) and bit-identical
/// CPU↔GPU. The origin (`|x|+|y| = 0`) returns `0` (guards the `0/0`); it is a single
/// degenerate texel a sweep has no defined angle for.
fn pseudo_angle(x: f32, y: f32) -> f32 {
    let denom = x.abs() + y.abs();
    if denom == 0.0 {
        return 0.0;
    }
    let p = y / denom; // [-1, 1]
    if x < 0.0 {
        2.0 - p
    } else if y < 0.0 {
        4.0 + p
    } else {
        p
    }
}

/// The pseudo-angle of a DEGREE angle, via the parabolic trig (a per-cook constant).
/// The `(cos, sin)` are the corrected-parabolic ones the box uses, so CPU and GPU agree;
/// the ~0.09% trig error only nudges the sector edge <0.1° (invisible), and it nudges it
/// the SAME way on both devices.
fn pseudo_of_deg(deg: f32) -> f32 {
    let (c, s) = cos_sin_cycles(deg / 360.0);
    pseudo_angle(c, s)
}

/// `x mod 4` into `[0, 4)` (floor-based, so bit-identical to the WGSL `floor`).
fn wrap04(x: f32) -> f32 {
    x - 4.0 * (x * 0.25).floor()
}

/// `x` folded into `[-period/2, period/2)` (the nearest-repetition distance). Floor-based
/// ⇒ bit-identical CPU↔GPU. `period = 0` never happens (repetitions ≥ 1 ⇒ period ≤ 4).
fn wrap_sym(x: f32, period: f32) -> f32 {
    x - period * (x / period + 0.5).floor()
}

/// The per-cook sector constants derived from the raw params (so the CPU computes them
/// once outside the hot loop; the GPU recomputes the same formula per instance — same
/// inputs, same result, parity holds). `full` is a genuine full disk (`end − start ≥
/// 360`) whose angular ramp is bypassed to `1` (a distance-from-mid model always seams at
/// the antipode; a full circle must skip it — this is what makes the neutral EXACT).
struct Sector {
    pa_mid: f32,
    pa_half: f32,
    period: f32,
    full: bool,
}

fn sector(start_angle: f32, end_angle: f32, repetitions: f32) -> Sector {
    let pa_start = pseudo_of_deg(start_angle);
    let pa_end = pseudo_of_deg(end_angle);
    let span = wrap04(pa_end - pa_start);
    let pa_half = span * 0.5;
    let pa_mid = wrap04(pa_start + pa_half);
    let reps = repetitions.round().max(1.0);
    Sector {
        pa_mid,
        pa_half,
        period: 4.0 / reps,
        full: (end_angle - start_angle).abs() >= 360.0,
    }
}

/// The raw sweep mask (before the `curve` and `invert`) at LOCAL offset `(lx, ly)` from
/// the centre — i.e. the offset already un-rotated into the field's frame. It is the
/// `min` of the radial ramp (inside the disk of `radius`) and the angular ramp (inside the
/// nearest repetition of the sector), each softened by `soft` (a fraction of its extent).
fn sweep_mask(lx: f32, ly: f32, radius: f32, soft: f32, sec: &Sector) -> f32 {
    let r = (lx * lx + ly * ly).sqrt();
    let rad = edge_ramp(r, radius, soft * radius);
    let ang = if sec.full {
        1.0
    } else {
        let pa = pseudo_angle(lx, ly);
        let d = wrap_sym(pa - sec.pa_mid, sec.period);
        edge_ramp(d, sec.pa_half, soft * sec.pa_half)
    };
    rad.min(ang)
}

/// GPU compute kernel (ADR-0126): a straight WGSL port of [`sweep_mask`] × [`curve`]
/// multiplied into the existing `falloff` — same `min`/`max`/`clamp`/`sqrt`/`floor` +
/// polynomials (HR-5), so parity holds within float ULPs. The sector constants are
/// recomputed per instance from the raw params (the CPU hoists them; the device redoes
/// the arithmetic — same formula, same result). `P` reads its `0` identity when absent
/// (the CPU's `unwrap_or([0,0])`); `falloff` `ReadWrite` from the `1.0` identity mirrors
/// the CPU (fields multiply, the column is always written).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let rs_p = read_P(i);\n\
        let rs_dx = rs_p.x - params.center_x;\n\
        let rs_dy = rs_p.y - params.center_y;\n\
        // Rotate the offset by -rotation into the field's local frame.\n\
        let rs_b = rs_cos_sin(params.rotation / 360.0);\n\
        let rs_lx =  rs_dx * rs_b.x + rs_dy * rs_b.y;\n\
        let rs_ly = -rs_dx * rs_b.y + rs_dy * rs_b.x;\n\
        let rs_soft = clamp(params.soft, 0.0, 1.0);\n\
        // Radial ramp: inside the disk of `radius`.\n\
        let rs_r = sqrt(rs_lx * rs_lx + rs_ly * rs_ly);\n\
        let rs_rad = rs_edge_ramp(rs_r, params.radius, rs_soft * params.radius);\n\
        // Angular ramp: inside the nearest repetition of the sector.\n\
        let rs_pa_start = rs_pseudo_of_deg(params.start_angle);\n\
        let rs_pa_end = rs_pseudo_of_deg(params.end_angle);\n\
        let rs_span = rs_wrap04(rs_pa_end - rs_pa_start);\n\
        let rs_pa_half = rs_span * 0.5;\n\
        let rs_pa_mid = rs_wrap04(rs_pa_start + rs_pa_half);\n\
        let rs_reps = max(rs_round(params.repetitions), 1.0);\n\
        let rs_period = 4.0 / rs_reps;\n\
        var rs_ang = 1.0;\n\
        if (abs(params.end_angle - params.start_angle) < 360.0) {\n\
            let rs_pa = rs_pseudo_angle(rs_lx, rs_ly);\n\
            let rs_d = rs_wrap_sym(rs_pa - rs_pa_mid, rs_period);\n\
            rs_ang = rs_edge_ramp(rs_d, rs_pa_half, rs_soft * rs_pa_half);\n\
        }\n\
        let rs_m = rs_curve(i32(rs_round(params.curve)), min(rs_rad, rs_ang));\n\
        var rs_f = rs_m;\n\
        if (params.invert >= 0.5) { rs_f = 1.0 - rs_m; }\n\
        write_falloff(i, read_falloff(i) * rs_f);\n",
    wgsl_lib: "\
        fn rs_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn rs_sin_cycles(phase: f32) -> f32 {\n\
            let ff = phase - floor(phase);\n\
            var p: f32;\n\
            if (ff < 0.5) { let u = ff * 2.0; p = 4.0 * u * (1.0 - u); }\n\
            else { let u = (ff - 0.5) * 2.0; p = -4.0 * u * (1.0 - u); }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n\
        fn rs_cos_sin(phase: f32) -> vec2<f32> {\n\
            return vec2<f32>(rs_sin_cycles(phase + 0.25), rs_sin_cycles(phase));\n\
        }\n\
        fn rs_curve(kind: i32, s: f32) -> f32 {\n\
            if (kind == 1) { return s * s; }\n\
            if (kind == 2) { return s * s * (3.0 - 2.0 * s); }\n\
            if (kind == 3) { return s * s * s * (s * (s * 6.0 - 15.0) + 10.0); }\n\
            return s;\n\
        }\n\
        fn rs_edge_ramp(d: f32, half: f32, soft: f32) -> f32 {\n\
            let a = abs(d);\n\
            let s = min(max(soft, 0.0), half);\n\
            if (s > 0.0) { return clamp((half - a) / s, 0.0, 1.0); }\n\
            return select(0.0, 1.0, a <= half);\n\
        }\n\
        fn rs_pseudo_angle(x: f32, y: f32) -> f32 {\n\
            let denom = abs(x) + abs(y);\n\
            if (denom == 0.0) { return 0.0; }\n\
            let p = y / denom;\n\
            if (x < 0.0) { return 2.0 - p; }\n\
            if (y < 0.0) { return 4.0 + p; }\n\
            return p;\n\
        }\n\
        fn rs_pseudo_of_deg(deg: f32) -> f32 {\n\
            let b = rs_cos_sin(deg / 360.0);\n\
            return rs_pseudo_angle(b.x, b.y);\n\
        }\n\
        fn rs_wrap04(x: f32) -> f32 {\n\
            return x - 4.0 * floor(x * 0.25);\n\
        }\n\
        fn rs_wrap_sym(x: f32, period: f32) -> f32 {\n\
            return x - period * floor(x / period + 0.5);\n\
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
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &[
        "radius",
        "start_angle",
        "end_angle",
        "repetitions",
        "soft",
        "center_x",
        "center_y",
        "rotation",
        "curve",
        "invert",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct FieldRadialSweep;

impl NodeOp for FieldRadialSweep {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let radius = ctx.param("radius");
        let soft = ctx.param("soft").clamp(0.0, 1.0);
        let (cx, cy) = (ctx.param("center_x"), ctx.param("center_y"));
        // The rotation basis and the sector's pseudo-bounds, computed ONCE per cook (a
        // per-cook constant): rotating a world offset by −rotation brings it into the
        // field's local frame, where `start`/`end` define the sector.
        let (rc, rs) = cos_sin_cycles(ctx.param("rotation") / 360.0);
        let sec = sector(
            ctx.param("start_angle"),
            ctx.param("end_angle"),
            ctx.param("repetitions"),
        );
        let curve_kind = ctx.param("curve").round() as i32;
        let invert = ctx.param("invert") >= 0.5;
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            let prev: Option<&[f32]> = match input.get("falloff") {
                Some(Column::Scalar(v)) => Some(v.as_slice()),
                _ => None,
            };
            let positions: &[[f32; 2]] = match input.get("P") {
                Some(Column::Vec2(v)) => v.as_slice(),
                _ => &[],
            };
            let fall = par_build(n, |i| {
                let p = positions.get(i).copied().unwrap_or([0.0, 0.0]);
                let (dx, dy) = (p[0] - cx, p[1] - cy);
                // Rotate the offset by −rotation into the field's local frame.
                let (lx, ly) = (dx * rc + dy * rs, -dx * rs + dy * rc);
                let m = curve(curve_kind, sweep_mask(lx, ly, radius, soft, &sec));
                let f = if invert { 1.0 - m } else { m };
                let base = prev.and_then(|v| v.get(i).copied()).unwrap_or(1.0);
                base * f
            });
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "falloff" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("falloff", Column::Scalar(fall));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via `ph2d-node-sync`
/// codegen) from `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FieldRadialSweep))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Radial Sweep",
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): the sweep's radius (gizmo-driven), the angular sector
/// (start/end in degrees), the repetition count, a normalized softness, a signed centre,
/// a named Curve selector, an Invert checkbox. `soft` is a FRACTION `[0,1]` of the extent
/// (dimensionless — it softens both the angular and the radial edge; see the module doc).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "start_angle",
        label: "Start Angle",
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "end_angle",
        label: "End Angle",
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "repetitions",
        label: "Repetitions",
        min: 1.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "soft",
        label: "Softness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_x",
        label: "Center X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "Center Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "rotation",
        label: "Rotation",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "curve",
        label: "Curve",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Quad", "Smooth", "Smoother"],
        },
    },
    ParamUiHint {
        param: "invert",
        label: "Invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("field.radial_sweep.test.src"),
        name: "field.radial_sweep.test.src",
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
    struct Src(Vec<[f32; 2]>);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(self.0.len()).with("P", Column::Vec2(self.0.clone())));
        }
    }
    struct Ops {
        src: Src,
    }
    impl Ops {
        fn new(pts: Vec<[f32; 2]>) -> Self {
            Ops { src: Src(pts) }
        }
    }
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&self.src),
                t if t == MANIFEST.id => Some(&FieldRadialSweep),
                _ => None,
            }
        }
    }

    fn falloff_of(g: &Graph, ops: &Ops, target: NodeId) -> Vec<f32> {
        let mut cook = Cook::new();
        let out = cook.cook(g, ops, target, 0.0).unwrap();
        match out[0].as_stream().get("falloff").unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("falloff must be a Scalar column"),
        }
    }

    fn chain() -> (Graph, NodeId) {
        let mut g = Graph::new();
        let src = g.add_node("field.radial_sweep.test.src");
        let sw = g.add_node("field.radial_sweep");
        g.connect(Edge {
            from: (src, 0),
            to: (sw, 0),
            delayed: false,
        })
        .unwrap();
        (g, sw)
    }

    // A point on the ray at `deg` degrees, at distance `r` from the origin — the honest
    // input for an angular test (via true trig, NOT the node's parabolic one). Kept well
    // inside the default `radius = 10` so the radial ramp is 1 and we isolate the angle.
    fn ray(deg: f32, r: f32) -> [f32; 2] {
        let a = deg.to_radians();
        [r * a.cos(), r * a.sin()]
    }

    #[test]
    fn default_wedge_is_one_inside_zero_outside() {
        // Default sector [0°, 60°], radius 10, soft 0.15, curve Smooth. A ray at 30°
        // (mid-sector) well inside the radius is fully in → 1; a ray at 120° is outside
        // the wedge → 0; a ray at 30° but BEYOND the radius (r = 20) is outside the disk
        // → 0.
        let (g, sw) = chain();
        let ops = Ops::new(vec![ray(30.0, 5.0), ray(120.0, 5.0), ray(30.0, 20.0)]);
        let got = falloff_of(&g, &ops, sw);
        assert!((got[0] - 1.0).abs() < 1e-4, "mid-sector inside: {}", got[0]);
        assert_eq!(got[1], 0.0, "outside the wedge");
        assert_eq!(got[2], 0.0, "beyond the radius");
    }

    #[test]
    fn rotation_spins_the_wedge() {
        // Rotate the field +90°: the [0,60] wedge now covers [90,150]. A ray at 120° was
        // OUT (default) and is now IN; a ray at 30° was IN and is now OUT.
        let (mut g, sw) = chain();
        g.set_param(sw, "soft", 0.0); // hard edges, so the membership test is crisp
        g.set_param(sw, "rotation", 90.0);
        let ops = Ops::new(vec![ray(120.0, 5.0), ray(30.0, 5.0)]);
        let got = falloff_of(&g, &ops, sw);
        assert_eq!(got[0], 1.0, "120° now inside the rotated wedge");
        assert_eq!(got[1], 0.0, "30° now outside");
    }

    #[test]
    fn repetitions_tile_the_sector_around_the_circle() {
        // 4 repetitions of a [0,30] wedge ⇒ wedges at [0,30], [90,120], [180,210],
        // [270,300]. A ray at 100° falls in the second copy → 1; a ray at 60° is in a GAP
        // between copies → 0.
        let (mut g, sw) = chain();
        g.set_param(sw, "soft", 0.0);
        g.set_param(sw, "end_angle", 30.0);
        g.set_param(sw, "repetitions", 4.0);
        let ops = Ops::new(vec![ray(100.0, 5.0), ray(60.0, 5.0), ray(15.0, 5.0)]);
        let got = falloff_of(&g, &ops, sw);
        assert_eq!(got[0], 1.0, "100° in the 2nd copy");
        assert_eq!(got[1], 0.0, "60° in a gap");
        assert_eq!(got[2], 1.0, "15° in the 1st copy");
    }

    #[test]
    fn full_circle_is_the_identity_disk() {
        // The neutral (D12): end − start ≥ 360 ⇒ a full disk. With a radius larger than
        // the points and soft 0, the mask is 1 everywhere in the disk — including the
        // ANTIPODE of the sector mid (180° from where a wedge would point), which a
        // distance-from-mid model would seam. Fields multiply, so falloff ← 1·falloff.
        let (mut g, sw) = chain();
        g.set_param(sw, "start_angle", 0.0);
        g.set_param(sw, "end_angle", 360.0);
        g.set_param(sw, "radius", 100.0);
        g.set_param(sw, "soft", 0.0);
        // Rays all around the circle, all inside the huge radius.
        let ops = Ops::new(vec![
            ray(0.0, 5.0),
            ray(90.0, 5.0),
            ray(180.0, 5.0),
            ray(270.0, 5.0),
            ray(210.0, 5.0),
        ]);
        assert_eq!(falloff_of(&g, &ops, sw), vec![1.0; 5]);
    }

    #[test]
    fn a_wider_wedge_covers_a_wider_arc() {
        // A 180° sector [0,180] with hard edges: rays in the upper half-plane are in,
        // rays in the lower half-plane are out — the pseudo-angle membership is EXACT
        // across the octant boundaries (0/90/180 all handled by the branch table).
        let (mut g, sw) = chain();
        g.set_param(sw, "soft", 0.0);
        g.set_param(sw, "end_angle", 180.0);
        let ops = Ops::new(vec![
            ray(45.0, 5.0),
            ray(135.0, 5.0),
            ray(225.0, 5.0),
            ray(315.0, 5.0),
        ]);
        assert_eq!(falloff_of(&g, &ops, sw), vec![1.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn radius_clips_the_disk() {
        // Along the mid-ray (30°, always inside the wedge), the mask is a radial edge_ramp:
        // 1 deep inside, 0 at/after the radius. soft 0 ⇒ hard edge at r = radius.
        let (mut g, sw) = chain();
        g.set_param(sw, "soft", 0.0);
        g.set_param(sw, "radius", 6.0);
        let ops = Ops::new(vec![ray(30.0, 1.0), ray(30.0, 5.9), ray(30.0, 6.1)]);
        let got = falloff_of(&g, &ops, sw);
        assert_eq!(got[0], 1.0);
        assert_eq!(got[1], 1.0);
        assert_eq!(got[2], 0.0);
    }

    #[test]
    fn soft_ramps_both_edges_between_zero_and_one() {
        // With soft > 0 the wedge and the disk both feather. A ray just inside the angular
        // edge of the default [0,60] wedge (say 58°) sits on the ramp → strictly between
        // 0 and 1; a ray near the radial edge does too. Curve Linear keeps the ramp plain.
        let (mut g, sw) = chain();
        g.set_param(sw, "curve", 0.0); // Linear, so we can reason about the ramp
        g.set_param(sw, "soft", 0.3);
        let ops = Ops::new(vec![ray(58.0, 5.0), ray(30.0, 9.6)]);
        let got = falloff_of(&g, &ops, sw);
        assert!(got[0] > 0.0 && got[0] < 1.0, "angular ramp: {}", got[0]);
        assert!(got[1] > 0.0 && got[1] < 1.0, "radial ramp: {}", got[1]);
    }

    #[test]
    fn invert_flips_the_mask() {
        // Default wedge: a mid-sector inside point → 1-1 = 0; an outside point → 1-0 = 1.
        let (mut g, sw) = chain();
        g.set_param(sw, "soft", 0.0);
        g.set_param(sw, "invert", 1.0);
        let ops = Ops::new(vec![ray(30.0, 5.0), ray(200.0, 5.0)]);
        assert_eq!(falloff_of(&g, &ops, sw), vec![0.0, 1.0]);
    }

    #[test]
    fn center_moves_the_pivot() {
        // Shift the centre to (5, 0): the sweep now pivots there. A point at (10, 0) has
        // local offset (5, 0) — angle 0°, inside [0,60], within radius 10 → 1. A point at
        // world (0,0) has local (−5,0) — angle 180°, outside the wedge → 0.
        let (mut g, sw) = chain();
        g.set_param(sw, "soft", 0.0);
        g.set_param(sw, "center_x", 5.0);
        let ops = Ops::new(vec![[10.0, 0.0], [0.0, 0.0]]);
        assert_eq!(falloff_of(&g, &ops, sw), vec![1.0, 0.0]);
    }

    #[test]
    fn a_prior_falloff_column_is_multiplied_not_overwritten() {
        static FSRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("field.radial_sweep.test.fsrc"),
            name: "field.radial_sweep.test.fsrc",
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
        struct FSrc;
        impl NodeOp for FSrc {
            fn manifest(&self) -> &'static NodeManifest {
                &FSRC_MAN
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                // 30° inside the default wedge, 200° outside; a prior falloff on each.
                ctx.emit(
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[4.33, 2.5], [-4.7, -1.7]]))
                        .with("falloff", Column::Scalar(vec![0.5, 0.9])),
                );
            }
        }
        struct FOps;
        impl OpResolver for FOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == FSRC_MAN.id => Some(&FSrc),
                    t if t == MANIFEST.id => Some(&FieldRadialSweep),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("field.radial_sweep.test.fsrc");
        let sw = g.add_node("field.radial_sweep");
        g.set_param(sw, "soft", 0.0);
        g.connect(Edge {
            from: (src, 0),
            to: (sw, 0),
            delayed: false,
        })
        .unwrap();
        // (4.33, 2.5) ≈ 30° at r=5 → mask 1; (−4.7,−1.7) ≈ 200° → mask 0. Composed:
        // 0.5·1, 0.9·0.
        let mut cook = Cook::new();
        let out = cook.cook(&g, &FOps, sw, 0.0).unwrap();
        match out[0].as_stream().get("falloff").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.5, 0.0]),
            _ => panic!("falloff"),
        }
    }

    #[test]
    fn pseudo_angle_is_monotone_with_true_angle() {
        // The spine of the HR-5 angular test: the diamond pseudo-angle preserves the
        // ORDER of the true angle over a full turn, so sector membership is exact. Sweep
        // 0..360° and assert the pseudo-angle strictly increases, staying in [0,4).
        let mut prev = -1.0;
        for deg in 0..360 {
            let a = (deg as f32).to_radians();
            let pa = pseudo_angle(a.cos(), a.sin());
            assert!((0.0..4.0).contains(&pa), "pa in range at {deg}: {pa}");
            assert!(pa > prev, "monotone at {deg}: {pa} !> {prev}");
            prev = pa;
        }
        // Anchors: the octant corners land on the integers.
        assert!((pseudo_angle(1.0, 0.0) - 0.0).abs() < 1e-6);
        assert!((pseudo_angle(0.0, 1.0) - 1.0).abs() < 1e-6);
        assert!((pseudo_angle(-1.0, 0.0) - 2.0).abs() < 1e-6);
        assert!((pseudo_angle(0.0, -1.0) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn empty_sector_masks_nothing() {
        // start == end ⇒ a zero-width wedge ⇒ the mask is 0 everywhere but the exact ray.
        let (mut g, sw) = chain();
        g.set_param(sw, "soft", 0.0);
        g.set_param(sw, "end_angle", 0.0); // == start_angle
        let ops = Ops::new(vec![ray(0.0, 5.0), ray(1.0, 5.0), ray(30.0, 5.0)]);
        let got = falloff_of(&g, &ops, sw);
        // On the ray (0°) it is 1; a degree off it is 0.
        assert_eq!(got[0], 1.0);
        assert_eq!(got[1], 0.0);
        assert_eq!(got[2], 0.0);
    }

    #[test]
    fn curves_are_monotone_and_endpoint_exact() {
        for k in 0..=3 {
            assert_eq!(curve(k, 0.0), 0.0, "curve {k} at 0");
            assert_eq!(curve(k, 1.0), 1.0, "curve {k} at 1");
        }
        assert_eq!(curve(1, 0.5), 0.25);
        assert_eq!(curve(2, 0.5), 0.5);
    }
}
