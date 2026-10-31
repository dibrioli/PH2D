// A advecção Jacobi/gather do Wet Paint na placa — a mesma lei de
// `ph2d_wet_paint::solver::advect_jacobi` (prepare → outflow → transport → commit),
// em f32. Todo mundo lê o estado do INÍCIO do passo; nada de atômicos.
// A mistura de cor é a média LINEAR ponderada (km_mixing = false, o default).

struct U {
    w: i32, h: i32, s: i32, reach: i32,
    // Retângulo varrido, inclusivo, em coordenadas de célula (1..=w, 1..=h).
    x0: i32, y0: i32, x1: i32, y1: i32,
    // Grade de fluxo: stride, largura, altura, razão fina/fluxo.
    fs: i32, fw: i32, fh: i32, rf: i32,
    half: i32, inv_rf: f32, _p0: f32, _p1: f32,
}

@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var<storage, read> flow_x: array<f32>;
@group(0) @binding(2) var<storage, read> flow_y: array<f32>;
@group(0) @binding(3) var<storage, read> act: array<u32>;
@group(0) @binding(4) var<storage, read_write> uv: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read_write> outflow: array<f32>;
@group(0) @binding(6) var<storage, read_write> st: array<vec2<f32>>;
@group(0) @binding(7) var<storage, read_write> rgb: array<f32>;
@group(0) @binding(8) var<storage, read_write> dst: array<vec2<f32>>;
@group(0) @binding(9) var<storage, read_write> drgb: array<f32>;

// A coordenada na grade de fluxo, em inteiro + fração (exata: sem `x·inv_rf` em f32 a 4096).
struct Fc { i0: i32, a: f32 }
fn flow_coord(c: i32, n: i32) -> Fc {
    let t = c - 1 - u.half;
    if t < 0 { return Fc(1, 0.0); }
    let q = t / u.rf;
    let i0 = q + 1;
    if i0 >= n { return Fc(n, 0.0); }
    return Fc(i0, f32(t - q * u.rf) * u.inv_rf);
}

// O retro-traço `s = c − uv`: base inteira + fração, tirados de `−uv` (pequeno) e não de
// `c − uv` (a 4096 o ulp de f32 seria 5e-4 na fração).
struct Bt { ok: bool, x0: i32, fx: f32, y0: i32, fy: f32 }
fn split(c: i32, d: f32) -> vec2<f32> {
    let fl = floor(d);
    var f = d - fl;
    var b = c + i32(fl);
    if f >= 1.0 { f = 0.0; b = b + 1; }
    return vec2<f32>(bitcast<f32>(b), f);
}
fn backtrace(x: i32, y: i32, v: vec2<f32>) -> Bt {
    let sx = split(x, -v.x);
    let sy = split(y, -v.y);
    let bx = bitcast<i32>(sx.x);
    let by = bitcast<i32>(sy.x);
    let fora = bx < 1 || bx > u.w || (bx == u.w && sx.y > 0.0)
        || by < 1 || by > u.h || (by == u.h && sy.y > 0.0);
    return Bt(!fora, bx, sx.y, by, sy.y);
}

fn corner_weight(c: i32, base: i32, frac: f32) -> f32 {
    if c == base { return 1.0 - frac; }
    if c == base + 1 { return frac; }
    return 0.0;
}

fn take_scale(o: f32) -> f32 {
    if o > 1.0 { return 1.0 / o; }
    return 1.0;
}

struct Cell { ok: bool, x: i32, y: i32, i: i32 }
fn cell(g: vec3<u32>) -> Cell {
    let x = u.x0 + i32(g.x);
    let y = u.y0 + i32(g.y);
    return Cell(x <= u.x1 && y <= u.y1, x, y, x + y * u.s);
}

@compute @workgroup_size(16, 16, 1)
fn prepare(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = cell(g);
    if !c.ok { return; }
    let cx = flow_coord(c.x, u.fw);
    let cy = flow_coord(c.y, u.fh);
    let i00 = cx.i0 + cy.i0 * u.fs;
    let a = cx.a;
    let b = cy.a;
    let w00 = (1.0 - a) * (1.0 - b);
    let w10 = a * (1.0 - b);
    let w01 = (1.0 - a) * b;
    let w11 = a * b;
    let ux = flow_x[i00] * w00 + flow_x[i00 + 1] * w10 + flow_x[i00 + u.fs] * w01 + flow_x[i00 + u.fs + 1] * w11;
    let uy = flow_y[i00] * w00 + flow_y[i00 + 1] * w10 + flow_y[i00 + u.fs] * w01 + flow_y[i00 + u.fs + 1] * w11;
    uv[c.i] = vec2<f32>(ux, uy);
}

@compute @workgroup_size(16, 16, 1)
fn outflow_pass(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = cell(g);
    if !c.ok { return; }
    var acc = 0.0;
    // A ordem da soma é a da CPU: linha-destino crescente, depois coluna.
    let ny0 = max(max(c.y - u.reach, 1), u.y0);
    let ny1 = min(min(c.y + u.reach, u.h), u.y1);
    let nx0 = max(c.x - u.reach, u.x0);
    let nx1 = min(c.x + u.reach, u.x1);
    for (var ny = ny0; ny <= ny1; ny++) {
        for (var nx = nx0; nx <= nx1; nx++) {
            let di = nx + ny * u.s;
            if act[di] == 0u { continue; }
            let t = backtrace(nx, ny, uv[di]);
            if !t.ok { continue; }
            let wy = corner_weight(c.y, t.y0, t.fy);
            if wy == 0.0 { continue; }
            if t.x0 == c.x {
                acc += (1.0 - t.fx) * wy;
            } else if t.x0 + 1 == c.x {
                acc += t.fx * wy;
            }
        }
    }
    outflow[c.i] = acc;
}

@compute @workgroup_size(16, 16, 1)
fn transport(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = cell(g);
    if !c.ok { return; }
    let i = c.i;
    let fm = st[i];
    let leave = min(outflow[i], 1.0);
    var nf = fm.x - fm.x * leave;
    var nm = fm.y - fm.y * leave;
    var col = vec3<f32>(rgb[3 * i], rgb[3 * i + 1], rgb[3 * i + 2]);
    if act[i] != 0u {
        let t = backtrace(c.x, c.y, uv[i]);
        if t.ok {
            let i00 = t.x0 + t.y0 * u.s;
            let i10 = i00 + 1;
            let i01 = i00 + u.s;
            let i11 = i01 + 1;
            let fx = t.fx;
            let fy = t.fy;
            let k00 = (1.0 - fx) * (1.0 - fy) * take_scale(outflow[i00]);
            let k10 = fx * (1.0 - fy) * take_scale(outflow[i10]);
            let k01 = (1.0 - fx) * fy * take_scale(outflow[i01]);
            let k11 = fx * fy * take_scale(outflow[i11]);
            let s00 = st[i00];
            let s10 = st[i10];
            let s01 = st[i01];
            let s11 = st[i11];
            let p00 = s00.y * k00;
            let p10 = s10.y * k10;
            let p01 = s01.y * k01;
            let p11 = s11.y * k11;
            let want = p00 + p10 + p01 + p11;
            if want > 0.0 {
                let inv = 1.0 / want;
                let c00 = vec3<f32>(rgb[3 * i00], rgb[3 * i00 + 1], rgb[3 * i00 + 2]);
                let c10 = vec3<f32>(rgb[3 * i10], rgb[3 * i10 + 1], rgb[3 * i10 + 2]);
                let c01 = vec3<f32>(rgb[3 * i01], rgb[3 * i01 + 1], rgb[3 * i01 + 2]);
                let c11 = vec3<f32>(rgb[3 * i11], rgb[3 * i11 + 1], rgb[3 * i11 + 2]);
                col = (c00 * p00 + c10 * p10 + c01 * p01 + c11 * p11) * inv;
                nm += want;
            }
            nf += s00.x * k00 + s10.x * k10 + s01.x * k01 + s11.x * k11;
        }
    }
    dst[i] = vec2<f32>(nf, nm);
    drgb[3 * i] = col.x;
    drgb[3 * i + 1] = col.y;
    drgb[3 * i + 2] = col.z;
}

@compute @workgroup_size(16, 16, 1)
fn commit(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = cell(g);
    if !c.ok { return; }
    let i = c.i;
    st[i] = dst[i];
    rgb[3 * i] = drgb[3 * i];
    rgb[3 * i + 1] = drgb[3 * i + 1];
    rgb[3 * i + 2] = drgb[3 * i + 2];
}
