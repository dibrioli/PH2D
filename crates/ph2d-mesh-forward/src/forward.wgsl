// ⭐⭐⭐ O DESENHISTA DE JOGO — uma passada por objecto, custo FIXO por pixel, nada acumulado entre
// quadros. Pede so' o que o WebGL2 / GLES3 da': uniformes, texturas (nada de armazenamento), atributos
// inteiros planos, profundidade com comparacao.
//
// As ranhuras sao preenchidas por `fonte.rs`: o material da casa (com a ranhura do ambiente ja'
// tomada por `env_radiance`/`env_irradiance` abaixo), o ceu de quem chama, e o olhar. ⚠️ Nunca
// escreva o NOME de uma ranhura num comentario: a substituicao nao sabe o que e' comentario.

struct Quadro {
    view_proj: mat4x4<f32>,
    sombra_vp: mat4x4<f32>,
    // xyz = olho; w = 1 perspectiva (a vista e' do olho ao ponto), 0 ortografica (a vista e' `dir`).
    olho: vec4<f32>,
    dir_vista: vec4<f32>,
    // x = altura do chao · y = ha' chao (0/1) · z = tangente da penumbra · w = aresta do texel de sombra (mundo)
    chao: vec4<f32>,
    // O quadrado do chao em xz: centro (x, z) e meia-aresta (x, z) — o do mapa de sombra.
    chao_xz: vec4<f32>,
    // x = profundidade do mapa em mundo (perto→longe) · y = vies · z = ha' sombra (0/1) · w = _
    sombra: vec4<f32>,
    // x = exposicao (stops) · y = codigo da vista · z = numero de luzes · w = _
    olhar: vec4<f32>,
    // pares (posicao, radiancia a 1), ate' `MAX_LUZES` luzes
    luzes: array<vec4<f32>, {MAX_LUZES2}>,
};

struct Objeto {
    modelo: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> quadro: Quadro;
@group(0) @binding(1) var<uniform> ceu: Ceu;
@group(0) @binding(2) var tabela_tex: texture_2d<f32>;
@group(0) @binding(3) var materiais: texture_2d<f32>;
@group(0) @binding(4) var mapa_sombra: texture_depth_2d;
@group(0) @binding(5) var compara: sampler_comparison;
@group(1) @binding(0) var<uniform> objeto: Objeto;

fn tabela_ler(i: u32) -> f32 {
    return textureLoad(tabela_tex, vec2<i32>(i32(i % {TAB_W}u), i32(i / {TAB_W}u)), 0).r;
}

{AMBIENTE}

// ⭐ O ambiente da lei do material = as DUAS partes do ceu, cada uma com o seu peso por pixel: a
// oclusao assada tapa o ceu, a sombra tapa a caixa. A lei e' LINEAR no ambiente, logo isto e'
// exactamente a soma de duas avaliacoes — por metade do preco.
var<private> peso_ceu: f32 = 1.0;
var<private> peso_caixa: f32 = 1.0;

fn env_radiance_da_cena(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {
    return ceu_radiance_sem_caixa(dir, shrink) * peso_ceu + ceu_radiance_da_caixa(dir, alpha) * peso_caixa;
}

fn env_irradiance_da_cena(n: vec3<f32>) -> vec3<f32> {
    return ceu_irradiance_sem_caixa(n) * peso_ceu + ceu_irradiance_da_caixa(n) * peso_caixa;
}

{MATERIAL}

{OLHAR}

fn material(k: u32) -> Mat {
    let r = i32(k);
    return Mat(
        textureLoad(materiais, vec2<i32>(0, r), 0),
        textureLoad(materiais, vec2<i32>(1, r), 0),
        textureLoad(materiais, vec2<i32>(2, r), 0),
        textureLoad(materiais, vec2<i32>(3, r), 0),
        textureLoad(materiais, vec2<i32>(4, r), 0),
        textureLoad(materiais, vec2<i32>(5, r), 0),
        textureLoad(materiais, vec2<i32>(6, r), 0),
        textureLoad(materiais, vec2<i32>(7, r), 0),
        textureLoad(materiais, vec2<i32>(8, r), 0),
        textureLoad(materiais, vec2<i32>(9, r), 0),
        textureLoad(materiais, vec2<i32>(10, r), 0),
        textureLoad(materiais, vec2<i32>(11, r), 0),
    );
}

fn vista(p: vec3<f32>) -> vec3<f32> {
    if (quadro.olho.w > 0.5) {
        return normalize(quadro.olho.xyz - p);
    }
    return -quadro.dir_vista.xyz;
}

// ⭐⭐ A SOMBRA DA CAIXA — penumbra de tamanho FISICO (PCSS): a caixa de luz tem raio angular, logo a
// sombra e' dura onde o objecto toca o chao e mole longe dele. Pontos fixos (nada de ruido rodado por
// pixel): a mesma imagem em todo quadro.
const DISCO: array<vec2<f32>, 16> = array<vec2<f32>, 16>(
    vec2<f32>(-0.94201624, -0.39906216), vec2<f32>(0.94558609, -0.76890725),
    vec2<f32>(-0.09418410, -0.92938870), vec2<f32>(0.34495938, 0.29387760),
    vec2<f32>(-0.91588581, 0.45771432), vec2<f32>(-0.81544232, -0.87912464),
    vec2<f32>(-0.38277543, 0.27676845), vec2<f32>(0.97484398, 0.75648379),
    vec2<f32>(0.44323325, -0.97511554), vec2<f32>(0.53742981, -0.47373420),
    vec2<f32>(-0.26496911, -0.41893023), vec2<f32>(0.79197514, 0.19090188),
    vec2<f32>(-0.24188840, 0.99706507), vec2<f32>(-0.81409955, 0.91437590),
    vec2<f32>(0.19984126, 0.78641367), vec2<f32>(0.14383161, -0.14100790),
);

fn visibilidade_da_caixa(p: vec3<f32>) -> f32 {
    if (quadro.sombra.z < 0.5) {
        return 1.0;
    }
    let c = quadro.sombra_vp * vec4<f32>(p, 1.0);
    let uv = vec2<f32>(c.x * 0.5 + 0.5, 0.5 - c.y * 0.5);
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
        return 1.0;
    }
    let dims = vec2<f32>(textureDimensions(mapa_sombra));
    let zr = c.z;
    let fundo = quadro.sombra.x;
    let texel = quadro.chao.w;
    let tan_p = quadro.chao.z;
    // 1) quem tapa: a profundidade media dos bloqueadores numa janela do tamanho da caixa vista daqui.
    let busca = clamp(tan_p * zr * fundo / texel, 1.0, 48.0);
    var soma = 0.0;
    var n = 0.0;
    for (var i = 0u; i < 16u; i = i + 1u) {
        let q = clamp(uv * dims + DISCO[i] * busca, vec2<f32>(0.0), dims - vec2<f32>(1.0));
        let d = textureLoad(mapa_sombra, vec2<i32>(q), 0);
        if (d < zr - quadro.sombra.y) {
            soma = soma + d;
            n = n + 1.0;
        }
    }
    if (n < 0.5) {
        return 1.0;
    }
    let zb = soma / n;
    // 2) a penumbra: a distancia ao bloqueador vezes a tangente da caixa, em texels.
    let raio = clamp(tan_p * (zr - zb) * fundo / texel, 1.0, 48.0);
    var vis = 0.0;
    for (var i = 0u; i < 16u; i = i + 1u) {
        let q = uv + DISCO[i] * raio / dims;
        vis = vis + textureSampleCompareLevel(mapa_sombra, compara, q, zr - quadro.sombra.y);
    }
    return vis / 16.0;
}

fn luz_das_lampadas(m: Mat, n: vec3<f32>, v: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
    var c = vec3<f32>(0.0);
    let k = u32(quadro.olhar.z);
    for (var i = 0u; i < k; i = i + 1u) {
        let d = quadro.luzes[2u * i].xyz - p;
        let dist = max(length(d), {PISO_LUZ});
        let l = d / max(length(d), 1.0e-6);
        c = c + mx_direct(m, n, v, l, quadro.luzes[2u * i + 1u].xyz / (dist * dist));
    }
    return c;
}

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) mundo: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) ao: f32,
    @location(3) @interpolate(flat, either) material: u32,
};

@vertex
fn vs_objeto(
    @location(0) p: vec3<f32>,
    @location(1) n: vec3<f32>,
    @location(2) ao: f32,
    @location(3) m: u32,
) -> VsOut {
    var o: VsOut;
    let w = objeto.modelo * vec4<f32>(p, 1.0);
    o.clip = quadro.view_proj * w;
    o.mundo = w.xyz;
    o.normal = (objeto.modelo * vec4<f32>(n, 0.0)).xyz;
    o.ao = ao;
    o.material = m;
    return o;
}

@fragment
fn fs_objeto(i: VsOut) -> @location(0) vec4<f32> {
    let m = material(i.material);
    let n = normalize(i.normal);
    let v = vista(i.mundo);
    peso_ceu = i.ao;
    peso_caixa = visibilidade_da_caixa(i.mundo);
    var c = mx_indirect(m, n, v);
    c = c + luz_das_lampadas(m, n, v, i.mundo);
    c = c + mx_emission(m, n, v);
    return vec4<f32>(vt_to_display(c, quadro.olhar.x, u32(quadro.olhar.y)), 1.0);
}

// ── A SOMBRA: so' profundidade, vista de cima ────────────────────────────────────────────────────
@vertex
fn vs_sombra(@location(0) p: vec3<f32>) -> @builtin(position) vec4<f32> {
    return quadro.sombra_vp * (objeto.modelo * vec4<f32>(p, 1.0));
}

// ── O CHAO QUE SO' RECEBE: aparece so' o quanto ele escurece ─────────────────────────────────────
struct ChaoOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) mundo: vec3<f32>,
};

@vertex
fn vs_chao(@builtin(vertex_index) k: u32) -> ChaoOut {
    // Um quadrado do tamanho do mapa de sombra (alem dele nao ha' sombra a receber).
    let canto = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
    );
    let ndc = canto[k];
    // O inverso do mapa de sombra em xz: o mapa e' ortografico e olha a direito para baixo.
    let inv = quadro.chao_xz;
    let p = vec3<f32>(inv.x + ndc.x * inv.z, quadro.chao.x, inv.y + ndc.y * inv.w);
    var o: ChaoOut;
    o.clip = quadro.view_proj * vec4<f32>(p, 1.0);
    o.mundo = p;
    return o;
}

const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

@fragment
fn fs_chao(i: ChaoOut) -> @location(0) vec4<f32> {
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let ceu_e = ceu_irradiance_sem_caixa(up);
    let caixa_e = ceu_irradiance_da_caixa(up);
    var lamp = vec3<f32>(0.0);
    let k = u32(quadro.olhar.z);
    for (var j = 0u; j < k; j = j + 1u) {
        let d = quadro.luzes[2u * j].xyz - i.mundo;
        let dist = max(length(d), {PISO_LUZ});
        let cosl = max(d.y / max(length(d), 1.0e-6), 0.0);
        lamp = lamp + quadro.luzes[2u * j + 1u].xyz * cosl / (dist * dist);
    }
    let sem = dot(ceu_e + caixa_e + lamp, LUMA);
    let com = dot(ceu_e + caixa_e * visibilidade_da_caixa(i.mundo) + lamp, LUMA);
    let escuro = clamp(1.0 - com / max(sem, 1.0e-6), 0.0, 1.0);
    return vec4<f32>(0.0, 0.0, 0.0, escuro);
}

