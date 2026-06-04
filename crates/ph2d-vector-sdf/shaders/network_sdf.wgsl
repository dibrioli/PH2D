// ph2d-vector-sdf — SDF draft compute (ADR-0065 phase 2).
//
// Per grid cell: the min unsigned distance to the uploaded directed boundary
// edges, signed by the NonZero winding number (negative inside). This mirrors
// the pure-Rust `network_sdf` (the parity oracle) for a single-region NonZero
// silhouette — the common draft operand. Determinism (§2.4): fixed grid, a
// per-thread ORDERED edge reduction (no atomics, no cross-thread races).

struct Globals {
    res: u32,
    edge_count: u32,
    _pad0: u32,
    _pad1: u32,
    bmin: vec2<f32>,
    bmax: vec2<f32>,
}

@group(0) @binding(0) var<uniform> g: Globals;
// Each edge is `(a.x, a.y, b.x, b.y)` — a directed boundary segment.
@group(0) @binding(1) var<storage, read> edges: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> out_sdf: array<f32>;

fn seg_dist(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let ab = b - a;
    let len2 = dot(ab, ab);
    var t = 0.0;
    if len2 > 0.0 {
        t = clamp(dot(p - a, ab) / len2, 0.0, 1.0);
    }
    return distance(a + ab * t, p);
}

fn is_left(a: vec2<f32>, b: vec2<f32>, p: vec2<f32>) -> f32 {
    return (b.x - a.x) * (p.y - a.y) - (p.x - a.x) * (b.y - a.y);
}

@compute @workgroup_size(8, 8, 1)
fn sdf_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= g.res || gid.y >= g.res {
        return;
    }
    let span = g.bmax - g.bmin;
    let u = (f32(gid.x) + 0.5) / f32(g.res);
    let v = (f32(gid.y) + 0.5) / f32(g.res);
    let p = g.bmin + vec2<f32>(span.x * u, span.y * v);

    var dist = 1e30;
    var wn = 0;
    for (var i = 0u; i < g.edge_count; i = i + 1u) {
        let e = edges[i];
        let a = e.xy;
        let b = e.zw;
        dist = min(dist, seg_dist(p, a, b));
        // NonZero winding number (Sunday's algorithm), in fixed edge order.
        if a.y <= p.y {
            if b.y > p.y && is_left(a, b, p) > 0.0 {
                wn = wn + 1;
            }
        } else {
            if b.y <= p.y && is_left(a, b, p) < 0.0 {
                wn = wn - 1;
            }
        }
    }
    let inside = wn != 0;
    out_sdf[gid.y * g.res + gid.x] = select(dist, -dist, inside);
}
