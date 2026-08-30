//! `BandBlit` — cola **uma faixa** de desenho sobre o acumulador do mundo (ADR-0154 Fase 2).
//!
//! Ver [`crate::world_rt::WorldRt`] para por que o alvo é de formato cru e a mistura é de hardware.
//! Este passe é deliberadamente pequeno: um triângulo de tela cheia, uma textura, dois
//! interruptores de convenção.
//!
//! ⛔ **Ele não decide ordem nenhuma.** Quem sabe a ordem é o `draw_bands` do shell; aqui só se
//! executa uma colagem.

use ph2d_gpu::GpuContext;

/// De onde vem a faixa — e a convenção de cor de cada uma.
///
/// ⚠️ **É um enum e não dois `bool`s soltos**, porque as duas convenções andam sempre juntas e a
/// combinação errada não falha alto: ela dá uma borda escura ou uma cor lavada.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BandSource {
    /// A saída do tonemap: vista `Bgra8UnormSrgb` (a amostragem descodifica) e **pré-multiplicada**.
    Sprites,
    /// O intermediário do Vello: `Rgba8Unorm` (sem descodificação) e com alfa **directa**.
    Vector,
}

impl BandSource {
    /// `(decode_srgb, premultiplied)` — o par que o shader consome.
    #[must_use]
    pub fn flags(self) -> (u32, u32) {
        match self {
            BandSource::Sprites => (1, 1),
            BandSource::Vector => (0, 0),
        }
    }
}

pub struct BandBlit {
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: [wgpu::Buffer; 2],
}

impl BandBlit {
    pub fn new(gpu: &GpuContext, target_format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-render band blit bgl"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-render band blit layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-render band blit shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/band_blit.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-render band blit pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    // ⚠️ **`One / OneMinusSrcAlpha` — o `over` pré-multiplicado.** O shader entrega
                    // sempre pré-multiplicado, e o alvo é de formato CRU: ⇒ a mistura acontece
                    // sobre os bytes codificados, que É o espaço do desenhista.
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render band blit sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        // Um uniforme por fonte, construído uma vez: os dois valores são constantes.
        let uniforms = [
            uniform_for(device, BandSource::Sprites),
            uniform_for(device, BandSource::Vector),
        ];
        Self {
            pipeline,
            bgl,
            sampler,
            uniforms,
        }
    }

    /// Cola `src` sobre `target`, com a convenção de `source`.
    ///
    /// ⚠️ `LoadOp::Load` — o alvo é um ACUMULADOR. Um `Clear` aqui apagaria as faixas anteriores, e
    /// o sintoma seria «só a última coisa desenhada aparece».
    pub fn blit(
        &self,
        gpu: &GpuContext,
        target: &wgpu::TextureView,
        src: &wgpu::TextureView,
        source: BandSource,
    ) {
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render band blit bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(src),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.uniforms[source as usize].as_entire_binding(),
                },
            ],
        });
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render band blit encoder"),
            });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-render band blit pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: ph2d_gpu::pass_profiler::render_writes("render.band_blit"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit(Some(enc.finish()));
    }
}

fn uniform_for(device: &wgpu::Device, source: BandSource) -> wgpu::Buffer {
    use wgpu::util::DeviceExt as _;
    let (decode, premul) = source.flags();
    let data: [u32; 4] = [decode, premul, 0, 0];
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ph2d-render band blit flags"),
        contents: bytemuck::cast_slice(&data),
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **As duas convenções são OPOSTAS em ambos os eixos**, e trocá-las não falha alto — dá uma
    /// borda escura ou uma cor lavada. Este gate é o que torna a troca observável.
    #[test]
    fn the_two_sources_disagree_on_both_conventions() {
        assert_eq!(BandSource::Sprites.flags(), (1, 1));
        assert_eq!(BandSource::Vector.flags(), (0, 0));
    }

    /// O índice do uniforme é o discriminante — um enum reordenado trocaria os dois em silêncio.
    #[test]
    fn the_uniform_index_follows_the_discriminant() {
        assert_eq!(BandSource::Sprites as usize, 0);
        assert_eq!(BandSource::Vector as usize, 1);
    }
}
