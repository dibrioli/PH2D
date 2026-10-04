//! ⭐⭐⭐ **AS CAPTURAS DE REFLEXO** — a peça brilhante mostra as VIZINHAS (e, com elas, as sombras
//! delas no chão). O idioma é o do Fortnite móvel / Unreal: um cubo 360° por peça, desenhado do centro
//! dela com o MESMO desenhista (as faces são o `luz_de_cena` de cada vizinha), pré-filtrado por
//! rugosidade (um nível por `α`, a pergunta do pré-filtro do céu) e REFEITO SÓ quando a chave muda —
//! girar a câmara não refaz nada.
//!
//! - **O que a captura guarda:** só as vizinhas, com cobertura (`rgb` pré-multiplicado, `a`). O céu e o
//!   chão continuam a lei exacta do ponto (`chao_tapa.rs`): sem paralaxe, e o chão lê TODAS as sombras
//!   (a máscara da zona da peça só vale sem captura).
//! - **Paralaxe:** a distância do centro às vizinhas vai na camada irmã; o raio `p + t·r` é posto na
//!   esfera dessa distância ([`PARALAXE`] vezes).
//! - **Cabe no WebGL2:** texturas (nada de armazenamento); as `6` faces lado a lado num atlas `3 × 2`
//!   (uma textura quadrada de `6` camadas o GLES faz CUBO à força — `wgpu-hal` `get_info_from_desc`);
//!   as capturas num `texture_2d_array` cujo número de camadas é potência de `2` (nunca múltiplo de `6`).
//! - **Sem cena-linear em `Rgba16Float`** (a placa sem o brilho) não há capturas, e vale a lei sem elas.
//! - **Uma peça sozinha** não tem vizinhas: não há capturas, e o quadro é o de sempre ao byte.

use super::QUADRO;

/// O lado do octaedro de uma captura no nível `0`, com a borda (`1` texel a toda a volta), e o de uma
/// face do cubo. ⭐ Medido (04/10, `o_espelho_mostra_as_vizinhas_como_no_cycles` e
/// `instrumento_custo_sondas`, 1080p, RTX 5060 Ti, load `< 4`):
///
/// | octaedro / face | esfera nítida (miolo / contorno) | esfera áspera | caixa nítida (miolo) | arrastar, 16 peças |
/// |---|---|---|---|---|
/// | `128 / 128` | `0,0050 / 0,103` | `0,027` | `0,060` | `+3,84 ms` |
/// | `256 / 128` | `0,0051 / 0,056` | `0,033` | `0,036` | `+3,92 ms` |
/// | `128 / 256` | `0,0050 / 0,097` | `0,027` | `0,059` | `+4,34 ms` |
/// | `256 / 256` | `0,0051 / 0,053` | `0,033` | `0,034` | `+4,78 ms` |
/// | **`512 / 256`** (com a busca do nítido) | `0,0052 / 0,026` | `0,034` | **`0,021`** | `~+8 ms` (load `~15`) |
///
/// ⭐ O report do dono (04/10, o cromo a encher o ecrã): a `256` o reflexo ampliado era BORRADO e a face
/// de cima da caixa refletida saía em dentes; a `512 / 256` limpa-os (foto da sonda de perto). O preço é
/// arrastar: com 16 peças de metal o quadro passa os `8 ms` — aberto no handoff.
///
/// O espelho PLANO perto da vizinha (a caixa) é quem pede o octaedro fino; as faces finas não lhe dão
/// nada. Memória: `~1,4 MB` por captura (as duas camadas com os níveis, meia precisão).
pub(crate) const LADO: u32 = 512;
pub(crate) const FACE: u32 = 256;
/// Os níveis: `α = (k / (NIVEIS − 1))²`, do espelho (`256`) ao mais largo (`8`).
pub(crate) const NIVEIS: u32 = 6;
/// As amostras do lóbulo por texel nos níveis `> 0` (filtradas: cada uma lê o nível do ângulo dela).
/// Medido (04/10, 16 peças a arrastar): `64` → `16` amostras tira `~1 ms` dos `2,5 ms` do pós-processamento.
const TAPS: u32 = 64;
/// ⭐ **A busca do reflexo nítido** (`sonda_le.wgsl`): passos iguais em ângulo, um a cada [`PASSO`]
/// texels do nível `0` do arco do raio, entre [`MARCHA_MIN`] e [`MARCHA_MAX`]; as bissecções e a
/// espessura relativa. ⛔ Medido (04/10, a câmara de perto da cena 42): `24` passos fixos deixavam
/// DENTES no topo da caixa refletida (a face vista de raspão saltava-se ao acaso), `96` limpavam-nos a
/// `+0,3–0,4 ms` por quadro; a busca HIERÁRQUICA sobre o mínimo da distância por nível (o Hi-Z) saltava
/// partes finas (o mínimo filtrado não é conservador) e deixava as bordas PIORES — recusada.
pub(crate) const MARCHA_MIN: u32 = 8;
pub(crate) const MARCHA_MAX: u32 = 48;
pub(crate) const PASSO: f32 = 2.0;
pub(crate) const REFINO: u32 = 6;
/// A espessura (relativa) que um cruzamento tolera por trás da vizinha, e a franja onde o peso cai a `0`.
/// ⛔ Medido (04/10, 4.º report do dono: «um espaço vazio onde os reflexos deviam sobrepor-se»): a `0,05` /
/// `0,1` o reflexo ALARGAVA além da vizinha (régua da orla, câmara do app: `0,0255`, `775` grosseiros) e, onde
/// outra vizinha ficava atrás, a borda certa parecia um buraco ao lado do resto alargado. `0,02` → `0,0169 /
/// 183`; `0,01` → `0,0166 / 169` (a junta `0,0065 / 0`, o contorno do par `465 → 344` grosseiros, o miolo de
/// perto `0,0126 → 0,0131`); `0,005` igual a `0,01` (satura: o resto é a face lateral, que a captura não vê).
pub(crate) const ESPESSURA: f32 = 0.01;
pub(crate) const FRANJA: f32 = 0.01;
/// ⭐ **A ARESTA entre duas vizinhas** (o 3.º report do dono, a «junta»): a captura guarda também a média
/// do QUADRADO da distância, que atravessa toda média (MSAA, octaedro, níveis, filtro linear); onde o
/// desvio passa esta fracção da distância média, o texel mistura superfícies de distâncias diferentes e a
/// distância lida é FANTASMA — a busca não aceita cruzamento ali.
pub(crate) const ARESTA: f32 = 0.1;
/// Numa aresta, até quantos texels adiante (no arco do raio) se procura a leitura pura da vizinha da frente.
pub(crate) const ARESTA_PASSOS: u32 = 4;
/// A cobertura a partir da qual a cor do cruzamento se lê (abaixo é a silhueta, meio céu).
pub(crate) const CHEIA: f32 = 0.95;
/// Até quantos níveis de borrão o reflexo usa a BUSCA (passando ao ponto fixo no último). ⛔ Medido
/// (04/10, o par a `0,3`): a `3` o lobo áspero via a fronteira acerta/falha da busca como uma aresta dura.
pub(crate) const LOD_BUSCA: f32 = 1.0;
/// Os níveis de borrão (abaixo de [`LOD_BUSCA`]) em que a busca passa ao ponto fixo. ⛔ Medido (04/10, a
/// junta a `0,05`): com a passagem em `1` nível o ponto fixo pesava `~37 %` no quase nítido e punha uma
/// ponte translúcida entre duas vizinhas (faixa `0,170`); só com a busca, `0,0185`.
pub(crate) const PASSAGEM: f32 = 0.5;
/// A cobertura a que um passo do ponto fixo pesa por inteiro (abaixo, mistura-se: sem aresta).
pub(crate) const COBERTURA_PLENA: f32 = 0.05;
/// No reflexo ÁSPERO a paralaxe só lê distâncias tão borradas como o lobo e mais estes níveis.
pub(crate) const ACIMA: f32 = 3.0;
/// ⭐ **A paralaxe do reflexo ÁSPERO** (o nítido é a busca, [`MARCHA_MAX`]): a esfera das vizinhas (a
/// distância média em toda a volta, `sonda_esfera`) e um passo por nível onde se lê a distância, do
/// GROSSO ao fino. Medido (04/10, octaedro `512`, `a_paralaxe_acerta_onde_o_raio_bate`: os raios do
/// oráculo que acertam uma vizinha, contra o ponto acertado visto do centro, médio / máx):
///
/// | passos | esfera | caixa |
/// |---|---|---|
/// | nenhum | `11,2° / 19,1°` | `12,0° / 21,6°` |
/// | `[5, 2, 1, 0]` sem a esfera das vizinhas | `0,09° / 19,1°` — o nível mais largo a `512` (16 px) já não sente a vizinha | `0,25° / 5,8°` |
/// | `[5, 2, 1, 0]` | `0,01° / 0,07°` | `0,18° / 5,48°` |
/// | **`[5, 3, 1, 0]`** | `0,01° / 0,07°` | `0,10° / 2,97°` |
///
/// (A `256`, sem a esfera, `[5, 2, 1, 0]` dava `0,01° / 0,10°` e `0,09° / 2,45°`; `[4, 2, 1, 0]` `19,1°`.)
pub(crate) const PARALAXE: [f32; 4] = [5.0, 3.0, 1.0, 0.0];
/// ⭐ O máximo de capturas — pela MEMÓRIA da placa: `~5,6 MB` cada uma a 512 (as duas camadas com os
/// níveis, meia precisão), `32` são `~180 MB`. (O WebGL2 deixaria `128`: `256` camadas ÷ 2.)
pub(crate) const MAX: usize = 32;
pub(crate) const FORMATO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// As faces: `(para onde olha, cima)`; a direita é `olha × cima`. A MESMA tabela no WGSL ([`constantes`]).
pub(super) const FACES: [([f32; 3], [f32; 3]); 6] = [
    ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    ([-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
    ([0.0, -1.0, 0.0], [0.0, 0.0, 1.0]),
    ([0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
    ([0.0, 0.0, -1.0], [0.0, 1.0, 0.0]),
];

fn cruz(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn v3(v: [f32; 3]) -> String {
    format!("vec3<f32>({:?}, {:?}, {:?})", v[0], v[1], v[2])
}

/// As constantes das capturas em WGSL, escritas a partir das daqui (nunca à mão).
pub(crate) fn constantes() -> String {
    let w: Vec<String> = FACES.iter().map(|f| v3(f.0)).collect();
    let up: Vec<String> = FACES.iter().map(|f| v3(f.1)).collect();
    let xi: Vec<String> = (0..TAPS)
        .map(|i| {
            format!(
                "vec2<f32>({:?}, {:?})",
                i as f32 / TAPS as f32,
                i.reverse_bits() as f32 / 4_294_967_296.0
            )
        })
        .collect();
    format!(
        "const SONDA_LADO: u32 = {LADO}u;\nconst SONDA_NIVEIS: u32 = {NIVEIS}u;\n\
         const SONDA_FACE: u32 = {FACE}u;\nconst SONDA_TAPS: u32 = {TAPS}u;\n\
         const SONDA_PASSOS: u32 = {np}u;\n\
         const SONDA_MARCHA_MIN: u32 = {MARCHA_MIN}u;\nconst SONDA_MARCHA_MAX: u32 = {MARCHA_MAX}u;\n\
         const SONDA_PASSO: f32 = {PASSO:?};\nconst SONDA_TEXEL: f32 = {texel:?};\n\
         const SONDA_REFINO: u32 = {REFINO}u;\n\
         const SONDA_ESPESSURA: f32 = {ESPESSURA:?};\nconst SONDA_FRANJA: f32 = {FRANJA:?};\n\
         const SONDA_ARESTA: f32 = {ARESTA:?};\nconst SONDA_ARESTA_PASSOS: u32 = {ARESTA_PASSOS}u;\nconst SONDA_CHEIA: f32 = {CHEIA:?};\n\
         const SONDA_LOD_BUSCA: f32 = {LOD_BUSCA:?};\nconst SONDA_PASSAGEM: f32 = {PASSAGEM:?};\nconst SONDA_COBERTURA_PLENA: f32 = {COBERTURA_PLENA:?};\n\
         const SONDA_ACIMA: f32 = {ACIMA:?};\n\
         const SONDA_PARALAXE: array<f32, {np}> = array<f32, {np}>({});\n\
         const SONDA_FACE_W: array<vec3<f32>, 6> = array<vec3<f32>, 6>({});\n\
         const SONDA_FACE_UP: array<vec3<f32>, 6> = array<vec3<f32>, 6>({});\n\
         const SONDA_HAMMERSLEY: array<vec2<f32>, {TAPS}> = array<vec2<f32>, {TAPS}>({});\n",
        PARALAXE.map(|x| format!("{x:?}")).join(", "),
        w.join(", "),
        up.join(", "),
        xi.join(", "),
        np = PARALAXE.len(),
        // O ângulo de um texel do nível `0` (90° por meia aresta do conteúdo).
        texel = std::f32::consts::FRAC_PI_2 / ((LADO - 2) as f32 * 0.5),
    )
}

/// O shader dos passes da captura (o octaedro, a cadeia e o pré-filtro).
fn fonte() -> String {
    format!(
        "{}\n{}\n{}",
        constantes(),
        ph2d_sky::wgsl::OCT,
        include_str!("sondas.wgsl")
    )
}

/// ⭐ **A face `f` de uma captura em `c`** — mundo → recorte (colunas), perspectiva de `90°`, a
/// profundidade em `[0, 1]` entre `perto` e `longe`.
pub(crate) fn face_vp(f: usize, c: [f32; 3], perto: f32, longe: f32) -> [[f32; 4]; 4] {
    let (w, up) = FACES[f];
    let dir = cruz(w, up);
    let s = longe / (longe - perto);
    let mut m = [[0.0f32; 4]; 4];
    for e in 0..3 {
        m[e] = [dir[e], up[e], w[e] * s, w[e]];
    }
    m[3] = [
        -dot(c, dir),
        -dot(c, up),
        -dot(c, w) * s - perto * s,
        -dot(c, w),
    ];
    m
}

/// O ladrilho da face `f` no atlas `3 × 2`: `(x, y)` em texels.
pub(super) fn ladrilho(f: usize) -> (u32, u32) {
    ((f as u32 % 3) * FACE, (f as u32 / 3) * FACE)
}

/// A caixa `(lo, hi)` do mundo cai (em parte) na face `f` da captura em `c`? — os quatro planos do
/// tronco de `90°` pelo centro; fora se os `8` cantos ficam todos do lado de fora de um deles.
pub(super) fn na_face(f: usize, c: [f32; 3], (lo, hi): ([f32; 3], [f32; 3])) -> bool {
    let (w, up) = FACES[f];
    let dir = cruz(w, up);
    let planos = [-1.0f32, 1.0].into_iter().flat_map(|sg| {
        [
            [0, 1, 2].map(|e| w[e] + sg * dir[e]),
            [0, 1, 2].map(|e| w[e] + sg * up[e]),
        ]
    });
    let cantos: Vec<[f32; 3]> = (0..8)
        .map(|k| {
            [
                if k & 1 == 0 { lo[0] } else { hi[0] } - c[0],
                if k & 2 == 0 { lo[1] } else { hi[1] } - c[1],
                if k & 4 == 0 { lo[2] } else { hi[2] } - c[2],
            ]
        })
        .collect();
    planos
        .into_iter()
        .all(|n| cantos.iter().any(|q| dot(*q, n) >= 0.0))
}

/// ⭐ **As capturas deste quadro.**
pub(crate) struct Plano {
    /// O centro de cada captura (a captura `s` é a da instância `s` da lista do quadro).
    pub centros: Vec<[f32; 3]>,
    /// A chave mudou: as capturas refazem-se neste quadro.
    pub refaz: bool,
    /// O enquadramento das sombras das capturas (a cena inteira, não a vista).
    pub ha_sombra: bool,
    pub ha_chao: bool,
}

pub(crate) struct Sondas {
    pub(super) faces: wgpu::RenderPipeline,
    pub(super) octa: wgpu::RenderPipeline,
    pub(super) desce: wgpu::RenderPipeline,
    pub(super) prefiltro: wgpu::RenderPipeline,
    /// O grupo de DESCER ao nível `k` (a cadeia `0..k` ligada, o resto a textura vazia).
    pub(super) binds: Vec<wgpu::BindGroup>,
    /// O grupo do PRÉ-FILTRO do nível `k` (a cadeia inteira).
    pub(super) binds_pre: Vec<wgpu::BindGroup>,
    pub(super) faces_cor: wgpu::TextureView,
    pub(super) faces_dist: wgpu::TextureView,
    pub(super) faces_prof: wgpu::TextureView,
    /// As faces com `MSAA` amostras (resolvidas em `faces_cor` e `faces_dist`).
    pub(super) faces_ms: [wgpu::TextureView; 2],
    /// A CADEIA: um nível por textura (`[cor, distância]`) — lida directamente, sem cópias.
    pub(super) cadeia: Vec<[wgpu::TextureView; 2]>,
    /// As capturas: `(capacidade, textura, vista de todas as camadas, [por captura][nível] = [cor,
    /// distância])`.
    pub arranjo: Arranjo,
    /// Os uniformes das faces: `(capacidade em faces, buffer)`.
    pub(super) quadros: Option<(usize, wgpu::Buffer)>,
    pub(super) chave: Vec<u8>,
    /// Quantas vezes as capturas foram refeitas (os gates do «refaz só quando muda»).
    pub refeitas: u64,
}

/// A matriz das capturas e as vistas de escrita de cada uma.
pub(crate) struct Arranjo {
    pub cap: usize,
    /// Só os gates a leem de volta (as vistas seguram a textura).
    #[cfg(test)]
    pub textura: wgpu::Texture,
    pub vista: wgpu::TextureView,
    pub alvos: Vec<Vec<[wgpu::TextureView; 2]>>,
}

/// O passo de um uniforme de face (o deslocamento mínimo de um uniforme é `256`).
pub(crate) const PASSO_QUADRO: u64 = ((QUADRO * 4) as u64).next_multiple_of(256);

fn textura(
    device: &wgpu::Device,
    (w, h, camadas): (u32, u32, u32),
    niveis: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-mesh-forward sonda"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: camadas,
        },
        mip_level_count: niveis,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

pub(super) fn arranjo(device: &wgpu::Device, cap: usize) -> Arranjo {
    // ⚠️ `2 · cap` com `cap` potência de 2: nunca múltiplo de 6 (o GLES faria cubos).
    debug_assert!(cap.is_power_of_two());
    let textura = textura(
        device,
        (LADO, LADO, 2 * cap as u32),
        NIVEIS,
        FORMATO,
        wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
    );
    let vista = textura.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });
    let alvos = (0..cap as u32)
        .map(|s| {
            (0..NIVEIS)
                .map(|k| {
                    [2 * s, 2 * s + 1].map(|c| {
                        textura.create_view(&wgpu::TextureViewDescriptor {
                            dimension: Some(wgpu::TextureViewDimension::D2),
                            base_mip_level: k,
                            mip_level_count: Some(1),
                            base_array_layer: c,
                            array_layer_count: Some(1),
                            ..Default::default()
                        })
                    })
                })
                .collect()
        })
        .collect();
    Arranjo {
        cap,
        #[cfg(test)]
        textura,
        vista,
        alvos,
    }
}

fn entrada_textura(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

/// A entrada das capturas no grupo `0` do desenhista (`binding 20`).
pub(crate) fn entrada(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2Array,
            multisampled: false,
        },
        count: None,
    }
}

fn alvos(n: usize) -> Vec<Option<wgpu::ColorTargetState>> {
    vec![
        Some(wgpu::ColorTargetState {
            format: FORMATO,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        });
        n
    ]
}

/// As ligações dos passes da captura: o atlas das faces (`0`, `1`), a cadeia da cor nível a nível
/// (`2..2 + NIVEIS`), a distância do nível anterior, o amostrador e o passo.
const LIG_DIST: u32 = 2 + NIVEIS;
const LIG_AMOSTRA: u32 = LIG_DIST + 1;
const LIG_PASSO: u32 = LIG_DIST + 2;

impl Sondas {
    /// ⭐ Os QUATRO pipelines das capturas (as faces; o octaedro; descer a cadeia; o pré-filtro) e as
    /// texturas fixas — compilados UMA vez, com o desenhista. ⛔ Medido (04/10,
    /// `instrumento_custo_sondas`, 16 peças a arrastar): com cópias para a cadeia (`25` operações por
    /// captura) o pós-processamento custava `4,3 ms`; a cadeia é uma textura por nível (sem cópias). ⛔ E
    /// a cadeia com o pré-filtro no MESMO passe só deixava este ler até ao nível `k − 1`: a 512 o áspero
    /// saía aos blocos (report do dono, 04/10) — passes separados.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn nova(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        modulo: &wgpu::ShaderModule,
        pl_g0g1: &wgpu::PipelineLayout,
        vertice: &[wgpu::VertexBufferLayout<'_>],
    ) -> Self {
        let faces = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-mesh-forward sonda faces"),
            layout: Some(pl_g0g1),
            vertex: wgpu::VertexState {
                module: modulo,
                entry_point: Some("vs_objeto"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: vertice,
            },
            fragment: Some(wgpu::FragmentState {
                module: modulo,
                entry_point: Some("fs_sonda"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &alvos(2),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::gpu_alvo::PROFUNDIDADE,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            // ⭐ O contorno das vizinhas com anti-serrilhado: de perto o cromo AMPLIA a captura, e os
            // degraus de uma face sem ele viam-se (report do dono, 04/10).
            multisample: wgpu::MultisampleState {
                count: crate::MSAA,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        let mut entradas: Vec<wgpu::BindGroupLayoutEntry> =
            (0..LIG_AMOSTRA).map(entrada_textura).collect();
        entradas.push(wgpu::BindGroupLayoutEntry {
            binding: LIG_AMOSTRA,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
        entradas.push(crate::gpu_ligacoes::uniforme(
            LIG_PASSO,
            true,
            wgpu::ShaderStages::FRAGMENT,
        ));
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward sondas"),
            entries: &entradas,
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-mesh-forward sondas"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let m2 = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh-forward sondas"),
            source: wgpu::ShaderSource::Wgsl(fonte().into()),
        });
        let quatro = alvos(4);
        let tela = |entrada: &str, alvos: &[Option<wgpu::ColorTargetState>]| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ph2d-mesh-forward sondas tela"),
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &m2,
                    entry_point: Some("vs_tela"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &m2,
                    entry_point: Some(entrada),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: alvos,
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let dois = alvos(2);
        let (octa, desce, prefiltro) = (
            tela("fs_octa", &quatro),
            tela("fs_desce", &dois),
            tela("fs_prefiltro", &dois),
        );
        let rt = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let tb = wgpu::TextureUsages::TEXTURE_BINDING;
        let vista = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas = (3 * FACE, 2 * FACE, 1);
        let faces_cor = vista(&textura(device, atlas, 1, FORMATO, rt | tb));
        let faces_dist = vista(&textura(device, atlas, 1, FORMATO, rt | tb));
        let msaa = |format| {
            vista(&device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-mesh-forward sonda msaa"),
                size: wgpu::Extent3d {
                    width: atlas.0,
                    height: atlas.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: crate::MSAA,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: rt,
                view_formats: &[],
            }))
        };
        let faces_ms = [msaa(FORMATO), msaa(FORMATO)];
        let faces_prof = msaa(crate::gpu_alvo::PROFUNDIDADE);
        let cadeia: Vec<[wgpu::TextureView; 2]> = (0..NIVEIS)
            .map(|k| {
                [0, 1].map(|_| {
                    vista(&textura(
                        device,
                        (LADO >> k, LADO >> k, 1),
                        1,
                        FORMATO,
                        rt | tb,
                    ))
                })
            })
            .collect();
        let vazia = vista(&textura(device, (1, 1, 1), 1, FORMATO, tb));
        let passo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh-forward sondas passo"),
            size: 256 * u64::from(NIVEIS),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut p = vec![0u8; 256 * NIVEIS as usize];
        for k in 0..NIVEIS {
            p[256 * k as usize..256 * k as usize + 4].copy_from_slice(&k.to_le_bytes());
        }
        queue.write_buffer(&passo, 0, &p);
        let amostra = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-mesh-forward sondas"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        // ⚠️ O passe que DESCE ao nível `k` escreve a cadeia `k`: só os níveis `< k` ficam ligados (uma
        // textura lida e escrita no mesmo passe é o que o WebGPU recusa e o GLES não garante); o
        // pré-filtro do nível `k` lê a cadeia inteira (escreve a captura) e a distância da cadeia `k`.
        let grupo = |ate: u32, dist: Option<u32>| {
            fn tv(v: &wgpu::TextureView) -> wgpu::BindingResource<'_> {
                wgpu::BindingResource::TextureView(v)
            }
            let mut e = vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: tv(&faces_cor),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: tv(&faces_dist),
                },
            ];
            for i in 0..NIVEIS {
                e.push(wgpu::BindGroupEntry {
                    binding: 2 + i,
                    resource: tv(if i < ate {
                        &cadeia[i as usize][0]
                    } else {
                        &vazia
                    }),
                });
            }
            e.push(wgpu::BindGroupEntry {
                binding: LIG_DIST,
                resource: tv(dist.map_or(&vazia, |d| &cadeia[d as usize][1])),
            });
            e.push(wgpu::BindGroupEntry {
                binding: LIG_AMOSTRA,
                resource: wgpu::BindingResource::Sampler(&amostra),
            });
            e.push(wgpu::BindGroupEntry {
                binding: LIG_PASSO,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &passo,
                    offset: 0,
                    size: std::num::NonZeroU64::new(16),
                }),
            });
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-mesh-forward sondas"),
                layout: &bgl,
                entries: &e,
            })
        };
        let binds = (0..NIVEIS).map(|k| grupo(k, k.checked_sub(1))).collect();
        let binds_pre = (0..NIVEIS).map(|k| grupo(NIVEIS, Some(k))).collect();
        Self {
            faces,
            octa,
            desce,
            prefiltro,
            binds,
            binds_pre,
            faces_cor,
            faces_dist,
            faces_prof,
            faces_ms,
            cadeia,
            arranjo: arranjo(device, 1),
            quadros: None,
            chave: Vec::new(),
            refeitas: 0,
        }
    }
}

/// O centro da caixa local `(lo, hi)` posta no mundo por `modelo`, e os cantos dela no mundo.
pub(super) fn caixa_no_mundo(
    modelo: &[[f32; 4]; 4],
    (lo, hi): ([f32; 3], [f32; 3]),
) -> ([f32; 3], [f32; 3]) {
    let (mut a, mut b) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for k in 0..8 {
        let c = [
            if k & 1 == 0 { lo[0] } else { hi[0] },
            if k & 2 == 0 { lo[1] } else { hi[1] },
            if k & 4 == 0 { lo[2] } else { hi[2] },
        ];
        for e in 0..3 {
            let w = modelo[3][e] + (0..3).map(|j| modelo[j][e] * c[j]).sum::<f32>();
            a[e] = a[e].min(w);
            b[e] = b[e].max(w);
        }
    }
    (a, b)
}
