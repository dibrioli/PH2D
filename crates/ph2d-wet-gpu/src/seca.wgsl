// A secagem do Wet Paint na placa (degrau 5): `drying_pass_jacobi_rows` → `dry_cell` →
// `assenta`/`levanta` da lei Cobre (`ph2d_wet_paint::assenta`), em f32. Duas passadas: `borda`
// (a janela 3×3 de `susp > 10` sobre o susp de ANTES, Jacobi) e `seca`. Cortes ao bit: SALTA
// (levanta com coberturas iguais) e SEM (volta sem canal corrigido sai). Desenho e tolerâncias:
// docs/Painter/49_realshader/PLACA_degrau5.md.

struct S {
    retention: f32, edge_dark: f32, base_evap: f32, ext_gran: f32,
    ext_stain: f32, evap_base: f32, rewet_base: f32, bypass: u32,
    filme: f32, esconde: f32, vela: f32, oleo: f32,
    s: u32, by0: u32, by1: u32, bx0: u32,
    bx1: u32, cross: u32, evap_calmo: f32, rewet_calmo: f32,
};

@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> film: array<f32>;
@group(0) @binding(2) var<storage, read_write> susp: array<f32>;
@group(0) @binding(3) var<storage, read_write> sett: array<f32>;
@group(0) @binding(4) var<storage, read_write> srgb: array<f32>;
@group(0) @binding(5) var<storage, read_write> trgb: array<f32>;
@group(0) @binding(6) var<storage, read> paper: array<f32>;
// x = vela, y = vidrado, z = óleo da célula, w = livre
@group(0) @binding(7) var<storage, read> aux: array<vec4<f32>>;
@group(0) @binding(8) var<storage, read_write> edge: array<u32>;
@group(0) @binding(9) var<storage, read> alfa: array<f32>;
// x = lo, y = hi da faixa da linha (span_x_of)
@group(0) @binding(10) var<storage, read> faixa: array<vec2<i32>>;
// A cadência por tile 64×64 (B13.4, `drying::por_tile`): 0 salta · 1 o par calmo · 2 o par do chamador.
// O 15 (e não o 11): o `seca_tinta.wgsl` vem colado a este e usa 11..14.
@group(0) @binding(15) var<storage, read> modo: array<u32>;

fn modo_da(i: u32) -> u32 {
    return modo[((i / u.s) >> 6u) * ((u.s + 63u) / 64u) + ((i % u.s) >> 6u)];
}

const PIGMENT: f32 = 10.0;

fn alpha_of_mass(m: f32) -> f32 {
    if !(m >= 1.0) { return 0.0; }
    if m >= 3000.0 { return 1.0; }
    return alfa[u32(m)];
}

struct Ec { filme: f32, esconde: f32, vela: f32, oleo: f32 };

fn tapa(e: Ec, a_f: f32) -> f32 {
    let b = 1.0 - a_f;
    if b <= 0.0 { return 1.0; }
    return 1.0 - pow(b, e.esconde);
}

// `entre_perfis`: sqrt(1 − cos²). `cross = 1`: 1 − cos² = |a×b|²/(|a|²|b|²) (Lagrange), sem o
// cancelamento de f32 perto de cos² = 1; `cross = 0`: a conta literal da CPU.
fn entre(pa: vec3<f32>, pb: vec3<f32>) -> f32 {
    let na = dot(pa, pa);
    let nb = dot(pb, pb);
    if na < 1e-9 || nb < 1e-9 { return 0.0; }
    if u.cross == 1u {
        let c = cross(pa, pb);
        return sqrt(clamp(dot(c, c) / (na * nb), 0.0, 1.0));
    }
    let d = dot(pa, pb);
    let cos2 = min(d * d / (na * nb), 1.0);
    return sqrt(1.0 - cos2);
}

fn dif(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return entre(vec3(255.0) - a, vec3(255.0) - b);
}

fn od(c: vec3<f32>) -> vec3<f32> {
    return vec3(255.0) + log(min(max(c, vec3(1.0)) / 255.0, vec3(1.0)));
}

fn d_de(vela: f32, sc: vec3<f32>, uc: vec3<f32>) -> f32 {
    if vela > 0.0 {
        let fa = vec3(255.0) - od(sc);
        let fb = vec3(255.0) - od(uc);
        if 0.7 / 0.5 * entre(fa, fb) < 0.1 { return 0.0; }
    }
    return dif(sc, uc);
}

fn molha(v: vec3<f32>, la: f32) -> vec3<f32> {
    return clamp((v - vec3(255.0 * (1.0 - la))) / la, vec3(0.0), vec3(255.0));
}

// `prepara(...).sobre_branco()` com molhada = 1 (o `seco()` de assenta/levanta).
fn ecra(e: Ec, sc: vec3<f32>, uc: vec3<f32>, a_s: f32, a_f: f32, h: f32) -> vec3<f32> {
    var c: vec3<f32>;
    var la: f32;
    if a_f <= 0.0 {
        c = sc; la = a_s;
    } else if a_s <= 0.0 {
        let a_t = a_f;
        c = molha(vec3(255.0) - a_t * (vec3(255.0) - uc), a_t);
        la = a_t;
    } else {
        let a_t = a_f;
        let d = d_de(e.vela, sc, uc);
        let a_e = a_t + max(h - a_t, 0.0) * d * (1.0 - max(e.oleo, 0.0));
        let t_s = vec3(255.0) - a_s * (vec3(255.0) - sc);
        let c_eff = vec3(255.0) - (a_t / a_e) * (vec3(255.0) - uc);
        la = a_s + a_e * (1.0 - a_s);
        var r = t_s * (1.0 - a_e) + c_eff * a_e;
        if e.filme > 0.0 {
            let kv = a_e * e.filme * dif(t_s, c_eff);
            r = max(r - kv * c_eff * (vec3(1.0) - t_s / 255.0), vec3(0.0));
        }
        if e.vela > 0.0 {
            let f = vec3(255.0) - a_t * (vec3(255.0) - uc);
            r = r + e.vela * (t_s * f / 255.0 - r);
        }
        r = min(r, max(t_s, uc));
        c = molha(r, la);
    }
    return vec3(255.0 * (1.0 - la)) + c * la;
}

// `resolve` com o SEM; `quem = 0`: ecrã(x, uc) (assenta) · `1`: ecrã(sc, x) (levanta).
fn resolve(e: Ec, x_in: vec3<f32>, alvo: vec3<f32>, cego: f32, quem: u32,
           fixo: vec3<f32>, a_s: f32, a_f: f32, h: f32) -> vec3<f32> {
    var x = x_in;
    var y: vec3<f32>;
    if quem == 0u { y = ecra(e, x, fixo, a_s, a_f, h); } else { y = ecra(e, fixo, x, a_s, a_f, h); }
    for (var it = 0; it < 4; it++) {
        let dv = abs(y - alvo);
        if dv.x < 0.01 && dv.y < 0.01 && dv.z < 0.01 { break; }
        var mexeu = false;
        for (var c = 0; c < 3; c++) {
            var xh = x;
            if x[c] + 0.5 <= 255.0 { xh[c] = x[c] + 0.5; } else { xh[c] = x[c] - 0.5; }
            var yh: vec3<f32>;
            if quem == 0u { yh = ecra(e, xh, fixo, a_s, a_f, h); } else { yh = ecra(e, fixo, xh, a_s, a_f, h); }
            let d = (yh[c] - y[c]) / (xh[c] - x[c]);
            if abs(d) < cego { continue; }
            let passo = clamp((y[c] - alvo[c]) / d, -48.0, 48.0);
            x[c] = clamp(x[c] - passo, 0.0, 255.0);
            if quem == 0u { y = ecra(e, x, fixo, a_s, a_f, h); } else { y = ecra(e, fixo, x, a_s, a_f, h); }
            mexeu = true;
        }
        if !mexeu { break; }
    }
    return x;
}

fn assenta(e: Ec, sc: vec3<f32>, uc: vec3<f32>, a_s: f32, a_f: f32, a_s2: f32, a_f2: f32) -> vec3<f32> {
    let h_af = tapa(e, a_f);
    let alvo = ecra(e, sc, uc, a_s, a_f, h_af);
    if a_s <= 0.0 && a_f2 <= 0.0 { return uc; }
    var a_dm = 0.0;
    if a_s < 1.0 { a_dm = clamp(1.0 - (1.0 - a_s2) / (1.0 - a_s), 0.0, 1.0); }
    var x0 = uc;
    if !(a_s <= 0.0 || a_s2 <= 0.0) {
        let r = ecra(e, sc, uc, a_s, a_dm, tapa(e, a_dm));
        x0 = clamp(vec3(255.0) - (vec3(255.0) - r) / a_s2, vec3(0.0), vec3(255.0));
    }
    if e.vela > 0.0 && a_s > 0.0 && a_s2 > 0.0 {
        let t_s = vec3(255.0) - a_s * (vec3(255.0) - sc);
        let f = vec3(255.0) - a_f * (vec3(255.0) - uc);
        let f2 = vec3(255.0) - a_f2 * (vec3(255.0) - uc);
        let t_s2 = select(t_s, t_s * f / f2, f2 > vec3(0.0));
        let produto = clamp(vec3(255.0) - (vec3(255.0) - t_s2) / a_s2, vec3(0.0), vec3(255.0));
        x0 = x0 + e.vela * (produto - x0);
    }
    var cego = 0.1;
    if e.vela > 0.0 { cego = 0.01; }
    var h2 = h_af;
    if a_f2 != a_f { h2 = tapa(e, a_f2); }
    return resolve(e, x0, alvo, cego, 0u, uc, a_s2, a_f2, h2);
}

fn levanta(e: Ec, sc: vec3<f32>, uc: vec3<f32>, a_s: f32, a_f: f32, a_s2: f32, a_f2: f32) -> vec3<f32> {
    var x: vec3<f32>;
    if a_f2 <= 0.0 {
        x = uc;
    } else {
        var x0 = sc;
        if a_f > 0.0 { x0 = uc + (sc - uc) * clamp((a_f2 - a_f) / a_f2, 0.0, 1.0); }
        if a_f > 0.0 && a_f == a_f2 && a_s == a_s2 {
            x = x0; // SALTA
        } else {
            let h_af = tapa(e, a_f);
            let alvo = ecra(e, sc, uc, a_s, a_f, h_af);
            var h2 = h_af;
            if a_f2 != a_f { h2 = tapa(e, a_f2); }
            x = resolve(e, x0, alvo, 0.1, 1u, sc, a_s2, a_f2, h2);
        }
    }
    if !(a_f > 0.0) { return sc; }
    return clamp(x, min(uc, sc), max(uc, sc));
}

fn ldx(i: u32) -> vec3<f32> { return vec3(srgb[3u * i], srgb[3u * i + 1u], srgb[3u * i + 2u]); }
fn ldt(i: u32) -> vec3<f32> { return vec3(trgb[3u * i], trgb[3u * i + 1u], trgb[3u * i + 2u]); }

fn dentro(gid: vec3<u32>) -> i32 {
    let y = u.by0 + gid.y;
    if y > u.by1 { return -1; }
    let x = i32(u.bx0 + gid.x);
    let f = faixa[y];
    if x < f.x || x > f.y { return -1; }
    return x + i32(y * u.s);
}

fn pig(j: i32) -> u32 { return select(0u, 1u, susp[u32(j)] > PIGMENT); }

@compute @workgroup_size(16, 16)
fn borda(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = dentro(gid);
    if i < 0 { return; }
    let s = i32(u.s);
    var n = 0u;
    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) { n += pig(i + dx + dy * s); }
    }
    edge[u32(i)] = n;
}

@compute @workgroup_size(16, 16)
fn seca(@builtin(global_invocation_id) gid: vec3<u32>) {
    let ii = dentro(gid);
    if ii < 0 { return; }
    let i = u32(ii);
    let film0 = film[i];
    let s_mass = susp[i];
    if film0 <= 0.0 && s_mass <= 0.0 { return; }
    let md = modo_da(i);
    if md == 0u { return; }
    let evb = select(u.evap_calmo, u.evap_base, md == 2u);
    let rwb = select(u.rewet_calmo, u.rewet_base, md == 2u);
    var susp_c = s_mass;
    var sett_c = sett[i];
    var susp_rgb = ldx(i);
    var sett_rgb = ldt(i);
    let ax = aux[i];
    var e = Ec(u.filme, u.esconde, u.vela, u.oleo);
    if ax.x > 0.0 { e.vela = ax.x; }
    if ax.y > 0.0 {
        if ax.y >= 1.0 { e.esconde = 2.0; } else { e.esconde = e.esconde + (2.0 - e.esconde) * ax.y; }
    }
    if ax.z > 0.0 && e.oleo >= 0.0 { e.oleo = min(ax.z / 0.04, 1.0); }

    var ev = 1.0;
    if film0 > 0.0 && sett_c < 1000.0 {
        let cnt = edge[i];
        if cnt == 9u { ev = 1.0; } else { ev = f32(cnt) / 9.0; }
    }
    let f = film0;
    var borda = u.edge_dark;
    if ax.w >= 0.0 { borda = ax.w; }
    var new_film = f * u.retention - evb * ((1.0 - ev) * borda + u.base_evap);
    if new_film < 0.0001 { new_film = 0.0; }
    var lost = 1.0;
    if f > 0.0 { lost = 1.0 - clamp(new_film / f, 0.0, 1.0); }
    let film_c = new_film;

    if lost > 0.0 && s_mass > 0.0 {
        var dm = s_mass * lost;
        if u.bypass == 0u {
            let bias = clamp(1.0 + u.ext_gran * 0.6 * (0.5 - paper[i]) * 2.0, 0.3, 1.7);
            dm = min(dm * bias, s_mass);
        }
        let a_sett = alpha_of_mass(sett_c);
        sett_rgb = assenta(e, sett_rgb, susp_rgb, a_sett, alpha_of_mass(s_mass),
                           alpha_of_mass(sett_c + dm), alpha_of_mass(s_mass - dm));
        sett_c = sett_c + dm;
        susp_c = max(s_mass - dm, 0.0);
    }

    let st = sett_c;
    if film_c > 0.0 && st > 0.0 {
        let excess = max(film_c - alpha_of_mass(susp_c), 0.0);
        var b = rwb * (1.0 + excess * 50.0);
        if u.bypass == 0u {
            let sm = select((1.0 - u.ext_stain) * 2.0, 1.0 + (0.5 - u.ext_stain) * 14.0, u.ext_stain < 0.5);
            b = b * sm;
        }
        b = clamp(b, 0.0, 1.0);
        let lift = st * b;
        if b > 0.0 && lift > 0.0 {
            let sf = susp_c;
            susp_rgb = levanta(e, sett_rgb, susp_rgb, alpha_of_mass(st), alpha_of_mass(sf),
                               alpha_of_mass(st - lift), alpha_of_mass(sf + lift));
            susp_c = sf + lift;
            sett_c = st - lift;
        }
    }

    film[i] = film_c;
    susp[i] = susp_c;
    sett[i] = sett_c;
    srgb[3u * i] = susp_rgb.x; srgb[3u * i + 1u] = susp_rgb.y; srgb[3u * i + 2u] = susp_rgb.z;
    trgb[3u * i] = sett_rgb.x; trgb[3u * i + 1u] = sett_rgb.y; trgb[3u * i + 2u] = sett_rgb.z;
}
