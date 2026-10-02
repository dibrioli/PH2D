//! ⭐⭐ **O DESENHISTA NA PLACA** — pipelines compilados UMA vez (mover, mudar a cor ou acrescentar
//! um objeto não compila nada: gate `nada_compila_ao_editar`), malhas subidas por id, e o quadro em
//! quatro passes: sombra → chão + objetos (MSAA, resolvido) → codificação → leitura.

use std::collections::BTreeMap;

use crate::gpu_alvo::{Alvos, PROFUNDIDADE, SAIDA};
use crate::{Ambiente, Cena, Malha};

#[path = "gpu_quadro.rs"]
mod quadro_impl;

/// Os bytes de um vértice: posição, normal, AO, material.
const VERTICE: u64 = 32;
/// O alinhamento do deslocamento dinâmico de um uniforme (o mínimo que todo aparelho aceita).
pub(crate) const SLOT: u64 = 256;
/// O tamanho do uniforme do quadro — o `Quadro` do WGSL.
pub(crate) const QUADRO: usize = 2 * 16 + 6 * 4 + 2 * crate::MAX_LUZES * 4;

struct MalhaGpu {
    vertices: wgpu::Buffer,
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
    g0_bgl: wgpu::BindGroupLayout,
    g0_sombra_bgl: wgpu::BindGroupLayout,
    g1_bgl: wgpu::BindGroupLayout,
    ecra_bgl: wgpu::BindGroupLayout,
    quadro: wgpu::Buffer,
    ceu: wgpu::Buffer,
    tabela: wgpu::TextureView,
    mapa_sombra: wgpu::TextureView,
    compara: wgpu::Sampler,
    liso: wgpu::Sampler,
    cobertura: crate::gpu_cobertura::Cobertura,
    materiais: Option<(u32, wgpu::Texture, wgpu::TextureView)>,
    objetos: Option<(u64, wgpu::Buffer, wgpu::BindGroup)>,
    malhas: BTreeMap<u64, MalhaGpu>,
    alvos: Option<Alvos>,
    pipelines: usize,
}

fn uniforme(binding: u32, dinamico: bool, vis: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: vis,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: dinamico,
            min_binding_size: None,
        },
        count: None,
    }
}

fn textura_float(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
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

    /// ⭐ **Quantos pipelines este desenhista já compilou** — fica em `6` para sempre (objeto, chão,
    /// sombra, cobertura, redução, codificação).
    #[must_use]
    pub fn pipelines_compilados(&self) -> usize {
        self.pipelines
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
        let g0_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward g0"),
            entries: &[
                uniforme(0, false, vf),
                uniforme(1, false, wgpu::ShaderStages::FRAGMENT),
                textura_float(2),
                textura_float(3),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        // ⚠️ O passe de sombra ESCREVE o mapa: não o pode ter ligado para leitura. Só o quadro.
        let g0_sombra_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward g0 sombra"),
            entries: &[uniforme(0, false, vf)],
        });
        let g1_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward g1"),
            entries: &[uniforme(0, true, wgpu::ShaderStages::VERTEX)],
        });
        let ecra_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh-forward ecra"),
            entries: &[textura_float(0)],
        });

        let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh-forward"),
            source: wgpu::ShaderSource::Wgsl(crate::fonte(ambiente).into()),
        });
        let modulo_ecra = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh-forward ecra"),
            source: wgpu::ShaderSource::Wgsl(crate::fonte::ECRA.into()),
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
        let vertice = [wgpu::VertexBufferLayout {
            array_stride: VERTICE,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &atr,
        }];
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

        let objeto = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            depth_stencil: profundidade(true, wgpu::CompareFunction::Less, wgpu::DepthBiasState::default()),
            multisample: msaa,
            multiview_mask: None,
            cache: None,
        });
        // ⚠️ O chão vem PRIMEIRO, sem escrever profundidade: ele só existe onde nenhum objeto está
        // (os objetos pintam por cima, opacos) — uma peça abaixo do chão continua inteira.
        let chao = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            depth_stencil: profundidade(false, wgpu::CompareFunction::Always, wgpu::DepthBiasState::default()),
            multisample: msaa,
            multiview_mask: None,
            cache: None,
        });
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
            primitive: prim,
            depth_stencil: profundidade(
                true,
                wgpu::CompareFunction::Less,
                wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
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
        let tabela = textura_de_floats(&device, &queue, ambiente.tabela);
        let mapa_sombra = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-mesh-forward mapa de sombra"),
                size: wgpu::Extent3d {
                    width: crate::SOMBRA_LADO,
                    height: crate::SOMBRA_LADO,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: PROFUNDIDADE,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default());
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
        let cobertura = crate::gpu_cobertura::Cobertura::nova(&device, &modulo, &pl_sombra, &so_posicao);
        Self {
            device,
            queue,
            cor,
            objeto,
            chao,
            sombra,
            ecra,
            g0_bgl,
            g0_sombra_bgl,
            g1_bgl,
            ecra_bgl,
            quadro,
            ceu,
            tabela,
            mapa_sombra,
            compara,
            liso,
            cobertura,
            materiais: None,
            objetos: None,
            malhas: BTreeMap::new(),
            alvos: None,
            pipelines: 6,
        }
    }

    /// ⭐ **Sobe (ou substitui) a malha `id`.**
    pub fn sobe(&mut self, id: u64, m: &Malha<'_>) {
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
        let vertices = buf("ph2d-mesh-forward vertices", &bytes, wgpu::BufferUsages::VERTEX);
        let indices = buf(
            "ph2d-mesh-forward indices",
            bytemuck::cast_slice(m.indices),
            wgpu::BufferUsages::INDEX,
        );
        self.malhas.insert(
            id,
            MalhaGpu {
                vertices,
                indices,
                n: m.indices.len() as u32,
                caixa: (lo, hi),
            },
        );
    }

    /// Esquece a malha `id`.
    pub fn esquece(&mut self, id: u64) {
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
            self.alvos = Some(Alvos::novos(&self.device, (w, h), self.cor, &self.ecra_bgl));
        }
        self.sobe_materiais(cena.materiais);
        let enquadra = quadro_impl::enquadra_sombra(cena, |id| self.malhas.get(&id).map(|m| m.caixa));
        let dados = quadro_impl::uniforme_do_quadro(cena, &enquadra);
        self.queue.write_buffer(&self.quadro, 0, bytemuck::cast_slice(&dados));
        let visiveis: Vec<&crate::Instancia> =
            cena.objetos.iter().filter(|o| self.malhas.contains_key(&o.malha)).collect();
        self.sobe_objetos(&visiveis);
        self.desenha(cena, &visiveis, enquadra.ha_sombra);
        self.le((w, h))
    }
}

/// Uma lista de floats numa textura `R32Float` de [`crate::TAB_W`] colunas.
fn textura_de_floats(device: &wgpu::Device, queue: &wgpu::Queue, v: &[f32]) -> wgpu::TextureView {
    let w = crate::TAB_W;
    let h = (v.len() as u32).div_ceil(w).max(1);
    let mut dados = v.to_vec();
    dados.resize((w * h) as usize, 0.0);
    let t = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-mesh-forward tabela"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &t,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(&dados),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    t.create_view(&wgpu::TextureViewDescriptor::default())
}

