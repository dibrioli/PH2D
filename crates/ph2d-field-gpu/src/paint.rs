//! ⭐⭐⭐ **O PASSE QUE PINTA** — do G-buffer que ficou no dispositivo aos bytes que a tela recebe.
//!
//! # ⛔⛔ Porque ele existe: o barramento, medido
//!
//! Até aqui o dispositivo marchava e **devolvia o G-buffer**: `t`, normal, sombra e oclusão por
//! pixel, `49,8 MB` a `1920×1080`. A CPU reconstruía o ponto de cada pixel, resolvia o material,
//! corria o OpenPBR, aplicava o olhar e escrevia os bytes — `36` dos `61 ms` do quadro assente.
//!
//! ⇒ as três leis do pintor já atravessaram ([`ph2d_material::wgsl`], o céu do chamador e o
//! [`ph2d_view_transform::wgsl`]) e a quarta é a do **dono** ([`ph2d_field_eval::owners::wgsl`]).
//! Com as quatro no dispositivo, o que volta é a **imagem**: `8,3 MB`.
//!
//! # ⚠️ Ele lê o grupo `0` da marcha e não o refaz
//!
//! O centro, a luz e a lista de bordas já lá estão. O pintor acrescenta um **grupo `1`** com o que
//! só ele lê — o céu, as tabelas, os materiais e a saída. *Uma segunda declaração do centro seria a
//! segunda resposta à mesma pergunta, e ela divergiria no dia em que um dos dois mudasse de
//! formato.*
//!
//! # ⚠️⚠️ As DUAS aproximações da borda são as da CPU, declaradas
//!
//! Um pixel de borda tem quatro sub-amostras marchadas, e delas guarda-se a **normal** — não o
//! ponto. ⇒ o **ponto** e o **material** do centro servem às quatro, exactamente como o
//! [`ph2d_field_render::shade_render`] faz. Numa silhueta entre duas peças de cores diferentes isto
//! pinta a borda com a cor da que o centro apanhou.

use ph2d_field_eval::owners::{Owners, wgsl::OwnersWgsl};

/// Tudo o que o pintor precisa e que **não** sai da marcha.
pub struct PaintSetup<'a> {
    /// A lei do dono desta peça. `None` numa peça de UMA folha — ver [`Owners`].
    pub owners: Option<&'a Owners>,
    /// Um material por folha, empacotado com o [`ph2d_material::wgsl::pack`] e concatenado.
    ///
    /// ⚠️ **A ordem é a das folhas do [`Owners`]**, e quem constrói um constrói o outro: uma lista
    /// com outra ordem pinta cada peça com a cor da vizinha, sem erro nenhum.
    pub materials: &'a [f32],
    /// O céu de quem chama, que tem de declarar **exactamente**:
    ///
    /// ```wgsl
    /// fn ceu_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32>
    /// fn ceu_irradiance(n: vec3<f32>) -> vec3<f32>
    /// ```
    ///
    /// ⚠️ **`ceu_*` e não `env_*`, desde o ricochete** (`docs/Render3d/08` §12): quem preenche o
    /// [`ph2d_material::wgsl::ENV_SLOT`] é o [`ambiente`] desta crate, e o céu é **um dos dois
    /// braços** dele — o outro é a luz que as superfícies devolvem.
    ///
    /// ⚠️ **Ele vem de fora de propósito:** o céu do produto vive na família `field3d`, que é
    /// composição, e esta crate não desenha estúdio nenhum.
    pub env_source: &'a str,
    /// O uniforme que o [`Self::env_source`] lê.
    pub env_consts: &'a [f32],
    /// O armazém que o [`Self::env_source`] lê.
    pub env_tables: &'a [f32],
    /// A radiância que cada lâmpada entrega a **uma** unidade de distância — o
    /// [`ph2d_field_render::PointLamp::radiance_at_one`]. As posições são as
    /// [`crate::trace::MarchSetup::lamps`], e as duas listas **têm de ter o mesmo comprimento e a
    /// mesma ordem**: quem monta uma monta a outra.
    pub lamp_radiance: [[f32; 3]; crate::trace::MAX_LAMPS],
    /// ⭐ **Quantas direcções o RICOCHETE percorre** — o mesmo `ao_rays` da marcha
    /// (`docs/Render3d/08`). `0` não compila nem despacha a passagem, e o canal fica vazio.
    ///
    /// ⚠️ **Ele vem do chamador e não do uniforme** porque quem decide COMPILAR um pipeline é o
    /// Rust, e o uniforme só é lido dentro do shader.
    pub ao_rays: u32,
    /// A exposição, em paragens.
    pub stops: f32,
    /// A vista, no código do [`ph2d_view_transform::wgsl::view_code`].
    pub view: u32,
    /// Os bytes EXACTOS que um pixel de fundo recebe — copiados, nunca reconvertidos.
    pub background: [u8; 4],
    /// A largura em MUNDO da fronteira entre dois materiais — ver o `BOUNDARY_PIXELS` do
    /// [`ph2d_field_render::shade_render`], que é quem a deriva.
    pub pixel_world: f32,
    /// ⭐⭐⭐ **A difusa BRANCA com que o CHÃO mede a luz** (`docs/Render3d/07`) — a
    /// [`ph2d_field_render::catcher_surface`], empacotada como as outras.
    ///
    /// ⚠️ Ela viaja **depois** dos materiais da peça, no índice `materiais`, e o guarda do
    /// [`PaintSetup::materials`] não a alcança de propósito: ela não é o material de folha nenhuma.
    pub catcher: &'a [f32],
    /// ⭐⭐⭐ **AS GÉMEAS FOSCAS** — os mesmos materiais de [`PaintSetup::materials`], na MESMA
    /// ordem, sem o lóbulo especular ([`ph2d_material::Surface::matte`]).
    ///
    /// ⚠️⚠️ **Elas viajam empacotadas e não se derivam aqui**, e a razão é que o pacote é
    /// **preparado**: o `specular_weight` entra no `modulated_eta_s`, no `main_alpha` e no
    /// escurecimento da base, e zerar o campo `a` do `emission_specweight` no shader **não** é a
    /// mesma superfície que a `matte()` prepara. *Uma segunda derivação da mesma lei, num shader, é
    /// como as duas divergem no dia em que o OpenPBR ganhar uma camada.*
    ///
    /// Vazia = o caminho de antes, ao bit: o ricochete lê o material de sempre.
    pub matte: &'a [f32],
}

/// ⭐⭐⭐ **QUANTOS ARMAZÉNS O PASSE QUE PINTA LIGA** — seis do grupo `0` e três do grupo `1`.
///
/// ⚠️ **O piso garantido da `wgpu` é `8`**, e é por isso que este número é público: quem não o
/// tiver cai na CPU em vez de ver a `wgpu` recusar o layout a meio de um quadro.
pub const ARMAZENS: u32 = 9;

/// Os buffers do grupo `0`, que a marcha já criou e escreveu.
pub(crate) struct Alvos<'a> {
    /// ⭐ **As LEIS da marcha, com as esculturas já substituídas** — o pintor marcha o ricochete
    /// com elas (`docs/Render3d/08` §12), e quem as compõe é quem também compõe o molde da marcha:
    /// *a origem que o texto indexa e a ordem dos vectores são a MESMA decisão.*
    pub leis: &'a str,
    /// A fita do campo — a mesma que a marcha compilou, pela mesma razão.
    pub fita: &'a ph2d_field_eval::wgsl::TapeWgsl,
    pub bgl: &'a wgpu::BindGroupLayout,
    /// As grades das esculturas — o pintor lê-as pela mesma lei que a marcha.
    pub grades: &'a wgpu::Buffer,
    pub setup: &'a wgpu::Buffer,
    pub k: &'a wgpu::Buffer,
    pub centro: &'a wgpu::Buffer,
    pub luz: &'a wgpu::Buffer,
    pub conta: &'a wgpu::Buffer,
    pub borda: &'a wgpu::Buffer,
}

/// ⛔ **A lei do dono numa peça de UMA folha** — a mesma resposta que o `Surfaces::owners: None` dá
/// na CPU, e não um caso especial: não perguntar é exactamente o custo zero.
const DONO_DE_UMA_FOLHA: &str = r"
struct Dono { a: u32, b: u32, t: f32 };
fn dono_mix(p: vec3<f32>, width: f32) -> Dono { return Dono(0u, 0u, 0.0); }
";

/// O corpo do pintor — o grupo `1`, as leis de leitura e as duas entradas.
///
/// ⚠️ `{BLUR_COS}` e `{PISO_LUZ}` são **lidos do ficheiro** que os declara, nunca escritos aqui: uma
/// constante transcrita é uma divergência à espera de um dia em que alguém mexa na outra.
const PINTOR: &str = r"
// ── o grupo 1: o que só o pintor lê ───────────────────────────────────────────────────────────
struct Pintor {
    knobs: vec4<f32>,  // stops, pixel_world, _, _
    fundo: vec4<f32>,  // o fundo em LINEAR pré-multiplicado, para a média da borda
    modo: vec4<u32>,   // view, bordas, fundo empacotado, materiais
    // ⭐ `x` = há gémeas foscas empacotadas (ver `ler_mat_fosca`). `0` é o caminho anterior ao
    // report do dono de 2026-09-17, ao bit.
    modo2: vec4<u32>,
    // ⭐ **Uma radiância por lâmpada**, na MESMA ordem das posições do `Setup`.
    lamp: array<vec4<f32>, {MAX_LAMPS}>,
};
@group(1) @binding(0) var<uniform> ceu: Ceu;
@group(1) @binding(1) var<uniform> pintor: Pintor;
@group(1) @binding(2) var<storage, read> tabela: array<f32>;
@group(1) @binding(3) var<storage, read> materiais: array<f32>;
@group(1) @binding(4) var<storage, read_write> saida: array<u32>;
// ⭐⭐⭐ **AS SONDAS** (`ph2d_field_render::probes`): `PROBE_GRID³` sondas × `SH_STRIDE` floats — os
// nove coeficientes esféricos por canal (27) e a bandeira «está dentro da peça» (o 28.º).
@group(1) @binding(5) var<storage, read_write> sondas: array<f32>;

const BLUR_COS: f32 = {BLUR_COS};
const PISO_LUZ: f32 = {PISO_LUZ};
const PROBE_GRID: u32 = {PROBE_GRID}u;
const PROBE_DIRS: u32 = {PROBE_DIRS}u;
const PROBE_MARGIN: f32 = {PROBE_MARGIN};
const SH_STRIDE: u32 = 28u;
// `4π / PROBE_DIRS`, formatado do MESMO f32 que a CPU calcula.
const SH_PESO: f32 = {SH_PESO};
const SH_A1: f32 = {SH_A1};
const SH_A2: f32 = {SH_A2};
const SQRT3: f32 = {SQRT3};
const PACKED: u32 = {PACKED}u;

// ⚠️ **A rede é `materiais[0]`**, e ela não é decorativa: o dono pode vir de uma peça que já mudou
// entre a marcha e a pintura. Ler fora do buffer devolveria lixo; pintar com o primeiro material é
// o que o artista lê como «ainda não actualizou», que é o que de facto aconteceu.
fn ler_mat(i: u32) -> Mat {
    var j = i;
    if (j >= pintor.modo.w) { j = 0u; }
    return mat_em(j * PACKED);
}

// ⭐⭐⭐ **A GÉMEA FOSCA do material `i`** — ver `ph2d_material::Surface::matte` e o
// `PaintSetup::matte`. Com `pintor.modo2.x == 0` não há gémeas empacotadas e ela devolve o material
// de sempre, que é o caminho anterior ao report do dono, ao bit.
fn ler_mat_fosca(i: u32) -> Mat {
    var j = i;
    if (j >= pintor.modo.w) { j = 0u; }
    if (pintor.modo2.x == 0u) { return mat_em(j * PACKED); }
    return mat_em((pintor.modo.w + 1u + j) * PACKED);
}

// ⭐ **O material do CHÃO** — a difusa branca que vive depois dos da peça. Ver `PaintSetup::catcher`.
fn mat_do_chao() -> Mat {
    return mat_em(pintor.modo.w * PACKED);
}

fn mat_em(o: u32) -> Mat {
    var m: Mat;
    m.base_color_weight    = vec4<f32>(materiais[o +  0u], materiais[o +  1u], materiais[o +  2u], materiais[o +  3u]);
    m.specular_color_metal = vec4<f32>(materiais[o +  4u], materiais[o +  5u], materiais[o +  6u], materiais[o +  7u]);
    m.coat_color_diffrough = vec4<f32>(materiais[o +  8u], materiais[o +  9u], materiais[o + 10u], materiais[o + 11u]);
    m.emission_specweight  = vec4<f32>(materiais[o + 12u], materiais[o + 13u], materiais[o + 14u], materiais[o + 15u]);
    m.darkening_coatweight = vec4<f32>(materiais[o + 16u], materiais[o + 17u], materiais[o + 18u], materiais[o + 19u]);
    m.attenuation_coatior  = vec4<f32>(materiais[o + 20u], materiais[o + 21u], materiais[o + 22u], materiais[o + 23u]);
    m.prepared             = vec4<f32>(materiais[o + 24u], materiais[o + 25u], materiais[o + 26u], materiais[o + 27u]);
    m.emissive             = vec4<f32>(materiais[o + 28u], materiais[o + 29u], materiais[o + 30u], materiais[o + 31u]);
    return m;
}

// A base é ortonormal, logo a transposta é a inversa — a mesma `ViewBasis::world_to_view` da CPU.
fn mundo_para_vista(w: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(dot(w, s.right), dot(w, s.up), dot(w, s.fwd));
}

// ⭐ **A OCLUSÃO SUAVIZADA** — o `ph2d_field_render::blur_occlusion`, célula a célula.
//
// ⚠️ Os vizinhos leem a oclusão CRUA (a CPU suaviza para uma cópia), e um pixel que não acerta
// devolve o valor dele sem tocar em nada — ali `n0` é o vector zero e a cerca da normal fecha
// sozinha, que é exactamente o `continue` da CPU.
fn ceu_em(x: u32, y: u32, i: u32, n0: vec3<f32>) -> f32 {
    var soma = 0.0;
    var cont = 0u;
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let xx = i32(x) + dx;
            let yy = i32(y) + dy;
            if (xx < 0 || yy < 0 || xx >= i32(s.w) || yy >= i32(s.h)) { continue; }
            let j = u32(yy) * s.w + u32(xx);
            let c = centro[j];
            if (c.x < 0.0) { continue; }
            if (dot(n0, c.yzw) < BLUR_COS) { continue; }
            soma = soma + luz[j * passo_da_luz()];
            cont = cont + 1u;
        }
    }
    if (cont > 0u) { return soma / f32(cont); }
    return luz[i * passo_da_luz()];
}

// A base é ortonormal, logo a transposta é a inversa — a volta do `mundo_para_vista`.
fn vista_para_mundo(v: vec3<f32>) -> vec3<f32> {
    return s.right * v.x + s.up * v.y + s.fwd * v.z;
}

// ⭐⭐⭐ **A RADIÂNCIA QUE SAI DO PONTO ACERTADO, por luz DIRECTA** — e é isto que faz o ricochete
// ser **UM**: a luz que sai dali não traz o que lhe chegou por sua vez.
//
// O `ph2d_field_render::bounce_slice`, passo 4, linha a linha.
fn devolvida_de(q: vec3<f32>, nq_vista: vec3<f32>, veio_de: vec3<f32>) -> vec3<f32> {
    // ⚠️ **Sem fronteira suavizada, e o doc da `Surfaces::of` diz porquê:** *«um raio de ricochete
    // não tem pixel — ele acerta um ponto —, e inventar-lhe uma largura seria inventar a
    // resposta»*. Só o dono (`.a`) é lido; a largura não muda quem ele é.
    // ⭐⭐⭐ **A GÉMEA FOSCA** — ver `ph2d_material::Surface::matte`, que traz a medição na peça do
    // dono: o lóbulo especular é uma quase-delta que `48` direcções FIXAS não amostram, e carregá-lo
    // faz o estimador **deixar de convergir** (`96` direcções leem PIOR que `48`).
    let m = ler_mat_fosca(dono_mix(q, pintor.knobs.y).a);
    // ⭐ O observador daquele ponto é **quem lhe perguntou**: o raio veio de `-veio_de`.
    let v = mundo_para_vista(-veio_de);
    let nq = vista_para_mundo(nq_vista);
    let erguido = q + nq * (s.hit_eps * 4.0);
    let piso = PISO_LUZ * PISO_LUZ;
    var sai = vec3<f32>(0.0);
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        let d = s.lamps[l].xyz - q;
        let cru = dot(d, d);
        // ⚠️ **O braço degenerado é o do sombreador**: abaixo do piso a direcção é a NORMAL, que é
        // o limite finito. A mesma lei da `shade_render::chega_da_lampada`.
        var to_light = nq_vista;
        var vis = 1.0;
        if (cru > piso) {
            let dist = sqrt(cru);
            let dir = d / dist;
            to_light = mundo_para_vista(dir);
            // ⭐ **Só quem VÊ a luz paga raio** — e isto não muda a resposta: com `N·L <= 0` o
            // `mx_direct` devolve zero seja qual for a visibilidade. É a mesma cerca do passe da
            // sombra, e o gate de paridade contra a referência de CPU é quem o prova.
            if (dot(nq, dir) > 0.0) {
                vis = visivel(erguido, dir, cerca_da_bola(erguido, dir, dist), 8.0);
            }
        }
        sai = sai + mx_direct(m, nq_vista, v, to_light, pintor.lamp[l].rgb * vis / max(cru, piso));
    }
    return sai;
}

// ⭐⭐⭐ **AS SONDAS DE IRRADIÂNCIA** — `ph2d_field_render::probes`, linha a linha (`docs/Render3d/08`
// §14). ⛔ Aqui viveu a recolha POR PIXEL (`ricochete_em`), que o report do dono de 2026-09-17
// (*«um reflexo mal feito»*) mostrou ser uma soma de projecções DURAS da peça — a fita está no
// cabeçalho daquele módulo. A cura é trocar ONDE se recolhe: em pontos fixos, com o pixel a
// interpolar.
fn sonda_raio() -> f32 { return max(s.ball_radius, 1e-3) * PROBE_MARGIN; }
fn sonda_passo() -> f32 { return 2.0 * sonda_raio() / f32(PROBE_GRID - 1u); }
fn sonda_canto() -> vec3<f32> { return s.ball_center - vec3<f32>(sonda_raio()); }
fn sonda_pos(x: u32, y: u32, z: u32) -> vec3<f32> {
    return sonda_canto() + vec3<f32>(f32(x), f32(y), f32(z)) * sonda_passo();
}
fn sonda_idx(x: u32, y: u32, z: u32) -> u32 { return (z * PROBE_GRID + y) * PROBE_GRID + x; }

// A base real de harmónicas esféricas até `l = 2` — os mesmos nove literais da CPU.
fn sh_base(d: vec3<f32>) -> array<f32, 9> {
    var y: array<f32, 9>;
    y[0] = 0.282095;
    y[1] = 0.488603 * d.y;
    y[2] = 0.488603 * d.z;
    y[3] = 0.488603 * d.x;
    y[4] = 1.092548 * d.x * d.y;
    y[5] = 1.092548 * d.y * d.z;
    y[6] = 0.315392 * (3.0 * d.z * d.z - 1.0);
    y[7] = 1.092548 * d.x * d.z;
    y[8] = 0.546274 * (d.x * d.x - d.y * d.y);
    return y;
}

// A irradiância (a média da radiância pesada pelo cosseno) que a sonda `k` entrega à normal `n`.
fn sh_irradiancia(k: u32, n: vec3<f32>) -> vec3<f32> {
    let b = k * SH_STRIDE;
    var y = sh_base(n);
    var e = vec3<f32>(0.0);
    for (var m: u32 = 0u; m < 9u; m = m + 1u) {
        var a = y[m];
        if (m >= 4u) { a = a * SH_A2; } else if (m >= 1u) { a = a * SH_A1; }
        e = e + a * vec3<f32>(sondas[b + m * 3u], sondas[b + m * 3u + 1u], sondas[b + m * 3u + 2u]);
    }
    return max(e, vec3<f32>(0.0));
}

// ⭐⭐⭐ **RECOLHER nos pixels**: as oito sondas da célula, pesadas por trilinear × «está à frente»
// — a `gather_probes` da CPU, na mesma ordem.
fn recolhe_sondas(p: vec3<f32>, n_vista: vec3<f32>) -> vec3<f32> {
    let n = vista_para_mundo(n_vista);
    let passo = sonda_passo();
    let canto = sonda_canto();
    let ng = f32(PROBE_GRID - 1u);
    let u = clamp((p - canto) / passo, vec3<f32>(0.0), vec3<f32>(ng));
    let c0 = min(vec3<u32>(floor(u)), vec3<u32>(PROBE_GRID - 2u));
    let f = clamp(u - vec3<f32>(c0), vec3<f32>(0.0), vec3<f32>(1.0));
    let lift = s.hit_eps * 4.0;
    var soma = vec3<f32>(0.0);
    var peso = 0.0;
    for (var dz: u32 = 0u; dz < 2u; dz = dz + 1u) {
        for (var dy: u32 = 0u; dy < 2u; dy = dy + 1u) {
            for (var dx: u32 = 0u; dx < 2u; dx = dx + 1u) {
                let x = c0.x + dx;
                let y = c0.y + dy;
                let z = c0.z + dz;
                let k = sonda_idx(x, y, z);
                if (sondas[k * SH_STRIDE + 27u] > 0.5) { continue; }
                let tri = select(1.0 - f.x, f.x, dx == 1u)
                        * select(1.0 - f.y, f.y, dy == 1u)
                        * select(1.0 - f.z, f.z, dz == 1u);
                let sp = sonda_pos(x, y, z);
                let para = sp - p;
                let dist = length(para);
                if (dist <= lift) { continue; }
                let dir = para / dist;
                let hf = (dot(n, dir) + 1.0) * 0.5;
                let w = tri * hf * hf;
                if (w <= 1e-6) { continue; }
                // ⛔ Aqui viveu um raio de visibilidade pixel→sonda; saiu por medição — ver a
                // recusa no doc da `gather_probes_por` da CPU (piorava o pé das paredes).
                soma = soma + w * sh_irradiancia(k, n);
                peso = peso + w;
            }
        }
    }
    if (peso > 0.0) { return soma / peso; }
    return vec3<f32>(0.0);
}

// ⭐⭐⭐ **ASSAR as sondas** — um GRUPO por sonda, uma thread por direcção, e a projecção em nove
// coeficientes reduzida na memória partilhada. Não há buffer de radiância: cada direcção marcha,
// pergunta a cor fosca de onde bateu (`devolvida_de`, a mesma do pixel) e some-se no coeficiente.
//
// ⚠️ **O controlo é UNIFORME de propósito:** todas as threads avaliam o mesmo `dentro` (o campo na
// posição da sonda), logo o `return` cedo é o mesmo para as 256 e as barreiras ficam legais.
var<workgroup> parcial: array<vec3<f32>, 256>;
@compute @workgroup_size(256, 1, 1)
fn assa_sondas(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
    let k = wg.x;
    let j = lid.x;
    let x = k % PROBE_GRID;
    let y = (k / PROBE_GRID) % PROBE_GRID;
    let z = k / (PROBE_GRID * PROBE_GRID);
    let pos = sonda_pos(x, y, z);
    let base = k * SH_STRIDE;
    // Colada à superfície conta como dentro: a marcha acertaria em todas as direcções no 1.º passo.
    let dentro = field(pos) < s.hit_eps * 4.0;
    if (dentro) {
        if (j == 0u) {
            for (var m: u32 = 0u; m < 27u; m = m + 1u) { sondas[base + m] = 0.0; }
            sondas[base + 27u] = 1.0;
        }
        return;
    }
    let d = direccao_do_cone(j, PROBE_DIRS);
    var r: Raio;
    r.o = pos;
    r.d = d;
    // Até onde um raio vai: a diagonal da caixa da grelha.
    let alcance = 2.0 * sonda_raio() * SQRT3;
    let h = marcha_ate(r, alcance);
    var l = vec3<f32>(0.0);
    if (h.x >= 0.0) { l = devolvida_de(pos + d * h.x, h.yzw, d); }
    var y9 = sh_base(d);
    for (var m: u32 = 0u; m < 9u; m = m + 1u) {
        workgroupBarrier();
        parcial[j] = (SH_PESO * y9[m]) * l;
        workgroupBarrier();
        for (var salto: u32 = 128u; salto > 0u; salto = salto >> 1u) {
            if (j < salto) { parcial[j] = parcial[j] + parcial[j + salto]; }
            workgroupBarrier();
        }
        if (j == 0u) {
            let v = parcial[0];
            sondas[base + m * 3u] = v.x;
            sondas[base + m * 3u + 1u] = v.y;
            sondas[base + m * 3u + 2u] = v.z;
        }
    }
    if (j == 0u) { sondas[base + 27u] = 0.0; }
}

// ⭐⭐⭐ **O RICOCHETE SUAVIZADO — a MESMA vizinhança do céu**, canal a canal.
//
// ⚠️⚠️ **Ele é suavizado pela mesma razão que a oclusão**, e o doc do
// `ph2d_field_render::blur_occlusion` escreve-a: o que aquele borrão apaga hoje não é ruído de
// amostragem — são as **estrias do conjunto discreto de direcções**. O ricochete corre no MESMO
// conjunto, logo tem a mesma assinatura. *E é por isso que ele tem de ser um CANAL: um valor
// calculado e consumido na mesma invocação não tem vizinhos para suavizar.*
// ⭐ **UMA passagem da recolha `3×3` guardada pela normal.** `liso = 0` lê os slots CRUS (o que a
// `borra_ricochete` faz); `liso = 1` lê os já borrados uma vez (o que o pintor faz ao ler) — e as
// duas juntas são as `ph2d_field_render::BOUNCE_BLUR_PASSES` passagens da lei.
//
// ⚠️ **É UMA função com dois chamadores de propósito:** duas cópias da mesma recolha divergiriam no
// dia em que alguém afinasse o `BLUR_COS` numa delas, e a segunda passagem deixaria de ser a mesma
// lei da primeira. *É a mesma razão que a `ph2d_field_render::occlusion::para_cada_vizinhanca` já
// escreve do lado da CPU.*
fn borra_de(x: u32, y: u32, i: u32, n0: vec3<f32>, liso: u32) -> vec3<f32> {
    var soma = vec3<f32>(0.0);
    var cont = 0u;
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let xx = i32(x) + dx;
            let yy = i32(y) + dy;
            if (xx < 0 || yy < 0 || xx >= i32(s.w) || yy >= i32(s.h)) { continue; }
            let j = u32(yy) * s.w + u32(xx);
            let c = centro[j];
            if (c.x < 0.0) { continue; }
            if (dot(n0, c.yzw) < BLUR_COS) { continue; }
            let b = base_do_ricochete(j) + liso * 3u;
            soma = soma + vec3<f32>(luz[b], luz[b + 1u], luz[b + 2u]);
            cont = cont + 1u;
        }
    }
    let b = base_do_ricochete(i) + liso * 3u;
    if (cont > 0u) { return soma / f32(cont); }
    return vec3<f32>(luz[b], luz[b + 1u], luz[b + 2u]);
}

fn borra_uma_vez(x: u32, y: u32, i: u32, n0: vec3<f32>) -> vec3<f32> {
    return borra_de(x, y, i, n0, 0u);
}

fn ricochete_no_pixel(x: u32, y: u32, i: u32, n0: vec3<f32>) -> vec3<f32> {
    return borra_de(x, y, i, n0, 1u);
}

// A luz que UM material devolve ao olho, já com o olhar — o `shade_render::radiance` da CPU.
fn luz_do_material(m: Mat, n: vec3<f32>, v: vec3<f32>, p: vec3<f32>, i: u32, ceu_vis: f32, ric: vec3<f32>) -> vec3<f32> {
    // ⭐⭐⭐ **A OCLUSÃO É A SOMBRA DO CÉU** — ela multiplica o que o AMBIENTE entrega, e mais nada.
    // Não toca nas lâmpadas (que têm sombra a sério) nem na emissão.
    var rgb = mx_indirect(m, n, v) * ceu_vis;
    // ⭐⭐⭐ **E A OUTRA METADE DO HEMISFÉRIO: a luz que as SUPERFÍCIES devolvem.**
    //
    // ⚠️ **É uma SEGUNDA chamada à mesma lei indirecta, com o ambiente trocado** — exactamente o
    // que o `shade_render` faz com o `SoIrradiancia`. Ela não leva o `ceu_vis`: a oclusão é a
    // sombra do CÉU, e a luz que vem das superfícies não é céu.
    //
    // ⛔ **Fora deste ramo o quadro é o de sempre, AO BIT** — `ricochete` nasce a zero e o
    // `ambiente_e_ricochete` a `false`.
    if (ric.x > 0.0 || ric.y > 0.0 || ric.z > 0.0) {
        ricochete = ric;
        ambiente_e_ricochete = true;
        rgb = rgb + mx_indirect(m, n, v);
        ambiente_e_ricochete = false;
    }
    let piso = PISO_LUZ * PISO_LUZ;
    let base = i * passo_da_luz();
    // ⭐⭐⭐ **AS LUZES-OBJECTO, uma a uma** — a direcção e a distância de cada saem do PONTO deste
    // pixel, e a soma é sobre a RADIÂNCIA, como a CPU faz.
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        let d = s.lamps[l].xyz - p;
        let cru = dot(d, d);
        // ⚠️ **O piso protege DUAS grandezas.** Abaixo dele a direcção é a NORMAL: com a luz sobre
        // o ponto, `d` é o vector ZERO, a direcção normalizada sai `(0,0,0)` e o pixel ficaria PRETO.
        var to_light = n;
        if (cru > piso) {
            let inv = 1.0 / sqrt(cru);
            to_light = mundo_para_vista(d * inv);
        }
        // ⭐ **A sombra entra na radiância que CHEGA** — não no `N·L` e não no resultado.
        let chega = pintor.lamp[l].rgb * luz[base + 1u + l] / max(cru, piso);
        rgb = rgb + mx_direct(m, n, v, to_light, chega);
    }
    return vt_to_display(rgb + mx_emission(m, n, v), pintor.knobs.x, pintor.modo.x);
}

// ⭐⭐ **Sombreia DUAS vezes e mistura o RESULTADO**, nunca os materiais: um metal e um dieléctrico
// a meio caminho não são um meio-metal. E só paga o dobro onde há fronteira.
fn radiancia(p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, i: u32, ceu_vis: f32, ric: vec3<f32>) -> vec3<f32> {
    let d = dono_mix(p, pintor.knobs.y);
    let ca = luz_do_material(ler_mat(d.a), n, v, p, i, ceu_vis, ric);
    if (d.t <= 0.0) { return ca; }
    let cb = luz_do_material(ler_mat(d.b), n, v, p, i, ceu_vis, ric);
    return ca + (cb - ca) * d.t;
}

// A curva do `ph2d_color::srgb::linear_to_srgb_byte`, com o mesmo arredondamento.
fn srgb_byte(linear: f32) -> u32 {
    let v = clamp(linear, 0.0, 1.0);
    var e = v * 12.92;
    if (v > 0.0031308) { e = 1.055 * pow(v, 1.0 / 2.4) - 0.055; }
    return u32(clamp(e * 255.0 + 0.5, 0.0, 255.0));
}

fn empacota(c: vec4<f32>) -> u32 {
    let a = u32(clamp(clamp(c.w, 0.0, 1.0) * 255.0 + 0.5, 0.0, 255.0));
    return srgb_byte(c.x) | (srgb_byte(c.y) << 8u) | (srgb_byte(c.z) << 16u) | (a << 24u);
}

// A direcção PARA o observador — o raio do traçado, ao contrário.
fn direccao_de_vista(d: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(-dot(d, s.right), -dot(d, s.up), -dot(d, s.fwd));
}

// ⭐⭐⭐ **A LEI DO CHÃO QUE SÓ RECEBE** — `luz que chega com a peça / luz que chegaria sem ela`, em
// luminância, sobre a difusa branca. É o `shade_render::catcher` da CPU, linha a linha.
//
// ⚠️ **As duas somas correm as mesmas contas na mesma ordem**, e onde nada tapa elas são o MESMO
// número: a razão sai exactamente `1` e o pixel fica com os bytes do fundo.
fn fator_do_chao(i: u32, q: vec3<f32>, v: vec3<f32>) -> f32 {
    let m = mat_do_chao();
    let n = mundo_para_vista(vec3<f32>(0.0, 1.0, 0.0));
    let ceu = mx_indirect(m, n, v);
    let base = i * passo_da_luz();
    var livre = ceu;
    var chega = ceu * luz[base];
    let piso = PISO_LUZ * PISO_LUZ;
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        let d = s.lamps[l].xyz - q;
        let cru = dot(d, d);
        var to_light = n;
        if (cru > piso) { to_light = mundo_para_vista(d * (1.0 / sqrt(cru))); }
        let rad = pintor.lamp[l].rgb / max(cru, piso);
        livre = livre + mx_direct(m, n, v, to_light, rad);
        chega = chega + mx_direct(m, n, v, to_light, rad * luz[base + 1u + l]);
    }
    let luma = vec3<f32>({LUMA_R}, {LUMA_G}, {LUMA_B});
    let a = luma.x * chega.x + luma.y * chega.y + luma.z * chega.z;
    let b = luma.x * livre.x + luma.y * livre.y + luma.z * livre.z;
    if (!(b > 0.0)) { return 1.0; }
    return clamp(a / b, 0.0, 1.0);
}

// O fundo com a sombra do chão por cima, em linear pré-multiplicado — o `shadowed_background` da CPU.
fn fundo_sombreado(f: f32) -> vec4<f32> {
    return vec4<f32>(pintor.fundo.rgb * f, (1.0 - f) + pintor.fundo.a * f);
}

// O factor do chão DESTE pixel — `1,0` quando ele não vê chão nenhum.
fn fator_no_pixel(j: u32, x: u32, y: u32) -> f32 {
    let r = ray_at_plane(raio(f32(x) + 0.5, f32(y) + 0.5));
    let q = chao_em(r);
    if (q.w == 0.0) { return 1.0; }
    return fator_do_chao(j, q.xyz, direccao_de_vista(r.d));
}

// ⭐⭐ **O factor de uma BORDA** — o do próprio pixel quando ele falha a peça; senão, a média dos
// vizinhos de cruz que a falham (esquerda, direita, cima, baixo — a ordem da CPU).
fn fator_da_borda(i: u32, x: u32, y: u32) -> f32 {
    if (s.chao == 0u) { return 1.0; }
    if (centro[i].x < 0.0) { return fator_no_pixel(i, x, y); }
    var soma = 0.0;
    var n = 0u;
    if (x > 0u && centro[i - 1u].x < 0.0) { soma = soma + fator_no_pixel(i - 1u, x - 1u, y); n = n + 1u; }
    if (x + 1u < s.w && centro[i + 1u].x < 0.0) { soma = soma + fator_no_pixel(i + 1u, x + 1u, y); n = n + 1u; }
    if (y > 0u && centro[i - s.w].x < 0.0) { soma = soma + fator_no_pixel(i - s.w, x, y - 1u); n = n + 1u; }
    if (y + 1u < s.h && centro[i + s.w].x < 0.0) { soma = soma + fator_no_pixel(i + s.w, x, y + 1u); n = n + 1u; }
    if (n == 0u) { return 1.0; }
    return soma / f32(n);
}

// ⭐⭐⭐ **O INTERIOR: um pixel, um material, uma escrita.**
// ⭐⭐⭐ **A PASSAGEM QUE RECOLHE O RICOCHETE NAS SONDAS E O GUARDA** (`docs/Render3d/08` §12, §14).
//
// ⚠️⚠️ **Ela vive no PINTOR e não na marcha, e é uma decisão:** as sondas precisam do MATERIAL do
// ponto acertado, e a tabela de materiais é o grupo `1` deste passe. ⇒ *quem marcha não sabe de que
// cor é o que ele acertou.*
//
// ⚠️ **E ela é um despacho SEPARADO porque a suavização precisa dos VIZINHOS** — a mesma razão que
// faz a re-amostragem da borda ser um segundo despacho e não uma linha dentro do primeiro.
@compute @workgroup_size(8, 8, 1)
fn pinta_ricochete(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let c = centro[i];
    if (c.x < 0.0) { return; }
    let r = ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5));
    let devolvida = recolhe_sondas(r.o + r.d * c.x, c.yzw);
    let b = base_do_ricochete(i);
    luz[b] = devolvida.x;
    luz[b + 1u] = devolvida.y;
    luz[b + 2u] = devolvida.z;
}

// ⭐⭐⭐ **A PRIMEIRA DAS DUAS PASSAGENS DE BORRÃO** — ver `ph2d_field_render::BOUNCE_BLUR_PASSES`.
//
// ⚠️⚠️ **Ela tem de ser um DESPACHO com destino PRÓPRIO.** A `ricochete_no_pixel` já faz uma recolha
// de `3×3` ao LER, o que dá a segunda passagem de graça; a primeira não pode ser feita no mesmo
// sítio porque escrever onde os vizinhos ainda estão a ler é uma **corrida**. ⇒ os seis slots do
// canal: o CRU e o de uma passagem.
//
// ⛔ **E ela não pode ser um núcleo MAIOR numa recolha só**, que seria de graça: medido na peça do
// dono, `5×5` e `7×7` numa passagem baixam o `p99` (`0,401 → 0,33`/`0,32`) e **sobem o MÁXIMO**
// (`1,81 → 2,84`/`3,21`), enquanto duas passagens de `3×3` baixam os dois (`0,293` / `1,50`).
// *A guarda da normal aplicada a CADA salto é transitiva — uma vizinhança GEODÉSICA, que não
// atravessa um vinco; uma recolha larga guardada pelo centro atravessa-o quando as duas pontas por
// acaso concordam.*
@compute @workgroup_size(8, 8, 1)
fn borra_ricochete(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let c = centro[i];
    let d = base_do_ricochete_liso(i);
    if (c.x < 0.0) {
        luz[d] = 0.0; luz[d + 1u] = 0.0; luz[d + 2u] = 0.0;
        return;
    }
    let v = borra_uma_vez(g.x, g.y, i, c.yzw);
    luz[d] = v.x;
    luz[d + 1u] = v.y;
    luz[d + 2u] = v.z;
}

@compute @workgroup_size(8, 8, 1)
fn pinta(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let c = centro[i];
    if (c.x < 0.0) {
        // ⭐ **O CHÃO**: onde ele é tapado o fundo escurece; onde nada o tapa a razão é exactamente
        // `1`. ⚠️ **O fundo é COPIADO** nesse caso, e não passa pela conversão — a cerca da CPU.
        let f = fator_no_pixel(i, g.x, g.y);
        if (f < 1.0) { saida[i] = empacota(fundo_sombreado(f)); } else { saida[i] = pintor.modo.z; }
        return;
    }
    let r = ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5));
    // ⭐ O PONTO reconstrói-se do `t` — a mesma álgebra do `Rays::point_at`.
    let p = r.o + r.d * c.x;
    let rgb = radiancia(
        p,
        c.yzw,
        direccao_de_vista(r.d),
        i,
        ceu_em(g.x, g.y, i, c.yzw),
        ricochete_no_pixel(g.x, g.y, i, c.yzw),
    );
    saida[i] = empacota(vec4<f32>(rgb, 1.0));
}

// ⭐⭐⭐ **A BORDA: quatro sub-amostras, média em LINEAR DE ECRÃ.**
//
// ⚠️ A média é das luzes **já transformadas** — é isso que o olho vê, e transformar a média de duas
// luzes da cena não é a mesma coisa.
@compute @workgroup_size(64, 1, 1)
fn pinta_bordas(@builtin(global_invocation_id) g: vec3<u32>) {
    let slot = g.x;
    if (slot >= pintor.modo.y) { return; }
    let i = bitcast<u32>(borda[slot * 5u].x);
    if (i >= s.w * s.h) { return; }
    let x = i % s.w;
    let y = i / s.w;
    let c = centro[i];
    let r = ray_at_plane(raio(f32(x) + 0.5, f32(y) + 0.5));
    let v = direccao_de_vista(r.d);
    let p = r.o + r.d * c.x;
    let ceu_vis = ceu_em(x, y, i, c.yzw);
    // ⚠️ **As sub-amostras partilham o ricochete do CENTRO**, exactamente como partilham o ponto e
    // o material — a mesma aproximação declarada da borda, e pela mesma razão.
    let ric = ricochete_no_pixel(x, y, i, c.yzw);
    // ⭐⭐ **O fundo de uma sub-amostra que falha é o fundo COM o chão** — sem isto a silhueta de
    // baixo pinta um fio do fundo limpo entre a peça e a sombra de contacto.
    let f_chao = fator_da_borda(i, x, y);
    var fundo = pintor.fundo;
    if (f_chao < 1.0) { fundo = fundo_sombreado(f_chao); }
    var acc = vec4<f32>(0.0);
    for (var j = 0u; j < 4u; j = j + 1u) {
        let q = borda[slot * 5u + 1u + j];
        var cor = fundo;
        if (q.x >= 0.0) { cor = vec4<f32>(radiancia(p, q.yzw, v, i, ceu_vis, ric), 1.0); }
        acc = acc + cor * 0.25;
    }
    saida[i] = empacota(acc);
}
";

/// ⭐⭐⭐ **O texto do pintor, composto** — as quatro leis mais o corpo.
///
/// ⚠️ **A ordem é a que o WGSL precisa para o `struct Ceu` existir antes do binding que o nomeia.**
/// O material traz a ranhura do ambiente já preenchida pelo céu de quem chama.
pub(crate) fn fonte(
    pintor: &PaintSetup<'_>,
    lei_do_dono: Option<&OwnersWgsl>,
    leis: &str,
) -> String {
    let dono = lei_do_dono.map_or(DONO_DE_UMA_FOLHA, |l| l.source.as_str());
    let material = ph2d_material::wgsl::SOURCE
        .replace(ph2d_material::wgsl::ENV_SLOT, &ambiente(pintor.env_source));
    let corpo = PINTOR
        .replace(
            "{BLUR_COS}",
            &formata(ph2d_field_render::OCCLUSION_BLUR_COS),
        )
        .replace(
            "{PISO_LUZ}",
            &formata(ph2d_field_render::POINT_LAMP_MIN_DISTANCE),
        )
        .replace("{PACKED}", &ph2d_material::wgsl::PACKED.to_string())
        .replace(
            "{PROBE_GRID}",
            &ph2d_field_render::probes::PROBE_GRID.to_string(),
        )
        .replace(
            "{PROBE_DIRS}",
            &ph2d_field_render::probes::PROBE_DIRS.to_string(),
        )
        .replace(
            "{PROBE_MARGIN}",
            &formata(ph2d_field_render::probes::PROBE_MARGIN),
        )
        // ⚠️ O MESMO f32 que a CPU calcula em tempo de execução — `4π/N` em `f32`.
        .replace(
            "{SH_PESO}",
            &formata(4.0 * std::f32::consts::PI / ph2d_field_render::probes::PROBE_DIRS as f32),
        )
        .replace(
            "{SH_A1}",
            &formata(ph2d_field_render::probes::SH_COSSENO[1]),
        )
        .replace(
            "{SH_A2}",
            &formata(ph2d_field_render::probes::SH_COSSENO[4]),
        )
        .replace("{SQRT3}", &formata(3.0f32.sqrt()))
        .replace("{MAX_LAMPS}", &crate::trace::MAX_LAMPS.to_string())
        .replace("{LUMA_R}", &formata(ph2d_field_render::GROUND_LUMA[0]))
        .replace("{LUMA_G}", &formata(ph2d_field_render::GROUND_LUMA[1]))
        .replace("{LUMA_B}", &formata(ph2d_field_render::GROUND_LUMA[2]));
    // ⚠️⚠️ **As LEIS da marcha entram aqui desde o ricochete** (`docs/Render3d/08` §12): ele marcha
    // a partir da superfície, logo o passe que PINTA precisa do campo, da marcha e da
    // visibilidade. ⛔ **E só as leis** — os dois kernels da marcha ficam de fora, senão este
    // módulo teria pontos de entrada que ninguém despacha.
    format!(
        "{}{leis}{material}\n{}\n{dono}\n{corpo}",
        crate::trace_wgsl::comum(),
        ph2d_view_transform::wgsl::SOURCE
    )
}

/// ⭐⭐⭐ **O AMBIENTE QUE O MATERIAL VÊ — o céu de quem chama, OU o ricochete.**
///
/// A lei indirecta do OpenPBR pergunta duas coisas ao ambiente (`env_radiance` e `env_irradiance`)
/// e é **linear** nas duas: o termo difuso entra pela irradiância, o especular pela radiância, e o
/// empilhamento é linear nas respostas. ⇒ *chamar a lei duas vezes com dois ambientes é somar os
/// dois*, que é exactamente o que o [`ph2d_field_render::shade_render`] faz com o `SoIrradiancia`.
///
/// ⚠️⚠️ **E tinham de ser DUAS chamadas e não um ambiente somado**, porque a parcela do céu leva a
/// oclusão por cima (`* ceu_vis`) e a do ricochete não: *a oclusão é a sombra do CÉU, e a luz que
/// vem das superfícies não é céu.*
///
/// ⛔ **É por isso que o [`PaintSetup::env_source`] declara `ceu_*` e não `env_*`:** quem manda no
/// ambiente é este despacho, e o céu de quem chama é um dos dois braços dele. Com o
/// `ambiente_e_ricochete` a `false` — o valor de nascença — o texto gerado entrega o céu **ao
/// bit**.
fn ambiente(ceu: &str) -> String {
    format!(
        r"{ceu}
// ⭐ O estado deste pixel, por invocação: o WGSL dá `var<private>`, que é exactamente isso.
var<private> ricochete: vec3<f32> = vec3<f32>(0.0);
var<private> ambiente_e_ricochete: bool = false;

fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {{
    // ⚠️ **O ricochete NÃO tem direcção**: ele é uma irradiância por pixel, e o lóbulo especular
    // pergunta *«que luz vem DAQUELA direcção»*. A resposta honesta é zero — a mesma que o
    // `SoIrradiancia` da CPU dá.
    if (ambiente_e_ricochete) {{ return vec3<f32>(0.0); }}
    return ceu_radiance(dir, alpha, shrink);
}}

fn env_irradiance(n: vec3<f32>) -> vec3<f32> {{
    if (ambiente_e_ricochete) {{ return ricochete; }}
    return ceu_irradiance(n);
}}
"
    )
}

/// ⚠️ Um `f32` que o WGSL leia como `f32` — sem isto um `0.9` inteiro sairia `0.9` e um `2` sairia
/// `2`, que ali é um literal **inteiro**.
fn formata(v: f32) -> String {
    let s = format!("{v:?}");
    if s.contains('.') || s.contains('e') {
        s
    } else {
        format!("{s}.0")
    }
}

/// ⭐⭐⭐ **A imagem, pintada onde os dados estão.**
// O dispositivo, a fila, o cache, o pedido, a lei do dono, os alvos da marcha, a tela e a contagem
// de bordas — oito coisas independentes, e uma struct só as renomearia.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pinta(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &mut crate::FieldPipelines,
    pintor: &PaintSetup<'_>,
    lei_do_dono: Option<&OwnersWgsl>,
    alvos: &Alvos<'_>,
    width: u32,
    height: u32,
    bordas: u64,
) -> Vec<u8> {
    let leis = alvos.leis;
    let fita = alvos.fita;
    use crate::trace::{armazem, uniforme};
    use wgpu::util::DeviceExt;

    let bgl1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("pintor"),
        entries: &[
            uniforme(0),
            uniforme(1),
            armazem(2, true),
            armazem(3, true),
            armazem(4, false),
            armazem(5, false),
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pintor"),
        bind_group_layouts: &[Some(alvos.bgl), Some(&bgl1)],
        immediate_size: 0,
    });
    // ⚠️ **O cache é o mesmo do traçado**, e a chave é o TEXTO: um arrasto de slider muda números e
    // não recompila nada, exactamente como na marcha.
    let fonte = fonte(pintor, lei_do_dono, leis);
    // ⚠️ **A fita é a MESMA da marcha, e tem de o ser:** o `k` que o grupo `0` liga já traz as
    // constantes dela, e um texto gerado de outra fita indexaria aquele armazém por outra
    // aritmética. *Era a fita VAZIA enquanto o pintor não marchava.*
    let p_pinta = cache
        .entry_with_layout(device, &fonte, fita, "pinta", Some(&layout))
        .clone();
    // ⭐ **O ricochete só compila quando ele vai de facto correr** — a mesma lei que a borda já
    // segue: *compilar é o caro*.
    let p_ricochete = (pintor.ao_rays > 0).then(|| {
        cache
            .entry_with_layout(device, &fonte, fita, "pinta_ricochete", Some(&layout))
            .clone()
    });
    // ⭐⭐⭐ As SONDAS assam-se antes do ricochete as ler (`ph2d_field_render::probes`).
    let p_assa = (pintor.ao_rays > 0).then(|| {
        cache
            .entry_with_layout(device, &fonte, fita, "assa_sondas", Some(&layout))
            .clone()
    });
    // ⭐ A PRIMEIRA das duas passagens de borrão — a segunda é a recolha que o pintor faz ao ler.
    let p_borra = (pintor.ao_rays > 0).then(|| {
        cache
            .entry_with_layout(device, &fonte, fita, "borra_ricochete", Some(&layout))
            .clone()
    });
    let p_bordas = (bordas > 0).then(|| {
        cache
            .entry_with_layout(device, &fonte, fita, "pinta_bordas", Some(&layout))
            .clone()
    });

    let bytes = |v: &[f32]| -> Vec<u8> { v.iter().flat_map(|f| f.to_le_bytes()).collect() };
    let ceu = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ceu"),
        contents: &bytes(pintor.env_consts),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let tabela = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("tabela"),
        contents: &bytes(pintor.env_tables),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let mut mats = if pintor.materials.is_empty() {
        vec![0.0f32; ph2d_material::wgsl::PACKED]
    } else {
        pintor.materials.to_vec()
    };
    #[allow(clippy::cast_possible_truncation)]
    let n_mats = (mats.len() / ph2d_material::wgsl::PACKED) as u32;
    // ⭐ **O material do chão vai no fim**, e o `n_mats` continua a contar só os da peça — é ele o
    // guarda do `ler_mat`, e o chão não é uma folha.
    mats.extend_from_slice(pintor.catcher);
    // ⭐ **As gémeas foscas vão a seguir ao chão**, e o `n_mats` continua a contar só as da peça —
    // ele é o guarda do `ler_mat`. A gémea de `k` mora em `(n_mats + 1 + k) * PACKED`.
    let tem_foscas = u32::from(!pintor.matte.is_empty());
    mats.extend_from_slice(pintor.matte);
    let materiais = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("materiais"),
        contents: &bytes(&mats),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let bg = pintor.background;
    let a = f32::from(bg[3]) / 255.0;
    let mut u: Vec<u8> = Vec::with_capacity(64 + crate::trace::MAX_LAMPS * 16);
    for f in [
        pintor.stops,
        pintor.pixel_world,
        0.0,
        0.0,
        // ⚠️ **O fundo da BORDA é LINEAR e PRÉ-MULTIPLICADO** — a média das quatro amostras corre
        // em linear de ecrã, e o alfa entra nela como as outras três componentes.
        ph2d_color::srgb::srgb_to_linear_byte(bg[0]) * a,
        ph2d_color::srgb::srgb_to_linear_byte(bg[1]) * a,
        ph2d_color::srgb::srgb_to_linear_byte(bg[2]) * a,
        a,
    ] {
        u.extend_from_slice(&f.to_le_bytes());
    }
    #[allow(clippy::cast_possible_truncation)]
    let n_bordas = bordas as u32;
    let empacotado = u32::from(bg[0])
        | (u32::from(bg[1]) << 8)
        | (u32::from(bg[2]) << 16)
        | (u32::from(bg[3]) << 24);
    for v in [
        pintor.view,
        n_bordas,
        empacotado,
        n_mats,
        tem_foscas,
        0,
        0,
        0,
    ] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    // ⚠️ **O array vai INTEIRO** — a mesma razão do `MarchSetup::lamps`: um `array<vec4, 8>` de
    // uniforme tem tamanho fixo.
    for r in pintor.lamp_radiance {
        for f in r {
            u.extend_from_slice(&f.to_le_bytes());
        }
        u.extend_from_slice(&0f32.to_le_bytes());
    }
    let ub_pintor = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("pintor"),
        contents: &u,
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let n = u64::from(width) * u64::from(height);
    let b_saida = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("imagem"),
        size: (n * 4).max(16),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    fn recurso(b: &wgpu::Buffer, i: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: i,
            resource: b.as_entire_binding(),
        }
    }
    let bg0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: alvos.bgl,
        entries: &[
            recurso(alvos.setup, 0),
            recurso(alvos.k, 1),
            recurso(alvos.centro, 2),
            recurso(alvos.luz, 3),
            recurso(alvos.conta, 4),
            recurso(alvos.borda, 5),
            recurso(alvos.grades, 6),
        ],
    });
    // ⭐ As sondas: `PROBE_GRID³ × 28` floats. ⚠️ Ele existe mesmo sem ricochete (a bandeira `0`
    // é «fora», e o pintor só o lê quando o despacho das sondas correu).
    let n_sondas = ph2d_field_render::probes::PROBE_GRID.pow(3);
    let b_sondas = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sondas"),
        size: (n_sondas * 28 * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let bg1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl1,
        entries: &[
            recurso(&ceu, 0),
            recurso(&ub_pintor, 1),
            recurso(&tabela, 2),
            recurso(&materiais, 3),
            recurso(&b_saida, 4),
            recurso(&b_sondas, 5),
        ],
    });

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    // ⚠️⚠️ **O ricochete ANTES da pintura, e a ordem é a lei**: a pintura lê a vizinhança `3×3` do
    // canal para o suavizar, logo ela precisa dele escrito em TODO o quadro — não só neste pixel.
    // *Escrito na mesma passagem, cada pixel leria oito vizinhos de um quadro que ainda não existe.*
    // ⭐⭐⭐ **As SONDAS primeiro**: um grupo de 256 threads por sonda — `PROBE_GRID³` grupos, que
    // cabem no limite de `65 535` por dimensão a `32³`. Elas não dependem da câmera; assá-las por
    // quadro assente custa `~1 ms` e uma cache por cena é a wave seguinte.
    if let Some(p) = &p_assa {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        #[allow(clippy::cast_possible_truncation)]
        cp.dispatch_workgroups(n_sondas as u32, 1, 1);
    }
    if let Some(p) = &p_ricochete {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        cp.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    // ⚠️ **E o BORRÃO é um TERCEIRO despacho, pela MESMA razão**: ele lê a vizinhança `3×3` do canal
    // CRU, logo precisa dele escrito em todo o quadro. *Escrever no mesmo sítio de onde os vizinhos
    // estão a ler é uma corrida, e é por isso que os slots do liso existem.*
    if let Some(p) = &p_borra {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        cp.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(&p_pinta);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        cp.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }
    // ⚠️ **A borda depois do interior, e a ordem é a lei**: ela SOBRESCREVE o pixel que o interior
    // acabou de escrever, exactamente como o laço em série da CPU faz depois das linhas.
    if let Some(p) = &p_bordas {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        cp.dispatch_workgroups(n_bordas.div_ceil(64), 1, 1);
    }
    let leitura = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("leitura"),
        size: (n * 4).max(16),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    enc.copy_buffer_to_buffer(&b_saida, 0, &leitura, 0, (n * 4).max(16));
    queue.submit([enc.finish()]);
    leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let dados = leitura.slice(..).get_mapped_range();
    #[allow(clippy::cast_possible_truncation)]
    let out = dados[..(n as usize) * 4].to_vec();
    drop(dados);
    out
}
