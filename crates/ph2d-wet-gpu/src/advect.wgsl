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
// `gx = 1 − fx` sai de `(fl + 1) − d` e não de `1 − f`: com `d = −1e-10` a fração `f` arredonda a 1 em f32 e o
// peso do canto de baixo seria 0, quando a CPU (f64) lhe dá 1e-10 — o resíduo que a parede entre traços vê
// (`susp > 0`). Sem o «embrulho» `f ≥ 1 ⇒ base + 1`: a base fica a da CPU (`floor` em f64).
struct Bt { ok: bool, x0: i32, fx: f32, y0: i32, fy: f32, gx: f32, gy: f32 }
fn split(c: i32, d: f32) -> vec3<f32> {
    let fl = floor(d);
    return vec3<f32>(bitcast<f32>(c + i32(fl)), d - fl, (fl + 1.0) - d);
}
fn backtrace(x: i32, y: i32, v: vec2<f32>) -> Bt {
    let sx = split(x, -v.x);
    let sy = split(y, -v.y);
    let bx = bitcast<i32>(sx.x);
    let by = bitcast<i32>(sy.x);
    let fora = bx < 1 || bx > u.w || (bx == u.w && sx.y > 0.0)
        || by < 1 || by > u.h || (by == u.h && sy.y > 0.0);
    return Bt(!fora, bx, sx.y, by, sy.y, sx.z, sy.z);
}

fn corner_weight(c: i32, base: i32, frac: f32, comp: f32) -> f32 {
    if c == base { return comp; }
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
            let wy = corner_weight(c.y, t.y0, t.fy, t.gy);
            if wy == 0.0 { continue; }
            if t.x0 == c.x {
                acc += t.gx * wy;
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
    // `m·(1 − leave)`: `1 − leave` é exacto em f32 (Sterbenz, leave ≥ 0,5), e o resíduo de uma célula que quase
    // se esvazia fica > 0 como o `m − m·leave` em f64 da CPU (o `m − m·leave` em f32 dava 0 — e a parede entre
    // traços pergunta `susp > 0`).
    var nf = fm.x * (1.0 - leave);
    var nm = fm.y * (1.0 - leave);
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
            let gx = t.gx;
            let gy = t.gy;
            let k00 = gx * gy * take_scale(outflow[i00]);
            let k10 = fx * gy * take_scale(outflow[i10]);
            let k01 = gx * fy * take_scale(outflow[i01]);
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

// `smooth_velocity_rows` sem gota: média de 5 pontos da velocidade persistente.
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
// O caminho do produto por omissão: sem ladeira do óleo, sem gota
// (`drip_head = 0`), sem fingering e sem backrun (knobs a 0). Ver PLACA_degrau3.md.

struct K {
    level_k: f32, level_clamp: f32, cap_k: f32, cap_gate: f32,
    // `visc_t`: o menor f32 cujo f64 passa `> viscosity` (a CPU compara em f64) — compara-se `>=`.
    visc_t: f32, brake_bias: f32, brake_reach: f32, dif: f32,
    // `project`: o knob `Projection`. `sedimenta`: fracções por eixo e sentido (−1/0/1).
    projection: f32, sed_ax: f32, sed_ay: f32, _k0: f32,
    // `freio_fino`: 1 = o freio do B13.10 (sonda na grade fina), 0 = o de antes (índice linear).
    sed_dx: i32, sed_dy: i32, freio_fino: i32, _k2: i32,
}
@group(0) @binding(12) var<storage, read> paper: array<f32>;
@group(0) @binding(13) var<storage, read_write> wet: array<u32>;
@group(0) @binding(14) var<storage, read> sett: array<f32>;
@group(0) @binding(15) var<uniform> k: K;

// `flow_rows`: o núcleo gather até ao freio; quem escreve `flow` é o `campo` (ou o `campo_oleo`).
struct CampoV { ok: bool, i: i32, fi: i32, f: f32, e: vec2<f32> }
fn campo_v(g: vec3<u32>) -> CampoV {
    let c = fcell(g);
    if !c.ok { return CampoV(false, 0, 0, 0.0, vec2<f32>(0.0, 0.0)); }
    let px = flow_probe(c.x, u.w);
    let pl = flow_probe(c.x - 1, u.w);
    let pr = flow_probe(c.x + 1, u.w);
    let ym = flow_probe(c.y, u.h) * u.s;
    let yu = flow_probe(c.y - 1, u.h) * u.s;
    let yd = flow_probe(c.y + 1, u.h) * u.s;
    let fi = px + ym;
    if act[fi] == 0u { return CampoV(false, 0, 0, 0.0, vec2<f32>(0.0, 0.0)); }
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
    // B13.10 ligado (`flow::sonda_do_freio`): o passo na grade FINA a partir da amostra `(px, fy)`, limites 2D.
    // Desligado (a omissão, `flow::sonda_do_freio_linear`): o índice linear em células de fluxo.
    var probe_f: i32;
    var dentro: bool;
    if k.freio_fino != 0 {
        let s_len = k.brake_reach / (sqrt(ex * ex + ey * ey) + 0.01);
        let qx = px + i32(trunc(ex * s_len));
        let qy = flow_probe(c.y, u.h) + i32(trunc(ey * s_len));
        let rows = i32(arrayLength(&act)) / u.s;
        probe_f = qy * u.s + qx;
        dentro = qx >= 0 && qx < u.s && qy >= 0 && qy < rows;
    } else {
        let s_len = k.brake_reach * k.dif / (sqrt(ex * ex + ey * ey) + 0.01);
        let dxs = i32(trunc(ex * s_len));
        let dys = i32(trunc(ey * s_len));
        let probe = i + dxs + dys * u.fs;
        probe_f = fi + dxs * u.rf + dys * u.rf * u.s;
        dentro = probe >= 0 && probe < i32(arrayLength(&flow_x)) && probe_f >= 0 && probe_f < i32(arrayLength(&act));
    }
    if dentro {
        let br = clamp(st[probe_f].x + (3.0 / 255.0) * f32(wet[probe_f]) - k.brake_bias, 0.05, 1.0);
        ex *= br;
        ey *= br;
    }
    return CampoV(true, i, fi, f, vec2<f32>(ex, ey));
}

@compute @workgroup_size(16, 16, 1)
fn campo(@builtin(global_invocation_id) g: vec3<u32>) {
    let r = campo_v(g);
    if !r.ok { return; }
    flow_x[r.i] = clamp(r.e.x, -u.max_v, u.max_v);
    flow_y[r.i] = clamp(r.e.y, -u.max_v, u.max_v);
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

// ---- bordas (`apply_boundaries`): o dreno de duas células em cada lado -----------------------

// A metade FINA (massa): film = susp = 0, wet = 1 nas linhas 0,1,h,h+1 e colunas 0,1,w,w+1.
@compute @workgroup_size(16, 16, 1)
fn bordas_massa(@builtin(global_invocation_id) g: vec3<u32>) {
    let x = i32(g.x);
    let y = i32(g.y);
    if x > u.w + 1 || y > u.h + 1 { return; }
    if !(y <= 1 || y >= u.h || x <= 1 || x >= u.w) { return; }
    let i = x + y * u.s;
    st[i] = vec2<f32>(0.0, 0.0);
    wet[i] = 1u;
}

// A metade de FLUXO (vel e flow): tangencial a 0, depois a normal para FORA a 0,1 — na ordem da CPU
// (topo → base, esquerda → direita: o último escrito vence nos cantos).
@compute @workgroup_size(16, 16, 1)
fn bordas_vel(@builtin(global_invocation_id) g: vec3<u32>) {
    let x = i32(g.x);
    let y = i32(g.y);
    if x > u.fw + 1 || y > u.fh + 1 { return; }
    let topo = y <= 1;
    let base = y >= u.fh;
    let esq = x <= 1;
    let dir = x >= u.fw;
    if !(topo || base || esq || dir) { return; }
    let i = x + y * u.fs;
    var vx = vel_x[i];
    var fx = flow_x[i];
    var vy = vel_y[i];
    var fy = flow_y[i];
    if topo || base { vx = 0.0; fx = 0.0; }
    if esq || dir { vy = 0.0; fy = 0.0; }
    if topo { vy = -0.1; fy = -0.1; }
    if base { vy = 0.1; fy = 0.1; }
    if esq { vx = -0.1; fx = -0.1; }
    if dir { vx = 0.1; fx = 0.1; }
    vel_x[i] = vx;
    flow_x[i] = fx;
    vel_y[i] = vy;
    flow_y[i] = fy;
}

// ---- `project` (n % 3 == 0): div em flow_x, pressão em flow_y, correção em vel ----------------

fn ativo_fluxo(x: i32, y: i32) -> bool {
    return act[flow_probe(x, u.w) + flow_probe(y, u.h) * u.s] != 0u;
}

// O rascunho a zero no retângulo de fluxo com a margem de 1 que o estêncil pede.
@compute @workgroup_size(16, 16, 1)
fn proj_zera(@builtin(global_invocation_id) g: vec3<u32>) {
    let x = u.fx0 - 1 + i32(g.x);
    let y = u.fy0 - 1 + i32(g.y);
    if x < 0 || y < 0 || x > min(u.fx1 + 1, u.fs - 1) || y > min(u.fy1 + 1, u.fh + 1) { return; }
    let i = x + y * u.fs;
    flow_x[i] = 0.0;
    flow_y[i] = 0.0;
}

@compute @workgroup_size(16, 16, 1)
fn proj_div(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = fcell(g);
    if !c.ok || !ativo_fluxo(c.x, c.y) { return; }
    let i = c.i;
    flow_x[i] = (vel_x[i - 1] - vel_x[i + 1] + vel_y[i - u.fs] - vel_y[i + u.fs]) * k.dif;
}

@compute @workgroup_size(16, 16, 1)
fn proj_prs(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = fcell(g);
    if !c.ok || !ativo_fluxo(c.x, c.y) { return; }
    let i = c.i;
    flow_y[i] = (flow_x[i] + 0.25 * (flow_x[i - 1] + flow_x[i + 1] + flow_x[i - u.fs] + flow_x[i + u.fs])) * 0.25;
}

@compute @workgroup_size(16, 16, 1)
fn proj_corr(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = fcell(g);
    if !c.ok || !ativo_fluxo(c.x, c.y) { return; }
    let i = c.i;
    var nvx = vel_x[i] - 0.5 * (flow_y[i + 1] - flow_y[i - 1]) * k.dif * k.projection;
    var nvy = vel_y[i] - 0.5 * (flow_y[i + u.fs] - flow_y[i - u.fs]) * k.dif * k.projection;
    if f32(c.x) - nvx < 1.0 || f32(c.x) - nvx > f32(u.fw) { nvx = 0.0; }
    if f32(c.y) - nvy < 1.0 || f32(c.y) - nvy > f32(u.fh) { nvy = 0.0; }
    vel_x[i] = nvx;
    vel_y[i] = nvy;
}

// ---- `sedimenta` (só com `p.sedimenta > 0`): o pigmento desce mais que a água -----------------
// Escreve `dst = (film, susp')` e `drgb`; o `commit` copia de volta (film inalterado).

fn liga(fc: f32, fd: f32) -> f32 {
    if fc <= 0.0 { return 0.0; }
    return max(min(fc, fd), 0.0) / fc;
}
fn sed_dentro(x: i32, y: i32) -> bool {
    return x >= 1 && x <= u.w && y >= 1 && y <= u.h;
}
fn sed_da(x: i32, y: i32) -> vec2<f32> {
    let i = x + y * u.s;
    var fx = 0.0;
    var fy = 0.0;
    if k.sed_dx != 0 && sed_dentro(x + k.sed_dx, y) {
        fx = k.sed_ax * liga(st[i].x, st[i + k.sed_dx].x);
    }
    if k.sed_dy != 0 && sed_dentro(x, y + k.sed_dy) {
        fy = k.sed_ay * liga(st[i].x, st[i + k.sed_dy * u.s].x);
    }
    return vec2<f32>(fx, fy);
}
fn cor(i: i32) -> vec3<f32> {
    return vec3<f32>(rgb[3 * i], rgb[3 * i + 1], rgb[3 * i + 2]);
}

@compute @workgroup_size(16, 16, 1)
fn sedimenta(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = cell(g);
    if !c.ok { return; }
    let i = c.i;
    let o = sed_da(c.x, c.y);
    let fica = st[i].y * (1.0 - o.x - o.y);
    var e1 = 0.0;
    var j1 = i;
    var e2 = 0.0;
    var j2 = i;
    if k.sed_dx != 0 && sed_dentro(c.x - k.sed_dx, c.y) {
        j1 = i - k.sed_dx;
        e1 = st[j1].y * sed_da(c.x - k.sed_dx, c.y).x;
    }
    if k.sed_dy != 0 && sed_dentro(c.x, c.y - k.sed_dy) {
        j2 = i - k.sed_dy * u.s;
        e2 = st[j2].y * sed_da(c.x, c.y - k.sed_dy).y;
    }
    let total = fica + e1 + e2;
    var col = cor(i);
    if e1 + e2 > 0.0 && total > 0.0 {
        col = (cor(i) * fica + cor(j1) * e1 + cor(j2) * e2) * (1.0 / total);
    }
    dst[i] = vec2<f32>(st[i].x, total);
    drgb[3 * i] = col.x;
    drgb[3 * i + 1] = col.y;
    drgb[3 * i + 2] = col.z;
}

// ---- `rebuild_active_region` (n % 2 == 0) --------------------------------------------------
// A máscara e a caixa nova. A SAIA é sequencial de cima para baixo, mas só dentro da COLUNA
// (lê e escreve `i−s, i, i+s`): uma thread por coluna, laço em y — a mesma ordem da CPU.
// `caixa`: 0..3 extensão do passe 1 (min x, max x, min y, max y) · 4 disparou · 5..8 a da saia ·
// 9 a saia disparou · 10..13 a caixa publicada (bx0, bx1, by0, by1) · 14 has_fluid.

@group(0) @binding(16) var<storage, read_write> act_w: array<u32>;
@group(0) @binding(17) var<storage, read_write> caixa: array<atomic<i32>>;

struct Janela { px0: i32, px1: i32, py0: i32, py1: i32 }
fn janela() -> Janela {
    return Janela(max(u.x0 - 2, 1), min(u.x1 + 2, u.w), max(u.y0 - 2, 1), min(u.y1 + 2, u.h));
}

@compute @workgroup_size(1, 1, 1)
fn re_zera() {
    atomicStore(&caixa[0], u.w + 1);
    atomicStore(&caixa[1], 0);
    atomicStore(&caixa[2], u.h + 1);
    atomicStore(&caixa[3], 0);
    atomicStore(&caixa[4], 0);
    atomicStore(&caixa[5], u.w + 1);
    atomicStore(&caixa[6], 0);
    atomicStore(&caixa[7], u.h + 1);
    atomicStore(&caixa[8], 0);
    atomicStore(&caixa[9], 0);
}

// Limpa a caixa anterior ±2 (até o anel quando ela toca a borda).
@compute @workgroup_size(16, 16, 1)
fn re_limpa(@builtin(global_invocation_id) g: vec3<u32>) {
    let x = i32(g.x);
    let y = i32(g.y);
    let j = janela();
    var cx0 = j.px0;
    if j.px0 == 1 { cx0 = 0; }
    var cx1 = j.px1;
    if j.px1 == u.w { cx1 = u.w + 1; }
    var cy0 = j.py0;
    if j.py0 == 1 { cy0 = 0; }
    var cy1 = j.py1;
    if j.py1 == u.h { cy1 = u.h + 1; }
    if x < cx0 || x > cx1 || y < cy0 || y > cy1 { return; }
    act_w[x + y * u.s] = 0u;
}

// Passe 1 em GATHER: a célula é ativa se algum trio horizontal centrado em x−1, x ou x+1 molha.
// O film é ≥ 0, então `a + b + c > 0` em f32 é o mesmo predicado da soma f64 da CPU.
@compute @workgroup_size(16, 16, 1)
fn re_passe1(@builtin(global_invocation_id) g: vec3<u32>) {
    let x = i32(g.x);
    let y = i32(g.y);
    let j = janela();
    let sx0 = max(j.px0, 2);
    let sx1 = min(j.px1, u.w - 1);
    let sy0 = max(j.py0, 3);
    let sy1 = min(j.py1, u.h - 2);
    if y < sy0 || y > sy1 || x < sx0 - 1 || x > sx1 + 1 { return; }
    var liga_aqui = false;
    for (var cx = x - 1; cx <= x + 1; cx++) {
        if cx < sx0 || cx > sx1 { continue; }
        let i = cx + y * u.s;
        if st[i - 1].x + st[i].x + st[i + 1].x > 0.0 {
            liga_aqui = true;
            if cx == x {
                atomicMin(&caixa[0], x);
                atomicMax(&caixa[1], x);
                atomicMin(&caixa[2], y);
                atomicMax(&caixa[3], y);
                atomicStore(&caixa[4], 1);
            }
        }
    }
    if liga_aqui { act_w[x + y * u.s] = 1u; }
}

@compute @workgroup_size(64, 1, 1)
fn re_saia(@builtin(global_invocation_id) g: vec3<u32>) {
    if atomicLoad(&caixa[4]) == 0 { return; }
    let x = i32(g.x);
    let kx0 = max(atomicLoad(&caixa[0]) - 2, 1);
    let kx1 = min(atomicLoad(&caixa[1]) + 2, u.w);
    let ky0 = max(atomicLoad(&caixa[2]) - 2, 1);
    let ky1 = min(atomicLoad(&caixa[3]) + 2, u.h);
    if x < kx0 || x > kx1 { return; }
    var y_lo = u.h + 1;
    var y_hi = 0;
    for (var y = ky0; y <= ky1; y++) {
        let i = x + y * u.s;
        if act_w[i - u.s] + act_w[i] + act_w[i + u.s] == 1u {
            act_w[i - u.s] = 2u;
            act_w[i] = 2u;
            if act_w[i + u.s] == 0u { act_w[i + u.s] = 2u; }
            y_lo = min(y_lo, y);
            y_hi = max(y_hi, y);
        }
    }
    if y_hi > 0 {
        atomicMin(&caixa[5], x);
        atomicMax(&caixa[6], x);
        atomicMin(&caixa[7], y_lo);
        atomicMax(&caixa[8], y_hi);
        atomicStore(&caixa[9], 1);
    }
}

@compute @workgroup_size(1, 1, 1)
fn re_caixa() {
    if atomicLoad(&caixa[4]) == 0 {
        atomicStore(&caixa[10], 0);
        atomicStore(&caixa[11], -1);
        atomicStore(&caixa[12], 0);
        atomicStore(&caixa[13], -1);
        atomicStore(&caixa[14], 0);
        return;
    }
    var b = 5;
    if atomicLoad(&caixa[9]) == 0 { b = 0; }
    atomicStore(&caixa[10], max(atomicLoad(&caixa[b]) - 5, 1));
    atomicStore(&caixa[11], min(atomicLoad(&caixa[b + 1]) + 5, u.w));
    atomicStore(&caixa[12], max(atomicLoad(&caixa[b + 2]) - 5, 1));
    atomicStore(&caixa[13], min(atomicLoad(&caixa[b + 3]) + 5, u.h));
    atomicStore(&caixa[14], 1);
}

// O `vmax` da advecção (`prepare`): máx |u| por componente nas células ativas da caixa. f32 ≥ 0 ordena
// como i32 pelos bits ⇒ `atomicMax` sobre `caixa[15]` e `caixa[16 + tile]` (zerados pela CPU antes do plano).
@compute @workgroup_size(16, 16, 1)
fn vmax_red(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = cell(g);
    if !c.ok || act[c.i] == 0u { return; }
    let v = uv[c.i];
    let b = bitcast<i32>(max(abs(v.x), abs(v.y)));
    atomicMax(&caixa[15], b);
    // O leque por tile 64×64 (B13.4, `drying::por_tile::vmax_por_tile`): `caixa[16 + tile]`.
    atomicMax(&caixa[16 + (c.y >> 6u) * ((u.s + 63) / 64) + (c.x >> 6u)], b);
}
