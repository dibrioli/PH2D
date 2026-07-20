#![forbid(unsafe_code)]
//! **`sim.collide`** — the world pushes BACK (Motion Nodes O4, doc 52).
//!
//! ## What was missing
//!
//! `motion.collide` is *push-apart*: instances relaxing off each other (PBD non-penetration).
//! It knows nothing about a floor, and nothing about velocity — it cannot, because outside a
//! simulation zone **there is no velocity to know about**. A stream re-authored from scratch
//! every frame has positions and no history, so a collision could only ever be a *shove*, never
//! a *bounce*.
//!
//! Inside a zone there is. `vel` lives in the state, so a collider can do the thing that makes a
//! collision look like one: **reflect it**.
//!
//! ## The contact, once
//!
//! Every shape reduces to the same two numbers — an outward unit normal `n` and a penetration
//! depth `d` — and then the response is written ONCE, for all of them:
//!
//! ```text
//!   p += n·d                      // out of the wall, exactly to its surface
//!   vn = v·n                      // …only if it is still moving INTO it (vn < 0)
//!   v  -= (1 + restitution)·vn·n  // reflect the normal part
//!   vt *= 1 - friction            // and bleed the tangential part
//! ```
//!
//! Writing it once per shape is how a collider grows a bug per shape. This one has three shapes
//! and one response.
//!
//! **Only a particle moving INTO the surface is reflected** (`vn < 0`). A particle already
//! sliding along a floor, or resting on it, has `vn ≈ 0` — reflecting it anyway is the classic
//! collider jitter: the thing buzzes on the ground forever, gaining energy from its own contact
//! test.
//!
//! **Restitution ≤ 1**, and the guard says so: a bounce that returns more than it took is a
//! machine for making energy, and it ends with the scene exploding.
//!
//! Transcendental-free (HR-5): the normals are geometry, not angles — a floor's normal is up, a
//! disc's is the radial direction. Nothing here needs a sine.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The shapes. Each one answers the same question — *how far inside am I, and which way is out?*
pub const SHAPE_FLOOR: i32 = 0;
/// A solid disc: the world is everything OUTSIDE it (an obstacle to fall around).
pub const SHAPE_DISC: i32 = 1;
/// A bowl: the world is everything INSIDE it (a container to rattle around in).
pub const SHAPE_BOWL: i32 = 2;

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("sim.collide"),
    name: "sim.collide",
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
        // 0 Floor (a horizontal line, world above it) · 1 Disc (solid obstacle) · 2 Bowl
        // (container).
        ParamSpec {
            name: "shape",
            default: 0.0,
        },
        // Floor: its height. Disc/Bowl: the centre and the radius.
        ParamSpec {
            name: "height",
            default: -2.0,
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
            name: "radius",
            default: 2.0,
        },
        // 0 = dead (it sticks) · 1 = perfectly elastic (it comes back as fast as it hit).
        ParamSpec {
            name: "restitution",
            default: 0.3,
        },
        // How much of the SLIDING speed the surface eats per contact. 0 = ice, 1 = glue.
        ParamSpec {
            name: "friction",
            default: 0.2,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The WGSL port of [`contact`] + [`respond`], element for element (ADR-0135 —
/// the sim-zone family on the GPU). Single-port, no clock, no grid: it reads `P`
/// and `vel` off the state and writes them back. The shape is a **uniform branch**
/// (every element shares the `shape` param), so it is coherent on the device.
///
/// The param clamps are the kernel's own (`radius.max(0)`, restitution/friction
/// clamped to `[0,1]`) — a clamp that lives only on the CPU is a divergence waiting
/// for a slider at its edge. Position is pushed out **unconditionally**; only the
/// velocity reflection is gated on `vn < 0` (already leaving ⇒ do not re-reflect,
/// the classic collider buzz), and the whole response is dropped when non-finite.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let sc_shape = i32(round(params.shape));\n\
        let sc_c = vec2<f32>(params.center_x, params.center_y);\n\
        let sc_radius = max(params.radius, 0.0);\n\
        let sc_rest = clamp(params.restitution, 0.0, 1.0);\n\
        let sc_fric = clamp(params.friction, 0.0, 1.0);\n\
        let sc_p = read_P(i);\n\
        let sc_v = read_vel(i);\n\
        var sc_hit = false;\n\
        var sc_n = vec2<f32>(0.0, 1.0);\n\
        var sc_depth = 0.0;\n\
        if (sc_shape == SC_DISC || sc_shape == SC_BOWL) {\n\
        \x20   let sc_d = sc_p - sc_c;\n\
        \x20   let sc_dist = sqrt(sc_d.x * sc_d.x + sc_d.y * sc_d.y);\n\
        \x20   // Dead centre has no way out: pick up rather than divide by zero.\n\
        \x20   var sc_dir = vec2<f32>(0.0, 1.0);\n\
        \x20   if (sc_dist > SC_EPS) { sc_dir = sc_d / sc_dist; }\n\
        \x20   if (sc_shape == SC_DISC) {\n\
        \x20       if (sc_dist < sc_radius) { sc_hit = true; sc_n = sc_dir; sc_depth = sc_radius - sc_dist; }\n\
        \x20   } else {\n\
        \x20       if (sc_dist > sc_radius) { sc_hit = true; sc_n = -sc_dir; sc_depth = sc_dist - sc_radius; }\n\
        \x20   }\n\
        } else {\n\
        \x20   // The floor: the world is everything above `height`, so out is up.\n\
        \x20   if (sc_p.y < params.height) { sc_hit = true; sc_n = vec2<f32>(0.0, 1.0); sc_depth = params.height - sc_p.y; }\n\
        }\n\
        var sc_out_p = sc_p;\n\
        var sc_out_v = sc_v;\n\
        if (sc_hit) {\n\
        \x20   let sc_rp = sc_p + sc_n * sc_depth;\n\
        \x20   var sc_rv = sc_v;\n\
        \x20   let sc_vn = dot(sc_rv, sc_n);\n\
        \x20   // Already leaving (or sliding): touching must not change it.\n\
        \x20   if (sc_vn < 0.0) {\n\
        \x20       let sc_bounce = (1.0 + sc_rest) * sc_vn;\n\
        \x20       let sc_reflected = sc_rv - sc_bounce * sc_n;\n\
        \x20       let sc_vn_out = dot(sc_reflected, sc_n);\n\
        \x20       let sc_tangent = sc_reflected - sc_vn_out * sc_n;\n\
        \x20       sc_rv = sc_vn_out * sc_n + sc_tangent * (1.0 - sc_fric);\n\
        \x20   }\n\
        \x20   if (collide_finite(sc_rp) && collide_finite(sc_rv)) {\n\
        \x20       sc_out_p = sc_rp;\n\
        \x20       sc_out_v = sc_rv;\n\
        \x20   }\n\
        }\n\
        write_P(i, sc_out_p);\n\
        write_vel(i, sc_out_v);\n",
    wgsl_lib: "\
        const SC_DISC: i32 = 1;\n\
        const SC_BOWL: i32 = 2;\n\
        const SC_EPS: f32 = 1.1920929e-7;\n\
        const SC_F32_MAX: f32 = 3.4028235e38;\n\
        fn collide_finite(v: vec2<f32>) -> bool {\n\
        \x20   return abs(v.x) <= SC_F32_MAX && abs(v.y) <= SC_F32_MAX;\n\
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
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[
        "shape",
        "height",
        "center_x",
        "center_y",
        "radius",
        "restitution",
        "friction",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// The contact at `p`: the outward unit normal, and how deep inside the surface it is.
/// `None` when the point is in the free world.
fn contact(
    shape: i32,
    p: [f32; 2],
    height: f32,
    c: [f32; 2],
    radius: f32,
) -> Option<([f32; 2], f32)> {
    match shape {
        SHAPE_DISC | SHAPE_BOWL => {
            let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
            let dist = (dx * dx + dy * dy).sqrt();
            // Dead centre of a disc has no "way out" — any direction is as good, so pick one
            // rather than dividing by zero and turning the element into a NaN.
            let n = if dist > f32::EPSILON {
                [dx / dist, dy / dist]
            } else {
                [0.0, 1.0]
            };
            if shape == SHAPE_DISC {
                (dist < radius).then_some((n, radius - dist))
            } else {
                (dist > radius).then_some(([-n[0], -n[1]], dist - radius))
            }
        }
        // The floor: the world is everything above `height`, so "out" is straight up.
        _ => (p[1] < height).then_some(([0.0, 1.0], height - p[1])),
    }
}

/// One contact response — the ONE place a collision is written, for every shape.
fn respond(
    p: &mut [f32; 2],
    v: &mut [f32; 2],
    n: [f32; 2],
    depth: f32,
    restitution: f32,
    friction: f32,
) {
    p[0] += n[0] * depth;
    p[1] += n[1] * depth;

    let vn = v[0] * n[0] + v[1] * n[1];
    // Already leaving (or sliding along) the surface: touching it must not change it. Reflecting
    // here is the classic collider jitter — the element buzzes on the ground forever, fed by its
    // own contact test.
    if vn >= 0.0 {
        return;
    }
    // Reflect the normal component, keep (and bleed) the tangential one.
    let bounce = (1.0 + restitution) * vn;
    let mut out = [v[0] - bounce * n[0], v[1] - bounce * n[1]];
    let vn_out = out[0] * n[0] + out[1] * n[1];
    let tangent = [out[0] - vn_out * n[0], out[1] - vn_out * n[1]];
    let keep = 1.0 - friction;
    out = [
        vn_out * n[0] + tangent[0] * keep,
        vn_out * n[1] + tangent[1] * keep,
    ];
    *v = out;
}

fn vec2(s: &Stream, name: &str, n: usize) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    }
}

/// The whole node: resolve each element's contact, respond, write `P` and `vel` back.
#[allow(clippy::too_many_arguments)]
fn collide(
    s: &Stream,
    shape: i32,
    height: f32,
    c: [f32; 2],
    radius: f32,
    restitution: f32,
    friction: f32,
) -> Stream {
    let n = s.count();
    let mut out = Stream::new(n);
    for (name, col) in s.columns() {
        if !matches!(name.as_str(), "P" | "vel") {
            out.set(name.clone(), col.clone());
        }
    }
    let mut p = vec2(s, "P", n);
    let mut v = vec2(s, "vel", n);
    for i in 0..n {
        if let Some((normal, depth)) = contact(shape, p[i], height, c, radius) {
            let (mut pi, mut vi) = (p[i], v[i]);
            respond(&mut pi, &mut vi, normal, depth, restitution, friction);
            if pi.iter().chain(&vi).all(|x| x.is_finite()) {
                p[i] = pi;
                v[i] = vi;
            }
        }
    }
    out.set("P", Column::Vec2(p));
    out.set("vel", Column::Vec2(v));
    out
}

struct SimCollide;

impl NodeOp for SimCollide {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let shape = ctx.param("shape").round() as i32;
        let height = ctx.param("height");
        let c = [ctx.param("center_x"), ctx.param("center_y")];
        let radius = ctx.param("radius").max(0.0);
        // A restitution above 1 returns more than it took — a machine for making energy, and the
        // scene ends up in orbit. Clamped, not trusted.
        let restitution = ctx.param("restitution").clamp(0.0, 1.0); // CLAMP-OK: const bounds
        let friction = ctx.param("friction").clamp(0.0, 1.0); // CLAMP-OK: const bounds
        let out = collide(
            ctx.input(0),
            shape,
            height,
            c,
            radius,
            restitution,
            friction,
        );
        ctx.emit(out);
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SimCollide))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Collider",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

/// Param UI hints. `shape` is a NAMED enum — a float slider would make the artist decode
/// "2" into Bowl, which is exactly the decode the segmented selector exists to abolish.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "shape",
        label: "Shape",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Floor", "Disc", "Bowl"],
        },
    },
    ParamUiHint {
        param: "height",
        label: "Height",
        min: -10.0,
        max: 10.0,
        step: 0.1,
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
        param: "radius",
        label: "Radius",
        min: 0.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "restitution",
        label: "Bounce",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "friction",
        label: "Friction",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
