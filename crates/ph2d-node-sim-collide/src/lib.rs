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
//!
//! ## The plane TILTS (doc 89, folha 13 P0)
//!
//! The floor's normal was the literal `vec2(0, 1)`, and the folha measured both ways around it
//! and refuted both: `motion.rotate` writes only the `rot` column — it never moves `P`, and
//! could not move `vel` either, so a rotate-collide-unrotate sandwich reflects against the
//! wrong frame; and chaining colliders *works* (each is `Pure`) but every floor is horizontal,
//! so a chain builds a **staircase**, never a ramp.
//!
//! So the shape carries its own orientation. The plane is the canonical analytic one — a unit
//! normal and a signed distance (the Hesse form):
//!
//! ```text
//!   world = { p : dot(p, n) >= offset },   n = the world's up-vector turned by `angle`
//!
//!     0°   n = ( 0,  1)   a FLOOR    (the world is above)
//!    ±θ    a RAMP, and a particle with friction < 1 SLIDES down it
//!    90°   n = (-1,  0)   a WALL     at x = -offset, world to its left
//!   180°   n = ( 0, -1)   a CEILING  at y = -offset, world below
//!   270°   n = ( 1,  0)   a WALL     at x =  offset, world to its right
//! ```
//!
//! ⚠️ **`offset` is the distance from the ORIGIN along the normal, not a Y coordinate** — and
//! that choice is what makes a wall placeable at all. The obvious alternative (*"the plane
//! pivots about `(0, height)`"*) reads better for a small ramp tilt, and it pins every wall to
//! `x = 0` for ever: at 90° the pivot lies **on** the plane, so the knob slides the wall along
//! itself and does nothing. The Hesse offset is never inert at any angle. The param is still
//! named `height` (a name is stored in every saved graph and cannot be renamed); its LABEL is
//! "Offset", because at 90° a height is not what the number is.
//!
//! ⚠️ **The tilt does not reach the Disc or the Bowl, and that is geometry**: both are
//! rotationally symmetric, so an angle on them would be a knob that provably changes nothing.
//! It is gated to the plane ([`PARAM_GATES`]).
//!
//! `angle = 0` gives `(0, 1)` **to the bit** — [`trig`]'s polynomial is exact at the cardinal
//! phases — and `dot(p, (0,1))` is `p.y`, so the untilted plane is the floor that shipped, term
//! for term.

use ph2d_node_registry::{
    NodeRegistry, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;
mod ui;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The shapes. Each one answers the same question — *how far inside am I, and which way is out?*
///
/// A half-plane, oriented by `angle`. **A floor is this plane at angle 0** — the default, and
/// the overwhelming case; the name is the general one because the same shape is also a wall and
/// a ceiling, and a "Floor" that can be a ceiling is a label that lies.
pub const SHAPE_PLANE: i32 = 0;
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

/// **The contact channel** (doc 89, folha 13 P1) — how deep this tick's collision pushed the
/// element back out, in world units, and `0` where nothing touched.
///
/// Until it existed a collision changed `P` and `vel` and **nothing downstream could tell**: the
/// step changes those two every tick anyway, so *"did it touch?"* was a question the graph could
/// not ask. One number answers it and the STRENGTH together, and it does so without two facts
/// that could disagree — contact is `depth > 0` **by construction**, because each shape's test
/// returns `Some` exactly when the depth it computes is strictly positive.
///
/// ⚠️ **It ACCUMULATES over a tick and the zone STRIPS it** — the exact shape of `accel`, for the
/// same two reasons. Colliders CHAIN (the ramp and the wall of demo 29 are two of them), so a
/// plain write would mean *"did the LAST collider touch me"*, which is a fact about graph order
/// and not about the world; `max` over the tick means *the deepest contact of this tick* and
/// reads the same however many colliders the artist stacks. And `sim.zone`'s `TRANSIENTS` must
/// strip it **in the commit that introduces it** — a contact flag that rides the state around
/// says "touched" on the tick after it stopped touching, and every reader downstream repeats the
/// lie (the mask that masked its own gravity, `sim.zone`'s own scar).
pub const HIT_COL: &str = "hit";

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
        // 0 Plane (a half-plane, world on the normal's side) · 1 Disc (solid obstacle) ·
        // 2 Bowl (container).
        ParamSpec {
            name: "shape",
            default: 0.0,
        },
        // Plane: the signed distance from the origin along its normal (the Hesse offset —
        // at angle 0 that IS the floor's height). Disc/Bowl: the centre and the radius.
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
        // ── The plane's TILT, in degrees (doc 89, folha 13 P0) ──────────────────
        // Appended for the same reason the three above were. 0 = the horizontal
        // floor this node always had, bit for bit.
        // **A ALEATORIEDADE DA RESTITUIÇÃO** (doc 89 folha 13). `0` = a restituição
        // autorada para toda a gente, bit-idêntico ao que sempre saiu. Ver [`RANDOMNESS`].
        ParamSpec {
            name: RANDOMNESS,
            default: 0.0,
        },
        // Qual espalhamento, quando há aleatoriedade. Gateado a ela: com `0` o cook nunca o
        // lê, e um knob que o cook não abre é um controlo morto.
        ParamSpec {
            name: SEED,
            default: 0.0,
        },
        ParamSpec {
            name: "angle",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
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

/// **Which way is up for a plane tilted by `angle`** — the other door, asked once per eval and
/// ported term for term into [`GPU_KERNEL`]. It is the world's up-vector turned by `angle`.
///
/// ⚠️ **The `sqrt` is not decoration.** The parabolic sine is ~0.09% off true trig, so `(c, s)`
/// is not exactly on the unit circle — and a normal that is not unit-length makes BOTH the
/// penetration depth and the reflection wrong by that much, because `respond` measures along it.
/// One `sqrt` fixes it, and `sqrt` is the one operation IEEE-754 pins exactly, so the CPU and
/// the device agree on it.
///
/// ⚠️ **No zero-guard, deliberately**: `trig::tests::stays_near_unit_circle` bounds the radius²
/// inside `[0.98, 1.02]`, so the divisor cannot approach zero. A branch that can never fire is a
/// defense no gate could ever prove.
///
/// `0.0 - s` rather than `-s` so that `angle = 0` lands on the literal `[0.0, 1.0]` the floor
/// used to carry, sign of zero and all: the two differ only at `s = +0`, which is precisely the
/// case the byte-identity claim is about.
pub fn plane_normal(angle_deg: f32) -> [f32; 2] {
    let (c, s) = trig::cos_sin_cycles(angle_deg / 360.0);
    let inv = 1.0 / (c * c + s * s).sqrt();
    [(0.0 - s) * inv, c * inv]
}

/// The contact at `p` for a disc of radius `r`: the outward unit normal, and how deep inside the
/// surface it is. `None` when it is clear of the surface.
///
/// `r` enters as the **Minkowski inflation** — the same shape grown outward — so there is still
/// one contact test per shape and one response for all of them. `r = 0` is the point collider,
/// term for term.
///
/// `plane_n` is the plane's normal (from [`plane_normal`]); the Disc and the Bowl ignore it,
/// because a circle turned is the same circle.
#[allow(clippy::too_many_arguments)]
fn contact(
    shape: i32,
    p: [f32; 2],
    height: f32,
    c: [f32; 2],
    radius: f32,
    r: f32,
    plane_n: [f32; 2],
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
        // The plane: the world is the side its normal points to, so "out" IS the normal — and
        // what touches it is the element's near face, `sd − r`. At `angle = 0` the normal is
        // `(0, 1)` to the bit, `sd` is `p[1]`, and this is the floor test verbatim.
        _ => {
            let sd = p[0] * plane_n[0] + p[1] * plane_n[1];
            (sd - r < height).then_some((plane_n, height - (sd - r)))
        }
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

/// A scalar column widened to `n`. **Absent reads as `0`** — the same identity the GPU binding
/// carries, so an incoming `hit` and a missing one mean the same thing on both paths.
fn scalars(s: &Stream, name: &str, n: usize) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) if v.len() == n => v.clone(),
        _ => vec![0.0; n],
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

/// O nome do param da aleatoriedade, e o da semente que a escolhe.
const RANDOMNESS: &str = "restitution_randomness";
const SEED: &str = "seed";

/// **A RESTITUIÇÃO DE UM ELEMENTO** — a ÚNICA porta, chamada pelo [`collide`] e portada
/// termo a termo para o [`GPU_KERNEL`].
///
/// `rest · (1 − randomness · h)`, com `h ∈ [0, 1)` do hash estável da identidade. Em
/// `randomness = 0` o factor é **exactamente** `1` em IEEE-754 (`1 − 0·h`), então a saída é
/// bit-idêntica à de antes deste param existir — não «quase», o mesmo número.
///
/// ⚠️ **Ela só TIRA, nunca acrescenta**, e é a leitura certa de um material: a restituição
/// autorada é o teto (a batida mais viva que aquele obstáculo devolve), e o acaso diz quanto
/// desta batida se perdeu. Um `±` centrado no valor autorado faria `restitution = 1` devolver
/// **mais** energia do que recebeu em metade dos elementos — a máquina de fazer energia que o
/// clamp do `eval` existe para impedir.
///
/// ⚠️ **A chave é a IDENTIDADE do elemento (`id`), não a posição dele na lista** — pela mesma
/// lei do `pick` do `motion.duplicator`: pôr um `motion.sort` no meio não pode redistribuir
/// quão saltitante cada partícula é. Sem coluna `id`, a posição é a única resposta disponível.
fn element_restitution(rest: f32, randomness: f32, seed: u32, key: u32) -> f32 {
    rest * (1.0 - randomness * hash::rand01(seed, key))
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
    plane_n: [f32; 2],
    rnd: (f32, u32),
) -> Stream {
    let n = s.count();
    let mut out = Stream::new(n);
    for (name, col) in s.columns() {
        if !matches!(name.as_str(), "P" | "vel") && name.as_str() != HIT_COL {
            out.set(name.clone(), col.clone());
        }
    }
    let mut p = vec2(s, "P", n);
    let mut v = vec2(s, "vel", n);
    let mut hit = scalars(s, HIT_COL, n);
    // Read unconditionally: `Point` ignores it, and a branch here would be a second place that
    // decides what the radius mode means.
    let size = sizes(s, n);
    let (mode, fixed, scale) = part;
    // A identidade de cada elemento. ⚠️ Lida uma vez: um `get` por elemento seria a mesma
    // pergunta `n` vezes, e a coluna AUSENTE tem de cair na posição — não em zero, que daria
    // a todos a mesma sorte (a armadilha que o `HAS_id` do kernel evita do outro lado).
    let ids = match s.get("id") {
        Some(Column::Scalar(v)) if v.len() == n => Some(v.clone()),
        _ => None,
    };
    let (randomness, seed) = rnd;
    for i in 0..n {
        let r = particle_radius(mode, fixed, scale, size[i]);
        if let Some((normal, depth)) = contact(shape, p[i], height, c, radius, r, plane_n) {
            let (mut pi, mut vi) = (p[i], v[i]);
            #[expect(clippy::cast_sign_loss, reason = "uma identidade e' um inteiro >= 0")]
            #[expect(clippy::cast_possible_truncation, reason = "idem")]
            let key = ids
                .as_ref()
                .map_or(i as u32, |v| v[i].max(0.0).round() as u32);
            let rest_i = element_restitution(restitution, randomness, seed, key);
            respond(&mut pi, &mut vi, normal, depth, rest_i, friction);
            if pi.iter().chain(&vi).all(|x| x.is_finite()) {
                p[i] = pi;
                v[i] = vi;
                // Written where the response LANDED. A dropped (non-finite) response moved
                // nothing, so reporting a contact there would say the node did something it
                // did not — the channel describes what happened, not what was attempted.
                hit[i] = hit[i].max(depth);
            }
        }
    }
    out.set("P", Column::Vec2(p));
    out.set("vel", Column::Vec2(v));
    out.set(HIT_COL, Column::Scalar(hit));
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
        // Once per eval: it is a param, so every element shares it.
        let plane_n = plane_normal(ctx.param("angle"));
        let randomness = ctx.param(RANDOMNESS).clamp(0.0, 1.0); // CLAMP-OK: const bounds
        #[expect(clippy::cast_sign_loss, reason = "a seed is a bit pattern")]
        #[expect(clippy::cast_possible_truncation, reason = "idem")]
        let seed = ctx.param(SEED).max(0.0).round() as u32;
        let out = collide(
            ctx.input(0),
            shape,
            height,
            c,
            radius,
            restitution,
            friction,
            part,
            plane_n,
            (randomness, seed),
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
    reg.register_param_ui(MANIFEST.id, ui::PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, ui::PARAM_UNITS);
    reg.register_param_gates(MANIFEST.id, ui::PARAM_GATES);
    reg.register_param_groups(MANIFEST.id, ui::PARAM_GROUPS);
    reg.register_param_gates_above(MANIFEST.id, ui::PARAM_GATES_ABOVE);
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[path = "gpu.rs"]
mod gpu;
use gpu::GPU_KERNEL;

#[path = "hash.rs"]
mod hash;

#[cfg(test)]
#[path = "randomness_tests.rs"]
mod randomness_tests;
