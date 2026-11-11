//! WGSL compute shaders for the GPU inpaint path — a faithful, op-for-op mirror
//! of the CPU reference (`nnf.rs` / `vote.rs`). The same per-pixel counter hash
//! (`hash.rs`, 32-bit so WGSL can reproduce it — no `u64`), the same jump-flood
//! propagation, the same random-search draw indexing, and the same gather-vote.
//! One thread per pixel; five entry points driven per pyramid level from
//! `gpu/mod.rs`. All of it is `u32` arithmetic on premultiplied RGBA8 — integer
//! SSD, ties to the lower source index, Q16 vote — so it is the CPU's byte (B38.3).
//!
//! The three per-pixel masks are packed into ONE `flags` buffer (bit0 = source,
//! bit1 = target, bit2 = hole) so the shader uses only 8 storage buffers — the
//! `max_storage_buffers_per_shader_stage` floor on the default device tier.

/// The single shader module source (all five entry points share the helpers).
pub const INPAINT_WGSL: &str = r#"
struct U {
    w: u32,
    h: u32,
    r: i32,
    step: i32,
    em_pass: u32,
    seed: u32,
    n_src: u32,
    max_r: i32,
};

@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var<storage, read_write> content: array<vec4<u32>>;
@group(0) @binding(2) var<storage, read> src: array<vec4<u32>>;
@group(0) @binding(3) var<storage, read> flags: array<u32>;
@group(0) @binding(4) var<storage, read> sources: array<u32>;
@group(0) @binding(5) var<storage, read_write> off_a: array<vec2<i32>>;
@group(0) @binding(6) var<storage, read_write> cost_a: array<u32>;
@group(0) @binding(7) var<storage, read_write> off_b: array<vec2<i32>>;
@group(0) @binding(8) var<storage, read_write> cost_b: array<u32>;

const SALT_INIT: u32 = 0x1117u;
const SALT_SEARCH: u32 = 0x5eed0000u;
const PESO_UM: u32 = 4261478400u; // 2^16 * 255^2

fn cl(v: i32, n: u32) -> i32 { return clamp(v, 0, i32(n) - 1); }

fn cidx(x: i32, y: i32) -> u32 {
    return u32(cl(y, u.h)) * u.w + u32(cl(x, u.w));
}

fn is_source(i: u32) -> bool { return (flags[i] & 1u) != 0u; }
fn is_target(i: u32) -> bool { return (flags[i] & 2u) != 0u; }
fn is_hole(i: u32) -> bool { return (flags[i] & 4u) != 0u; }

fn get_content(x: i32, y: i32) -> vec4<u32> { return content[cidx(x, y)]; }
fn get_src(x: i32, y: i32) -> vec4<u32> { return src[cidx(x, y)]; }

// Integer SSD of the (2r+1)^2 patch: target read from evolving content, source from src.
fn ssd(tx: i32, ty: i32, sx: i32, sy: i32) -> u32 {
    var acc = 0u;
    for (var dy = -u.r; dy <= u.r; dy = dy + 1) {
        for (var dx = -u.r; dx <= u.r; dx = dx + 1) {
            let a = get_content(tx + dx, ty + dy);
            let b = get_src(sx + dx, sy + dy);
            let d = max(a, b) - min(a, b);
            acc = acc + dot(d, d);
        }
    }
    return acc;
}

// Lower score, then the lower (clamped) source index.
fn better(c: u32, si: u32, best: u32, bi: u32) -> bool {
    return c < best || (c == best && si < bi);
}

fn hash32(x0: u32) -> u32 {
    var x = x0;
    x = x ^ (x >> 17u); x = x * 0xed5ad4bbu;
    x = x ^ (x >> 11u); x = x * 0xac4c1b51u;
    x = x ^ (x >> 15u); x = x * 0x31848babu;
    x = x ^ (x >> 14u);
    return x;
}

fn rand_u32(seed: u32, idx: u32, salt: u32, draw: u32) -> u32 {
    let a = hash32(salt ^ (draw * 0x85ebca77u));
    let b = hash32((idx * 0x9e3779b1u) ^ a);
    return hash32(seed ^ b);
}

fn rand_range(seed: u32, idx: u32, salt: u32, draw: u32, lo: i32, hi: i32) -> i32 {
    if (hi <= lo) { return lo; }
    let span = u32(hi - lo + 1);
    return lo + i32(rand_u32(seed, idx, salt, draw) % span);
}

@compute @workgroup_size(64)
fn init_nnf(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= u.w * u.h) { return; }
    if (!is_target(idx)) { return; }
    let tx = i32(idx % u.w);
    let ty = i32(idx / u.w);
    let pick = rand_u32(u.seed, idx, SALT_INIT, 0u) % u.n_src;
    let si = sources[pick];
    let sx = i32(si % u.w);
    let sy = i32(si / u.w);
    off_a[idx] = vec2<i32>(sx - tx, sy - ty);
    cost_a[idx] = ssd(tx, ty, sx, sy);
}

@compute @workgroup_size(64)
fn cost_refresh(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= u.w * u.h) { return; }
    if (!is_target(idx)) { return; }
    let tx = i32(idx % u.w);
    let ty = i32(idx / u.w);
    let o = off_a[idx];
    cost_a[idx] = ssd(tx, ty, tx + o.x, ty + o.y);
}

@compute @workgroup_size(64)
fn propagate(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= u.w * u.h) { return; }
    if (!is_target(idx)) {
        off_b[idx] = off_a[idx];
        cost_b[idx] = cost_a[idx];
        return;
    }
    let tx = i32(idx % u.w);
    let ty = i32(idx / u.w);
    var dxs = array<i32, 8>(1, -1, 0, 0, 1, 1, -1, -1);
    var dys = array<i32, 8>(0, 0, 1, -1, 1, -1, 1, -1);
    var best_off = off_a[idx];
    var best = cost_a[idx];
    var bi = cidx(tx + best_off.x, ty + best_off.y);
    for (var i = 0; i < 8; i = i + 1) {
        let qx = cl(tx + dxs[i] * u.step, u.w);
        let qy = cl(ty + dys[i] * u.step, u.h);
        let cand = off_a[u32(qy) * u.w + u32(qx)];
        let sx = tx + cand.x;
        let sy = ty + cand.y;
        let si = cidx(sx, sy);
        if (is_source(si)) {
            let c = ssd(tx, ty, sx, sy);
            if (better(c, si, best, bi)) { best = c; best_off = cand; bi = si; }
        }
    }
    off_b[idx] = best_off;
    cost_b[idx] = best;
}

@compute @workgroup_size(64)
fn random_search(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= u.w * u.h) { return; }
    if (!is_target(idx)) { return; }
    let tx = i32(idx % u.w);
    let ty = i32(idx / u.w);
    let salt = SALT_SEARCH + u.em_pass;
    var best_off = off_a[idx];
    var best = cost_a[idx];
    var bi = cidx(tx + best_off.x, ty + best_off.y);
    var radius = u.max_r;
    var k = 0u;
    loop {
        if (radius < 1) { break; }
        let jx = rand_range(u.seed, idx, salt, 2u * k, -radius, radius);
        let jy = rand_range(u.seed, idx, salt, 2u * k + 1u, -radius, radius);
        let sx = cl(tx + best_off.x + jx, u.w);
        let sy = cl(ty + best_off.y + jy, u.h);
        let si = u32(sy) * u.w + u32(sx);
        if (is_source(si)) {
            let c = ssd(tx, ty, sx, sy);
            if (better(c, si, best, bi)) { best = c; best_off = vec2<i32>(sx - tx, sy - ty); bi = si; }
        }
        radius = radius / 2;
        k = k + 1u;
    }
    off_a[idx] = best_off;
    cost_a[idx] = best;
}

@compute @workgroup_size(64)
fn vote(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= u.w * u.h) { return; }
    if (!is_hole(idx)) { return; }
    let px = i32(idx % u.w);
    let py = i32(idx / u.w);
    var sum = vec4<u32>(0u, 0u, 0u, 0u);
    var wsum = 0u;
    for (var cdy = -u.r; cdy <= u.r; cdy = cdy + 1) {
        for (var cdx = -u.r; cdx <= u.r; cdx = cdx + 1) {
            let cx = px + cdx;
            let cy = py + cdy;
            if (cx < 0 || cx >= i32(u.w) || cy < 0 || cy >= i32(u.h)) { continue; }
            let ci = u32(cy) * u.w + u32(cx);
            if (!is_target(ci)) { continue; }
            let o = off_a[ci];
            let w = PESO_UM / (65025u + cost_a[ci]);
            let col = get_src(cx + o.x - cdx, cy + o.y - cdy);
            sum = sum + w * col;
            wsum = wsum + w;
        }
    }
    if (wsum > 0u) {
        content[idx] = (sum + vec4<u32>(wsum / 2u)) / wsum;
    }
}
"#;
