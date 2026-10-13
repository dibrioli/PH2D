// ⭐ W5o — a BASE da pirâmide do miolo num despacho só: os níveis `[b, l]` (os mais grossos, que cabem
// na memória de grupo) resolvem-se num grupo de 256 fios, cada varrida separada por uma barreira em vez
// de um despacho (medido na W5o: ~1,2 µs por despacho, 600 por quadro, e um nível de 5×3 com 64). A
// conta de cada texel é a do `c3_miolo.wgsl`, na mesma ordem — as funções aqui são espelho das de lá.
//
// `miolo_base`: a cascata (`miolo_inicia` no nível `l`, as relaxações, e `miolo_desce` + relaxações até
// `b`); `miolo_erro_base`: o V-ciclo a partir do defeito do nível `b` (`miolo_restringe` até `l`,
// `miolo_erro_inicia`, as relaxações do erro, e `miolo_erro_sobe` + relaxações até `b`). A saída é a
// solução (ou o erro) do nível `b` — o que o despacho seguinte lê. `CAP` (texels) é trocado no texto
// ao criar o módulo: o limite de memória de grupo do dispositivo ÷ 52 B.

struct NivelBase {
    // A região resolvida no nível, `[inicio, fim)` (a do `c3_miolo.wgsl` no mesmo nível).
    inicio: vec2<u32>,
    fim: vec2<u32>,
    // O nível inteiro e onde ele começa nos vetores do grupo.
    tamanho: vec2<u32>,
    kh2: f32,
    // Os pares de relaxações da cascata neste nível.
    pares: u32,
    deslocamento: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
}

struct Base {
    b: u32,
    l: u32,
    suaviza: u32,
    _p: u32,
    // Índice = o nível (0 não se usa).
    niveis: array<NivelBase, 17>,
}

@group(0) @binding(0) var<uniform> base: Base;
// A pirâmide do conhecido inteira (o mip `m` é o nível `m + 1`).
@group(0) @binding(1) var conhecido: texture_2d<f32>;
// A pirâmide do defeito inteira (`miolo_erro_base` lê o nível `b`).
@group(0) @binding(2) var defeito: texture_2d<f32>;
@group(0) @binding(3) var saida: texture_storage_2d<rgba32float, write>;

const AMORTECE: f32 = 0.8;
const FIOS: u32 = 256u;

var<workgroup> kr: array<f32, CAP>;
var<workgroup> kg: array<f32, CAP>;
var<workgroup> kb: array<f32, CAP>;
var<workgroup> ka: array<f32, CAP>;
// Os dois lados do Jacobi (a e b do `c3_miolo.wgsl`).
var<workgroup> ar: array<f32, CAP>;
var<workgroup> ag: array<f32, CAP>;
var<workgroup> ab: array<f32, CAP>;
var<workgroup> br: array<f32, CAP>;
var<workgroup> bg: array<f32, CAP>;
var<workgroup> bb: array<f32, CAP>;
// O defeito de cada nível.
var<workgroup> dr: array<f32, CAP>;
var<workgroup> dg: array<f32, CAP>;
var<workgroup> db: array<f32, CAP>;

const VIZ: array<vec2<i32>, 4> = array<vec2<i32>, 4>(
    vec2<i32>(1, 0), vec2<i32>(-1, 0), vec2<i32>(0, 1), vec2<i32>(0, -1)
);

fn na_regiao(l: u32, t: vec2<i32>) -> bool {
    let n = base.niveis[l];
    return all(t >= vec2<i32>(n.inicio)) && all(t < vec2<i32>(n.fim));
}

fn idx(l: u32, t: vec2<i32>) -> u32 {
    let n = base.niveis[l];
    return n.deslocamento + u32(t.y) * n.tamanho.x + u32(t.x);
}

fn texel(l: u32, i: u32) -> vec2<i32> {
    let w = base.niveis[l].tamanho.x;
    return vec2<i32>(i32(i % w), i32(i / w));
}

fn n_texels(l: u32) -> u32 {
    let t = base.niveis[l].tamanho;
    return t.x * t.y;
}

fn k_em(i: u32) -> vec4<f32> {
    return vec4<f32>(kr[i], kg[i], kb[i], ka[i]);
}

fn ler(lado: u32, i: u32) -> vec3<f32> {
    if lado == 0u {
        return vec3<f32>(ar[i], ag[i], ab[i]);
    }
    return vec3<f32>(br[i], bg[i], bb[i]);
}

fn escrever(lado: u32, i: u32, v: vec3<f32>) {
    if lado == 0u {
        ar[i] = v.r;
        ag[i] = v.g;
        ab[i] = v.b;
    } else {
        br[i] = v.r;
        bg[i] = v.g;
        bb[i] = v.b;
    }
}

fn d_em(i: u32) -> vec3<f32> {
    return vec3<f32>(dr[i], dg[i], db[i]);
}

// O conhecido dos níveis `[b, l]` para o grupo.
fn carregar_conhecido(lid: u32) {
    for (var l = base.b; l <= base.l; l++) {
        for (var i = lid; i < n_texels(l); i += FIOS) {
            let t = texel(l, i);
            let k = textureLoad(conhecido, t, i32(l) - 1);
            let j = idx(l, t);
            kr[j] = k.r;
            kg[j] = k.g;
            kb[j] = k.b;
            ka[j] = k.a;
        }
    }
    workgroupBarrier();
}

// O `relaxado` do `c3_miolo.wgsl`, sobre o lado `de` no nível `l`.
fn relaxado(l: u32, t: vec2<i32>, k: vec4<f32>, de: u32) -> vec3<f32> {
    if k.a >= 1.0 {
        return k.rgb;
    }
    var soma = vec3<f32>(0.0);
    var m = 0.0;
    for (var j = 0; j < 4; j++) {
        let v = t + VIZ[j];
        if na_regiao(l, v) && ka[idx(l, v)] >= 0.0 {
            soma += ler(de, idx(l, v));
            m += 1.0;
        }
    }
    let livre = select(k.rgb, soma / (m + base.niveis[l].kh2), m > 0.0);
    return mix(livre, k.rgb, k.a);
}

// Uma varrida do `miolo_relaxa`: lado `de` → o outro.
fn relaxa(l: u32, de: u32, lid: u32) {
    for (var i = lid; i < n_texels(l); i += FIOS) {
        let t = texel(l, i);
        if !na_regiao(l, t) {
            continue;
        }
        let j = idx(l, t);
        let k = k_em(j);
        var v = vec3<f32>(0.0);
        if k.a >= 0.0 {
            v = relaxado(l, t, k, de);
        }
        escrever(1u - de, j, v);
    }
    workgroupBarrier();
}

// O `grosso_em` do `c3_miolo.wgsl`: a bilinear do lado `a` do nível `l + 1` no texel `t` do nível `l`.
fn grosso_em(l: u32, t: vec2<i32>) -> vec4<f32> {
    let acima = l + 1u;
    let n = vec2<i32>(base.niveis[acima].tamanho);
    let u = (vec2<f32>(t) + 0.5) * 0.5 - 0.5;
    let b = vec2<i32>(floor(u));
    let fr = u - vec2<f32>(b);
    var soma = vec3<f32>(0.0);
    var peso = 0.0;
    for (var j = 0; j < 4; j++) {
        let o = vec2<i32>(j & 1, j >> 1);
        let c = clamp(b + o, vec2<i32>(0), n - 1);
        if !na_regiao(acima, c) || ka[idx(acima, c)] < 0.0 {
            continue;
        }
        let wx = select(1.0 - fr.x, fr.x, o.x == 1);
        let wy = select(1.0 - fr.y, fr.y, o.y == 1);
        soma += wx * wy * ler(0u, idx(acima, c));
        peso += wx * wy;
    }
    return vec4<f32>(soma / max(peso, 1e-20), peso);
}

// A saída: o lado `a` do nível `b`.
fn gravar_saida(lid: u32) {
    let b = base.b;
    for (var i = lid; i < n_texels(b); i += FIOS) {
        let t = texel(b, i);
        if na_regiao(b, t) {
            textureStore(saida, t, vec4<f32>(ler(0u, idx(b, t)), 1.0));
        }
    }
}

@compute @workgroup_size(256)
fn miolo_base(@builtin(local_invocation_index) lid: u32) {
    carregar_conhecido(lid);
    let topo = base.l;
    // `miolo_inicia`.
    for (var i = lid; i < n_texels(topo); i += FIOS) {
        let t = texel(topo, i);
        if na_regiao(topo, t) {
            let j = idx(topo, t);
            let k = k_em(j);
            escrever(0u, j, select(k.rgb, vec3<f32>(0.0), k.a < 0.0));
        }
    }
    workgroupBarrier();
    for (var l = topo; l >= base.b; l--) {
        if l < topo {
            // `miolo_desce`.
            for (var i = lid; i < n_texels(l); i += FIOS) {
                let t = texel(l, i);
                if !na_regiao(l, t) {
                    continue;
                }
                let j = idx(l, t);
                let k = k_em(j);
                var v = vec3<f32>(0.0);
                if k.a >= 0.0 {
                    let g = grosso_em(l, t);
                    let grosso = select(k.rgb, g.rgb, g.a > 0.0);
                    v = mix(grosso, k.rgb, k.a);
                }
                escrever(0u, j, v);
            }
            workgroupBarrier();
        }
        for (var p = 0u; p < base.niveis[l].pares; p++) {
            relaxa(l, 0u, lid);
            relaxa(l, 1u, lid);
        }
        if l == base.b {
            break;
        }
    }
    gravar_saida(lid);
}

// Uma varrida do `miolo_erro_relaxa`: lado `de` → o outro.
fn erro_relaxa(l: u32, de: u32, lid: u32) {
    for (var i = lid; i < n_texels(l); i += FIOS) {
        let t = texel(l, i);
        if !na_regiao(l, t) {
            continue;
        }
        let j = idx(l, t);
        let k = k_em(j);
        var v = vec3<f32>(0.0);
        if k.a == 0.0 {
            var soma = vec3<f32>(0.0);
            var m = 0.0;
            for (var q = 0; q < 4; q++) {
                let w = t + VIZ[q];
                if na_regiao(l, w) && ka[idx(l, w)] >= 0.0 {
                    soma += ler(de, idx(l, w));
                    m += 1.0;
                }
            }
            let d = d_em(j);
            let e = ler(de, j);
            var livre = d;
            if m > 0.0 {
                livre = (1.0 - k.a) * soma / (m + base.niveis[l].kh2) + d;
            }
            v = mix(e, livre, AMORTECE);
        }
        escrever(1u - de, j, v);
    }
    workgroupBarrier();
}

@compute @workgroup_size(256)
fn miolo_erro_base(@builtin(local_invocation_index) lid: u32) {
    carregar_conhecido(lid);
    let b = base.b;
    let topo = base.l;
    // O defeito do nível `b` (o `miolo_restringe` de fora já o escreveu).
    for (var i = lid; i < n_texels(b); i += FIOS) {
        let t = texel(b, i);
        let d = textureLoad(defeito, t, i32(b) - 1).rgb;
        let j = idx(b, t);
        dr[j] = d.r;
        dg[j] = d.g;
        db[j] = d.b;
    }
    workgroupBarrier();
    // `miolo_restringe` até ao topo.
    for (var l = b + 1u; l <= topo; l++) {
        let baixo = l - 1u;
        let n = vec2<i32>(base.niveis[baixo].tamanho);
        let sigma = (16.0 + base.niveis[l].kh2) / (4.0 + base.niveis[l].kh2);
        for (var i = lid; i < n_texels(l); i += FIOS) {
            let t = texel(l, i);
            if !na_regiao(l, t) {
                continue;
            }
            let j = idx(l, t);
            var v = vec3<f32>(0.0);
            if ka[j] == 0.0 {
                var soma = vec3<f32>(0.0);
                var presentes = 0.0;
                for (var q = 0; q < 4; q++) {
                    let c = 2 * t + vec2<i32>(q & 1, q >> 1);
                    if any(c >= n) || !na_regiao(baixo, c) || ka[idx(baixo, c)] < 0.0 {
                        continue;
                    }
                    soma += d_em(idx(baixo, c));
                    presentes += 1.0;
                }
                v = sigma * soma / max(presentes, 1.0);
            }
            dr[j] = v.r;
            dg[j] = v.g;
            db[j] = v.b;
        }
        workgroupBarrier();
    }
    // `miolo_erro_inicia`.
    for (var i = lid; i < n_texels(topo); i += FIOS) {
        let t = texel(topo, i);
        if na_regiao(topo, t) {
            let j = idx(topo, t);
            var e = vec3<f32>(0.0);
            if ka[j] == 0.0 {
                e = d_em(j);
            }
            escrever(0u, j, e);
        }
    }
    workgroupBarrier();
    for (var l = topo; l >= b; l--) {
        if l < topo {
            // `miolo_erro_sobe`.
            for (var i = lid; i < n_texels(l); i += FIOS) {
                let t = texel(l, i);
                if !na_regiao(l, t) {
                    continue;
                }
                let j = idx(l, t);
                var e = vec3<f32>(0.0);
                if ka[j] == 0.0 {
                    let g = grosso_em(l, t);
                    e = select(vec3<f32>(0.0), g.rgb, g.a > 0.0);
                }
                escrever(0u, j, e);
            }
            workgroupBarrier();
        }
        let pares = select(base.suaviza, base.niveis[l].pares, l == topo);
        for (var p = 0u; p < pares; p++) {
            erro_relaxa(l, 0u, lid);
            erro_relaxa(l, 1u, lid);
        }
        if l == b {
            break;
        }
    }
    gravar_saida(lid);
}
