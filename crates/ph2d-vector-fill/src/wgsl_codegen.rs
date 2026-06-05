//! DAG → WGSL lowering (ADR-0060 §2.3, spec §5.3) + the topology hash that keys
//! the [`crate::cache`].
//!
//! ## Shape of the output
//!
//! The emitted source is **embeddable**: a `struct FillParams` + its `@group(0)
//! @binding(0)` uniform, the helper functions the topology actually uses, and a
//! `fn fill_main(coord: vec2<f32>) -> vec4<f32>`. The renderer (Coordenador
//! wiring) drops this into its fragment pipeline and calls `fill_main`.
//!
//! Nodes are lowered as **SSA** `let v{id}` bindings in dependency order — each
//! node is computed exactly once and shared subgraphs are not recomputed (a
//! straight reading of the spec's "function chain" but DAG-correct).
//!
//! ## Why animation never recompiles
//!
//! Two invariants together kill the compile stutter (Antigravity critique B):
//! 1. **No param ever appears in the WGSL.** Colours/frequencies/positions are
//!    read from the UBO (`params.colors[i]`, `params.scalars[i]`, …).
//! 2. **No enum ever branches the WGSL.** Noise kind / blend mode / coord mode /
//!    math op are emitted as *complete* `switch idx { … }` helpers, indexed by a
//!    `u32` from the UBO. Changing one is a `write_buffer`, not a recompile.
//!
//! So the only inputs to [`topology_hash`] are node *tags* (param-free) +
//! connections + the output id + the backend id. Any pure param/enum animation
//! maps to the same hash → a cache hit → zero recompiles.
//!
//! ## Parity (handoff §3 — reuse, don't reinvent)
//!
//! All 2D noise is built on [`ph2d_expr`]'s `ph2d_noise1` (the deterministic
//! integer hash, emitted verbatim from [`ph2d_expr::wgsl_prelude`]); `mix` is the
//! WGSL builtin, matching `ph2d_expr`'s `Func::Mix` semantics. The CPU evaluator
//! in [`crate::eval`] mirrors every formula here line-for-line.

use crate::{FillCodegenError, FillGraph, FillNode, MAX_FILL_NODES, MAX_GRADIENT_STOPS, NodeId};
use core::fmt::Write as _;

use crate::ubo::STOP_POS_LANES;

/// A 256-bit blake3 over a graph's *topology* (never its params) + the backend
/// id — the [`crate::cache`] key. Two graphs that differ only in animatable
/// values share a hash, which is what makes the cache hit-rate exceed 95%.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TopologyHash(pub [u8; 32]);

impl core::fmt::Debug for TopologyHash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TopologyHash({})", self.hex())
    }
}

impl TopologyHash {
    /// Lower-case hex (used in cache filenames / logs).
    pub fn hex(&self) -> String {
        let mut s = String::with_capacity(64);
        for b in self.0 {
            let _ = write!(s, "{b:02x}");
        }
        s
    }
}

/// A validated, embeddable fill shader + its cache key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledFill {
    /// Embeddable WGSL: `FillParams` + helpers + `fn fill_main(...)`.
    pub wgsl: String,
    pub topology_hash: TopologyHash,
}

impl CompiledFill {
    /// Approximate retained size (LRU accounting).
    pub fn byte_size(&self) -> usize {
        self.wgsl.len() + core::mem::size_of::<Self>()
    }
}

/// Which prelude helpers the reachable topology pulls in. Emitting only what is
/// used keeps shaders lean while each included helper still carries *all* its
/// enum variants (so kind/mode/op changes stay recompile-free).
#[derive(Default, Clone, Copy)]
struct Needs {
    noise: bool,
    voronoi: bool,
    stops: bool,
    linear: bool,
    radial: bool,
    mix: bool,
    coord: bool,
    math: bool,
    random: bool,
    bump: bool,
}

impl Needs {
    fn scan(graph: &FillGraph, order: &[NodeId]) -> Self {
        let mut n = Needs::default();
        for id in order {
            match graph.node(*id) {
                FillNode::Noise { .. } => n.noise = true,
                FillNode::Voronoi { .. } => n.voronoi = true,
                FillNode::Ramp { .. } => n.stops = true,
                FillNode::LinearGradient { .. } => {
                    n.stops = true;
                    n.linear = true;
                }
                FillNode::RadialGradient { .. } => {
                    n.stops = true;
                    n.radial = true;
                }
                FillNode::Mix { .. } => n.mix = true,
                FillNode::Coord { .. } => n.coord = true,
                FillNode::Math { .. } => n.math = true,
                FillNode::Random { .. } => n.random = true,
                FillNode::Bump { .. } => n.bump = true,
                _ => {}
            }
        }
        n
    }

    /// `ph2d_cell2`/cellular are shared by noise + voronoi.
    fn needs_cell2(&self) -> bool {
        self.noise || self.voronoi
    }
    /// `ph2d_noise1` (reused from `ph2d-expr`) underpins `ph2d_cell2` (so every
    /// noise/voronoi path) and `Random`.
    fn needs_noise1(&self) -> bool {
        self.needs_cell2() || self.random
    }
}

/// Lower a validated graph to embeddable WGSL. Returns
/// [`FillCodegenError::NotYetImplemented`] if a W6-stub node is reachable.
pub fn codegen(graph: &FillGraph) -> Result<String, FillCodegenError> {
    graph.validate()?;
    let order = graph.dependency_order()?;

    // Reject reachable stubs up front (handoff §2.X frontier).
    for id in &order {
        let node = graph.node(*id);
        if node.is_stub() {
            return Err(FillCodegenError::NotYetImplemented {
                node: id.0,
                kind: node.topology_tag(),
            });
        }
    }

    let needs = Needs::scan(graph, &order);
    let mut wgsl = String::with_capacity(2048);

    wgsl.push_str(&fill_params_struct());

    if needs.needs_noise1() {
        // `wgsl_prelude()` already ends in a newline; the helper consts each
        // begin with one, so a single blank line separates every block.
        wgsl.push_str(ph2d_expr::wgsl_prelude());
    }
    if needs.needs_cell2() {
        wgsl.push_str(CELL2);
    }
    if needs.noise {
        wgsl.push_str(VALUE_NOISE);
        wgsl.push_str(GRAD2);
        wgsl.push_str(SIMPLEX);
        wgsl.push_str(PERLIN);
        wgsl.push_str(FBM);
    }
    if needs.needs_cell2() {
        wgsl.push_str(CELLULAR);
    }
    if needs.noise {
        wgsl.push_str(NOISE_DISPATCH);
    }
    if needs.random {
        wgsl.push_str(RANDOM);
    }
    if needs.stops {
        wgsl.push_str(EVAL_STOPS);
    }
    if needs.linear {
        wgsl.push_str(LINEAR_GRADIENT);
    }
    if needs.radial {
        wgsl.push_str(RADIAL_GRADIENT);
    }
    if needs.mix {
        wgsl.push_str(MIX_BLEND);
    }
    if needs.coord {
        wgsl.push_str(COORD_TRANSFORM);
    }
    if needs.math {
        wgsl.push_str(MATH_OP);
    }
    if needs.bump {
        wgsl.push_str(BUMP);
    }

    wgsl.push_str(&fill_main(graph, &order)?);
    Ok(wgsl)
}

/// Emit `fn fill_main` — one SSA `let v{id}` per reachable node, in dependency
/// order, returning the output node's variable.
fn fill_main(graph: &FillGraph, order: &[NodeId]) -> Result<String, FillCodegenError> {
    let mut s = String::with_capacity(256);
    s.push_str("\nfn fill_main(coord: vec2<f32>) -> vec4<f32> {\n");
    for id in order {
        let node = graph.node(*id);
        let ty = node.output_type().wgsl();
        let expr = node_expr(graph, *id, node)?;
        let _ = writeln!(s, "    let v{}: {} = {};", id.0, ty, expr);
    }
    let _ = writeln!(s, "    return v{};", graph.output_node_id.0);
    s.push_str("}\n");
    Ok(s)
}

/// The WGSL expression that computes `node` (id `id`), referencing its input
/// drivers' `v{src}` variables or per-type defaults for unconnected ports.
fn node_expr(graph: &FillGraph, id: NodeId, node: &FillNode) -> Result<String, FillCodegenError> {
    let i = id.0;
    // Default for an unconnected port: uv → ambient `coord`, scalar → 0, colour
    // → transparent (mirrors `eval`'s "missing input is a default" rule).
    let inp = |port: u32, default: &str| -> String {
        match graph.driver(id, port) {
            Some(src) => format!("v{}", src.0),
            None => default.to_string(),
        }
    };
    let uv = |port: u32| inp(port, "coord");
    let scalar = |port: u32| inp(port, "0.0");
    let color = |port: u32| inp(port, "vec4<f32>(0.0, 0.0, 0.0, 0.0)");

    Ok(match node {
        FillNode::Solid { .. } => format!("params.colors[{i}]"),
        FillNode::LinearGradient { .. } => format!("ph2d_linear_gradient({}, {i}u)", uv(0)),
        FillNode::RadialGradient { .. } => format!("ph2d_radial_gradient({}, {i}u)", uv(0)),
        FillNode::Noise { .. } => format!(
            "ph2d_noise_dispatch({p}, params.ucontrol[{i}].x, params.scalars[{i}].x, \
             params.scalars[{i}].y, params.scalars[{i}].z, params.ucontrol[{i}].y)",
            p = uv(0)
        ),
        FillNode::Voronoi { .. } => format!(
            "ph2d_cellular({p} * f32(params.ucontrol[{i}].x), params.scalars[{i}].x)",
            p = uv(0)
        ),
        FillNode::Ramp { .. } => format!("ph2d_eval_stops({}, {i}u)", scalar(0)),
        FillNode::Mix { .. } => format!(
            "ph2d_mix_blend({a}, {b}, params.scalars[{i}].x, params.ucontrol[{i}].x)",
            a = color(0),
            b = color(1)
        ),
        FillNode::Bump { .. } => format!("ph2d_bump({}, params.scalars[{i}].x)", scalar(0)),
        FillNode::Coord { .. } => {
            format!("ph2d_coord_transform({}, params.ucontrol[{i}].x)", uv(0))
        }
        FillNode::Math { .. } => format!(
            "ph2d_math_op({a}, {b}, params.ucontrol[{i}].x)",
            a = scalar(0),
            b = scalar(1)
        ),
        FillNode::Time => "params.time".to_string(),
        FillNode::Random { .. } => format!("ph2d_random({}, params.ucontrol[{i}].x)", uv(0)),
        // Stubs are rejected in `codegen` before this point.
        _ => {
            return Err(FillCodegenError::NotYetImplemented {
                node: i,
                kind: node.topology_tag(),
            });
        }
    })
}

/// The `FillParams` uniform block — generated from the crate consts so the WGSL
/// can never drift from [`crate::ubo::FillParamsUbo`].
fn fill_params_struct() -> String {
    format!(
        "// === ph2d-vector-fill generated shader (ADR-0060) ===\n\
         struct FillParams {{\n\
         \x20   colors: array<vec4<f32>, {n}>,\n\
         \x20   scalars: array<vec4<f32>, {n}>,\n\
         \x20   ucontrol: array<vec4<u32>, {n}>,\n\
         \x20   stop_colors: array<array<vec4<f32>, {s}>, {n}>,\n\
         \x20   stop_pos: array<array<vec4<f32>, {l}>, {n}>,\n\
         \x20   time: f32,\n\
         }};\n\
         @group(0) @binding(0) var<uniform> params: FillParams;\n",
        n = MAX_FILL_NODES,
        s = MAX_GRADIENT_STOPS,
        l = STOP_POS_LANES,
    )
}

/// Canonical topology + backend → blake3. Connection order is normalized (sorted)
/// so the hash is insertion-order-independent; node order is *kept* (it is the
/// UBO slot / `v{id}` identity, hence structural).
pub fn topology_hash(graph: &FillGraph, backend_id: &str) -> TopologyHash {
    let mut h = blake3::Hasher::new();
    h.update(b"ph2d-vector-fill-topology-v1\0");
    h.update(backend_id.as_bytes());
    h.update(&[0]);
    h.update(&(graph.nodes.len() as u32).to_le_bytes());
    for node in &graph.nodes {
        h.update(node.topology_tag().as_bytes());
        h.update(&[0]);
    }
    h.update(&graph.output_node_id.0.to_le_bytes());
    let mut conns: Vec<[u32; 4]> = graph
        .connections
        .iter()
        .map(|c| [c.from_node.0, c.from_output, c.to_node.0, c.to_input])
        .collect();
    conns.sort_unstable();
    for c in conns {
        for v in c {
            h.update(&v.to_le_bytes());
        }
    }
    TopologyHash(*h.finalize().as_bytes())
}

/// Wrap embeddable source in a minimal fragment entry so naga validates the full
/// binding-usage path (and any fragment-only builtins like `dpdx` in `Bump`).
pub(crate) fn validation_module(embeddable: &str) -> String {
    format!(
        "{embeddable}\n\
         @fragment\n\
         fn ph2d_fill_fs(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {{\n\
         \x20   return fill_main(uv);\n\
         }}\n"
    )
}

// ───────────────────────────── static WGSL helpers ─────────────────────────────
// Each is a fixed string (deterministic codegen). 2D noise reuses `ph2d_noise1`
// from `ph2d-expr`; `crate::eval` mirrors every formula.

const CELL2: &str = "\nfn ph2d_cell2(c: vec2<f32>) -> f32 {\n    return ph2d_noise1(c.x * 113.0 + c.y * 271.7);\n}\n";

// 6.2831855 is the f32 value of TAU (== `core::f32::consts::TAU`), kept literal
// here and named in `eval::grad2` so the CPU mirror reads the identical f32.
const GRAD2: &str = "\nfn ph2d_grad2(c: vec2<f32>) -> vec2<f32> {\n    let a = ph2d_cell2(c) * 6.2831855;\n    return vec2<f32>(cos(a), sin(a));\n}\n";

const VALUE_NOISE: &str = "\nfn ph2d_value_noise(p: vec2<f32>) -> f32 {\n    let i = floor(p);\n    let f = p - i;\n    let u = f * f * (3.0 - 2.0 * f);\n    let a = ph2d_cell2(i + vec2<f32>(0.0, 0.0));\n    let b = ph2d_cell2(i + vec2<f32>(1.0, 0.0));\n    let c = ph2d_cell2(i + vec2<f32>(0.0, 1.0));\n    let d = ph2d_cell2(i + vec2<f32>(1.0, 1.0));\n    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);\n}\n";

const SIMPLEX: &str = "\nfn ph2d_simplex_corner(c: vec2<f32>, x: f32, y: f32) -> f32 {\n    var tt = 0.5 - x * x - y * y;\n    if (tt < 0.0) { return 0.0; }\n    tt = tt * tt;\n    let g = ph2d_grad2(c);\n    return tt * tt * (g.x * x + g.y * y);\n}\nfn ph2d_simplex(p: vec2<f32>) -> f32 {\n    let F2 = 0.3660254;\n    let G2 = 0.21132487;\n    let s = (p.x + p.y) * F2;\n    let i = floor(p.x + s);\n    let j = floor(p.y + s);\n    let t = (i + j) * G2;\n    let x0 = p.x - (i - t);\n    let y0 = p.y - (j - t);\n    var i1 = 0.0;\n    var j1 = 0.0;\n    if (x0 > y0) { i1 = 1.0; } else { j1 = 1.0; }\n    let x1 = x0 - i1 + G2;\n    let y1 = y0 - j1 + G2;\n    let x2 = x0 - 1.0 + 2.0 * G2;\n    let y2 = y0 - 1.0 + 2.0 * G2;\n    var n = 0.0;\n    n = n + ph2d_simplex_corner(vec2<f32>(i, j), x0, y0);\n    n = n + ph2d_simplex_corner(vec2<f32>(i + i1, j + j1), x1, y1);\n    n = n + ph2d_simplex_corner(vec2<f32>(i + 1.0, j + 1.0), x2, y2);\n    return clamp(70.0 * n * 0.5 + 0.5, 0.0, 1.0);\n}\n";

const PERLIN: &str = "\nfn ph2d_perlin(p: vec2<f32>) -> f32 {\n    let i = floor(p);\n    let f = p - i;\n    let u = f * f * (3.0 - 2.0 * f);\n    let g00 = ph2d_grad2(i + vec2<f32>(0.0, 0.0));\n    let g10 = ph2d_grad2(i + vec2<f32>(1.0, 0.0));\n    let g01 = ph2d_grad2(i + vec2<f32>(0.0, 1.0));\n    let g11 = ph2d_grad2(i + vec2<f32>(1.0, 1.0));\n    let n00 = dot(g00, f - vec2<f32>(0.0, 0.0));\n    let n10 = dot(g10, f - vec2<f32>(1.0, 0.0));\n    let n01 = dot(g01, f - vec2<f32>(0.0, 1.0));\n    let n11 = dot(g11, f - vec2<f32>(1.0, 1.0));\n    let nx0 = mix(n00, n10, u.x);\n    let nx1 = mix(n01, n11, u.x);\n    return clamp(mix(nx0, nx1, u.y) * 0.5 + 0.5, 0.0, 1.0);\n}\n";

const FBM: &str = "\nfn ph2d_fbm(p: vec2<f32>, lac: f32, pers: f32, oct: u32) -> f32 {\n    var amp = 1.0;\n    var freq = 1.0;\n    var sum = 0.0;\n    var norm = 0.0;\n    let n = min(oct, 8u);\n    for (var k = 0u; k < n; k = k + 1u) {\n        sum = sum + amp * ph2d_value_noise(p * freq);\n        norm = norm + amp;\n        amp = amp * pers;\n        freq = freq * lac;\n    }\n    return sum / max(norm, 1e-5);\n}\n";

const CELLULAR: &str = "\nfn ph2d_cellular(p: vec2<f32>, jitter: f32) -> f32 {\n    let i = floor(p);\n    let f = p - i;\n    var md = 8.0;\n    for (var dy = -1.0; dy <= 1.0; dy = dy + 1.0) {\n        for (var dx = -1.0; dx <= 1.0; dx = dx + 1.0) {\n            let o = vec2<f32>(dx, dy);\n            let cc = i + o;\n            let fp = vec2<f32>(ph2d_cell2(cc), ph2d_cell2(cc + vec2<f32>(19.3, 71.7)));\n            let diff = o + jitter * fp - f;\n            md = min(md, dot(diff, diff));\n        }\n    }\n    return clamp(sqrt(md), 0.0, 1.0);\n}\n";

const NOISE_DISPATCH: &str = "\nfn ph2d_noise_dispatch(p: vec2<f32>, kind: u32, freq: f32, lac: f32, pers: f32, oct: u32) -> f32 {\n    let q = p * freq;\n    switch kind {\n        case 0u: { return ph2d_simplex(q); }\n        case 1u: { return ph2d_perlin(q); }\n        case 2u: { return ph2d_cellular(q, 1.0); }\n        case 3u: { return ph2d_fbm(q, lac, pers, oct); }\n        default: { return 0.0; }\n    }\n}\n";

const RANDOM: &str = "\nfn ph2d_random(p: vec2<f32>, seed: u32) -> f32 {\n    return ph2d_noise1(p.x * 12.9898 + p.y * 78.233 + f32(seed));\n}\n";

const EVAL_STOPS: &str = "\nfn ph2d_stop_pos(node: u32, k: u32) -> f32 {\n    return params.stop_pos[node][k / 4u][k % 4u];\n}\nfn ph2d_eval_stops(t: f32, node: u32) -> vec4<f32> {\n    let n = params.ucontrol[node].x;\n    if (n == 0u) { return vec4<f32>(t, t, t, 1.0); }\n    let tc = clamp(t, 0.0, 1.0);\n    if (tc <= ph2d_stop_pos(node, 0u)) { return params.stop_colors[node][0]; }\n    let last = n - 1u;\n    if (tc >= ph2d_stop_pos(node, last)) { return params.stop_colors[node][last]; }\n    var result = params.stop_colors[node][0];\n    for (var i = 1u; i < n; i = i + 1u) {\n        let pa = ph2d_stop_pos(node, i - 1u);\n        let pb = ph2d_stop_pos(node, i);\n        if (tc >= pa && tc <= pb) {\n            let f = (tc - pa) / max(pb - pa, 1e-5);\n            result = mix(params.stop_colors[node][i - 1u], params.stop_colors[node][i], f);\n        }\n    }\n    return result;\n}\n";

const LINEAR_GRADIENT: &str = "\nfn ph2d_linear_gradient(uv: vec2<f32>, node: u32) -> vec4<f32> {\n    let angle = params.scalars[node].x;\n    let dir = vec2<f32>(cos(angle), sin(angle));\n    return ph2d_eval_stops(dot(uv, dir), node);\n}\n";

const RADIAL_GRADIENT: &str = "\nfn ph2d_radial_gradient(uv: vec2<f32>, node: u32) -> vec4<f32> {\n    let center = vec2<f32>(params.scalars[node].x, params.scalars[node].y);\n    let radius = max(params.scalars[node].z, 1e-5);\n    return ph2d_eval_stops(length(uv - center) / radius, node);\n}\n";

const MIX_BLEND: &str = "\nfn ph2d_blend(a: vec4<f32>, b: vec4<f32>, mode: u32) -> vec4<f32> {\n    switch mode {\n        case 0u: {\n            let oa = b.a + a.a * (1.0 - b.a);\n            let rgb = (b.rgb * b.a + a.rgb * a.a * (1.0 - b.a)) / max(oa, 1e-5);\n            return vec4<f32>(rgb, oa);\n        }\n        case 1u: { return a * b; }\n        case 2u: { return vec4<f32>(1.0) - (vec4<f32>(1.0) - a) * (vec4<f32>(1.0) - b); }\n        case 3u: {\n            let lo = 2.0 * a * b;\n            let hi = vec4<f32>(1.0) - 2.0 * (vec4<f32>(1.0) - a) * (vec4<f32>(1.0) - b);\n            return select(lo, hi, a > vec4<f32>(0.5));\n        }\n        case 4u: { return min(a + b, vec4<f32>(1.0)); }\n        case 5u: { return b; }\n        default: { return b; }\n    }\n}\nfn ph2d_mix_blend(a: vec4<f32>, b: vec4<f32>, factor: f32, mode: u32) -> vec4<f32> {\n    return mix(a, ph2d_blend(a, b, mode), clamp(factor, 0.0, 1.0));\n}\n";

const COORD_TRANSFORM: &str = "\nfn ph2d_coord_transform(p: vec2<f32>, mode: u32) -> vec2<f32> {\n    switch mode {\n        case 0u: { return p; }\n        case 1u: { return p; }\n        case 2u: { return p; }\n        case 3u: { return vec2<f32>(length(p), atan2(p.y, p.x)); }\n        default: { return p; }\n    }\n}\n";

const MATH_OP: &str = "\nfn ph2d_math_op(a: f32, b: f32, op: u32) -> f32 {\n    switch op {\n        case 0u: { return a + b; }\n        case 1u: { return a - b; }\n        case 2u: { return a * b; }\n        case 3u: { return a / b; }\n        case 4u: { return min(a, b); }\n        case 5u: { return max(a, b); }\n        case 6u: { return abs(a); }\n        case 7u: { return sin(a); }\n        case 8u: { return cos(a); }\n        case 9u: { return floor(a); }\n        case 10u: { return a - floor(a); }\n        case 11u: { return sqrt(a); }\n        case 12u: { return pow(a, b); }\n        case 13u: { return a - b * floor(a / b); }\n        default: { return a; }\n    }\n}\n";

const BUMP: &str = "\nfn ph2d_bump(h: f32, strength: f32) -> vec3<f32> {\n    let dx = dpdx(h) * strength;\n    let dy = dpdy(h) * strength;\n    return normalize(vec3<f32>(-dx, -dy, 1.0));\n}\n";
