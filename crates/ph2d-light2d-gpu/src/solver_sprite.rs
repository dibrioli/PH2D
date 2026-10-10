//! **A pipeline dos sprites na C0** (W3) — irmã da das formas, criada no primeiro quadro que tem
//! sprites (uma cena sem eles não aloca nada a mais). Desenha DENTRO do passe C0 das formas: os mesmos
//! alvos e a mesma mistura ([`alvos_c0`], a lei única das duas pipelines).

use std::ops::Range;

use ph2d_gpu::GpuContext;

use crate::captura_sprite::{Amostragem, FormaSprite};
use crate::shaders;

/// Os formatos dos alvos da C0: emissão, cobertura, τ dentro e (⭐ W5c, com o rebote) o albedo.
pub(crate) type FormatosC0 = [Option<wgpu::TextureFormat>; 4];

/// Os alvos da C0 e a MISTURA de cada um — a lei única das duas pipelines: emissão do dono pelo
/// maior (um só, no caso comum), cobertura pela soma, τ de dentro pelo produto, albedo pelo maior.
pub(crate) fn alvos_c0(formatos: FormatosC0) -> [Option<wgpu::ColorTargetState>; 4] {
    let soma = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    };
    let maior = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Max,
    };
    let multiplica = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero,
        dst_factor: wgpu::BlendFactor::Src,
        operation: wgpu::BlendOperation::Add,
    };
    let alvo = |formato: Option<wgpu::TextureFormat>, b: wgpu::BlendComponent| {
        formato.map(|format| wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState { color: b, alpha: b }),
            write_mask: wgpu::ColorWrites::ALL,
        })
    };
    [
        alvo(formatos[0], maior),
        alvo(formatos[1], soma),
        alvo(formatos[2], multiplica),
        alvo(formatos[3], maior),
    ]
}

/// ⭐ Os alvos da captura FINA dos opacos (W5a): os MESMOS da C0 sem a emissão (a saída do fragmento
/// para ela é descartada).
pub(crate) fn alvos_c0_opacos(formatos: FormatosC0) -> [Option<wgpu::ColorTargetState>; 4] {
    let [_, cobertura, tau, _] = alvos_c0(formatos);
    [None, cobertura, tau, None]
}

struct Residente {
    buffer: wgpu::Buffer,
    cap: usize,
    grupo: wgpu::BindGroup,
}

pub(crate) struct C0Sprites {
    pipeline: wgpu::RenderPipeline,
    layout_formas: wgpu::BindGroupLayout,
    layout_textura: wgpu::BindGroupLayout,
    amostra: wgpu::Sampler,
    amostra_exata: wgpu::Sampler,
    residente: Option<Residente>,
    /// O rótulo do buffer das formas (o G5 conta a geometria por ele).
    rotulo_do_buffer: &'static str,
}

impl C0Sprites {
    pub(crate) fn novo(
        device: &wgpu::Device,
        alvos: &[Option<wgpu::ColorTargetState>; 4],
        rotulo_do_buffer: &'static str,
    ) -> Self {
        let layout_formas = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d c0 sprites"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let layout_textura = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d c0 sprites textura"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("luz2d c0_sprite.wgsl"),
            source: wgpu::ShaderSource::Wgsl(shaders::C0_SPRITE.into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("luz2d c0 sprites"),
            bind_group_layouts: &[Some(&layout_formas), Some(&layout_textura)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("luz2d c0 sprites"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &modulo,
                entry_point: Some("vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &modulo,
                entry_point: Some("fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: alvos,
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        // As subamostras caem entre texels da textura: bilinear, preso à borda (o UV fora do quad já
        // é descartado pela conta, não pelo sampler).
        let amostra = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("luz2d c0 sprites amostra"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let amostra_exata = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("luz2d c0 sprites amostra exata"),
            ..Default::default()
        });
        Self {
            pipeline,
            layout_formas,
            layout_textura,
            amostra,
            amostra_exata,
            residente: None,
            rotulo_do_buffer,
        }
    }

    /// Sobe as formas (o buffer cresce por potências de dois, nunca encolhe) e devolve um grupo de
    /// textura por lote — tudo pronto ANTES do passe que os desenha.
    pub(crate) fn preparar(
        &mut self,
        gpu: &GpuContext,
        uni_c0: &wgpu::Buffer,
        formas: &[FormaSprite],
        lotes: &[(usize, Range<u32>)],
        texturas: &[(&wgpu::TextureView, Amostragem)],
    ) -> Vec<(wgpu::BindGroup, Range<u32>)> {
        if formas.is_empty() {
            return Vec::new();
        }
        let device = &gpu.device;
        if self.residente.as_ref().is_none_or(|r| r.cap < formas.len()) {
            let cap = formas.len().next_power_of_two();
            let buffer = crate::solver::criar_buffer(
                device,
                self.rotulo_do_buffer,
                (cap * size_of::<FormaSprite>()) as u64,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            );
            let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c0 sprites"),
                layout: &self.layout_formas,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uni_c0.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            });
            self.residente = Some(Residente { buffer, cap, grupo });
        }
        let r = self.residente.as_ref().expect("criado acima");
        gpu.queue
            .write_buffer(&r.buffer, 0, bytemuck::cast_slice(formas));
        lotes
            .iter()
            .filter_map(|(t, intervalo)| {
                let (vista, amostragem) = texturas.get(*t)?;
                let amostra = match amostragem {
                    Amostragem::Bilinear => &self.amostra,
                    Amostragem::Exata => &self.amostra_exata,
                };
                let g = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("luz2d c0 sprites textura"),
                    layout: &self.layout_textura,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(vista),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(amostra),
                        },
                    ],
                });
                Some((g, intervalo.clone()))
            })
            .collect()
    }

    /// Desenha os lotes preparados no passe C0 aberto.
    pub(crate) fn desenhar(
        &self,
        p: &mut wgpu::RenderPass<'_>,
        lotes: &[(wgpu::BindGroup, Range<u32>)],
    ) {
        let Some(r) = &self.residente else {
            return;
        };
        if lotes.is_empty() {
            return;
        }
        p.set_pipeline(&self.pipeline);
        p.set_bind_group(0, &r.grupo, &[]);
        for (g, intervalo) in lotes {
            p.set_bind_group(1, g, &[]);
            p.draw(0..4, intervalo.clone());
        }
    }

    pub(crate) fn bytes_residentes(&self) -> u64 {
        self.residente
            .as_ref()
            .map_or(0, |r| (r.cap * size_of::<FormaSprite>()) as u64)
    }
}
