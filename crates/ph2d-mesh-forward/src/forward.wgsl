// ⭐⭐⭐ O DESENHISTA DE JOGO — uma passada por objecto, custo FIXO por pixel, nada acumulado entre
// quadros. Pede so' o que o WebGL2 / GLES3 da': uniformes, texturas (nada de armazenamento), atributos
// inteiros planos, profundidade com comparacao.
//
// As ranhuras sao preenchidas por `fonte.rs`: o material da casa (com a ranhura do ambiente ja'
// tomada por `env_radiance`/`env_irradiance` abaixo), o ceu de quem chama, e o olhar. ⚠️ Nunca
// escreva o NOME de uma ranhura num comentario: a substituicao nao sabe o que e' comentario.

{ESTILO}

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
    // A camada de estilo (a arrumacao do `ph2d_style::wgsl::pack`).
    estilo: Estilo,
    // x = o raio da bola da PECA (a curvatura viaja em `H * raio`) · yzw = _
    peca: vec4<f32>,
    // O CEU FOTOGRAFICO: xy = (cos, sin) do giro · z = forca · w = ligado (0/1)
    foto: vec4<f32>,
    // x = _ · y = alfa do fundo · z = ha' fundo (0/1) · w = _
    foto_fundo: vec4<f32>,
    // O SOL do ceu fotografico: xyz = direccao PARA ele no referencial do CEU · w = ha' sol (0/1)
    sol: vec4<f32>,
    // rgb = a radiancia da calote (ja' com a forca do ceu e o peso da luz-chave) · w = _
    sol_rad: vec4<f32>,
    // O inverso do `view_proj` (o fundo pergunta a direccao de cada pixel).
    inv_view_proj: mat4x4<f32>,
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
@group(0) @binding(6) var cobertura: texture_2d<f32>;
@group(0) @binding(7) var liso: sampler;
@group(0) @binding(8) var ceu_foto: texture_2d<f32>;
@group(0) @binding(9) var sol_tab: texture_2d<f32>;
@group(1) @binding(0) var<uniform> objeto: Objeto;

fn tabela_ler(i: u32) -> f32 {
    return textureLoad(tabela_tex, vec2<i32>(i32(i % {TAB_W}u), i32(i / {TAB_W}u)), 0).r;
}

{AMBIENTE}

fn sky_atlas(x: u32, y: u32) -> vec3<f32> {
    return textureLoad(ceu_foto, vec2<i32>(i32(x), i32(y)), 0).rgb;
}

fn sky_sol_ler(i: u32) -> f32 {
    return textureLoad(sol_tab, vec2<i32>(i32(i % {TAB_W}u), i32(i / {TAB_W}u)), 0).r;
}

{CEU_FOTO}

// ⭐⭐ A parte SEM caixa: o ceu fotografico (girado, com a forca) quando ligado, o de quem chama senao.
fn foto_ligada() -> bool {
    return quadro.foto.w > 0.5;
}

fn ceu_rad_sem(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {
    if (foto_ligada()) {
        return sky_radiance(sky_gira(dir, quadro.foto.xy), alpha) * quadro.foto.z;
    }
    return ceu_radiance_sem_caixa(dir, shrink);
}

fn ceu_irr_sem(n: vec3<f32>) -> vec3<f32> {
    if (foto_ligada()) {
        return sky_irradiance(sky_gira(n, quadro.foto.xy)) * quadro.foto.z;
    }
    return ceu_irradiance_sem_caixa(n);
}

// ⭐⭐ A parte DA caixa (a luz que a SOMBRA tapa): sob o ceu fotografico e' o SOL dele (tirado do
// panorama: ceu sem sol + sol = o panorama), senao a caixa de quem chama. Sem sol, nada.
fn caixa_rad(dir: vec3<f32>, alpha: f32) -> vec3<f32> {
    if (foto_ligada()) {
        if (quadro.sol.w < 0.5) {
            return vec3<f32>(0.0);
        }
        let c = sky_sol_cos(sky_gira(dir, quadro.foto.xy), quadro.sol.xyz);
        return quadro.sol_rad.rgb * sky_sol_tabela(alpha, c);
    }
    return ceu_radiance_da_caixa(dir, alpha);
}

fn caixa_irr(n: vec3<f32>) -> vec3<f32> {
    if (foto_ligada()) {
        return caixa_rad(n, 1.0);
    }
    return ceu_irradiance_da_caixa(n);
}

// ⭐ O ambiente da lei do material = as DUAS partes do ceu, cada uma com o seu peso por pixel: a
// oclusao assada tapa o ceu, a sombra tapa a caixa. A lei e' LINEAR no ambiente, logo isto e'
// exactamente a soma de duas avaliacoes — por metade do preco.
var<private> peso_ceu: f32 = 1.0;
var<private> peso_caixa: f32 = 1.0;

fn env_radiance_da_cena(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {
    return ceu_rad_sem(dir, alpha, shrink) * peso_ceu + caixa_rad(dir, alpha) * peso_caixa;
}

fn env_irradiance_da_cena(n: vec3<f32>) -> vec3<f32> {
    return ceu_irr_sem(n) * peso_ceu + caixa_irr(n) * peso_caixa;
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
    // A curvatura media `H` (com sinal) ao passo do MATERIAL e ao do ESTILO.
    @location(4) k_mat: f32,
    @location(5) k_estilo: f32,
};

@vertex
fn vs_objeto(
    @location(0) p: vec3<f32>,
    @location(1) n: vec3<f32>,
    @location(2) ao: f32,
    @location(3) m: u32,
    @location(4) k: vec2<f32>,
) -> VsOut {
    var o: VsOut;
    let w = objeto.modelo * vec4<f32>(p, 1.0);
    o.clip = quadro.view_proj * w;
    o.mundo = w.xyz;
    o.normal = (objeto.modelo * vec4<f32>(n, 0.0)).xyz;
    o.ao = ao;
    o.material = m;
    o.k_mat = k.x;
    o.k_estilo = k.y;
    return o;
}

// A subsuperficie MACICA le a curvatura do PIXEL (o modulo), escrita antes de compor — o gemeo do
// `Surface::at_curvature` da CPU e do `com_a_curvatura` do Render tracado.
fn com_a_curvatura(m_in: Mat, k: f32) -> Mat {
    var m = m_in;
    if (m.ss_color_weight.a <= 0.0 || m.ss_brdf_thin.a > 0.5) { return m; }
    m.ss_btdf_curv.a = abs(k);
    return m;
}

// A luz que o pixel devolve ao olho, em CENA-linear (antes da exposicao e do olhar) — a ordem do
// Render tracado: o indirecto, a SATURACAO dele (antes das lampadas: saturar no fim saturaria o
// realce do sol), as lampadas, e o ESTILO entre a fisica e o olhar, com a emissao.
fn luz_de_cena(i: VsOut) -> vec3<f32> {
    let m = com_a_curvatura(material(i.material), i.k_mat);
    let n = normalize(i.normal);
    let v = vista(i.mundo);
    peso_ceu = i.ao;
    peso_caixa = visibilidade_da_caixa(i.mundo);
    var c = mx_indirect(m, n, v);
    c = st_saturate_indirect(quadro.estilo, c);
    c = c + luz_das_lampadas(m, n, v, i.mundo);
    return st_apply(quadro.estilo, c + mx_emission(m, n, v), abs(dot(n, v)), i.k_estilo * quadro.peca.x);
}

@fragment
fn fs_objeto(i: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(vt_to_display(luz_de_cena(i), quadro.olhar.x, u32(quadro.olhar.y)), 1.0);
}

// ⭐ Com o BRILHO ligado o mesmo passe escreve tambem a cena-linear (o que o brilho le): o alvo 0
// continua a sair ja' no olhar, logo o anti-serrilhado das bordas e' o de sempre.
struct DoisAlvos {
    @location(0) olhar: vec4<f32>,
    @location(1) cena: vec4<f32>,
};

@fragment
fn fs_objeto_brilho(i: VsOut) -> DoisAlvos {
    let c = luz_de_cena(i);
    var o: DoisAlvos;
    o.olhar = vec4<f32>(vt_to_display(c, quadro.olhar.x, u32(quadro.olhar.y)), 1.0);
    o.cena = vec4<f32>(c, 1.0);
    return o;
}

// ── A SOMBRA: so' profundidade, vista de cima ────────────────────────────────────────────────────
@vertex
fn vs_sombra(@location(0) p: vec3<f32>) -> @builtin(position) vec4<f32> {
    return quadro.sombra_vp * (objeto.modelo * vec4<f32>(p, 1.0));
}

// ── A COBERTURA vista de cima: r = ha' objecto, g = a profundidade dele vezes a cobertura ──────────
struct CobOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) z: f32,
};

@vertex
fn vs_cobertura(@location(0) p: vec3<f32>) -> CobOut {
    var o: CobOut;
    o.clip = quadro.sombra_vp * (objeto.modelo * vec4<f32>(p, 1.0));
    o.z = o.clip.z;
    return o;
}

@fragment
fn fs_cobertura(i: CobOut) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, clamp(i.z, 0.0, 1.0), 0.0, 1.0);
}

// ⭐⭐ A SOMBRA QUE POUSA NO CHAO: o nivel da cobertura cujo borrao tem o tamanho FISICO da penumbra
// (`tangente da caixa x distancia ao bloqueador`) — duro onde a peca encosta, mole longe dela.
// Devolve (visibilidade da caixa, visibilidade do ceu). ⭐ A do CEU e' o escurecimento de contacto:
// a fraccao do ceu que um objecto a altura `h` tapa ~ a cobertura numa janela do tamanho de `h`.
fn visibilidade_do_chao(p: vec3<f32>) -> vec2<f32> {
    if (quadro.sombra.z < 0.5) {
        return vec2<f32>(1.0);
    }
    let c = quadro.sombra_vp * vec4<f32>(p, 1.0);
    let uv = vec2<f32>(c.x * 0.5 + 0.5, 0.5 - c.y * 0.5);
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
        return vec2<f32>(1.0);
    }
    let lado = f32(textureDimensions(cobertura, 0).x);
    let topo = f32(textureNumLevels(cobertura)) - 1.0;
    let texel = 2.0 * quadro.chao_xz.z / lado;
    let tan_p = quadro.chao.z;
    let fundo = quadro.sombra.x;
    // 1) quem tapa, e a que altura: a caixa vista daqui ate' ao topo da cena.
    let busca = tan_p * c.z * fundo;
    let a = textureSampleLevel(cobertura, liso, uv, clamp(log2(max(2.0 * busca / texel, 1.0)), 0.0, topo));
    if (a.r < 1.0e-3) {
        return vec2<f32>(1.0);
    }
    let zb = a.g / a.r;
    let h = max(c.z - zb, 0.0) * fundo;
    // 2) a penumbra: o diametro do borrao e' 2 x tangente x distancia ao bloqueador.
    let w = tan_p * h;
    let b = textureSampleLevel(cobertura, liso, uv, clamp(log2(max(2.0 * w / texel, 1.0)), 0.0, topo));
    return vec2<f32>(1.0 - clamp(b.r, 0.0, 1.0), ceu_do_chao(uv, c.z, texel, fundo, topo));
}

// ⭐⭐ O CEU QUE O CHAO VE — oclusao por HORIZONTE sobre o mapa de alturas visto de cima (o HBAO de
// terreno): em cada uma de 8 direccoes, o angulo do horizonte (o topo mais alto visto dali); a
// fraccao do ceu ponderada pelo cosseno que uma fatia ve e' cos^2 desse angulo.
// ⛔ Medido (02/10): a cobertura media numa janela dava 5 % de escurecimento ao lado de uma esfera
// pousada, onde a conta fisica da' ~25 %.
const PASSOS_CEU: array<f32, 5> = array<f32, 5>(0.02, 0.05, 0.1, 0.2, 0.4);

fn ceu_do_chao(uv: vec2<f32>, zc: f32, texel: f32, fundo: f32, topo: f32) -> f32 {
    var vis = 0.0;
    for (var d = 0u; d < 8u; d = d + 1u) {
        let ang = f32(d) * 0.7853982;
        let dir = vec2<f32>(cos(ang), sin(ang));
        var horizonte = 0.0;
        for (var k = 0u; k < 5u; k = k + 1u) {
            let dist = PASSOS_CEU[k];
            let q = uv + dir * (dist / (texel * f32(textureDimensions(cobertura, 0).x)));
            let lod = clamp(log2(max(dist / texel * 0.25, 1.0)), 0.0, topo);
            let a = textureSampleLevel(cobertura, liso, q, lod);
            if (a.r > 0.5) {
                let h = max(zc - a.g / a.r, 0.0) * fundo;
                horizonte = max(horizonte, atan2(h, dist));
            }
        }
        let c = cos(horizonte);
        vis = vis + c * c;
    }
    return vis / 8.0;
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
    return escuro_do_chao(i);
}

// ⚠️ O chao NAO entra no brilho (a lei da CPU: so' os pixeis da PECA) — o alvo 1 tem mascara vazia.
@fragment
fn fs_chao_brilho(i: ChaoOut) -> DoisAlvos {
    var o: DoisAlvos;
    o.olhar = escuro_do_chao(i);
    o.cena = vec4<f32>(0.0);
    return o;
}

fn escuro_do_chao(i: ChaoOut) -> vec4<f32> {
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let ceu_e = ceu_irr_sem(up);
    let caixa_e = caixa_irr(up);
    var lamp = vec3<f32>(0.0);
    let k = u32(quadro.olhar.z);
    for (var j = 0u; j < k; j = j + 1u) {
        let d = quadro.luzes[2u * j].xyz - i.mundo;
        let dist = max(length(d), {PISO_LUZ});
        let cosl = max(d.y / max(length(d), 1.0e-6), 0.0);
        lamp = lamp + quadro.luzes[2u * j + 1u].xyz * cosl / (dist * dist);
    }
    let sem = dot(ceu_e + caixa_e + lamp, LUMA);
    let vis = visibilidade_do_chao(i.mundo);
    let com = dot(ceu_e * vis.y + caixa_e * vis.x + lamp, LUMA);
    let escuro = clamp(1.0 - com / max(sem, 1.0e-6), 0.0, 1.0);
    return vec4<f32>(0.0, 0.0, 0.0, escuro);
}

// ── O CEU ATRAS DA PECA (so' com o ceu fotografico e o fundo ligado) ─────────────────────────────
struct FundoOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs_fundo(@builtin(vertex_index) k: u32) -> FundoOut {
    let p = vec2<f32>(f32((k << 1u) & 2u) * 2.0 - 1.0, f32(k & 2u) * 2.0 - 1.0);
    var o: FundoOut;
    o.clip = vec4<f32>(p, 1.0, 1.0);
    o.ndc = p;
    return o;
}

// A luz do ceu na direccao do pixel, filtrada a `alfa do fundo`, em cena-linear.
fn luz_do_fundo(ndc: vec2<f32>) -> vec3<f32> {
    let perto = quadro.inv_view_proj * vec4<f32>(ndc, 0.0, 1.0);
    let longe = quadro.inv_view_proj * vec4<f32>(ndc, 1.0, 1.0);
    let dir = longe.xyz / longe.w - perto.xyz / perto.w;
    return sky_radiance(sky_gira(dir, quadro.foto.xy), quadro.foto_fundo.y) * quadro.foto.z;
}

@fragment
fn fs_fundo(i: FundoOut) -> @location(0) vec4<f32> {
    return vec4<f32>(vt_to_display(luz_do_fundo(i.ndc), quadro.olhar.x, u32(quadro.olhar.y)), 1.0);
}

// ⚠️ O fundo NAO entra no brilho (a lei da CPU: so' os pixeis da PECA), como o chao.
@fragment
fn fs_fundo_brilho(i: FundoOut) -> DoisAlvos {
    var o: DoisAlvos;
    o.olhar = vec4<f32>(vt_to_display(luz_do_fundo(i.ndc), quadro.olhar.x, u32(quadro.olhar.y)), 1.0);
    o.cena = vec4<f32>(0.0);
    return o;
}
