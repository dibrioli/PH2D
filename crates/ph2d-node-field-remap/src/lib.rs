#![forbid(unsafe_code)]
//! `field.remap` — the Motion **remapper** of a `falloff` mask: the downstream node
//! the whole `field.*` family DEFERS its remapping to (D1). Where a spatial field
//! (`field.box`/`radial_sweep`) shapes a raw mask by position, this one TRANSFORMS an
//! existing mask value-by-value — it reads `falloff`, rewrites `falloff` (it does NOT
//! multiply into it; a remap is not a field, it is a transfer function). It is the
//! C4D **Remapping** tab as a node: `inner_offset` (the solid-core plateau), a
//! `contour` transfer (None/Quadratic/Step/Quantize — **Curve** arrives with the A1
//! widget), an output `[min, max]` range with a `multiplier`, a `clamp`, an `invert`,
//! and a `strength` that blends input→remapped.
//!
//! **Transcendental-free (HR-5).** Every transfer is `min`/`max`/`clamp`/`floor`/`round`
//! and polynomials, so the mask is bit-identical CPU↔GPU (within FMA ULPs). The GPU
//! kernel is a straight WGSL port; there is no `P` to read — a remap is position-blind,
//! it reads the mask alone.
//!
//! **The neutral (D12).** `strength = 0` is a byte-identical passthrough (the mask is
//! returned unchanged) — the "off" that never mutates the scene. A fresh node, however,
//! does something VISIBLE: it is born `contour = Quadratic` with a gentle `curvature`,
//! so dropping it on a soft field sharpens the ramp — the sign that a remap is working,
//! trivially reset to `None`/Linear.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{par_build, Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.remap"),
    name: "field.remap",
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
            name: "inner_offset",
            default: 0.0,
        },
        ParamSpec {
            // 0 None/Linear · 1 Quadratic · 2 Step · 3 Quantize · 4 Curve (A1, future).
            name: "contour",
            default: 1.0,
        },
        ParamSpec {
            // The Quadratic bend, [-1, 1]: >0 ease-in (t²), <0 ease-out. Inert on the
            // other contour modes.
            name: "curvature",
            default: 0.5,
        },
        ParamSpec {
            // The level count for Step/Quantize (≥ 2). Inert on the other modes.
            name: "steps",
            default: 4.0,
        },
        ParamSpec {
            name: "min",
            default: 0.0,
        },
        ParamSpec {
            name: "max",
            default: 1.0,
        },
        ParamSpec {
            name: "multiplier",
            default: 1.0,
        },
        ParamSpec {
            name: "clamp",
            default: 1.0,
        },
        ParamSpec {
            name: "invert",
            default: 0.0,
        },
        ParamSpec {
            name: "strength",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Round half-away-from-zero — matching Rust's `f32::round` (the WGSL `round` is
/// half-even), so Step/Quantize land on the same level on both devices.
fn round_haz(x: f32) -> f32 {
    if x >= 0.0 {
        (x + 0.5).floor()
    } else {
        (x - 0.5).ceil()
    }
}

/// The **contour** transfer on `t ∈ [0,1]` — the C4D Remapping shapes, HR-5. `None`
/// (0) is the identity; `Quadratic` (1) bends by `curvature` (`>0` ease-in `t²`, `<0`
/// ease-out, `0` linear); `Step` (2) is a `steps`-level FLOOR staircase (holds, hits 0
/// and 1); `Quantize` (3) is a `steps`-level ROUND (nearest). `Curve` (4) is the A1
/// spline — a passthrough until the widget lands (documented, not a silent gap).
fn contour(mode: i32, t: f32, curvature: f32, steps: f32) -> f32 {
    match mode {
        1 => {
            let ease_in = t * t;
            let ease_out = 1.0 - (1.0 - t) * (1.0 - t);
            if curvature >= 0.0 {
                t + (ease_in - t) * curvature
            } else {
                t + (ease_out - t) * (-curvature)
            }
        }
        2 => {
            let n = steps.max(2.0);
            let k = (t * n).floor().min(n - 1.0);
            k / (n - 1.0)
        }
        3 => {
            let n = steps.max(2.0);
            round_haz(t * (n - 1.0)) / (n - 1.0)
        }
        _ => t, // None/Linear (0) and Curve (4, future) both pass through for now.
    }
}

/// The full remap of one mask value `t_in` — the SAME pipeline the WGSL kernel runs.
/// Invert flips the input; `inner_offset` expands the solid core (values above the
/// offset saturate to 1); the contour shapes it; the `[min, max]` range + `multiplier`
/// scale it; `clamp` bounds it to `[0,1]`; and `strength` blends input→remapped (so
/// `strength = 0` is an exact passthrough).
// The 11 params are the C4D Remapping pipeline; a struct would be a second model of the
// same knobs the `NodeManifest` already lists (mirrors `field_gizmo`'s `view_from_params`).
#[allow(clippy::too_many_arguments)]
fn remap(
    t_in: f32,
    inner_offset: f32,
    mode: i32,
    curvature: f32,
    steps: f32,
    lo: f32,
    hi: f32,
    multiplier: f32,
    clamp: bool,
    invert: bool,
    strength: f32,
) -> f32 {
    let mut t = if invert { 1.0 - t_in } else { t_in };
    // Inner offset: `1 − io` is the input level that reaches full effect; above it the
    // core is a flat 1. Clamped below 1 so the denominator is never 0.
    let io = inner_offset.clamp(0.0, 0.999);
    t = (t / (1.0 - io)).min(1.0);
    t = contour(mode, t, curvature, steps);
    let mut mapped = (lo + t * (hi - lo)) * multiplier;
    if clamp {
        mapped = mapped.clamp(0.0, 1.0);
    }
    t_in + (mapped - t_in) * strength
}

/// GPU compute kernel (ADR-0126): a straight WGSL port of [`remap`] × [`contour`] over
/// the existing `falloff` — same `min`/`max`/`clamp`/`floor` + polynomials (HR-5), so
/// parity holds within float ULPs. There is no `P` binding — a remap is position-blind.
/// `falloff` `ReadWrite` from the `1.0` identity mirrors the CPU (an absent mask reads
/// full effect); it is REWRITTEN, not multiplied (a remap is a transfer function).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let rm_in = read_falloff(i);\n\
        var rm_t = rm_in;\n\
        if (params.invert >= 0.5) { rm_t = 1.0 - rm_t; }\n\
        let rm_io = clamp(params.inner_offset, 0.0, 0.999);\n\
        rm_t = min(rm_t / (1.0 - rm_io), 1.0);\n\
        rm_t = rm_contour(i32(rm_round(params.contour)), rm_t, params.curvature, params.steps);\n\
        var rm_mapped = (params.min + rm_t * (params.max - params.min)) * params.multiplier;\n\
        if (params.clamp >= 0.5) { rm_mapped = clamp(rm_mapped, 0.0, 1.0); }\n\
        let rm_out = rm_in + (rm_mapped - rm_in) * params.strength;\n\
        write_falloff(i, rm_out);\n",
    wgsl_lib: "\
        fn rm_round(x: f32) -> f32 {\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn rm_contour(mode: i32, t: f32, curvature: f32, steps: f32) -> f32 {\n\
            if (mode == 1) {\n\
                let ease_in = t * t;\n\
                let ease_out = 1.0 - (1.0 - t) * (1.0 - t);\n\
                if (curvature >= 0.0) { return t + (ease_in - t) * curvature; }\n\
                return t + (ease_out - t) * (-curvature);\n\
            }\n\
            if (mode == 2) {\n\
                let n = max(steps, 2.0);\n\
                let k = min(floor(t * n), n - 1.0);\n\
                return k / (n - 1.0);\n\
            }\n\
            if (mode == 3) {\n\
                let n = max(steps, 2.0);\n\
                return rm_round(t * (n - 1.0)) / (n - 1.0);\n\
            }\n\
            return t;\n\
        }\n",
    bindings: &[ColumnBinding {
        column: "falloff",
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [1.0; 4],
        port: 0,
    }],
    params: &[
        "inner_offset",
        "contour",
        "curvature",
        "steps",
        "min",
        "max",
        "multiplier",
        "clamp",
        "invert",
        "strength",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct FieldRemap;

impl NodeOp for FieldRemap {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let inner_offset = ctx.param("inner_offset");
        let mode = ctx.param("contour").round() as i32;
        let curvature = ctx.param("curvature");
        let steps = ctx.param("steps");
        let lo = ctx.param("min");
        let hi = ctx.param("max");
        let multiplier = ctx.param("multiplier");
        let clamp = ctx.param("clamp") >= 0.5;
        let invert = ctx.param("invert") >= 0.5;
        let strength = ctx.param("strength");
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // The mask being remapped; absent → 1.0 (full effect), the CPU/GPU identity.
            let prev: Option<&[f32]> = match input.get("falloff") {
                Some(Column::Scalar(v)) => Some(v.as_slice()),
                _ => None,
            };
            let fall = par_build(n, |i| {
                let t_in = prev.and_then(|v| v.get(i).copied()).unwrap_or(1.0);
                remap(
                    t_in,
                    inner_offset,
                    mode,
                    curvature,
                    steps,
                    lo,
                    hi,
                    multiplier,
                    clamp,
                    invert,
                    strength,
                )
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
    reg.register(Box::new(FieldRemap))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Remap",
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): the C4D Remapping controls. `contour` is the transfer
/// selector (Curve is the A1 spline, greyed until the widget lands); `curvature`
/// bends the Quadratic; `steps` counts the Step/Quantize levels; `strength` is the
/// input→remapped blend (0 = passthrough).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "inner_offset",
        label: "Inner Offset",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "contour",
        label: "Contour",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["None", "Quadratic", "Step", "Quantize", "Curve"],
        },
    },
    ParamUiHint {
        param: "curvature",
        label: "Curvature",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "steps",
        label: "Steps",
        min: 2.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "min",
        label: "Min",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "max",
        label: "Max",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "multiplier",
        label: "Multiplier",
        min: 0.0,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "clamp",
        label: "Clamp",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: "invert",
        label: "Invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
