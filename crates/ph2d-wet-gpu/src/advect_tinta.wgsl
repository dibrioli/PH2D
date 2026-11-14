// A lei `Pigmento` no passo da placa (E11, F4_2 §6). Concatenado a `advect.wgsl`; as constantes vêm do cabeçalho que
// `tinta.rs` gera da CPU. As somas (`Soma`: v, L, a, b, t, m) são planas, 6 f32 por célula: a mesma combinação linear
// que a massa (`solver/tinta_transporte.rs`); o meio pela média do volume molhado (`solver/meio_transporte.rs`).

@group(0) @binding(18) var<storage, read_write> filme: array<f32>;
@group(0) @binding(19) var<storage, read_write> dfilme: array<f32>;
@group(0) @binding(20) var<storage, read_write> meio: array<f32>;
@group(0) @binding(21) var<storage, read_write> dmeio: array<f32>;
@group(0) @binding(22) var<storage, read> corpo: array<f32>;
@group(0) @binding(23) var<storage, read> aberto: array<f32>;
@group(0) @binding(24) var<storage, read_write> abertura: array<f32>;
@group(0) @binding(25) var<storage, read_write> fecham: array<atomic<u32>>;

struct S6 { a: vec3<f32>, b: vec3<f32> }

fn le_filme(i: i32) -> S6 {
    let j = 6 * i;
    return S6(vec3<f32>(filme[j], filme[j + 1], filme[j + 2]), vec3<f32>(filme[j + 3], filme[j + 4], filme[j + 5]));
}

fn escreve_dfilme(i: i32, a: vec3<f32>, b: vec3<f32>) {
    let j = 6 * i;
    dfilme[j] = max(a.x, 0.0);
    dfilme[j + 1] = a.y;
    dfilme[j + 2] = a.z;
    dfilme[j + 3] = b.x;
    dfilme[j + 4] = b.y;
    dfilme[j + 5] = max(b.z, 0.0);
}

fn molhado(i: i32) -> f32 {
    return st[i].y + K_W * st[i].x;
}

@compute @workgroup_size(16, 16, 1)
fn t_transporte(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = cell(g);
    if !c.ok { return; }
    let i = c.i;
    let fica = 1.0 - min(outflow[i], 1.0);
    let s0 = le_filme(i);
    var a = s0.a * fica;
    var b = s0.b * fica;
    let m0 = meio[i];
    var m = m0;
    if act[i] != 0u {
        let t = backtrace(c.x, c.y, uv[i]);
        if t.ok {
            let i00 = t.x0 + t.y0 * u.s;
            let i10 = i00 + 1;
            let i01 = i00 + u.s;
            let i11 = i01 + 1;
            let k00 = t.gx * t.gy * take_scale(outflow[i00]);
            let k10 = t.fx * t.gy * take_scale(outflow[i10]);
            let k01 = t.gx * t.fy * take_scale(outflow[i01]);
            let k11 = t.fx * t.fy * take_scale(outflow[i11]);
            let want = st[i00].y * k00 + st[i10].y * k10 + st[i01].y * k01 + st[i11].y * k11;
            if want > 0.0 {
                let s00 = le_filme(i00);
                let s10 = le_filme(i10);
                let s01 = le_filme(i01);
                let s11 = le_filme(i11);
                a += s00.a * k00 + s10.a * k10 + s01.a * k01 + s11.a * k11;
                b += s00.b * k00 + s10.b * k10 + s01.b * k01 + s11.b * k11;
            }
            if meio[i00] != m0 || meio[i10] != m0 || meio[i01] != m0 || meio[i11] != m0 {
                let q = molhado(i) * fica;
                let v00 = k00 * molhado(i00);
                let v10 = k10 * molhado(i10);
                let v01 = k01 * molhado(i01);
                let v11 = k11 * molhado(i11);
                let peso = q + v00 + v10 + v01 + v11;
                if peso > 0.0 {
                    let soma = q * m0 + v00 * meio[i00] + v10 * meio[i10] + v01 * meio[i01] + v11 * meio[i11];
                    m = clamp(soma / peso, 0.0, 1.0);
                }
            }
        }
    }
    escreve_dfilme(i, a, b);
    dmeio[i] = m;
}

@compute @workgroup_size(16, 16, 1)
fn t_commit(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = cell(g);
    if !c.ok { return; }
    let j = 6 * c.i;
    filme[j] = dfilme[j];
    filme[j + 1] = dfilme[j + 1];
    filme[j + 2] = dfilme[j + 2];
    filme[j + 3] = dfilme[j + 3];
    filme[j + 4] = dfilme[j + 4];
    filme[j + 5] = dfilme[j + 5];
    meio[c.i] = dmeio[c.i];
}

// A `sedimenta` sobre as somas: lê o `st` (o film não muda nesse passe) e escreve o `dfilme`; o `t_commit` copia.
@compute @workgroup_size(16, 16, 1)
fn t_sedimenta(@builtin(global_invocation_id) g: vec3<u32>) {
    let c = cell(g);
    if !c.ok { return; }
    let i = c.i;
    let o = sed_da(c.x, c.y);
    let f = 1.0 - o.x - o.y;
    let s0 = le_filme(i);
    var a = s0.a * f;
    var b = s0.b * f;
    if k.sed_dx != 0 && sed_dentro(c.x - k.sed_dx, c.y) {
        let w = sed_da(c.x - k.sed_dx, c.y).x;
        let s1 = le_filme(i - k.sed_dx);
        a += s1.a * w;
        b += s1.b * w;
    }
    if k.sed_dy != 0 && sed_dentro(c.x, c.y - k.sed_dy) {
        let w = sed_da(c.x, c.y - k.sed_dy).y;
        let s2 = le_filme(i - k.sed_dy * u.s);
        a += s2.a * w;
        b += s2.b * w;
    }
    escreve_dfilme(i, a, b);
}

// `drena_o_anel`: o dreno das bordas leva o filme com o `susp`.
@compute @workgroup_size(16, 16, 1)
fn t_bordas(@builtin(global_invocation_id) g: vec3<u32>) {
    let x = i32(g.x);
    let y = i32(g.y);
    if x > u.w + 1 || y > u.h + 1 { return; }
    if !(y <= 1 || y >= u.h || x <= 1 || x >= u.w) { return; }
    let j = 6 * (x + y * u.s);
    filme[j] = 0.0;
    filme[j + 1] = 0.0;
    filme[j + 2] = 0.0;
    filme[j + 3] = 0.0;
    filme[j + 4] = 0.0;
    filme[j + 5] = 0.0;
}

// O `campo` com o que o meio e o corpo acrescentam: a tensão de cedência (`ladeira_do_oleo::cede`, Bingham) e a mobilidade
// (`meio::mobilidade`). Com `meio = corpo = 0` é o `campo`.
@compute @workgroup_size(16, 16, 1)
fn campo_oleo(@builtin(global_invocation_id) g: vec3<u32>) {
    let r = campo_v(g);
    if !r.ok { return; }
    var e = r.e;
    let m = meio[r.fi];
    let movel = st[r.fi].y;
    var tau = 0.0;
    if m > 0.0 && movel > 0.0 {
        let phi = movel / (movel + K_W * max(r.f, 0.0));
        let x = max((phi - PHI_C) / (1.0 - PHI_C), 0.0);
        tau = TAU0 * min(m, 1.0) * (x * x);
    }
    if corpo[r.fi] > 0.0 && movel + sett[r.fi] > 0.0 {
        tau += TAU_CORPO * corpo[r.fi];
    }
    if tau > 0.0 {
        let v = sqrt(e.x * e.x + e.y * e.y);
        if v <= tau { e = vec2<f32>(0.0, 0.0); } else { e *= (v - tau) / v; }
    }
    if m > 0.0 { e *= pow(ETA_OLEO, -min(m, 1.0)); }
    flow_x[r.i] = clamp(e.x, -u.max_v, u.max_v);
    flow_y[r.i] = clamp(e.y, -u.max_v, u.max_v);
}

// A cura na folha inteira (`drying::cura`): a abertura decai onde há aberto com meio. O FECHO é da CPU (a óptica da camada
// fina); aqui só se conta quem o pede, e o passo desce, fecha e sobe. `fecham[1..=2]` = as linhas curadas (`min`, `max`): a
// descida suja traz a abertura delas, que podem estar fora da caixa. Lê antes de escrever: a escrita atómica disputada custa, e
// só as primeiras células de cada extremo a pagam.
@compute @workgroup_size(16, 16, 1)
fn t_cura(@builtin(global_invocation_id) g: vec3<u32>) {
    let x = i32(g.x);
    let y = i32(g.y);
    if x > u.w + 1 || y > u.h + 1 { return; }
    let i = x + y * u.s;
    let j = 6 * i;
    let v = aberto[j];
    if v <= 0.0 { return; }
    let m = clamp(aberto[j + 5] / v, 0.0, 1.0);
    if m <= 0.0 { return; }
    let o = abertura[i] * exp(-DT_CURA / (TAU_OLEO_S / m));
    abertura[i] = o;
    let linha = g.y;
    if atomicLoad(&fecham[1]) > linha { atomicMin(&fecham[1], linha); }
    if atomicLoad(&fecham[2]) < linha { atomicMax(&fecham[2], linha); }
    if o < O_FECHA && st[i].x <= 0.0 && filme[j] <= 0.0 {
        atomicAdd(&fecham[0], 1u);
    }
}
