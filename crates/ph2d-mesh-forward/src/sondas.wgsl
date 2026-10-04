// AS CAPTURAS DE REFLEXO, do atlas das faces ao octaedro pre-filtrado (`gpu_sondas.rs`). Passes de ecra
// cheio que leem texturas que o passe NAO escreve (o GLES do WebGL2 nao garante ler e escrever a mesma
// textura no mesmo passe, nem noutro nivel): a CADEIA tem uma textura por nivel, e o passe do nivel k
// escreve a cadeia k e o nivel k da captura, lendo os niveis < k.
//
// O octaedro de cada nivel `k` tem lado `LADO >> k` com UMA linha de borda a toda a volta (o texel de
// borda guarda a direccao da DOBRA): a leitura bilinear nao tem costura.

@group(0) @binding(0) var faces_cor: texture_2d<f32>;
@group(0) @binding(1) var faces_dist: texture_2d<f32>;
@group(0) @binding(2) var cadeia_0: texture_2d<f32>;
@group(0) @binding(3) var cadeia_1: texture_2d<f32>;
@group(0) @binding(4) var cadeia_2: texture_2d<f32>;
@group(0) @binding(5) var cadeia_3: texture_2d<f32>;
@group(0) @binding(6) var cadeia_4: texture_2d<f32>;
@group(0) @binding(7) var cadeia_5: texture_2d<f32>;
// A distancia do nivel anterior.
@group(0) @binding(8) var cadeia_dist: texture_2d<f32>;
@group(0) @binding(9) var amostra: sampler;
struct Passo {
    // x = o nivel que o passe escreve
    k: vec4<u32>,
};
@group(0) @binding(10) var<uniform> passo: Passo;

// As quatro saidas: a cadeia do nivel (cor, distancia) e o nivel da captura (cor, distancia) — a cor
// pre-multiplicada com a cobertura, a distancia ao centro da captura vezes a cobertura.
struct Quatro {
    @location(0) cadeia_cor: vec4<f32>,
    @location(1) cadeia_dist: vec4<f32>,
    @location(2) cor: vec4<f32>,
    @location(3) dist: vec4<f32>,
};

@vertex
fn vs_tela(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(p[k], 0.0, 1.0);
}

// A direccao do ponto `q` (em texels) do nivel `k`: o conteudo e' `[1, lado - 1)`, a borda dobra.
fn dir_do_texel(q: vec2<f32>, k: u32) -> vec3<f32> {
    let w = f32(SONDA_LADO >> k);
    return sky_de_oct((q - vec2<f32>(1.0)) / (w - 2.0) * 2.0 - vec2<f32>(1.0));
}

fn uv_no_nivel(d: vec3<f32>, k: u32) -> vec2<f32> {
    let w = f32(SONDA_LADO >> k);
    return (vec2<f32>(1.0) + (sky_oct(d) * 0.5 + vec2<f32>(0.5)) * (w - 2.0)) / w;
}

// O ponto do atlas das faces (3 x 2 ladrilhos) que ve a direccao `d`: a face do eixo maior.
fn uv_da_face(d: vec3<f32>) -> vec2<f32> {
    let a = abs(d);
    var f = 0u;
    if (a.y >= a.x && a.y >= a.z) {
        f = select(3u, 2u, d.y >= 0.0);
    } else if (a.z >= a.x) {
        f = select(5u, 4u, d.z >= 0.0);
    } else {
        f = select(1u, 0u, d.x >= 0.0);
    }
    let w = SONDA_FACE_W[f];
    let up = SONDA_FACE_UP[f];
    let xy = vec2<f32>(dot(d, cross(w, up)), dot(d, up)) / dot(d, w);
    let meio = 0.5 / f32(SONDA_FACE);
    let st = clamp(vec2<f32>(0.5 + 0.5 * xy.x, 0.5 - 0.5 * xy.y), vec2<f32>(meio), vec2<f32>(1.0 - meio));
    return (vec2<f32>(f32(f % 3u), f32(f / 3u)) + st) / vec2<f32>(3.0, 2.0);
}

const SUB: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(0.25, 0.25), vec2<f32>(0.75, 0.25), vec2<f32>(0.25, 0.75), vec2<f32>(0.75, 0.75),
);

// O nivel 0: quatro direccoes por texel lidas no atlas das faces — a cadeia 0 e o nivel 0 da captura
// (o espelho) sao o mesmo.
@fragment
fn fs_octa(@builtin(position) q: vec4<f32>) -> Quatro {
    var cor = vec4<f32>(0.0);
    var dist = vec4<f32>(0.0);
    for (var i = 0u; i < 4u; i = i + 1u) {
        let uv = uv_da_face(dir_do_texel(floor(q.xy) + SUB[i], 0u));
        cor = cor + textureSampleLevel(faces_cor, amostra, uv, 0.0) * 0.25;
        dist = dist + textureSampleLevel(faces_dist, amostra, uv, 0.0) * 0.25;
    }
    return Quatro(cor, dist, cor, dist);
}

// O nivel i da cadeia (uma textura por nivel).
fn le_nivel(i: u32, uv: vec2<f32>) -> vec4<f32> {
    if (i == 0u) {
        return textureSampleLevel(cadeia_0, amostra, uv, 0.0);
    }
    if (i == 1u) {
        return textureSampleLevel(cadeia_1, amostra, uv, 0.0);
    }
    if (i == 2u) {
        return textureSampleLevel(cadeia_2, amostra, uv, 0.0);
    }
    if (i == 3u) {
        return textureSampleLevel(cadeia_3, amostra, uv, 0.0);
    }
    if (i == 4u) {
        return textureSampleLevel(cadeia_4, amostra, uv, 0.0);
    }
    return textureSampleLevel(cadeia_5, amostra, uv, 0.0);
}

// A cadeia num nivel continuo ate' `topo`: os dois niveis vizinhos, cada um na coordenada dele.
fn le_cadeia(d: vec3<f32>, lod: f32, topo: u32) -> vec4<f32> {
    let l = clamp(lod, 0.0, f32(topo));
    let k0 = min(u32(l), topo);
    let a = le_nivel(k0, uv_no_nivel(d, k0));
    if (k0 >= topo) {
        return a;
    }
    let b = le_nivel(k0 + 1u, uv_no_nivel(d, k0 + 1u));
    return mix(a, b, l - f32(k0));
}

// O nivel k > 0: a cadeia k a partir da k - 1 (quatro direccoes por texel), e o nivel k da captura — a
// media sob o lobulo GGX de alfa = (k / (niveis - 1))^2 com N = V = R, pesada por N.L (a pergunta do
// pre-filtro do ceu, `ph2d_sky::prefiltro`), por amostragem de Hammersley FILTRADA: cada amostra le o
// nivel da cadeia cujo texel tem o angulo solido dela (sem o vies +1, que o ceu mediu a dobrar o erro),
// ate' ao k - 1 (o k nasce neste passe).
@fragment
fn fs_nivel(@builtin(position) q: vec4<f32>) -> Quatro {
    let k = passo.k.x;
    var cc = vec4<f32>(0.0);
    var cd = vec4<f32>(0.0);
    for (var i = 0u; i < 4u; i = i + 1u) {
        let uv = uv_no_nivel(dir_do_texel(floor(q.xy) + SUB[i], k), k - 1u);
        cc = cc + le_nivel(k - 1u, uv) * 0.25;
        cd = cd + textureSampleLevel(cadeia_dist, amostra, uv, 0.0) * 0.25;
    }
    let n = dir_do_texel(q.xy, k);
    let r = f32(k) / f32(SONDA_NIVEIS - 1u);
    let a2 = r * r * r * r;
    let s = select(-1.0, 1.0, n.z >= 0.0);
    let ia = -1.0 / (s + n.z);
    let ib = n.x * n.y * ia;
    let t = vec3<f32>(1.0 + s * n.x * n.x * ia, s * ib, -s * n.x);
    let b = vec3<f32>(ib, s + n.y * n.y * ia, -n.y);
    let w = f32(SONDA_LADO - 2u);
    let omega_p = 4.0 * 3.14159265 / (w * w);
    var soma = vec4<f32>(0.0);
    var pesos = 0.0;
    for (var i = 0u; i < SONDA_TAPS; i = i + 1u) {
        let xi = SONDA_HAMMERSLEY[i];
        let c2 = (1.0 - xi.x) / (1.0 + (a2 - 1.0) * xi.x);
        let ch = sqrt(c2);
        let sh = sqrt(max(1.0 - c2, 0.0));
        let phi = 6.28318531 * xi.y;
        let h = t * (sh * cos(phi)) + b * (sh * sin(phi)) + n * ch;
        let l = 2.0 * ch * h - n;
        let nl = dot(n, l);
        if (nl <= 0.0) {
            continue;
        }
        let den = c2 * (a2 - 1.0) + 1.0;
        let pdf = a2 / (3.14159265 * den * den) * 0.25;
        let omega_s = 1.0 / (f32(SONDA_TAPS) * pdf);
        let lod = max(0.5 * log2(omega_s / omega_p), 0.0);
        soma = soma + le_cadeia(l, lod, k - 1u) * nl;
        pesos = pesos + nl;
    }
    return Quatro(cc, cd, soma / max(pesos, 1.0e-6), cd);
}
