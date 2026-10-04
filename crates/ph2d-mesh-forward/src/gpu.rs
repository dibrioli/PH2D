//! ⭐⭐ **O DESENHISTA NA PLACA** — pipelines compilados UMA vez (mover, mudar a cor ou acrescentar
//! um objeto não compila nada: gate `nada_compila_ao_editar`), malhas subidas por id, e o quadro:
//! sombra → chão + objetos (MSAA, resolvido; com o brilho, também a cena-linear) → a cadeia do
//! brilho ([`crate::gpu_brilho`]) → codificação (com o halo) → leitura.

use std::collections::BTreeMap;

use crate::gpu_alvo::{Alvos, PROFUNDIDADE, SAIDA};
use crate::gpu_ligacoes::{textura_float, uniforme};
use crate::{Ambiente, Cena, Malha};

#[path = "gpu_quadro.rs"]
mod quadro_impl;
#[path = "gpu_sombra.rs"]
mod sombra_impl;
#[path = "gpu_sondas.rs"]
pub(crate) mod sondas_impl;
#[path = "gpu_sondas_passes.rs"]
pub(crate) mod sondas_passes;
#[path = "gpu_texturas.rs"]
mod texturas;

/// Os bytes de um vértice: posição, normal, AO, material.
const VERTICE: u64 = 32;
/// O alinhamento do deslocamento dinâmico de um uniforme (o mínimo que todo aparelho aceita).
pub(crate) const SLOT: u64 = 256;
/// O tamanho do uniforme do quadro — o `Quadro` do WGSL.
pub(crate) const QUADRO: usize = 2 * 16
    + 6 * 4
    + ph2d_style::wgsl::PACKED
    + 4
    + 4
    + 4
    + 4
    + 4
    + 16
    + 4
    + 16
    + 2 * crate::MAX_LUZES * 4;

struct MalhaGpu {
    vertices: wgpu::Buffer,
    /// As duas curvaturas (`[material, estilo]`) por vértice, num buffer à parte: assadas depois da
    /// malha e refeitas quando a suavidade muda, sem resubir a malha ([`Forward::sobe_curvatura`]).
    curvatura: wgpu::Buffer,
    n_vertices: usize,
    indices: wgpu::Buffer,
    n: u32,
    /// A caixa local `(min, max)` — o mapa de sombra enquadra o mundo de todos os objetos.
    caixa: ([f32; 3], [f32; 3]),
}

/// ⭐ **O desenhista.**
pub struct Forward {
    device: wgpu::Device,
    queue: wgpu::Queue,
    cor: wgpu::TextureFormat,
    objeto: wgpu::RenderPipeline,
    chao: wgpu::RenderPipeline,
    sombra: wgpu::RenderPipeline,
    ecra: wgpu::RenderPipeline,
    /// O céu fotográfico atrás da peça.
    fundo: wgpu::RenderPipeline,
    /// Os pipelines do brilho — `None` onde a placa não desenha `Rgba16Float` com `4×` (ali não há
    /// cena-linear para ler, e o painel esconde as fileiras: [`Forward::tem_brilho`]).
    brilho: Option<crate::gpu_brilho::Brilho>,
    ecra_ub: wgpu::Buffer,
    g0_bgl: wgpu::BindGroupLayout,
    g0_sombra_bgl: wgpu::BindGroupLayout,
    g1_bgl: wgpu::BindGroupLayout,
    ecra_bgl: wgpu::BindGroupLayout,
    quadro: wgpu::Buffer,
    ceu: wgpu::Buffer,
    tabela: wgpu::TextureView,
    /// O atlas do céu fotográfico ([`Forward::sobe_ceu`]); `None` até ao 1.º céu — a ligação lê
    /// então um texel vazio, e a cena ignora a [`crate::Foto`].
    foto: Option<(wgpu::Texture, wgpu::TextureView)>,
    foto_vazia: wgpu::TextureView,
    /// O sol do céu subido (`None` = o céu não tem sol); a ligação lê então `sol_vazia`.
    sol: Option<texturas::SolGpu>,
    sol_vazia: wgpu::TextureView,
    triplanar: crate::gpu_triplanar::TexturasGpu,
    mapas_sombra: [wgpu::TextureView; sombra_impl::NIVEIS],
    compara: wgpu::Sampler,
    liso: wgpu::Sampler,
    cobertura: crate::gpu_cobertura::Cobertura,
    contacto: crate::gpu_contacto::Contacto,
    materiais: Option<(u32, wgpu::Texture, wgpu::TextureView)>,
    objetos: Option<(u64, wgpu::Buffer, wgpu::BindGroup)>,
    malhas: BTreeMap<u64, MalhaGpu>,
    alvos: Option<Alvos>,
    pipelines: usize,
    /// ⭐ As capturas de reflexo — `None` onde a placa não desenha `Rgba16Float` (como o brilho).
    sondas: Option<sondas_impl::Sondas>,
    /// O que o grupo `0` liga no lugar delas quando não há.
    sondas_vazia: wgpu::TextureView,
    /// Os gates ligam e desligam as capturas (a régua de controlo).
    reflexos: bool,
    /// Sobe a cada envio que muda o que as capturas veem (malhas, curvaturas, grelhas, céu, texturas).
    geracao: u64,
}

fn atributos() -> [wgpu::VertexAttribute; 4] {
    [
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 0,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 12,
            shader_location: 1,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32,
            offset: 24,
            shader_location: 2,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Uint32,
            offset: 28,
            shader_location: 3,
        },
    ]
}

const CURVATURAS: [wgpu::VertexAttribute; 1] = [wgpu::VertexAttribute {
    format: wgpu::VertexFormat::Float32x2,
    offset: 0,
    shader_location: 4,
}];

impl Forward {
    /// ⭐ Cria o desenhista NO aparelho do celular ([`crate::gpu_alvo::aparelho_em`]).
    #[must_use]
    pub fn no_aparelho(ambiente: &Ambiente<'_>) -> Option<Self> {
        Self::no_backend(wgpu::Backends::all(), ambiente)
    }

    /// O mesmo, num backend escolhido.
    #[must_use]
    pub fn no_backend(backends: wgpu::Backends, ambiente: &Ambiente<'_>) -> Option<Self> {
        let (device, queue, adapter) = crate::gpu_alvo::aparelho_em(backends)?;
        let cor = crate::gpu_alvo::formato_da_cor(&adapter);
        Some(Self::new(device, queue, cor, ambiente))
    }

    /// O formato da cor do quadro (`Rgba16Float`, ou `Rgba8UnormSrgb` onde não houver).
    #[must_use]
    pub fn formato(&self) -> wgpu::TextureFormat {
        self.cor
    }

    /// ⭐ **Quantos pipelines este desenhista já compilou** — fixo desde que nasce: `10` (objeto,
    /// chão, fundo, sombra, cobertura de cima e de baixo, redução, céu do chão e o borrão dele,
    /// codificação) e, onde a placa desenha `Rgba16Float`, `+5` do brilho (objeto, chão e fundo com
    /// a cena-linear, descer, subir) e `+4` das capturas de reflexo (faces, octaedro, cadeia,
    /// pré-filtro). Ligar, desligar ou mexer no brilho, trocar de céu, ou refazer capturas, não
    /// compila nada.
    #[must_use]
    pub fn pipelines_compilados(&self) -> usize {
        self.pipelines
    }

    /// ⭐ **Esta placa desenha o brilho?** — só com a cena-linear em `Rgba16Float` (ver
    /// [`crate::gpu_alvo::formato_da_cor`]); sem ela o brilho não existe aqui, e quem pinta o painel
    /// não oferece as fileiras dele.
    #[must_use]
    pub fn tem_brilho(&self) -> bool {
        self.brilho.is_some()
    }

    /// ⭐ **Há um céu fotográfico subido?** — sem ele a [`crate::Foto`] da cena é ignorada.
    #[must_use]
    pub fn tem_ceu(&self) -> bool {
        self.foto.is_some()
    }

    #[must_use]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        cor: wgpu::TextureFormat,
        ambiente: &Ambiente<'_>,
    ) -> Self {
        let vf = wgpu::ShaderStages::VERTEX_FRAGMENT;
        let g0_bgl = crate::gpu_ligacoes::g0(&device);
        // ⚠️ O passe de sombra ESCREVE o mapa: não o pode ter ligado para leitura. Só o quadro.
        let g0_sombra_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward g0 sombra"),
            entries: &[uniforme(0, false, vf)],
        });
        let g1_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward g1"),
            entries: &[uniforme(0, true, vf)],
        });
        let ecra_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward ecra"),
            entries: &[
                textura_float(0),
                textura_float(1),
                uniforme(2, false, wgpu::ShaderStages::FRAGMENT),
            ],
        });

        let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh-forward"),
            source: wgpu::ShaderSource::Wgsl(crate::fonte(ambiente).into()),
        });
        let modulo_ecra = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh-forward ecra"),
            source: wgpu::ShaderSource::Wgsl(crate::fonte::ecra().into()),
        });
        let layout = |bgls: &[&wgpu::BindGroupLayout]| {
            let v: Vec<Option<&wgpu::BindGroupLayout>> = bgls.iter().copied().map(Some).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-mesh-forward layout"),
                bind_group_layouts: &v,
                immediate_size: 0,
            })
        };
        let atr = atributos();
        let vertice = [
            wgpu::VertexBufferLayout {
                array_stride: VERTICE,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &atr,
            },
            wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &CURVATURAS,
            },
        ];
        let so_posicao = [wgpu::VertexBufferLayout {
            array_stride: VERTICE,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &atr[..1],
        }];
        let prim = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        };
        let msaa = wgpu::MultisampleState {
            count: crate::MSAA,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };
        let profundidade = |escreve: bool, compara: wgpu::CompareFunction, bias| {
            Some(wgpu::DepthStencilState {
                format: PROFUNDIDADE,
                depth_write_enabled: Some(escreve),
                depth_compare: Some(compara),
                stencil: wgpu::StencilState::default(),
                bias,
            })
        };
        let alvo = |blend| {
            [Some(wgpu::ColorTargetState {
                format: cor,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })]
        };
        let opaco = alvo(None);
        let sobre = alvo(Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING));
        let pl_g0g1 = layout(&[&g0_bgl, &g1_bgl]);
        let pl_g0 = layout(&[&g0_bgl]);
        let pl_sombra = layout(&[&g0_sombra_bgl, &g1_bgl]);
        let pl_ecra = layout(&[&ecra_bgl]);
        let opts = wgpu::PipelineCompilationOptions::default;

        let objeto_d = wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-mesh-forward objeto"),
            layout: Some(&pl_g0g1),
            vertex: wgpu::VertexState {
                module: &modulo,
                entry_point: Some("vs_objeto"),
                compilation_options: opts(),
                buffers: &vertice,
            },
            fragment: Some(wgpu::FragmentState {
                module: &modulo,
                entry_point: Some("fs_objeto"),
                compilation_options: opts(),
                targets: &opaco,
            }),
            primitive: prim,
            depth_stencil: profundidade(
                true,
                wgpu::CompareFunction::Less,
                wgpu::DepthBiasState::default(),
            ),
            multisample: msaa,
            multiview_mask: None,
            cache: None,
        };
        let objeto = device.create_render_pipeline(&objeto_d);
        // ⚠️ O chão vem PRIMEIRO, sem escrever profundidade: ele só existe onde nenhum objeto está
        // (os objetos pintam por cima, opacos) — uma peça abaixo do chão continua inteira.
        let chao_d = wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-mesh-forward chao"),
            layout: Some(&pl_g0),
            vertex: wgpu::VertexState {
                module: &modulo,
                entry_point: Some("vs_chao"),
                compilation_options: opts(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &modulo,
                entry_point: Some("fs_chao"),
                compilation_options: opts(),
                targets: &sobre,
            }),
            primitive: prim,
            depth_stencil: profundidade(
                false,
                wgpu::CompareFunction::Always,
                wgpu::DepthBiasState::default(),
            ),
            multisample: msaa,
            multiview_mask: None,
            cache: None,
        };
        let chao = device.create_render_pipeline(&chao_d);
        // O fundo vem antes do chão, opaco, sem profundidade (está no infinito).
        let fundo_d = wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-mesh-forward fundo"),
            vertex: wgpu::VertexState {
                module: &modulo,
                entry_point: Some("vs_fundo"),
                compilation_options: opts(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &modulo,
                entry_point: Some("fs_fundo"),
                compilation_options: opts(),
                targets: &opaco,
            }),
            ..chao_d.clone()
        };
        let fundo = device.create_render_pipeline(&fundo_d);
        let brilho = (cor == crate::gpu_brilho::LINEAR)
            .then(|| crate::gpu_brilho::Brilho::novo(&device, cor, &objeto_d, &chao_d, &fundo_d));
        let sombra = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-mesh-forward sombra"),
            layout: Some(&pl_sombra),
            vertex: wgpu::VertexState {
                module: &modulo,
                entry_point: Some("vs_sombra"),
                compilation_options: opts(),
                buffers: &so_posicao,
            },
            fragment: None,
            // ⭐ O mapa guarda as faces de TRÁS (as que não olham a luz): o raio SAI do sólido aí, e é
            // essa a distância que dá a penumbra junto do contacto (as malhas do campo são fechadas e
            // CCW para fora). Na projecção da luz as faces que a olham saem CW, logo `Back` descarta-as.
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..prim
            },
            // ⚠️ SEM viés na placa: ele existia contra a «acne» das faces que olham a luz, que este mapa
            // já não guarda — e o viés por inclinação empurrava a face de trás de uma caixa (rasante
            // ao sol) para lá do chão, que saía aceso logo atrás dela (medido no oráculo, 03/10).
            depth_stencil: profundidade(
                true,
                wgpu::CompareFunction::Less,
                wgpu::DepthBiasState::default(),
            ),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let ecra = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-mesh-forward ecra"),
            layout: Some(&pl_ecra),
            vertex: wgpu::VertexState {
                module: &modulo_ecra,
                entry_point: Some("vs_ecra"),
                compilation_options: opts(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &modulo_ecra,
                entry_point: Some("fs_ecra"),
                compilation_options: opts(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: SAIDA,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: prim,
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let quadro = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh-forward quadro"),
            size: (QUADRO * 4) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut ceu_dados = [0.0f32; 16];
        for (d, s) in ceu_dados.iter_mut().zip(ambiente.constantes) {
            *d = *s;
        }
        let ceu = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh-forward ceu"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&ceu, 0, bytemuck::cast_slice(&ceu_dados));
        let tabela = texturas::textura_de_floats(&device, &queue, ambiente.tabela);
        let foto_vazia = texturas::vazia(&device, &queue);
        let triplanar = crate::gpu_triplanar::TexturasGpu::vazias(&device);
        let sol_vazia = texturas::textura_de_floats(&device, &queue, &[0.0]);
        let mapas_sombra = sombra_impl::mapas(&device);
        let compara = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-mesh-forward compara"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let liso = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-mesh-forward liso"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let ecra_ub = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh-forward ecra"),
            size: crate::gpu_brilho::ECRA,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sondas = brilho
            .is_some()
            .then(|| sondas_impl::Sondas::nova(&device, &queue, &modulo, &pl_g0g1, &vertice));
        let pipelines = if brilho.is_some() { 19 } else { 10 };
        let sondas_vazia = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-mesh-forward sondas vazias"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 2,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: sondas_impl::FORMATO,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
        let contacto = crate::gpu_contacto::Contacto::novo(&device);
        let cobertura =
            crate::gpu_cobertura::Cobertura::nova(&device, &modulo, &pl_sombra, &so_posicao);
        Self {
            device,
            queue,
            cor,
            objeto,
            chao,
            sombra,
            ecra,
            fundo,
            brilho,
            ecra_ub,
            g0_bgl,
            g0_sombra_bgl,
            g1_bgl,
            ecra_bgl,
            quadro,
            ceu,
            tabela,
            foto: None,
            foto_vazia,
            sol: None,
            sol_vazia,
            triplanar,
            mapas_sombra,
            compara,
            liso,
            cobertura,
            contacto,
            materiais: None,
            objetos: None,
            malhas: BTreeMap::new(),
            alvos: None,
            pipelines,
            sondas,
            sondas_vazia,
            reflexos: true,
            geracao: 0,
        }
    }

    /// ⭐ **Sobe (ou substitui) a malha `id`.**
    pub fn sobe(&mut self, id: u64, m: &Malha<'_>) {
        self.geracao += 1;
        let mut bytes: Vec<u8> = Vec::with_capacity(m.posicoes.len() * VERTICE as usize);
        let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        for (i, p) in m.posicoes.iter().enumerate() {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
            bytes.extend_from_slice(bytemuck::cast_slice(p));
            bytes.extend_from_slice(bytemuck::cast_slice(&m.normais[i]));
            bytes.extend_from_slice(&m.ao[i].to_le_bytes());
            bytes.extend_from_slice(&m.material[i].to_le_bytes());
        }
        let buf = |label, conteudo: &[u8], usage| {
            let b = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: (conteudo.len() as u64).max(4).next_multiple_of(4),
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(&b, 0, conteudo);
            b
        };
        let vertices = buf(
            "ph2d-mesh-forward vertices",
            &bytes,
            wgpu::BufferUsages::VERTEX,
        );
        let indices = buf(
            "ph2d-mesh-forward indices",
            bytemuck::cast_slice(m.indices),
            wgpu::BufferUsages::INDEX,
        );
        let n_vertices = m.posicoes.len();
        let curvatura = buf(
            "ph2d-mesh-forward curvatura",
            &vec![0u8; n_vertices * 8],
            wgpu::BufferUsages::VERTEX,
        );
        // Outra malha no mesmo id: a grelha do contacto da antiga já não é a dela.
        self.contacto.esquece(id);
        self.cobertura.ceu.malha_mudou();
        self.malhas.insert(
            id,
            MalhaGpu {
                vertices,
                curvatura,
                n_vertices,
                indices,
                n: m.indices.len() as u32,
                caixa: (lo, hi),
            },
        );
    }

    /// ⭐ **As curvaturas da malha `id`** — `[material, estilo]` por vértice, `H` com sinal
    /// (`1/mundo`): a do MATERIAL ao passo de precisão (a subsuperfície maciça lê o módulo), a do
    /// ESTILO ao passo da suavidade. Nascem a zero com a malha; mudar a suavidade sobe SÓ isto. Um
    /// tamanho que não bate com os vértices é recusado (`false`).
    pub fn sobe_curvatura(&mut self, id: u64, k: &[[f32; 2]]) -> bool {
        let Some(m) = self.malhas.get(&id) else {
            return false;
        };
        if k.len() != m.n_vertices {
            return false;
        }
        self.geracao += 1;
        if !k.is_empty() {
            self.queue
                .write_buffer(&m.curvatura, 0, bytemuck::cast_slice(k));
        }
        true
    }

    /// Esquece a malha `id`.
    /// ⭐ **Sobe a grelha do contacto da malha `id`** ([`ph2d_contacto::Grade`], no referencial
    /// da malha): o céu que ela tapa às outras peças. Uma malha sem grelha não tapa ninguém.
    pub fn sobe_contacto(&mut self, id: u64, g: &ph2d_contacto::Grade) {
        self.geracao += 1;
        (self.contacto).sobe(&self.device, &self.queue, id, g);
    }

    pub fn esquece(&mut self, id: u64) {
        self.geracao += 1;
        self.contacto.esquece(id);
        self.cobertura.ceu.malha_mudou();
        self.malhas.remove(&id);
    }

    /// As malhas subidas.
    #[must_use]
    pub fn malhas(&self) -> usize {
        self.malhas.len()
    }

    /// ⭐⭐ **O QUADRO** — RGBA8 por pixel, no formato do `premultiplicado::para_ecra` do Render
    /// (sRGB da cor des-premultiplicada, vezes o alfa). `None` quando o tamanho é nulo.
    #[must_use]
    pub fn quadro(&mut self, cena: &Cena<'_>) -> Option<Vec<u8>> {
        let (w, h) = cena.tamanho;
        if w == 0 || h == 0 {
            return None;
        }
        if self.alvos.as_ref().is_none_or(|a| a.tamanho != (w, h)) {
            self.alvos = Some(Alvos::novos(
                &self.device,
                (w, h),
                self.cor,
                &self.ecra_bgl,
                &self.ecra_ub,
            ));
        }
        let brilho =
            self.prepara_brilho(&cena.brilho.sanitized(), (w, h), cena.exposicao, cena.vista);
        let mat_dados = self.sobe_materiais(cena.materiais, cena.texturas);
        let chave = sombra_impl::chave(cena, self.foto.is_some(), self.sol.as_ref());
        let enquadra = sombra_impl::enquadra(
            cena,
            chave,
            |id| self.malhas.get(&id).map(|m| m.caixa),
            true,
        );
        let chao = cena.chao.unwrap_or(0.0);
        let (ceu, vp, objs) = (enquadra.ceu, &enquadra.ceu_vp, cena.objetos);
        (self.cobertura.ceu).prepara(&self.queue, ceu, vp, chao, objs);
        let dados = quadro_impl::uniforme_do_quadro(
            cena,
            &enquadra,
            self.foto.is_some(),
            self.sol.as_ref(),
            false,
        );
        self.queue
            .write_buffer(&self.quadro, 0, bytemuck::cast_slice(&dados));
        let visiveis: Vec<&crate::Instancia> = cena
            .objetos
            .iter()
            .filter(|o| self.malhas.contains_key(&o.malha))
            .collect();
        let atrib = self.atribui_sondas(&visiveis);
        let obj_dados = self.sobe_objetos(&visiveis, &atrib);
        let plano = self.planeia_sondas(cena, &visiveis, &atrib, (&obj_dados, &mat_dados));
        self.contacto.prepara(&self.queue, &visiveis);
        self.desenha(
            cena,
            &visiveis,
            (enquadra.ha_sombra, enquadra.ha_chao),
            brilho,
            plano.as_ref(),
        );
        self.le((w, h))
    }
}
