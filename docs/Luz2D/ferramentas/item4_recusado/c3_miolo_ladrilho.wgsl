// ⛔ RECUSADO (item 4, 24/10; BUGS_luz2d.md bug 7, «O miolo na integrada»): RTX ×1,53–1,68, Radeon ×1,47–2,40,
// 0 texels diferentes. Guardado como registo; para o remedir: concatenar em `shaders::C3_MIOLO`, um campo
// `Miolo::ladrilho` (1 = relaxa/defeito/erro_relaxa, 2 = + desce/erro_sobe/soma) e, no `pipeline` do
// `solver_miolo.rs`, a entrada `<nome>_lad` no lugar de `<nome>`.
// ⭐ Item 4 (24/10) — o LADRILHO do miolo: as passagens por texel dos níveis grandes com as vizinhas
// lidas UMA vez por grupo na memória dele (`Miolo::ladrilho`). Na Radeon integrada o miolo é limitado
// pelas LEITURAS de textura (2 CUs; ~9 `textureLoad` por texel na relaxação, ~9 na bilinear); aqui cada
// texel do grupo 8×8 lê ~1,6 vizinhas por textura (o ladrilho 10×10, ou 6×6 do nível de cima). A conta
// e a ordem das vizinhas são as das passagens de `c3_miolo.wgsl` — `relaxado_de`, `erro_relaxado_de` e a
// bilinear do `grosso_em` —, os mesmos valores lidos: o mesmo bit.

const LAD: i32 = 10;
// O nível deste passe: a `entrada` rgb e o alfa do `conhecido` (−1 fora da região), com a borda de 1.
var<workgroup> lad_e: array<vec3<f32>, 100>;
var<workgroup> lad_k: array<f32, 100>;
// O nível de cima (a bilinear): a `entrada` rgb e se a vizinha conta (1) ou não (−1).
var<workgroup> lad_ge: array<vec3<f32>, 36>;
var<workgroup> lad_gp: array<f32, 36>;

// O ladrilho deste nível: o texel `t` do grupo `wg` está em `t − origem`.
fn lad_carrega(wg: vec2<u32>, lid: u32) -> vec2<i32> {
    let origem = vec2<i32>(wg * 8u + nv.inicio) - 1;
    for (var c = lid; c < 100u; c += 64u) {
        let g = origem + vec2<i32>(i32(c) % LAD, i32(c) / LAD);
        var e = vec3<f32>(0.0);
        var k = -1.0;
        if na_regiao(g) {
            k = textureLoad(conhecido, g, 0).a;
            e = textureLoad(entrada, g, 0).rgb;
        }
        lad_e[c] = e;
        lad_k[c] = k;
    }
    return origem;
}

fn lad_i(l: vec2<i32>) -> u32 {
    return u32(l.y * LAD + l.x);
}

// A soma das vizinhas presentes de `l` (no ladrilho), na ordem de `VIZ`.
fn lad_soma(l: vec2<i32>) -> vec4<f32> {
    var soma = vec3<f32>(0.0);
    var m = 0.0;
    for (var j = 0; j < 4; j++) {
        let i = lad_i(l + VIZ[j]);
        if lad_k[i] >= 0.0 {
            soma += lad_e[i];
            m += 1.0;
        }
    }
    return vec4<f32>(soma, m);
}

fn lad_relaxado(l: vec2<i32>, k: vec4<f32>) -> vec3<f32> {
    if k.a >= 1.0 {
        return k.rgb;
    }
    let s = lad_soma(l);
    return relaxado_de(k, s.rgb, s.a);
}

@compute @workgroup_size(8, 8)
fn miolo_relaxa_lad(
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) lid: u32,
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let origem = lad_carrega(wg.xy, lid);
    workgroupBarrier();
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    if k.a < 0.0 {
        textureStore(saida, t, vec4<f32>(0.0));
        return;
    }
    textureStore(saida, t, vec4<f32>(lad_relaxado(t - origem, k), 1.0));
}

@compute @workgroup_size(8, 8)
fn miolo_defeito_lad(
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) lid: u32,
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let origem = lad_carrega(wg.xy, lid);
    workgroupBarrier();
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    if k.a < 0.0 {
        textureStore(saida, t, vec4<f32>(0.0));
        return;
    }
    let l = t - origem;
    textureStore(saida, t, vec4<f32>(lad_relaxado(l, k) - lad_e[lad_i(l)], 1.0));
}

@compute @workgroup_size(8, 8)
fn miolo_erro_relaxa_lad(
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) lid: u32,
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let origem = lad_carrega(wg.xy, lid);
    workgroupBarrier();
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    if k.a != 0.0 {
        textureStore(saida, t, vec4<f32>(0.0));
        return;
    }
    let l = t - origem;
    let s = lad_soma(l);
    let d = textureLoad(conhecido_acima, t, 0).rgb;
    textureStore(saida, t, vec4<f32>(erro_relaxado_de(k, s.rgb, s.a, d, lad_e[lad_i(l)]), 1.0));
}

// O ladrilho do nível de CIMA que a bilinear dos 8×8 texels do grupo lê: devolve a origem dele.
fn lad_carrega_grosso(wg: vec2<u32>, lid: u32) -> vec2<i32> {
    let f = vec2<f32>(wg * 8u + nv.inicio);
    let origem = vec2<i32>(floor(f * 0.5 - 0.25));
    let n = vec2<i32>(textureDimensions(entrada));
    if lid < 36u {
        let c = clamp(origem + vec2<i32>(i32(lid) % 6, i32(lid) / 6), vec2<i32>(0), n - 1);
        var e = vec3<f32>(0.0);
        var p = -1.0;
        if na_outra(c) && textureLoad(conhecido_acima, c, 0).a >= 0.0 {
            e = textureLoad(entrada, c, 0).rgb;
            p = 1.0;
        }
        lad_ge[lid] = e;
        lad_gp[lid] = p;
    }
    return origem;
}

// O `grosso_em` pelo ladrilho de cima (a mesma conta, as mesmas vizinhas: o `clamp` está no ladrilho).
fn lad_grosso_em(t: vec2<i32>, origem: vec2<i32>) -> vec4<f32> {
    let u = (vec2<f32>(t) + 0.5) * 0.5 - 0.5;
    let b = vec2<i32>(floor(u));
    let fr = u - vec2<f32>(b);
    var soma = vec3<f32>(0.0);
    var peso = 0.0;
    for (var j = 0; j < 4; j++) {
        let o = vec2<i32>(j & 1, j >> 1);
        let l = b + o - origem;
        let i = u32(l.y * 6 + l.x);
        if lad_gp[i] < 0.0 {
            continue;
        }
        let wx = select(1.0 - fr.x, fr.x, o.x == 1);
        let wy = select(1.0 - fr.y, fr.y, o.y == 1);
        soma += wx * wy * lad_ge[i];
        peso += wx * wy;
    }
    return vec4<f32>(soma / max(peso, 1e-20), peso);
}

@compute @workgroup_size(8, 8)
fn miolo_desce_lad(
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) lid: u32,
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let origem = lad_carrega_grosso(wg.xy, lid);
    workgroupBarrier();
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    if k.a < 0.0 {
        textureStore(saida, t, vec4<f32>(0.0));
        return;
    }
    let g = lad_grosso_em(t, origem);
    let grosso = select(k.rgb, g.rgb, g.a > 0.0);
    textureStore(saida, t, vec4<f32>(mix(grosso, k.rgb, k.a), 1.0));
}

@compute @workgroup_size(8, 8)
fn miolo_erro_sobe_lad(
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) lid: u32,
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let origem = lad_carrega_grosso(wg.xy, lid);
    workgroupBarrier();
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    var e = vec3<f32>(0.0);
    if k.a == 0.0 {
        let g = lad_grosso_em(t, origem);
        e = select(vec3<f32>(0.0), g.rgb, g.a > 0.0);
    }
    textureStore(saida, t, vec4<f32>(e, 1.0));
}

@compute @workgroup_size(8, 8)
fn miolo_soma_lad(
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) lid: u32,
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let origem = lad_carrega_grosso(wg.xy, lid);
    workgroupBarrier();
    let t = alvo(gid);
    if !na_regiao(t) {
        return;
    }
    let k = textureLoad(conhecido, t, 0);
    let u = textureLoad(transmitida, t, 0).rgb;
    if k.a < 0.0 {
        textureStore(saida, t, vec4<f32>(0.0));
        return;
    }
    var e = vec3<f32>(0.0);
    if k.a < 1.0 {
        let g = lad_grosso_em(t, origem);
        e = select(vec3<f32>(0.0), g.rgb, g.a > 0.0);
    }
    textureStore(saida, t, vec4<f32>(u + e, 1.0));
}
