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
//!
//! ## The thing that collides has a SIZE (doc 89, folha 13 P0)
//!
//! This collider collided a **point**, and a point is the one radius that is always wrong: an
//! element resting on a floor puts its CENTRE on the floor, so **a sprite sinks by exactly half
//! its height** (measured: a 1×1 quad on a floor at `y = −2` comes to rest with its bottom edge
//! at −2.5). Compensating with `height` only works for a UNIFORM size — `size` is a per-element
//! column and `height` is a param, one number per tick even when driven.
//!
//! The cure is the **Minkowski inflation**: collide the point against the same shape *grown
//! outward by `r`*. One term per shape, and [`respond`] is untouched — the response still has
//! exactly one implementation.
//!
//! ```text
//!   Floor   p.y − r < height          out is up,   depth = height − (p.y − r)
//!   Disc    dist    < radius + r      out is radial
//!   Bowl    dist    > radius − r      in  is radial   (clamped at 0, see `contact`)
//! ```
//!
//! Where `r` comes from is [`particle_radius`] — the ONE door both this eval and the WGSL ask,
//! because two answers to *"how big is this particle?"* diverge the moment a slider moves.
//! `radius_from = Point` (the default) makes it `0` ⇒ **byte-identical to the point collider**.

use ph2d_node_registry::{
    NodeRegistry, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
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

/// **A point** — the collider as it was before the radius existed. The default, so a document
/// that never touches this control collides exactly what it collided yesterday.
pub const RADIUS_POINT: i32 = 0;
/// One radius in world units, the same for every element.
pub const RADIUS_FIXED: i32 = 1;
/// The element's OWN size: the circle inscribed in the sprite the renderer draws.
pub const RADIUS_SIZE: i32 = 2;

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
        // ── The particle's OWN radius (doc 89, folha 13 P0) ─────────────────────
        // APPENDED, never inserted: a param list is read positionally by anything
        // that stores an index, and the three below must not renumber `shape`.
        //
        // 0 Point (what this node always did) · 1 Fixed · 2 Sprite Size.
        ParamSpec {
            name: "radius_from",
            default: 0.0,
        },
        // The Fixed radius, in world units.
        ParamSpec {
            name: "particle_radius",
            default: 0.25,
        },
        // Sprite Size: how many INSCRIBED radii big the collider is. 1 = exactly the
        // circle inside the sprite; ≈1.41 is the one around a square one.
        ParamSpec {
            name: "size_scale",
            default: 1.0,
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
        // The particle's own radius — `particle_radius`, term for term and in the\n\
        // same multiply order, so the two paths cannot answer this differently.\n\
        let sc_size = read_size(i);\n\
        var sc_r = 0.0;\n\
        if (i32(round(params.radius_from)) == SC_R_FIXED) {\n\
        \x20   sc_r = max(params.particle_radius, 0.0);\n\
        } else if (i32(round(params.radius_from)) == SC_R_SIZE) {\n\
        \x20   sc_r = max(min(abs(sc_size.x), abs(sc_size.y)) * 0.5 * params.size_scale, 0.0);\n\
        }\n\
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
        \x20       // The obstacle GROWS by the radius; the container SHRINKS by it.\n\
        \x20       let sc_grown = sc_radius + sc_r;\n\
        \x20       if (sc_dist < sc_grown) { sc_hit = true; sc_n = sc_dir; sc_depth = sc_grown - sc_dist; }\n\
        \x20   } else {\n\
        \x20       let sc_inner = max(sc_radius - sc_r, 0.0);\n\
        \x20       if (sc_dist > sc_inner) { sc_hit = true; sc_n = -sc_dir; sc_depth = sc_dist - sc_inner; }\n\
        \x20   }\n\
        } else {\n\
        \x20   // The floor: the world is everything above `height`, so out is up — and\n\
        \x20   // what touches it is the element's BOTTOM, `p.y - r`.\n\
        \x20   if (sc_p.y - sc_r < params.height) { sc_hit = true; sc_n = vec2<f32>(0.0, 1.0); sc_depth = params.height - (sc_p.y - sc_r); }\n\
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
        const SC_R_FIXED: i32 = 1;\n\
        const SC_R_SIZE: i32 = 2;\n\
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
        // READ-ONLY: the collider never resizes anything, it only asks how big the
        // thing it is catching is. `SIZE_IDENTITY = [1, 1]` — the same absence the
        // CPU's `sizes()` and the lowering itself fall back to.
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
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
        "radius_from",
        "particle_radius",
        "size_scale",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **How big element `i` is** — the ONE door, asked by [`SimCollide::eval`] and ported term for
/// term into [`GPU_KERNEL`]. Two answers to this would diverge the first time a slider moved,
/// and the divergence would be a sprite resting at a different height on the two paths.
///
/// **`Sprite Size` reads the INSCRIBED circle**, `min(|w|,|h|)·0.5`, and the halving is not a
/// convention: `sprite.wgsl` expands a unit quad in `[-0.5, 0.5]` by `size`, so a sprite spans
/// `±size/2`. The *inscribed* circle rather than the circumscribed one because a circle that
/// pokes OUT of the sprite makes it hover with a gap the artist can see and cannot fix by
/// editing the art, while a circle inside it at worst lets a corner clip — which is what a round
/// collider under a square sprite always does. `size_scale` reaches the circumscribed circle
/// (√2 for a square) for whoever wants it.
///
/// An **absent `size` column is `[1, 1]`** on both paths — `SIZE_IDENTITY`, which is *also* the
/// shell's `default_size`, so it is literally the quad the renderer draws there (one number, not
/// two). The `abs` is because a mirrored sprite is the same size, not a negative one.
pub fn particle_radius(mode: i32, fixed: f32, scale: f32, size: [f32; 2]) -> f32 {
    match mode {
        RADIUS_FIXED => fixed.max(0.0),
        RADIUS_SIZE => (size[0].abs().min(size[1].abs()) * 0.5 * scale).max(0.0),
        // Point (and any value a hand-edited document invents): no radius at all.
        _ => 0.0,
    }
}

/// The contact at `p` for a disc of radius `r`: the outward unit normal, and how deep inside the
/// surface it is. `None` when it is clear of the surface.
///
/// `r` enters as the **Minkowski inflation** — the same shape grown outward — so there is still
/// one contact test per shape and one response for all of them. `r = 0` is the point collider,
/// term for term.
fn contact(
    shape: i32,
    p: [f32; 2],
    height: f32,
    c: [f32; 2],
    radius: f32,
    r: f32,
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
                // A solid obstacle GROWS by the particle's radius: the centre of a disc of
                // radius `r` can never be closer than `r` to the surface.
                let grown = radius + r;
                (dist < grown).then_some((n, grown - dist))
            } else {
                // A container SHRINKS by it, for the same reason — and is clamped at 0: a
                // particle wider than its bowl has nowhere to be, and the honest answer is the
                // centre, not a push through it and out the other side.
                let inner = (radius - r).max(0.0);
                (dist > inner).then_some(([-n[0], -n[1]], dist - inner))
            }
        }
        // The floor: the world is everything above `height`, so "out" is straight up — and what
        // touches it is the element's BOTTOM, `p.y − r`.
        _ => (p[1] - r < height).then_some(([0.0, 1.0], height - (p[1] - r))),
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

/// Every element's `size`, widened to `n`. **Absent reads as `SIZE_IDENTITY`** — the unit quad
/// the lowering itself falls back to, and the same identity the GPU binding carries.
fn sizes(s: &Stream, n: usize) -> Vec<[f32; 2]> {
    match s.get("size") {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![ph2d_nodegraph::attr::SIZE_IDENTITY; n],
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
    part: (i32, f32, f32),
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
    // Read unconditionally: `Point` ignores it, and a branch here would be a second place that
    // decides what the radius mode means.
    let size = sizes(s, n);
    let (mode, fixed, scale) = part;
    for i in 0..n {
        let r = particle_radius(mode, fixed, scale, size[i]);
        if let Some((normal, depth)) = contact(shape, p[i], height, c, radius, r) {
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
        let part = (
            ctx.param("radius_from").round() as i32,
            ctx.param("particle_radius"),
            ctx.param("size_scale"),
        );
        let out = collide(
            ctx.input(0),
            shape,
            height,
            c,
            radius,
            restitution,
            friction,
            part,
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
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
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
        param: "radius_from",
        label: "Radius From",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            // Reads as a sentence, and "Point" names the default HONESTLY: what the
            // collider did before it could know how big anything was.
            labels: &["Point", "Fixed", "Sprite Size"],
        },
    },
    ParamUiHint {
        param: "particle_radius",
        label: "Particle Radius",
        min: 0.0,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "size_scale",
        label: "Size Scale",
        // 1 = the circle inscribed in the sprite; 1.414… = the one around a square
        // one. Above ~2 the collider is visibly bigger than the art it catches.
        min: 0.0,
        max: 3.0,
        step: 0.01,
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

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "height",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "radius",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "particle_radius",
        unit: ParamUnit::Length,
    },
];

/// **Only the controls this shape and this radius mode actually READ.**
///
/// Before this, every one of the ten was painted always — so a Disc showed a `Height` the kernel
/// never looks at, and a Floor showed a `Center X`/`Center Y`/`Radius` it never looks at either:
/// four dead knobs, which is the thing this codebase refuses (`sim.collide` was one of the nodes
/// the doc-89 folha flagged for it). The gate is side-metadata — a param with no entry here is
/// always shown, which is what all ten meant yesterday.
///
/// ⚠️ `radius` appears in the **shape** group and `particle_radius` in the **radius** one on
/// purpose: they are two different lengths (*how big is the WORLD* × *how big is the THING*),
/// and the labels say so ("Radius" × "Particle Radius").
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: "height",
        when: "shape",
        values: &[SHAPE_FLOOR],
    },
    ph2d_node_registry::ParamGate {
        param: "center_x",
        when: "shape",
        values: &[SHAPE_DISC, SHAPE_BOWL],
    },
    ph2d_node_registry::ParamGate {
        param: "center_y",
        when: "shape",
        values: &[SHAPE_DISC, SHAPE_BOWL],
    },
    ph2d_node_registry::ParamGate {
        param: "radius",
        when: "shape",
        values: &[SHAPE_DISC, SHAPE_BOWL],
    },
    ph2d_node_registry::ParamGate {
        param: "particle_radius",
        when: "radius_from",
        values: &[RADIUS_FIXED],
    },
    ph2d_node_registry::ParamGate {
        param: "size_scale",
        when: "radius_from",
        values: &[RADIUS_SIZE],
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
