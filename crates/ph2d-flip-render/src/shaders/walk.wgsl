// **O PERCURSO POR LADRILHO, NO DEVICE** — o port do `binning.rs::walk_pixel` + `tau.rs`
// (doc 12 §14). Um workgroup de 16×16 É um ladrilho: a lista que ele lê é a mesma para as 256
// threads, que é a razão inteira de o binning existir.
//
// ⚠️ **Isto é um ESPELHO, não uma segunda lei.** Cada função abaixo tem o irmão em Rust nomeado
// no comentário, e o gate `the_device_walk_is_the_cpu_walk` compara os dois no MESMO desenho. As
// operações estão na MESMA ORDEM de propósito — em `f32` a ordem é o valor (a lição que o
// `dab_weight` do passo 3 pagou nascendo vermelho por `p*p*(3−2p)` contra `3p²−2p³`).

struct Pt {
    pos: vec2<f32>,
    width: f32,
    opacity: f32,
    color: vec4<f32>,
}

struct Stroke {
    first_point: u32,
    point_count: u32,
    flags: u32,
    hardness: f32,
    material: u32,
    tip: u32,
    dot_spacing: f32,
    ref_width: f32,
}

struct Seg {
    stroke: u32,
    a: u32,
    b: u32,
}

struct Screen {
    world_to_clip: mat4x4<f32>,
    // xy = viewport, z = px_per_world, w = pad
    view: vec4<f32>,
    // x = tile, y = cols, z = rows, w = pad
    grid: vec4<u32>,
}

@group(0) @binding(0) var<uniform> sc: Screen;
@group(0) @binding(1) var<storage, read> points: array<Pt>;
@group(0) @binding(2) var<storage, read> strokes: array<Stroke>;
@group(0) @binding(3) var<storage, read> ranges: array<vec2<u32>>;
@group(0) @binding(4) var<storage, read> segs: array<Seg>;
// ⚠️ **UMA saída, e é uma TEXTURA** — a que o produto de fato usa (o `hdr` premult 16F do
// `FlipCompose`). Uma 2ª saída em buffer daria dois caminhos para o mesmo pixel, e a paridade
// passaria a medir um deles enquanto o outro shipa.
@group(0) @binding(5) var out_tex: texture_storage_2d<rgba16float, write>;

// `pack.rs`
const FLAG_CLOSED: u32 = 1u;
const FLAG_START_FLAT: u32 = 2u;
// `binning.rs::MIN_WIDTH_PX`
const MIN_WIDTH_PX: f32 = 1.3;
// `tau.rs`
const PAINTER_SPACING: f32 = 0.10;
const MIN_PITCH_PX: f32 = 0.25;
const SUB: f32 = 4.0;
const F_MAX: f32 = 16.0;

// `ScreenSpace::point_px`
fn point_px(w: vec2<f32>) -> vec2<f32> {
    let m = sc.world_to_clip;
    let cx = m[0][0] * w.x + m[1][0] * w.y + m[3][0];
    let cy = m[0][1] * w.x + m[1][1] * w.y + m[3][1];
    let cw = m[0][3] * w.x + m[1][3] * w.y + m[3][3];
    var inv = 1.0;
    if (cw != 0.0) {
        inv = 1.0 / cw;
    }
    return vec2<f32>(
        (cx * inv * 0.5 + 0.5) * sc.view.x,
        (cy * inv * 0.5 + 0.5) * sc.view.y,
    );
}

// `ScreenSpace::radius_px`
fn radius_px(width: f32) -> f32 {
    return max(width * 0.5 * sc.view.z, MIN_WIDTH_PX * 0.5);
}

// `binning.rs::closest_on_seg` — devolve (t, cx, cy).
fn closest_on_seg(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> vec3<f32> {
    let v = b - a;
    let len2 = dot(v, v);
    var t = 0.0;
    if (len2 > 1e-12) {
        t = clamp(dot(p - a, v) / len2, 0.0, 1.0);
    }
    let c = a + v * t;
    return vec3<f32>(t, c.x, c.y);
}

// `tau.rs::dab_weight`
fn dab_weight(dn: f32, hardness: f32) -> f32 {
    let h = clamp(hardness, 0.0, 1.0);
    if (h >= 1.0) {
        return select(0.0, 1.0, dn < 1.0);
    }
    let remapped = clamp((dn - h) / (1.0 - h), 0.0, 1.0);
    if (remapped >= 1.0) {
        return 0.0;
    }
    let q = 1.0 - remapped;
    return 3.0 * q * q - 2.0 * q * q * q;
}

// `tau.rs::f_of`
fn f_of(dn: f32, hardness: f32) -> f32 {
    let w = dab_weight(dn, hardness);
    if (w >= 1.0) {
        return F_MAX;
    }
    return min(-log(1.0 - w), F_MAX);
}

// `binning.rs::stroke_silhouette` — devolve (sd, near.x, near.y, dist); `sd = 1e30` se vazio.
fn stroke_silhouette(lo: u32, hi: u32, p: vec2<f32>) -> vec4<f32> {
    var best = vec4<f32>(1e30, 0.0, 0.0, 0.0);
    for (var k = lo; k < hi; k = k + 1u) {
        let s = segs[k];
        let pa = points[s.a];
        let pb = points[s.b];
        let sa = point_px(pa.pos);
        let sb = point_px(pb.pos);
        let tc = closest_on_seg(p, sa, sb);
        let dist = length(p - vec2<f32>(tc.y, tc.z));
        let r = radius_px(pa.width) * (1.0 - tc.x) + radius_px(pb.width) * tc.x;
        let sd = dist - r;
        if (sd < best.x) {
            best = vec4<f32>(sd, tc.y, tc.z, dist);
        }
    }
    return best;
}

// `tau.rs::end_dab` — o termo de fronteira, SÓ no começo (a assimetria da referência).
fn end_dab(sid: u32, hardness: f32, p: vec2<f32>, tau: ptr<function, f32>, acc: ptr<function, vec4<f32>>) {
    let st = strokes[sid];
    if ((st.flags & FLAG_CLOSED) != 0u || st.point_count == 0u) {
        return;
    }
    if ((st.flags & FLAG_START_FLAT) != 0u) {
        return;
    }
    let pt = points[st.first_point];
    let s = point_px(pt.pos);
    let r = max(radius_px(pt.width), 1e-4);
    let dn = length(p - s) / r;
    let d_tau = 0.5 * f_of(dn, hardness);
    if (d_tau <= 0.0) {
        return;
    }
    *tau = *tau + d_tau;
    *acc = *acc + vec4<f32>(
        pt.color.x * d_tau,
        pt.color.y * d_tau,
        pt.color.z * d_tau,
        pt.color.w * pt.opacity * d_tau,
    );
}

// `tau.rs::stroke_tau` — devolve (tau, acc) com a cor já dividida; tau <= 0 ⇒ sem depósito.
fn stroke_tau(lo: u32, hi: u32, sid: u32, hardness: f32, p: vec2<f32>, out_rgb: ptr<function, vec4<f32>>) -> f32 {
    var tau = 0.0;
    var acc = vec4<f32>(0.0);
    for (var k = lo; k < hi; k = k + 1u) {
        let s = segs[k];
        let pa = points[s.a];
        let pb = points[s.b];
        let sa = point_px(pa.pos);
        let sb = point_px(pb.pos);
        let ra = radius_px(pa.width);
        let rb = radius_px(pb.width);
        let v = sb - sa;
        let len2 = dot(v, v);
        if (len2 <= 1e-12) {
            continue;
        }
        let len = sqrt(len2);

        let rmax = max(ra, rb);
        let w = sa - p;
        let wv = dot(w, v);
        let ww = dot(w, w);
        let disc = wv * wv - len2 * (ww - rmax * rmax);
        if (disc <= 0.0) {
            continue;
        }
        let sq = sqrt(disc);
        let t0 = clamp((-wv - sq) / len2, 0.0, 1.0);
        let t1 = clamp((-wv + sq) / len2, 0.0, 1.0);
        if (t1 <= t0) {
            continue;
        }

        let pitch_min = max(PAINTER_SPACING * 2.0 * min(ra, rb), MIN_PITCH_PX);
        let ds = pitch_min / SUB;
        let n = max(ceil(len / ds), 1.0);
        let step = len / n;
        let i0 = u32(max(floor(t0 * len / step - 0.5), 0.0));
        let i1 = u32(clamp(ceil(t1 * len / step - 0.5), 0.0, n - 1.0));

        for (var i = i0; i <= i1; i = i + 1u) {
            let t = clamp((f32(i) + 0.5) * step / len, 0.0, 1.0);
            let sp = sa + v * t;
            let r = max(ra * (1.0 - t) + rb * t, 1e-4);
            let dn = length(p - sp) / r;
            let fv = f_of(dn, hardness);
            if (fv <= 0.0) {
                continue;
            }
            let pitch = max(PAINTER_SPACING * 2.0 * r, MIN_PITCH_PX);
            let d_tau = fv * step / pitch;
            tau = tau + d_tau;
            let op = pa.opacity * (1.0 - t) + pb.opacity * t;
            let col = pa.color * (1.0 - t) + pb.color * t;
            acc = acc + vec4<f32>(col.x * d_tau, col.y * d_tau, col.z * d_tau, col.w * op * d_tau);
        }
    }
    end_dab(sid, hardness, p, &tau, &acc);
    if (tau <= 0.0) {
        return 0.0;
    }
    *out_rgb = acc / tau;
    return tau;
}

@compute @workgroup_size(16, 16, 1)
fn walk(@builtin(global_invocation_id) gid: vec3<u32>) {
    let w = u32(sc.view.x);
    let h = u32(sc.view.y);
    if (gid.x >= w || gid.y >= h) {
        return;
    }
    let p = vec2<f32>(f32(gid.x) + 0.5, f32(gid.y) + 0.5);
    var acc = vec4<f32>(0.0);

    // `TileBins::tile_of_pixel`
    let tile = sc.grid.x;
    let tx = gid.x / tile;
    let ty = gid.y / tile;
    if (tx < sc.grid.y && ty < sc.grid.z) {
        let rng = ranges[ty * sc.grid.y + tx];
        let lo = rng.x;
        let hi = rng.x + rng.y;
        // `binning.rs::walk_list` — o scan de RUN. A lista já vem ordenada por (traço, segmento),
        // e é essa ordem que torna o agrupamento correto sem ordenar nada aqui.
        var i = lo;
        loop {
            if (i >= hi) { break; }
            let sid = segs[i].stroke;
            var j = i;
            loop {
                if (j >= hi || segs[j].stroke != sid) { break; }
                j = j + 1u;
            }
            // `binning.rs::stroke_deposit`
            let sil = stroke_silhouette(i, j, p);
            let edge = 0.5 - sil.x;
            if (sil.x < 1e29 && edge > 0.0) {
                var p_eval = p;
                if (sil.x > -0.5 && sil.w > 1e-6) {
                    let f = (sil.x + 0.5) * 0.5 / sil.w;
                    p_eval = p + (vec2<f32>(sil.y, sil.z) - p) * f;
                }
                var rgba = vec4<f32>(0.0);
                let tau = stroke_tau(i, j, sid, strokes[sid].hardness, p_eval, &rgba);
                if (tau > 0.0) {
                    let cover = (1.0 - exp(-tau)) * min(edge, 1.0);
                    if (cover > 0.0) {
                        let a = clamp(rgba.w * cover, 0.0, 1.0);
                        acc = vec4<f32>(
                            rgba.x * a + acc.x * (1.0 - a),
                            rgba.y * a + acc.y * (1.0 - a),
                            rgba.z * a + acc.z * (1.0 - a),
                            a + acc.w * (1.0 - a),
                        );
                    }
                }
            }
            i = j;
        }
    }
    textureStore(out_tex, vec2<i32>(i32(gid.x), i32(gid.y)), acc);
}
