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
    // x = profundidade do mapa em mundo (perto→longe) · y = vies · z = ha' sombra (0/1) · w = ha' chao (0/1)
    sombra: vec4<f32>,
    // x = exposicao (stops) · y = codigo da vista · z = numero de luzes · w = 1 dentro de uma captura
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
    // A COBERTURA vista de cima (o mapa de sombra olha ao longo da chave, que so' no estudio e' +y).
    ceu_vp: mat4x4<f32>,
    // x = meia-aresta · y = profundidade em mundo · z = tangente da caixa de cima · w = 1 o chao le a
    // chave na cobertura (estudio) / 0 no mapa de sombra (o sol)
    ceu: vec4<f32>,
    // O inverso do `view_proj` (o fundo pergunta a direccao de cada pixel).
    inv_view_proj: mat4x4<f32>,
    // pares (posicao, radiancia a 1), ate' `MAX_LUZES` luzes
    luzes: array<vec4<f32>, {MAX_LUZES2}>,
};

struct Objeto {
    modelo: mat4x4<f32>,
    // x = o indice da instancia na lista do quadro (o contacto nao a deixa tapar-se a si) ·
    // yzw = a pegada no chao (centro x, z e raio): sem captura, o reflexo so' le a sombra DELA.
    extra: vec4<f32>,
    // A CAPTURA DE REFLEXO da peca (`gpu_sondas.rs`): x = a camada (-1 = sem) · yzw = o centro.
    sonda: vec4<f32>,
};

// A tabela do contacto (`gpu_contacto.rs`): por instancia com grelha, as linhas da afim mundo -> grelha
// e (telha x, telha y, indice da instancia, _) em texels do atlas.
struct EntradaContacto {
    a0: vec4<f32>,
    a1: vec4<f32>,
    a2: vec4<f32>,
    t: vec4<f32>,
};
struct TabelaContacto {
    // x = quantas entradas · y = o lado do atlas em texels (x e y)
    n: vec4<f32>,
    e: array<EntradaContacto, {MAX_CONTACTO}>,
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
// Os niveis mais grossos do mapa de sombra (o mesmo enquadramento a 1/4 e 1/16 do lado).
@group(0) @binding(10) var mapa_sombra_1: texture_depth_2d;
@group(0) @binding(11) var mapa_sombra_2: texture_depth_2d;
// A TEXTURA TRIPLANAR: cor (sRGB) e nrh (normal rgb + rugosidade), uma camada por textura.
@group(0) @binding(12) var tri_cor_tex: texture_2d_array<f32>;
@group(0) @binding(13) var tri_nrh_tex: texture_2d_array<f32>;
@group(0) @binding(14) var tri_amostrador: sampler;
// O ceu que o chao ve (`gpu_ceu_chao.rs`).
@group(0) @binding(15) var ceu_chao: texture_2d<f32>;
@group(0) @binding(16) var<uniform> contacto_tab: TabelaContacto;
@group(0) @binding(17) var contacto_0: texture_3d<f32>;
@group(0) @binding(18) var contacto_1: texture_3d<f32>;
@group(0) @binding(19) var contacto_2: texture_3d<f32>;
// As capturas de reflexo: a camada 2s e' a cor pre-filtrada da captura s, a 2s + 1 a distancia.
@group(0) @binding(20) var sondas: texture_2d_array<f32>;
@group(1) @binding(0) var<uniform> objeto: Objeto;

fn tri_cor_ler(camada: i32, uv: vec2<f32>, lod: f32) -> vec4<f32> {
    return textureSampleLevel(tri_cor_tex, tri_amostrador, uv, camada, lod);
}

fn tri_nrh_ler(camada: i32, uv: vec2<f32>, lod: f32) -> vec4<f32> {
    return textureSampleLevel(tri_nrh_tex, tri_amostrador, uv, camada, lod);
}

{TRIPLANAR}

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
// ⭐⭐ O CHAO QUE TAPA (`chao_tapa.rs`): o ponto do pixel e se ha' chao. A metade de baixo da parte
// sem caixa perde `L_baixo x D`: o chao no lugar do ceu, escurecido onde as pecas o tapam.
var<private> ponto: vec3<f32> = vec3<f32>(0.0);
var<private> chao_tapa_ligado: bool = false;

fn ceu_de_baixo() -> vec3<f32> {
    return ceu_irr_sem(vec3<f32>(0.0, -1.0, 0.0));
}

fn env_radiance_da_cena(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {
    var ceu = ceu_rad_sem(dir, alpha, shrink);
    if (chao_tapa_ligado) {
        ceu = ceu * (1.0 - chao_reflexo_no_pixel(ponto, dir, alpha));
    }
    // ⭐⭐ Com a captura as VIZINHAS tapam o reflexo pela cobertura dela (e nao pelo contacto mole).
    if (sonda_camada >= 0) {
        let s = sonda_no_pixel(ponto, dir, alpha);
        return (s.rgb + (1.0 - s.a) * ceu) * peso_espec + caixa_rad(dir, alpha) * peso_caixa;
    }
    return ceu * peso_ceu + caixa_rad(dir, alpha) * peso_caixa;
}

fn env_irradiance_da_cena(n: vec3<f32>) -> vec3<f32> {
    var ceu = ceu_irr_sem(n);
    if (chao_tapa_ligado) {
        ceu = max(ceu - ceu_de_baixo() * chao_tapa_no_pixel(ponto, n), vec3<f32>(0.0));
    }
    return ceu * peso_ceu + caixa_irr(n) * peso_caixa;
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

// A textura do material k: as colunas depois do `pack` (`gpu_triplanar::colunas`).
struct Textura { ligada: bool, par: TriParams, m0: vec4<f32>, m1: vec4<f32>, m2: vec4<f32> };

fn textura(k: u32) -> Textura {
    let r = i32(k);
    let a = textureLoad(materiais, vec2<i32>({COL_TEX}, r), 0);
    let b = textureLoad(materiais, vec2<i32>({COL_TEX} + 1, r), 0);
    return Textura(
        a.x >= 0.0,
        TriParams(a.yz, a.w, b.x, b.w, i32(a.x), b.y > 0.5, b.z > 0.5),
        textureLoad(materiais, vec2<i32>({COL_TEX} + 2, r), 0),
        textureLoad(materiais, vec2<i32>({COL_TEX} + 3, r), 0),
        textureLoad(materiais, vec2<i32>({COL_TEX} + 4, r), 0),
    );
}

// As linhas da afim mundo -> folha aplicadas a um vector (sem a translacao) e a transposta.
fn folha_v(t: Textura, v: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(dot(t.m0.xyz, v), dot(t.m1.xyz, v), dot(t.m2.xyz, v));
}

fn mundo_v(t: Textura, v: vec3<f32>) -> vec3<f32> {
    return t.m0.xyz * v.x + t.m1.xyz * v.y + t.m2.xyz * v.z;
}

fn vista(p: vec3<f32>) -> vec3<f32> {
    if (quadro.olho.w > 0.5) {
        return normalize(quadro.olho.xyz - p);
    }
    return -quadro.dir_vista.xyz;
}

{PCSS}

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

{CONTACTO}

// ⭐⭐ O CEU QUE AS OUTRAS PECAS TAPAM a este ponto (`ph2d_contacto`): o produto das visibilidades,
// cada uma lida na grelha da peca no referencial dela. A propria instancia nao entra.
fn contacto(p: vec3<f32>, n: vec3<f32>) -> f32 {
    var vis = 1.0;
    let k = u32(contacto_tab.n.x);
    let atlas = vec3<f32>(contacto_tab.n.y, contacto_tab.n.y, f32({LADO_CONTACTO}));
    for (var j = 0u; j < k; j = j + 1u) {
        let e = contacto_tab.e[j];
        if (e.t.z == objeto.extra.x) {
            continue;
        }
        let u = vec3<f32>(dot(e.a0.xyz, p) + e.a0.w, dot(e.a1.xyz, p) + e.a1.w, dot(e.a2.xyz, p) + e.a2.w);
        let b = ct_borda(u);
        if (b.w <= 0.0) {
            continue;
        }
        let nl = normalize(vec3<f32>(dot(e.a0.xyz, n), dot(e.a1.xyz, n), dot(e.a2.xyz, n)));
        let tc = (vec3<f32>(e.t.xy, 0.0) + b.xyz * f32({LADO_CONTACTO} - 1) + vec3<f32>(0.5)) / atlas;
        let c0 = textureSampleLevel(contacto_0, liso, tc, 0.0);
        let c1 = textureSampleLevel(contacto_1, liso, tc, 0.0);
        let c2 = textureSampleLevel(contacto_2, liso, tc, 0.0);
        vis = vis * (1.0 - b.w * ct_oclusao(c0, c1, c2, nl));
    }
    return vis;
}

// A luz que o pixel devolve ao olho, em CENA-linear (antes da exposicao e do olhar) — a ordem do
// Render tracado: o indirecto, a SATURACAO dele (antes das lampadas: saturar no fim saturaria o
// realce do sol), as lampadas, e o ESTILO entre a fisica e o olhar, com a emissao.
fn luz_de_cena(i: VsOut) -> vec3<f32> {
    // As derivadas em fluxo UNIFORME, antes de qualquer ramo.
    let dx = dpdx(i.mundo);
    let dy = dpdy(i.mundo);
    var m = com_a_curvatura(material(i.material), i.k_mat);
    var n = normalize(i.normal);
    let t = textura(i.material);
    if (t.ligada) {
        let pf = folha_v(t, i.mundo) + vec3<f32>(t.m0.w, t.m1.w, t.m2.w);
        let r = tri_avalia(t.par, pf, normalize(folha_v(t, n)), folha_v(t, dx), folha_v(t, dy));
        m = mx_at_base_color(m, m.base_color_weight.rgb * r.cor);
        if (t.par.tem_rugosidade) {
            m = mx_at_roughness(m, r.rugosidade);
        }
        if (t.par.tem_normal && t.par.relevo != 0.0) {
            n = normalize(mundo_v(t, r.normal));
        }
    }
    let v = vista(i.mundo);
    peso_ceu = i.ao * contacto(i.mundo, n);
    peso_caixa = visibilidade_da_caixa(i.mundo);
    ponto = i.mundo;
    chao_tapa_ligado = quadro.chao.y > 0.5 && quadro.sombra.w > 0.5;
    // Dentro de uma captura (`olhar.w`) ninguem le capturas.
    sonda_camada = -1;
    if (objeto.sonda.x >= 0.0 && quadro.olhar.w < 0.5) {
        sonda_camada = i32(objeto.sonda.x);
    }
    peso_espec = i.ao;
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

// ── A SOMBRA: so' profundidade, vista da chave ───────────────────────────────────────────────────
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
    o.clip = quadro.ceu_vp * (objeto.modelo * vec4<f32>(p, 1.0));
    o.z = o.clip.z;
    return o;
}

@fragment
fn fs_cobertura(i: CobOut) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, clamp(i.z, 0.0, 1.0), 0.0, 1.0);
}

@fragment
fn fs_cobertura_baixo(i: CobOut) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, clamp(i.z, 0.0, 1.0), 0.0);
}

// O ceu que o chao ve (`ceu_do_chao`) e o chao que tapa as pecas (`chao_tapa`): `chao_tapa.rs`.
// O escurecimento do chao que o REFLEXO desta peca le. Com a captura, o chao todo (as vizinhas vem com
// as sombras delas); sem ela, so' o da zona da peca (`chao_tapa::pegada`).
fn sombra_propria(x: vec3<f32>, escuro: f32) -> f32 {
    if (sonda_camada >= 0) {
        return escuro;
    }
    let r = max(objeto.extra.w, 1.0e-4);
    let d = length(x.xz - objeto.extra.yz);
    return escuro * (1.0 - clamp((d - 1.5 * r) / (1.5 * r), 0.0, 1.0));
}
{CHAO_TAPA}

{SONDAS}

// ⭐⭐ A SOMBRA QUE POUSA NO CHAO: o nivel da cobertura cujo borrao tem o tamanho FISICO da penumbra
// (`tangente da caixa x distancia ao bloqueador`) — duro onde a peca encosta, mole longe dela.
// Devolve (visibilidade da caixa, visibilidade do ceu). A do CEU vem do passe do ceu do chao.
fn visibilidade_do_chao(p: vec3<f32>) -> vec2<f32> {
    if (quadro.sombra.w < 0.5) {
        return vec2<f32>(1.0);
    }
    let c = quadro.ceu_vp * vec4<f32>(p, 1.0);
    let uv = vec2<f32>(c.x * 0.5 + 0.5, 0.5 - c.y * 0.5);
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
        return vec2<f32>(1.0);
    }
    let lado = f32(textureDimensions(cobertura, 0).x);
    let topo = f32(textureNumLevels(cobertura)) - 1.0;
    let texel = 2.0 * quadro.ceu.x / lado;
    let tan_p = quadro.ceu.z;
    let fundo = quadro.ceu.y;
    let ceu = ceu_do_chao(p);
    // 1) quem tapa, e a que altura: a caixa vista daqui ate' ao topo da cena.
    let busca = tan_p * c.z * fundo;
    let a = textureSampleLevel(cobertura, liso, uv, clamp(log2(max(2.0 * busca / texel, 1.0)), 0.0, topo));
    if (a.r < 1.0e-3) {
        return vec2<f32>(1.0, ceu);
    }
    let zb = a.g / a.r;
    let h = max(c.z - zb, 0.0) * fundo;
    // 2) a penumbra: o diametro do borrao e' 2 x tangente x distancia ao bloqueador.
    let w = tan_p * h;
    let b = textureSampleLevel(cobertura, liso, uv, clamp(log2(max(2.0 * w / texel, 1.0)), 0.0, topo));
    return vec2<f32>(1.0 - clamp(b.r, 0.0, 1.0), ceu);
}

// ── O CHAO QUE SO' RECEBE: aparece so' o quanto ele escurece ─────────────────────────────────────
struct ChaoOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) mundo: vec3<f32>,
};

@vertex
fn vs_chao(@builtin(vertex_index) k: u32) -> ChaoOut {
    // O quadrado do chao: a cobertura e, sob o sol, as sombras compridas que ele deita.
    let canto = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
    );
    let ndc = canto[k];
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
    // A chave: no estudio a caixa larga (25 graus) pela cobertura; sob o sol, o mapa dele (PCSS).
    var chave = vis.x;
    if (quadro.ceu.w < 0.5) {
        chave = visibilidade_da_caixa(i.mundo);
    }
    let com = dot(ceu_e * vis.y + caixa_e * chave + lamp, LUMA);
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
