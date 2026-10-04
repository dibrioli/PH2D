// ⭐⭐⭐⭐ THE SURFACE HEAT (docs/3D/30 §14, W6) — the low-pass of a neighbourhood
// adjustment on a SURFACE (the sculpt piece's reticle), in place of the grid's separable
// blur. `e^{-tA} u` by the Chebyshev polynomial of the reticle's laplacian `A = M⁻¹L`:
// the SAME law and the SAME coefficients as the CPU (`ph2d_mesh_colors::difusao`,
// `Difusao::desfoca`), written term for term:
//
//   T₀ = A u;  y = u − d₀·T₀;  T₁ = X T₀;  T_{k+1} = 2 X T_k − T_{k−1};  y −= d_k·T_k
//   X z = escala·A z − z   (escala = 2/λ_sup),   (A z)_a = inv_m_a · Σ_b w_ab (z_a − z_b)
//
// The buffers hold the work texture's texels in fold order (sample i ↔ texel
// (i % width, i / width)); the steps touch the first `n` (the samples), the fold's tail
// keeps what the load wrote.

struct HeatPass {
    n: u32,      // the graph's rows (samples)
    total: u32,  // width · height (the buffers, with the fold's tail)
    width: u32,  // fold width
    mode: u32,   // load: 1 = premultiply; step: 0 = T₀, 1 = T₁, 2 = recurrence; store: 1 = scalar
    coef: f32,   // d_k
    escala: f32, // 2 / λ_sup
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<uniform> hp: HeatPass;
@group(0) @binding(1) var<storage, read> ini: array<u32>;
@group(0) @binding(2) var<storage, read> viz: array<u32>;
@group(0) @binding(3) var<storage, read> peso: array<f32>;
@group(0) @binding(4) var<storage, read> inv_massa: array<f32>;
@group(0) @binding(5) var heat_src: texture_2d<f32>;
@group(0) @binding(6) var heat_dst: texture_storage_2d<rgba32float, write>;
@group(0) @binding(7) var<storage, read_write> u_out: array<vec4<f32>>;
@group(0) @binding(8) var<storage, read> t_cur: array<vec4<f32>>;
@group(0) @binding(9) var<storage, read> t_prev: array<vec4<f32>>;
@group(0) @binding(10) var<storage, read_write> t_out: array<vec4<f32>>;
@group(0) @binding(11) var<storage, read_write> y_buf: array<vec4<f32>>;

// The work texture → u and y (premultiplied on read like the grid's first blur pass).
@compute @workgroup_size(8, 8, 1)
fn cs_heat_load(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.y * hp.width + g.x;
    if g.x >= hp.width || i >= hp.total {
        return;
    }
    var c = textureLoad(heat_src, vec2<i32>(g.xy), 0);
    if hp.mode != 0u {
        c = vec4<f32>(c.rgb * c.a, c.a);
    }
    u_out[i] = c;
    y_buf[i] = c;
}

// `(A t_cur)_a`.
fn a_vezes(a: u32) -> vec4<f32> {
    let k = inv_massa[a];
    if k == 0.0 {
        return vec4<f32>(0.0);
    }
    let za = t_cur[a];
    var s = vec4<f32>(0.0);
    for (var e = ini[a]; e < ini[a + 1u]; e = e + 1u) {
        s = s + peso[e] * (za - t_cur[viz[e]]);
    }
    return k * s;
}

// One term of the polynomial over the samples (1-D, 256 per workgroup, 2-D grid of groups).
@compute @workgroup_size(256, 1, 1)
fn cs_heat_step(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(num_workgroups) nw: vec3<u32>,
    @builtin(local_invocation_index) li: u32,
) {
    let i = (wid.y * nw.x + wid.x) * 256u + li;
    if i >= hp.n {
        return;
    }
    let a = a_vezes(i);
    var v = a;
    if hp.mode != 0u {
        let x = hp.escala * a - t_cur[i];
        v = x;
        if hp.mode == 2u {
            v = 2.0 * x - t_prev[i];
        }
    }
    t_out[i] = v;
    y_buf[i] = y_buf[i] - hp.coef * v;
}

// Below the polynomial's error the heat is ZERO (`RESTO` of `difusao.rs`, the same rule): a
// colour whose coverage is below it is zero entirely; a scalar field (the S/H luma) stays.
const RESTO: f32 = 1e-6;

// y → the destination work texture (still premultiplied: the combine un-premultiplies).
@compute @workgroup_size(8, 8, 1)
fn cs_heat_store(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.y * hp.width + g.x;
    if g.x >= hp.width || i >= hp.total {
        return;
    }
    var o = y_buf[i];
    if hp.mode == 0u && abs(o.a) < RESTO {
        o = vec4<f32>(0.0);
    }
    textureStore(heat_dst, vec2<i32>(g.xy), o);
}
