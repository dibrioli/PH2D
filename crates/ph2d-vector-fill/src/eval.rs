//! CPU **reference** evaluator (spec §5.3 / handoff DoD "nodes pilot work
//! offline"). This is the semantics the WGSL in [`crate::wgsl_codegen`] must
//! match: every helper below mirrors a WGSL helper line-for-line, and both read
//! the *same* [`FillParamsUbo`] slots — so CPU and GPU consume identical param
//! values (the deterministic integer base reuses [`ph2d_expr`]'s `noise1`).
//!
//! Float-arithmetic ordering can still differ CPU↔GPU (FMA); bit-identical
//! output is the **opt-in deterministic mode** (ADR-0060 §2.6), proven later by
//! the cross-OS GPU render gate. Here the contract is *formula* parity.
//!
//! `Bump` is the one node the CPU cannot reproduce — it needs screen-space
//! derivatives (`dpdx`/`dpdy`) that only exist in a fragment stage — so CPU eval
//! returns the flat normal `(0, 0, 1)` for it (documented limitation).

use glam::{Vec2, Vec3};

use crate::poisson_cpu::{FieldResolver, NoFields};
use crate::ubo::FillParamsUbo;
use crate::{FillCodegenError, FillGraph, FillNode, FillType, MAX_GRADIENT_STOPS, NodeId};

/// A value flowing on a wire during evaluation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FillValue {
    Scalar(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Color([f32; 4]),
}

impl FillValue {
    fn as_scalar(self) -> f32 {
        match self {
            FillValue::Scalar(v) => v,
            FillValue::Vec2(v) => v.x,
            FillValue::Vec3(v) => v.x,
            FillValue::Color(c) => c[0],
        }
    }
    fn as_vec2(self) -> Vec2 {
        match self {
            FillValue::Vec2(v) => v,
            FillValue::Scalar(v) => Vec2::splat(v),
            FillValue::Vec3(v) => v.truncate(),
            FillValue::Color(c) => Vec2::new(c[0], c[1]),
        }
    }
    fn as_color(self) -> [f32; 4] {
        match self {
            FillValue::Color(c) => c,
            FillValue::Scalar(v) => [v, v, v, 1.0],
            FillValue::Vec2(v) => [v.x, v.y, 0.0, 1.0],
            FillValue::Vec3(v) => [v.x, v.y, v.z, 1.0],
        }
    }
}

/// Evaluate the graph at `coord`, returning the output node's colour (linear
/// RGBA). `ubo` carries the params (use [`FillParamsUbo::from_graph`] for the
/// authored state, or an animated snapshot). Convenience wrapper over
/// [`eval_color_with_fields`] with **no** diffusion fields: a reachable
/// `MeshGradient` therefore renders transparent (use the `_with_fields` form to
/// supply solved [`crate::ColorField`]s). Errors only if a CPU-unevaluable node
/// ([`FillNode::lacks_cpu_eval`]) is reachable.
pub fn eval_color(
    graph: &FillGraph,
    coord: Vec2,
    ubo: &FillParamsUbo,
) -> Result<[f32; 4], FillCodegenError> {
    eval_color_with_fields(graph, coord, ubo, &NoFields)
}

/// As [`eval_color`], but `fields` resolves each reachable
/// [`FillNode::MeshGradient`]'s `gradient_id` to a solved
/// [`crate::ColorField`], which the node samples bilinearly at `coord` (W7
/// step 2). An unresolved gradient renders transparent.
pub fn eval_color_with_fields(
    graph: &FillGraph,
    coord: Vec2,
    ubo: &FillParamsUbo,
    fields: &dyn FieldResolver,
) -> Result<[f32; 4], FillCodegenError> {
    let order = graph.dependency_order()?;
    let mut values: Vec<Option<FillValue>> = vec![None; graph.nodes.len()];

    for id in &order {
        let node = graph.node(*id);
        if node.lacks_cpu_eval() {
            return Err(FillCodegenError::NotYetImplemented {
                node: id.0,
                kind: node.topology_tag(),
            });
        }
        let v = eval_node(graph, *id, node, coord, ubo, &values, fields);
        values[id.0 as usize] = Some(v);
    }

    Ok(values[graph.output_node_id.0 as usize]
        .expect("output node evaluated")
        .as_color())
}

fn eval_node(
    graph: &FillGraph,
    id: NodeId,
    node: &FillNode,
    coord: Vec2,
    ubo: &FillParamsUbo,
    values: &[Option<FillValue>],
    fields: &dyn FieldResolver,
) -> FillValue {
    let i = id.0 as usize;
    // Input lookup with per-type defaults (mirrors `node_expr` in codegen).
    let input = |port: u32, ty: FillType| -> FillValue {
        match graph.driver(id, port) {
            Some(src) => values[src.0 as usize].unwrap_or(default_value(ty)),
            None => match ty {
                FillType::Vec2 => FillValue::Vec2(coord),
                _ => default_value(ty),
            },
        }
    };

    match node {
        FillNode::Solid { .. } => FillValue::Color(ubo.colors[i]),
        FillNode::LinearGradient { .. } => {
            FillValue::Color(linear_gradient(input(0, FillType::Vec2).as_vec2(), i, ubo))
        }
        FillNode::RadialGradient { .. } => {
            FillValue::Color(radial_gradient(input(0, FillType::Vec2).as_vec2(), i, ubo))
        }
        FillNode::Noise { .. } => {
            let p = input(0, FillType::Vec2).as_vec2();
            let s = ubo.scalars[i];
            let kind = ubo.ucontrol[i][0];
            let oct = ubo.ucontrol[i][1];
            FillValue::Scalar(noise_dispatch(p, kind, s[0], s[1], s[2], oct))
        }
        FillNode::Voronoi { .. } => {
            let p = input(0, FillType::Vec2).as_vec2();
            let cells = ubo.ucontrol[i][0] as f32;
            FillValue::Scalar(cellular(p * cells, ubo.scalars[i][0]))
        }
        FillNode::Ramp { .. } => {
            FillValue::Color(eval_stops(input(0, FillType::Scalar).as_scalar(), i, ubo))
        }
        FillNode::Mix { .. } => {
            let a = input(0, FillType::Color).as_color();
            let b = input(1, FillType::Color).as_color();
            FillValue::Color(mix_blend(a, b, ubo.scalars[i][0], ubo.ucontrol[i][0]))
        }
        FillNode::Bump { .. } => FillValue::Vec3(Vec3::new(0.0, 0.0, 1.0)),
        FillNode::Coord { .. } => FillValue::Vec2(coord_transform(
            input(0, FillType::Vec2).as_vec2(),
            ubo.ucontrol[i][0],
        )),
        FillNode::Math { .. } => {
            let a = input(0, FillType::Scalar).as_scalar();
            let b = input(1, FillType::Scalar).as_scalar();
            FillValue::Scalar(math_op(a, b, ubo.ucontrol[i][0]))
        }
        FillNode::Time => FillValue::Scalar(ubo.time),
        FillNode::Random { .. } => FillValue::Scalar(random(
            input(0, FillType::Vec2).as_vec2(),
            ubo.ucontrol[i][0],
        )),
        // Mesh gradient: sample the host-supplied solved field at `coord`
        // (W7 step 2). Unresolved id → transparent (the field isn't ready yet).
        FillNode::MeshGradient { gradient_id } => FillValue::Color(
            fields
                .resolve(*gradient_id)
                .map_or([0.0, 0.0, 0.0, 0.0], |field| field.sample(coord)),
        ),
        // The remaining 4 resource stubs are rejected before this point.
        FillNode::Pattern { .. }
        | FillNode::ProceduralShader { .. }
        | FillNode::Image { .. }
        | FillNode::ImageSample { .. } => FillValue::Color([0.0, 0.0, 0.0, 0.0]),
    }
}

fn default_value(ty: FillType) -> FillValue {
    match ty {
        FillType::Scalar => FillValue::Scalar(0.0),
        FillType::Vec2 => FillValue::Vec2(Vec2::ZERO),
        FillType::Vec3 => FillValue::Vec3(Vec3::ZERO),
        FillType::Color => FillValue::Color([0.0, 0.0, 0.0, 0.0]),
    }
}

// ───────────────────────── helpers (mirror wgsl_codegen) ─────────────────────────

fn lerpf(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}
fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        lerpf(a[0], b[0], t),
        lerpf(a[1], b[1], t),
        lerpf(a[2], b[2], t),
        lerpf(a[3], b[3], t),
    ]
}

/// `ph2d_cell2` — 2D lattice hash on the reused 1D `noise1`.
fn cell2(c: Vec2) -> f32 {
    ph2d_expr::eval::noise1(c.x * 113.0 + c.y * 271.7)
}

fn grad2(c: Vec2) -> Vec2 {
    // Same f32 as the WGSL literal `6.2831855` in `wgsl_codegen::GRAD2`.
    let a = cell2(c) * core::f32::consts::TAU;
    Vec2::new(a.cos(), a.sin())
}

fn value_noise(p: Vec2) -> f32 {
    let i = p.floor();
    let f = p - i;
    let u = f * f * (Vec2::splat(3.0) - 2.0 * f);
    let a = cell2(i + Vec2::new(0.0, 0.0));
    let b = cell2(i + Vec2::new(1.0, 0.0));
    let c = cell2(i + Vec2::new(0.0, 1.0));
    let d = cell2(i + Vec2::new(1.0, 1.0));
    lerpf(lerpf(a, b, u.x), lerpf(c, d, u.x), u.y)
}

fn simplex_corner(c: Vec2, x: f32, y: f32) -> f32 {
    let mut tt = 0.5 - x * x - y * y;
    if tt < 0.0 {
        return 0.0;
    }
    tt *= tt;
    let g = grad2(c);
    tt * tt * (g.x * x + g.y * y)
}

fn simplex(p: Vec2) -> f32 {
    const F2: f32 = 0.3660254;
    const G2: f32 = 0.21132487;
    let s = (p.x + p.y) * F2;
    let i = (p.x + s).floor();
    let j = (p.y + s).floor();
    let t = (i + j) * G2;
    let x0 = p.x - (i - t);
    let y0 = p.y - (j - t);
    let (i1, j1) = if x0 > y0 { (1.0, 0.0) } else { (0.0, 1.0) };
    let x1 = x0 - i1 + G2;
    let y1 = y0 - j1 + G2;
    let x2 = x0 - 1.0 + 2.0 * G2;
    let y2 = y0 - 1.0 + 2.0 * G2;
    let mut n = 0.0;
    n += simplex_corner(Vec2::new(i, j), x0, y0);
    n += simplex_corner(Vec2::new(i + i1, j + j1), x1, y1);
    n += simplex_corner(Vec2::new(i + 1.0, j + 1.0), x2, y2);
    (70.0 * n * 0.5 + 0.5).clamp(0.0, 1.0)
}

fn perlin(p: Vec2) -> f32 {
    let i = p.floor();
    let f = p - i;
    let u = f * f * (Vec2::splat(3.0) - 2.0 * f);
    let g00 = grad2(i + Vec2::new(0.0, 0.0));
    let g10 = grad2(i + Vec2::new(1.0, 0.0));
    let g01 = grad2(i + Vec2::new(0.0, 1.0));
    let g11 = grad2(i + Vec2::new(1.0, 1.0));
    let n00 = g00.dot(f - Vec2::new(0.0, 0.0));
    let n10 = g10.dot(f - Vec2::new(1.0, 0.0));
    let n01 = g01.dot(f - Vec2::new(0.0, 1.0));
    let n11 = g11.dot(f - Vec2::new(1.0, 1.0));
    let nx0 = lerpf(n00, n10, u.x);
    let nx1 = lerpf(n01, n11, u.x);
    (lerpf(nx0, nx1, u.y) * 0.5 + 0.5).clamp(0.0, 1.0)
}

fn fbm(p: Vec2, lac: f32, pers: f32, oct: u32) -> f32 {
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut sum = 0.0;
    let mut norm = 0.0;
    for _ in 0..oct.min(8) {
        sum += amp * value_noise(p * freq);
        norm += amp;
        amp *= pers;
        freq *= lac;
    }
    sum / norm.max(1e-5)
}

fn cellular(p: Vec2, jitter: f32) -> f32 {
    let i = p.floor();
    let f = p - i;
    let mut md = 8.0_f32;
    let mut dy = -1.0;
    while dy <= 1.0 {
        let mut dx = -1.0;
        while dx <= 1.0 {
            let o = Vec2::new(dx, dy);
            let cc = i + o;
            let fp = Vec2::new(cell2(cc), cell2(cc + Vec2::new(19.3, 71.7)));
            let diff = o + jitter * fp - f;
            md = md.min(diff.dot(diff));
            dx += 1.0;
        }
        dy += 1.0;
    }
    md.sqrt().clamp(0.0, 1.0)
}

fn noise_dispatch(p: Vec2, kind: u32, freq: f32, lac: f32, pers: f32, oct: u32) -> f32 {
    let q = p * freq;
    match kind {
        0 => simplex(q),
        1 => perlin(q),
        2 => cellular(q, 1.0),
        3 => fbm(q, lac, pers, oct),
        _ => 0.0,
    }
}

fn random(p: Vec2, seed: u32) -> f32 {
    ph2d_expr::eval::noise1(p.x * 12.9898 + p.y * 78.233 + seed as f32)
}

fn stop_pos(node: usize, k: usize, ubo: &FillParamsUbo) -> f32 {
    ubo.stop_pos[node][k / 4][k % 4]
}

fn eval_stops(t: f32, node: usize, ubo: &FillParamsUbo) -> [f32; 4] {
    let n = ubo.ucontrol[node][0] as usize;
    if n == 0 {
        return [t, t, t, 1.0];
    }
    let n = n.min(MAX_GRADIENT_STOPS);
    let tc = t.clamp(0.0, 1.0);
    if tc <= stop_pos(node, 0, ubo) {
        return ubo.stop_colors[node][0];
    }
    let last = n - 1;
    if tc >= stop_pos(node, last, ubo) {
        return ubo.stop_colors[node][last];
    }
    let mut result = ubo.stop_colors[node][0];
    for k in 1..n {
        let pa = stop_pos(node, k - 1, ubo);
        let pb = stop_pos(node, k, ubo);
        if tc >= pa && tc <= pb {
            let f = (tc - pa) / (pb - pa).max(1e-5);
            result = lerp4(ubo.stop_colors[node][k - 1], ubo.stop_colors[node][k], f);
        }
    }
    result
}

fn linear_gradient(uv: Vec2, node: usize, ubo: &FillParamsUbo) -> [f32; 4] {
    let angle = ubo.scalars[node][0];
    let dir = Vec2::new(angle.cos(), angle.sin());
    eval_stops(uv.dot(dir), node, ubo)
}

fn radial_gradient(uv: Vec2, node: usize, ubo: &FillParamsUbo) -> [f32; 4] {
    let center = Vec2::new(ubo.scalars[node][0], ubo.scalars[node][1]);
    let radius = ubo.scalars[node][2].max(1e-5);
    eval_stops((uv - center).length() / radius, node, ubo)
}

fn blend(a: [f32; 4], b: [f32; 4], mode: u32) -> [f32; 4] {
    match mode {
        0 => {
            let oa = b[3] + a[3] * (1.0 - b[3]);
            let inv = (oa).max(1e-5);
            [
                (b[0] * b[3] + a[0] * a[3] * (1.0 - b[3])) / inv,
                (b[1] * b[3] + a[1] * a[3] * (1.0 - b[3])) / inv,
                (b[2] * b[3] + a[2] * a[3] * (1.0 - b[3])) / inv,
                oa,
            ]
        }
        1 => [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]],
        2 => [
            1.0 - (1.0 - a[0]) * (1.0 - b[0]),
            1.0 - (1.0 - a[1]) * (1.0 - b[1]),
            1.0 - (1.0 - a[2]) * (1.0 - b[2]),
            1.0 - (1.0 - a[3]) * (1.0 - b[3]),
        ],
        3 => {
            let ov = |ac: f32, bc: f32| {
                if ac > 0.5 {
                    1.0 - 2.0 * (1.0 - ac) * (1.0 - bc)
                } else {
                    2.0 * ac * bc
                }
            };
            [
                ov(a[0], b[0]),
                ov(a[1], b[1]),
                ov(a[2], b[2]),
                ov(a[3], b[3]),
            ]
        }
        4 => [
            (a[0] + b[0]).min(1.0),
            (a[1] + b[1]).min(1.0),
            (a[2] + b[2]).min(1.0),
            (a[3] + b[3]).min(1.0),
        ],
        _ => b,
    }
}

fn mix_blend(a: [f32; 4], b: [f32; 4], factor: f32, mode: u32) -> [f32; 4] {
    lerp4(a, blend(a, b, mode), factor.clamp(0.0, 1.0))
}

fn coord_transform(p: Vec2, mode: u32) -> Vec2 {
    match mode {
        3 => Vec2::new(p.length(), p.y.atan2(p.x)),
        // Local / World / Screen — identity until the renderer supplies matrices.
        _ => p,
    }
}

fn math_op(a: f32, b: f32, op: u32) -> f32 {
    match op {
        0 => a + b,
        1 => a - b,
        2 => a * b,
        3 => a / b,
        4 => a.min(b),
        5 => a.max(b),
        6 => a.abs(),
        7 => a.sin(),
        8 => a.cos(),
        9 => a.floor(),
        10 => a - a.floor(),
        11 => a.sqrt(),
        12 => a.powf(b),
        13 => a - b * (a / b).floor(),
        _ => a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Connection, FillNode, GradientStop, MathOp, NoiseKind};
    use ph2d_color::OklchColor;
    use smallvec::smallvec;

    #[test]
    fn solid_round_trips_authored_color() {
        let g = FillGraph {
            nodes: smallvec![FillNode::Solid {
                color: OklchColor::opaque(0.7, 0.05, 200.0)
            }],
            connections: smallvec![],
            output_node_id: NodeId(0),
        };
        let ubo = FillParamsUbo::from_graph(&g);
        let c = eval_color(&g, Vec2::new(0.3, 0.4), &ubo).unwrap();
        let want = OklchColor::opaque(0.7, 0.05, 200.0).to_linear().as_array();
        assert_eq!(c, want);
    }

    #[test]
    fn noise_into_ramp_is_in_gamut_and_deterministic() {
        // Coord(local) → Noise(fbm) → Ramp.  Pilot path (handoff DoD).
        let g = FillGraph {
            nodes: smallvec![
                FillNode::Coord {
                    mode: crate::CoordMode::Local
                },
                FillNode::Noise {
                    kind: NoiseKind::Fbm {
                        lacunarity: 2.0,
                        persistence: 0.5
                    },
                    frequency: 3.0,
                    octaves: 4,
                },
                FillNode::Ramp {
                    palette: smallvec![
                        GradientStop::new(OklchColor::opaque(0.1, 0.0, 0.0), 0.0),
                        GradientStop::new(OklchColor::opaque(0.9, 0.1, 60.0), 1.0),
                    ],
                },
            ],
            connections: smallvec![
                Connection::new(NodeId(0), NodeId(1), 0),
                Connection::new(NodeId(1), NodeId(2), 0),
            ],
            output_node_id: NodeId(2),
        };
        g.validate().unwrap();
        let ubo = FillParamsUbo::from_graph(&g);
        let c1 = eval_color(&g, Vec2::new(1.7, 2.3), &ubo).unwrap();
        let c2 = eval_color(&g, Vec2::new(1.7, 2.3), &ubo).unwrap();
        assert_eq!(c1, c2, "eval must be pure");
        for ch in c1 {
            assert!((0.0..=1.0).contains(&ch), "channel out of gamut: {ch}");
        }
    }

    #[test]
    fn math_fract_uses_wgsl_semantics() {
        // ph2d-expr gotcha: WGSL fract(-0.25) = 0.75, not Rust's -0.25.
        let g = FillGraph {
            nodes: smallvec![
                FillNode::Time,
                FillNode::Math { op: MathOp::Fract },
                FillNode::Ramp {
                    palette: smallvec![]
                },
            ],
            connections: smallvec![
                Connection::new(NodeId(0), NodeId(1), 0),
                Connection::new(NodeId(1), NodeId(2), 0),
            ],
            output_node_id: NodeId(2),
        };
        let mut ubo = FillParamsUbo::from_graph(&g);
        ubo.set_time(-0.25);
        let c = eval_color(&g, Vec2::ZERO, &ubo).unwrap();
        // Ramp with empty palette → grayscale of the fract result (0.75).
        assert!((c[0] - 0.75).abs() < 1e-6, "got {}", c[0]);
    }

    #[test]
    fn mesh_gradient_samples_solved_field() {
        // W7 step 2: a MeshGradient node evaluates by sampling its solved
        // ColorField at `coord` — eval must equal a direct `field.sample`.
        use crate::diffusion_curve::{DiffusionCurve, DiffusionCurveSet};
        use crate::poisson_cpu::{FieldStore, Resolution, solve_color_field};

        let red = OklchColor::opaque(0.63, 0.26, 29.0);
        let blue = OklchColor::opaque(0.45, 0.31, 264.0);
        let set = DiffusionCurveSet::from_curves([DiffusionCurve::straight(
            Vec2::new(0.5, 0.0),
            Vec2::new(0.5, 1.0),
            red,
            blue,
        )]);
        let field = solve_color_field(&set, Resolution::square(65).unwrap());
        let mut store = FieldStore::new();
        store.insert(7, field.clone());

        let g = FillGraph {
            nodes: smallvec![FillNode::MeshGradient { gradient_id: 7 }],
            connections: smallvec![],
            output_node_id: NodeId(0),
        };
        g.validate().unwrap();
        let ubo = FillParamsUbo::from_graph(&g);

        let coord = Vec2::new(0.1, 0.5); // far-left of the red/blue split
        let got = eval_color_with_fields(&g, coord, &ubo, &store).unwrap();
        assert_eq!(
            got,
            field.sample(coord),
            "MeshGradient must sample its field"
        );
        assert!(got[0] > got[2], "far-left should lean red (R>B): {got:?}");
    }

    #[test]
    fn mesh_gradient_unresolved_is_transparent() {
        // Bare eval_color supplies no fields → an unresolved gradient renders
        // transparent (a missing resource must not fail the whole graph).
        let g = FillGraph {
            nodes: smallvec![FillNode::MeshGradient { gradient_id: 99 }],
            connections: smallvec![],
            output_node_id: NodeId(0),
        };
        let ubo = FillParamsUbo::from_graph(&g);
        let c = eval_color(&g, Vec2::new(0.5, 0.5), &ubo).unwrap();
        assert_eq!(c, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn codegen_still_rejects_mesh_gradient() {
        // CPU eval handles MeshGradient (step 2), but WGSL codegen still cannot:
        // it needs the GPU texture binding (renderer wiring, step 3).
        let g = FillGraph {
            nodes: smallvec![FillNode::MeshGradient { gradient_id: 1 }],
            connections: smallvec![],
            output_node_id: NodeId(0),
        };
        let err = crate::wgsl_codegen::codegen(&g).unwrap_err();
        assert!(
            matches!(err, FillCodegenError::NotYetImplemented { .. }),
            "codegen should defer MeshGradient, got {err:?}"
        );
    }
}
