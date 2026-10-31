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
    half: i32, inv_rf: f32, gx: f32, gy: f32,
    max_v: f32, _p2: f32, _p3: f32, _p4: f32,
    // Retângulo na grade de fluxo, inclusivo.
    fx0: i32, fy0: i32, fx1: i32, fy1: i32,
}

@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var<storage, read_write> flow_x: array<f32>;
@group(0) @binding(2) var<storage, read_write> flow_y: array<f32>;
@group(0) @binding(3) var<storage, read> act: array<u32>;
@group(0) @binding(4) var<storage, read_write> uv: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read_write> outflow: array<f32>;
@group(0) @binding(6) var<storage, read_write> st: array<vec2<f32>>;
@group(0) @binding(7) var<storage, read_write> rgb: array<f32>;
@group(0) @binding(8) var<storage, read_write> dst: array<vec2<f32>>;
@group(0) @binding(9) var<storage, read_write> drgb: array<f32>;
@group(0) @binding(10) var<storage, read_write> vel_x: array<f32>;
@group(0) @binding(11) var<storage, read_write> vel_y: array<f32>;

// A coordenada na grade de fluxo, em inteiro + fração (exata: sem `x·inv_rf` em f32 a 4096).
// A posição fina é `b + f` (`b` inteiro, `f ∈ [0,1)`); `n` = largura da grade de fluxo.
struct Fc { i0: i32, a: f32 }
fn flow_coord_frac(b: i32, f: f32, n: i32) -> Fc {
    let t = b - 1 - u.half;
    var q = t / u.rf;
    if t < 0 && q * u.rf != t { q = q - 1; } // divisão de piso
    var i0 = q + 1;
    var a = (f32(t - q * u.rf) + f) * u.inv_rf;
    if a >= 1.0 { a = 0.0; i0 = i0 + 1; }
    if i0 < 1 { return Fc(1, 0.0); }
    if i0 >= n { return Fc(n, 0.0); }
    return Fc(i0, a);
}
fn flow_coord(c: i32, n: i32) -> Fc { return flow_coord_frac(c, 0.0, n); }

// O fluxo bilinear numa posição de fluxo (a ordem de termos do `flow_at_point`).
fn flow_bilin(cx: Fc, cy: Fc) -> vec2<f32> {
    let i00 = cx.i0 + cy.i0 * u.fs;
    let a = cx.a;
    let b = cy.a;
    let w00 = (1.0 - a) * (1.0 - b);
    let w10 = a * (1.0 - b);
    let w01 = (1.0 - a) * b;
    let w11 = a * b;
    let ux = flow_x[i00] * w00 + flow_x[i00 + 1] * w10 + flow_x[i00 + u.fs] * w01 + flow_x[i00 + u.fs + 1] * w11;
    let uy = flow_y[i00] * w00 + flow_y[i00 + 1] * w10 + flow_y[i00 + u.fs] * w01 + flow_y[i00 + u.fs + 1] * w11;
    return vec2<f32>(ux, uy);
}

// `flow::flow_probe`: a célula fina que representa a célula de fluxo `c`.
fn flow_probe(c: i32, w: i32) -> i32 {
    if c <= 0 { return 0; }
    let fw = (w + u.rf - 1) / u.rf;
    if c > fw { return w + 1; }
    return min((c - 1) * u.rf + 1 + u.half, w);
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
    uv[c.i] = flow_bilin(flow_coord(c.x, u.fw), flow_coord(c.y, u.fh));
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

// ---- grade de FLUXO ------------------------------------------------------------------------

struct FCell { ok: bool, x: i32, y: i32, i: i32 }
fn fcell(g: vec3<u32>) -> FCell {
    let x = u.fx0 + i32(g.x);
    let y = u.fy0 + i32(g.y);
    return FCell(x <= u.fx1 && y <= u.fy1, x, y, x + y * u.fs);
}

// `smooth_velocity_rows` sem gota e sem paredes: média de 5 pontos da velocidade persistente.
@compute @workgroup_size(16, 16, 1)
fn suaviza(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = fcell(g);
    if !c.ok { return; }
    let fi = flow_probe(c.x, u.w) + flow_probe(c.y, u.h) * u.s;
    if act[fi] == 0u { return; }
    let i = c.i;
    var fx = vel_x[i];
    var fy = vel_y[i];
    // A CPU compara `film as f64 > 0.05`: o menor f32 acima de 0,05 (f64) é 0,05f32.
    if st[fi].x >= 0.05 {
        fx = 0.2 * vel_x[i] + 0.2 * (vel_x[i - 1] + vel_x[i + 1] + vel_x[i - u.fs] + vel_x[i + u.fs]);
        fy = 0.2 * vel_y[i] + 0.2 * (vel_y[i - 1] + vel_y[i + 1] + vel_y[i - u.fs] + vel_y[i + u.fs]);
    }
    flow_x[i] = clamp(fx, -u.max_v, u.max_v);
    flow_y[i] = clamp(fy, -u.max_v, u.max_v);
}

// `momentum_rows`: o fluxo na origem do retro-traço + gravidade × água daqui.
@compute @workgroup_size(16, 16, 1)
fn momentum(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = fcell(g);
    if !c.ok { return; }
    let px = flow_probe(c.x, u.w);
    let py = flow_probe(c.y, u.h);
    if py < u.y0 || py > u.y1 || px < u.x0 || px > u.x1 { return; }
    let i = px + py * u.s;
    if act[i] == 0u {
        vel_x[c.i] = 0.0;
        vel_y[c.i] = 0.0;
        return;
    }
    let v = flow_bilin(flow_coord(px, u.fw), flow_coord(py, u.fh));
    let t = backtrace(px, py, v);
    if !t.ok { return; }
    let sf = flow_bilin(flow_coord_frac(t.x0, t.fx, u.fw), flow_coord_frac(t.y0, t.fy, u.fh));
    let f = st[i].x;
    vel_x[c.i] = clamp(sf.x + u.gx * f, -u.max_v, u.max_v);
    vel_y[c.i] = clamp(sf.y + u.gy * f, -u.max_v, u.max_v);
}

// ---- o CAMPO de fluxo (`build_flow_field_jacobi`, quadros n % 4 == 0) ----------------------
// O caminho do produto por omissão: sem ladeira do óleo, sem paredes (traço vazio), sem gota
// (`drip_head = 0`), sem fingering e sem backrun (knobs a 0). Ver PLACA_degrau3.md.

struct K {
    level_k: f32, level_clamp: f32, cap_k: f32, cap_gate: f32,
    // `visc_t`: o menor f32 cujo f64 passa `> viscosity` (a CPU compara em f64) — compara-se `>=`.
    visc_t: f32, brake_bias: f32, brake_reach: f32, dif: f32,
}
@group(0) @binding(12) var<storage, read> paper: array<f32>;
@group(0) @binding(13) var<storage, read_write> wet: array<u32>;
@group(0) @binding(14) var<storage, read> sett: array<f32>;
@group(0) @binding(15) var<uniform> k: K;

// `flow_rows`: o núcleo gather — escreve `flow` e mais nada; lê o `wet` do INÍCIO do passo.
@compute @workgroup_size(16, 16, 1)
fn campo(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = fcell(g);
    if !c.ok { return; }
    let px = flow_probe(c.x, u.w);
    let pl = flow_probe(c.x - 1, u.w);
    let pr = flow_probe(c.x + 1, u.w);
    let ym = flow_probe(c.y, u.h) * u.s;
    let yu = flow_probe(c.y - 1, u.h) * u.s;
    let yd = flow_probe(c.y + 1, u.h) * u.s;
    let fi = px + ym;
    if act[fi] == 0u { return; }
    let i = c.i;
    let f = st[fi].x;
    var ex = vel_x[i];
    var ey = vel_y[i];
    // Leveling: do grosso para o fino, clampado por eixo.
    ex += clamp((st[pl + ym].x - st[pr + ym].x) * k.level_k * k.dif, -k.level_clamp, k.level_clamp);
    ey += clamp((st[px + yu].x - st[px + yd].x) * k.level_k * k.dif, -k.level_clamp, k.level_clamp);
    // Capilar: só tinta fina segue o dente do papel.
    if st[fi].y + sett[fi] < k.cap_gate {
        let pc = paper[fi];
        let a = paper[pl + ym];
        let b = paper[pr + ym];
        var dx: f32;
        if a > pc { if pc < b { dx = a - b; } else { dx = pc - b; } }
        else if pc < b { dx = a - pc; } else { dx = a - b; }
        ex += dx * k.cap_k * k.dif;
        let up = paper[px + yu];
        let dn = paper[px + yd];
        var dy: f32;
        if up > pc { if pc < dn { dy = up - dn; } else { dy = pc - dn; } }
        else if pc < dn { dy = up - pc; } else { dy = up - dn; }
        ey += dy * k.cap_k * k.dif;
    }
    // Viscosidade: média de 5 pontos do campo persistente.
    if f >= k.visc_t {
        ex = 0.2 * ex + 0.2 * (vel_x[i - 1] + vel_x[i + 1] + vel_x[i - u.fs] + vel_x[i + u.fs]);
        ey = 0.2 * ey + 0.2 * (vel_y[i - 1] + vel_y[i + 1] + vel_y[i - u.fs] + vel_y[i + u.fs]);
    }
    // Freio de absorvência por look-ahead.
    let s_len = k.brake_reach * k.dif / (sqrt(ex * ex + ey * ey) + 0.01);
    let dxs = i32(trunc(ex * s_len));
    let dys = i32(trunc(ey * s_len));
    let probe = i + dxs + dys * u.fs;
    let probe_f = fi + dxs * u.rf + dys * u.rf * u.s;
    if probe >= 0 && probe < i32(arrayLength(&flow_x)) && probe_f >= 0 && probe_f < i32(arrayLength(&act)) {
        let br = clamp(st[probe_f].x + (3.0 / 255.0) * f32(wet[probe_f]) - k.brake_bias, 0.05, 1.0);
        ex *= br;
        ey *= br;
    }
    flow_x[i] = clamp(ex, -u.max_v, u.max_v);
    flow_y[i] = clamp(ey, -u.max_v, u.max_v);
}

// `wet_stamp_rows`: film fundo marca a folha úmida (corre DEPOIS do `campo`).
@compute @workgroup_size(16, 16, 1)
fn carimbo(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = fcell(g);
    if !c.ok { return; }
    let fi = flow_probe(c.x, u.w) + flow_probe(c.y, u.h) * u.s;
    if act[fi] == 0u { return; }
    if st[fi].x > 3.0 {
        wet[fi] = u32(clamp(2.0 - 2.0 * paper[fi], 0.0, 1.0) * 255.0);
    }
}
