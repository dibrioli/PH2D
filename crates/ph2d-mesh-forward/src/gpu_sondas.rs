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

use super::{Forward, QUADRO, quadro_impl};
use crate::{Cena, Instancia};

/// O lado do octaedro de uma captura no nível `0`, com a borda (`1` texel a toda a volta).
pub(crate) const LADO: u32 = 128;
/// Os níveis: `α = (k / (NIVEIS − 1))²`, do espelho (`128`) ao mais largo (`4`).
pub(crate) const NIVEIS: u32 = 6;
/// O lado de uma face do cubo.
pub(crate) const FACE: u32 = 128;
/// As amostras do lóbulo por texel nos níveis `> 0` (filtradas: cada uma lê o nível do ângulo dela).
const TAPS: u32 = 64;
/// ⭐ **A paralaxe**: um passo por nível onde se lê a distância, do GROSSO ao fino. O primeiro, grosso,
/// sente a vizinha mais perto mesmo quando a direcção crua não a vê. Medido (03/10, `sonda_da_paralaxe`,
/// os raios do oráculo que acertam uma vizinha, contra o ponto acertado visto do centro): a esfera
/// `[1, 1]` `3,60°` médio / `19,1°` máx (a direcção crua não via nada e o passo desistia), `[4, 2, 1]`
/// `0,05°` / `0,44°`, `[4, 2, 1, 0]` `0,02°` / `0,22°`; a caixa `[1, 1]` `0,24°` / `2,9°`, `[4, 2, 1, 0]`
/// `0,13°` / `2,9°`. Mais passos não ganham nada.
const PARALAXE: [f32; 4] = [4.0, 2.0, 1.0, 0.0];
/// ⭐ O máximo de capturas: duas camadas por captura nos `256` do `max_texture_array_layers` do WebGL2.
pub(crate) const MAX: usize = 128;
pub(crate) const FORMATO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// As faces: `(para onde olha, cima)`; a direita é `olha × cima`. A MESMA tabela no WGSL ([`constantes`]).
const FACES: [([f32; 3], [f32; 3]); 6] = [
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
         const SONDA_PARALAXE: array<f32, {np}> = array<f32, {np}>({});\n\
         const SONDA_FACE_W: array<vec3<f32>, 6> = array<vec3<f32>, 6>({});\n\
         const SONDA_FACE_UP: array<vec3<f32>, 6> = array<vec3<f32>, 6>({});\n\
         const SONDA_HAMMERSLEY: array<vec2<f32>, {TAPS}> = array<vec2<f32>, {TAPS}>({});\n",
        PARALAXE.map(|x| format!("{x:?}")).join(", "),
        w.join(", "),
        up.join(", "),
        xi.join(", "),
        np = PARALAXE.len(),
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

/// ⭐ **As capturas deste quadro.**
pub(crate) struct Plano {
    /// Quantas capturas (a captura `s` é a da instância `s` da lista do quadro).
    pub n: usize,
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
    pub(super) bind: wgpu::BindGroup,
    pub(super) faces_cor: wgpu::TextureView,
    pub(super) faces_dist: wgpu::TextureView,
    pub(super) faces_prof: wgpu::TextureView,
    pub(super) cadeia: [wgpu::Texture; 2],
    /// A passagem de cada nível: `[cor, distância]` (textura e vista).
    pub(super) tmp: Vec<[(wgpu::Texture, wgpu::TextureView); 2]>,
    /// As capturas: `(capacidade, textura, vista de todas as camadas)`.
    pub arranjo: (usize, wgpu::Texture, wgpu::TextureView),
    /// Os uniformes das faces: `(capacidade em faces, buffer)`.
    pub(super) quadros: Option<(usize, wgpu::Buffer)>,
    chave: Vec<u8>,
    /// Quantas vezes as capturas foram refeitas (os gates do «refaz só quando muda»).
    pub refeitas: u64,
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

fn arranjo(device: &wgpu::Device, cap: usize) -> (usize, wgpu::Texture, wgpu::TextureView) {
    // ⚠️ `2 · cap` com `cap` potência de 2: nunca múltiplo de 6 (o GLES faria cubos).
    debug_assert!(cap.is_power_of_two());
    let t = textura(
        device,
        (LADO, LADO, 2 * cap as u32),
        NIVEIS,
        FORMATO,
        wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
    );
    let v = t.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });
    (cap, t, v)
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

fn alvos_duplos() -> [Option<wgpu::ColorTargetState>; 2] {
    let a = Some(wgpu::ColorTargetState {
        format: FORMATO,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    });
    [a.clone(), a]
}

impl Sondas {
    /// ⭐ Os QUATRO pipelines das capturas (as faces, o octaedro, a cadeia, o pré-filtro) e as texturas
    /// fixas — compilados UMA vez, com o desenhista.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn nova(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        modulo: &wgpu::ShaderModule,
        pl_g0g1: &wgpu::PipelineLayout,
        vertice: &[wgpu::VertexBufferLayout<'_>],
    ) -> Self {
        let alvos = alvos_duplos();
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
                targets: &alvos,
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::gpu_alvo::PROFUNDIDADE,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward sondas"),
            entries: &[
                entrada_textura(0),
                entrada_textura(1),
                entrada_textura(2),
                entrada_textura(3),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                crate::gpu_ligacoes::uniforme(5, true, wgpu::ShaderStages::FRAGMENT),
            ],
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
        let tela = |entrada: &str| {
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
                    targets: &alvos,
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let (octa, desce, prefiltro) = (tela("fs_octa"), tela("fs_desce"), tela("fs_prefiltro"));
        let rt = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let tb = wgpu::TextureUsages::TEXTURE_BINDING;
        let vista = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas = (3 * FACE, 2 * FACE, 1);
        let faces_cor = vista(&textura(device, atlas, 1, FORMATO, rt | tb));
        let faces_dist = vista(&textura(device, atlas, 1, FORMATO, rt | tb));
        let faces_prof = vista(&textura(
            device,
            atlas,
            1,
            crate::gpu_alvo::PROFUNDIDADE,
            rt,
        ));
        let cadeia = [0, 1].map(|_| {
            textura(
                device,
                (LADO, LADO, 1),
                NIVEIS,
                FORMATO,
                tb | wgpu::TextureUsages::COPY_DST,
            )
        });
        let tmp = (0..NIVEIS)
            .map(|k| {
                [0, 1].map(|_| {
                    let t = textura(
                        device,
                        (LADO >> k, LADO >> k, 1),
                        1,
                        FORMATO,
                        rt | wgpu::TextureUsages::COPY_SRC,
                    );
                    let v = vista(&t);
                    (t, v)
                })
            })
            .collect();
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
        let (vc0, vc1) = (vista(&cadeia[0]), vista(&cadeia[1]));
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh-forward sondas"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&faces_cor),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&faces_dist),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&vc0),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&vc1),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&amostra),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &passo,
                        offset: 0,
                        size: std::num::NonZeroU64::new(16),
                    }),
                },
            ],
        });
        Self {
            faces,
            octa,
            desce,
            prefiltro,
            bind,
            faces_cor,
            faces_dist,
            faces_prof,
            cadeia,
            tmp,
            arranjo: arranjo(device, 1),
            quadros: None,
            chave: Vec::new(),
            refeitas: 0,
        }
    }
}

/// O centro da caixa local `(lo, hi)` posta no mundo por `modelo`, e os cantos dela no mundo.
fn caixa_no_mundo(modelo: &[[f32; 4]; 4], (lo, hi): ([f32; 3], [f32; 3])) -> ([f32; 3], [f32; 3]) {
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

impl Forward {
    /// Os gates ligam e desligam as capturas (a régua de controlo).
    #[cfg(test)]
    pub(crate) fn liga_reflexos(&mut self, liga: bool) {
        self.reflexos = liga;
    }

    /// A camada `camada` das capturas no nível `k`, lida de volta (`lado × lado` texels RGBA, linha a linha).
    #[cfg(test)]
    pub(crate) fn le_sonda(&self, camada: u32, k: u32) -> Option<Vec<[f32; 4]>> {
        let s = self.sondas.as_ref()?;
        let w = LADO >> k;
        let bpr = (w * 8).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh-forward sonda lida"),
            size: u64::from(bpr * w),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &s.arranjo.1,
                mip_level: k,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: camada,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpr),
                    rows_per_image: Some(w),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: w,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([enc.finish()]);
        let fatia = buffer.slice(..);
        fatia.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let dados = fatia.get_mapped_range();
        let mut out = Vec::with_capacity((w * w) as usize);
        for y in 0..w as usize {
            let linha: &[half::f16] =
                bytemuck::cast_slice(&dados[y * bpr as usize..y * bpr as usize + w as usize * 8]);
            out.extend(linha.chunks(4).map(|c| [0, 1, 2, 3].map(|e| c[e].to_f32())));
        }
        Some(out)
    }

    /// Quantas vezes as capturas foram refeitas.
    #[cfg(test)]
    pub(crate) fn sondas_refeitas(&self) -> u64 {
        self.sondas.as_ref().map_or(0, |s| s.refeitas)
    }

    /// ⭐ **Quem tem captura**: o campo `sonda` do `Objeto` de cada instância do quadro — `(camada, centro)`
    /// ou `−1`. Só com vizinhas (duas peças ou mais), com a placa que as desenha, até [`MAX`].
    pub(super) fn atribui_sondas(&self, objs: &[&Instancia]) -> Vec<[f32; 4]> {
        let liga = self.reflexos && self.sondas.is_some() && objs.len() >= 2;
        objs.iter()
            .enumerate()
            .map(|(i, o)| match self.malhas.get(&o.malha) {
                Some(m) if liga && i < MAX => {
                    let (a, b) = caixa_no_mundo(&o.modelo, m.caixa);
                    [
                        2.0 * i as f32,
                        0.5 * (a[0] + b[0]),
                        0.5 * (a[1] + b[1]),
                        0.5 * (a[2] + b[2]),
                    ]
                }
                _ => [-1.0, 0.0, 0.0, 0.0],
            })
            .collect()
    }

    /// ⭐⭐ **O plano das capturas**: os uniformes das faces (o enquadramento das sombras da cena INTEIRA
    /// — a da vista mudaria com a câmara) e a CHAVE: tudo o que as faces leem. Igual à do quadro anterior
    /// ⇒ as capturas ficam como estão.
    pub(super) fn planeia_sondas(
        &mut self,
        cena: &Cena<'_>,
        objs: &[&Instancia],
        atrib: &[[f32; 4]],
        extra: (&[u8], &[f32]),
    ) -> Option<Plano> {
        let n = atrib.iter().filter(|a| a[0] >= 0.0).count();
        if n == 0 {
            return None;
        }
        let chave_luz = super::sombra_impl::chave(cena, self.foto.is_some(), self.sol.as_ref());
        let e = super::sombra_impl::enquadra(
            cena,
            chave_luz,
            |id| self.malhas.get(&id).map(|m| m.caixa),
            false,
        );
        let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        for o in objs {
            if let Some(m) = self.malhas.get(&o.malha) {
                let (a, b) = caixa_no_mundo(&o.modelo, m.caixa);
                for k in 0..3 {
                    lo[k] = lo[k].min(a[k]);
                    hi[k] = hi[k].max(b[k]);
                }
            }
        }
        let diag = (0..3).map(|k| (hi[k] - lo[k]).powi(2)).sum::<f32>().sqrt();
        let longe = 2.0 * diag + 1.0e-3;
        let perto = longe * 1.0e-4;
        let passo = PASSO_QUADRO as usize;
        let mut dados = vec![0u8; 6 * n * passo];
        for (s, a) in atrib.iter().take(n).enumerate() {
            let c = [a[1], a[2], a[3]];
            for f in 0..6 {
                let cam = crate::Camera {
                    view_proj: face_vp(f, c, perto, longe),
                    olho: c,
                    perspectiva: true,
                    dir_vista: FACES[f].0,
                };
                let u = quadro_impl::uniforme_do_quadro(
                    &Cena {
                        camera: cam,
                        ..*cena
                    },
                    &e,
                    self.foto.is_some(),
                    self.sol.as_ref(),
                    true,
                );
                let at = (6 * s + f) * passo;
                dados[at..at + QUADRO * 4].copy_from_slice(bytemuck::cast_slice(&u));
            }
        }
        let mut chave = dados.clone();
        chave.extend_from_slice(extra.0);
        chave.extend_from_slice(bytemuck::cast_slice(extra.1));
        chave.extend_from_slice(&self.geracao.to_le_bytes());
        let sondas = self.sondas.as_mut()?;
        let cap = n.next_power_of_two();
        if sondas.arranjo.0 < cap {
            sondas.arranjo = arranjo(&self.device, cap);
            sondas.chave.clear();
        }
        if sondas.quadros.as_ref().is_none_or(|(c, _)| *c < 6 * cap) {
            let b = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-mesh-forward sondas quadros"),
                size: (6 * cap) as u64 * PASSO_QUADRO,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            sondas.quadros = Some((6 * cap, b));
        }
        let refaz = sondas.chave != chave;
        if refaz {
            if let Some((_, b)) = &sondas.quadros {
                self.queue.write_buffer(b, 0, &dados);
            }
            sondas.chave = chave;
            sondas.refeitas += 1;
        }
        Some(Plano {
            n,
            refaz,
            ha_sombra: e.ha_sombra,
            ha_chao: e.ha_chao,
        })
    }
}
