//! ⭐ **O rebote na placa** (W5c, a lei do plano §2.4) — o passe depois da C0: a emissão de cada
//! texel de um corpo opaco que reflete ganha `min(albedo, A_MAX) × S`, a luz que a superfície recebeu
//! no quadro ANTERIOR (a fonte, escrita pela `c3_rebote` depois das cascatas). Um rebote por quadro.
//! A fonte lê-se no MUNDO: a grade anda (pan, oitava) e o texel de hoje acha-a onde ela estava.
//! Módulo-filho do [`super`].

use super::*;
use crate::config::Grade;

/// O espelho de `Rebote` do `c0_rebote.wgsl` (32 B).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformeRebote {
    escala: [f32; 2],
    desloc: [f32; 2],
    anterior: [f32; 2],
    valida: f32,
    albedo_max: f32,
}

pub(super) struct SomaDoRebote {
    /// O 4.º alvo da C0.
    pub(super) alvo_albedo: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    grupo: wgpu::BindGroup,
    uniforme: wgpu::Buffer,
    /// A grade em que a fonte foi escrita (a do quadro anterior); `None` antes do 1.º.
    pub(super) anterior: Option<Grade>,
    fonte: wgpu::Texture,
    /// ⭐ W5d: o 2.º rebote do quadro — a fonte lida na MESMA grade (o uniforme e o grupo dela).
    na_mesma: Option<wgpu::BindGroup>,
}

impl Solver {
    /// ⭐ W5c: a FONTE do rebote do último quadro (rgb = a luz que a superfície recebe, `F/2π`;
    /// a = 1 onde há fonte), na grade das cascatas — para quem precisa de a medir.
    #[must_use]
    pub fn fonte_do_rebote(&self) -> Option<&wgpu::Texture> {
        self.rebote.as_ref().map(|r| &r.fonte)
    }
}

impl SomaDoRebote {
    pub(super) fn novo(
        device: crate::Placa<'_>,
        lista: &PassList,
        vistas: &[wgpu::TextureView],
        [albedo, fonte]: [usize; 2],
        (uniforme, textura_da_fonte): (wgpu::Buffer, wgpu::Texture),
        // ⭐ W5d: o uniforme do 2.º rebote (a mesma grade `g`) e a fila que o escreve.
        mesma: Option<(wgpu::Buffer, &Grade, &wgpu::Queue)>,
    ) -> Self {
        let textura = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: FLOAT,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luz2d c0 rebote"),
            entries: &[
                entrada_uniforme(0, wgpu::ShaderStages::FRAGMENT),
                textura(1),
                textura(2),
            ],
        });
        let grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luz2d c0 rebote"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforme.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&vistas[albedo]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&vistas[fonte]),
                },
            ],
        });
        let na_mesma = mesma.map(|(u, g, fila)| {
            fila.write_buffer(
                &u,
                0,
                bytemuck::bytes_of(&UniformeRebote {
                    escala: [1.0; 2],
                    desloc: [0.0; 2],
                    anterior: [g.largura as f32, g.altura as f32],
                    valida: 1.0,
                    albedo_max: ph2d_light2d_ecs::ALBEDO_MAX,
                }),
            );
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("luz2d c0 rebote (na mesma grade)"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: u.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&vistas[albedo]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&vistas[fonte]),
                    },
                ],
            })
        });
        let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("luz2d c0_rebote.wgsl"),
            source: wgpu::ShaderSource::Wgsl(shaders::C0_REBOTE.into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("luz2d c0 rebote"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        // Soma na cor; o alfa (1/alcance do emissor dono) fica.
        let soma = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("luz2d c0 rebote"),
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: lista.texturas[EMISSAO].formato,
                    blend: Some(wgpu::BlendState {
                        color: soma,
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::COLOR,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            alvo_albedo: vistas[albedo].clone(),
            pipeline,
            grupo,
            uniforme,
            anterior: None,
            fonte: textura_da_fonte,
            na_mesma,
        }
    }

    /// ⭐ W5d — o 2.º rebote do quadro: a soma na `emissao` com a fonte que a C3 acabou de escrever
    /// (na mesma grade); sem carimbos. Devolve se gravou.
    pub(super) fn gravar_na_mesma(
        &self,
        enc: &mut wgpu::CommandEncoder,
        emissao: &wgpu::TextureView,
    ) -> bool {
        let Some(grupo) = &self.na_mesma else {
            return false;
        };
        let mut p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("luz2d C0 rebote (2.º)"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: emissao,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        p.set_pipeline(&self.pipeline);
        p.set_bind_group(0, grupo, &[]);
        p.draw(0..3, 0..1);
        true
    }

    /// Grava a soma na `emissao` da grade `g` (com a fonte do quadro anterior, se houver).
    pub(super) fn gravar(
        &self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        g: &Grade,
        emissao: &wgpu::TextureView,
        tempos: Option<&Tempos>,
    ) {
        let u = self.anterior.map_or(
            UniformeRebote {
                escala: [1.0; 2],
                desloc: [0.0; 2],
                anterior: [0.0; 2],
                valida: 0.0,
                albedo_max: ph2d_light2d_ecs::ALBEDO_MAX,
            },
            |a| UniformeRebote {
                escala: [(g.texel_m / a.texel_m) as f32; 2],
                desloc: [0, 1].map(|k| ((g.origem_m[k] - a.origem_m[k]) / a.texel_m) as f32),
                anterior: [a.largura as f32, a.altura as f32],
                valida: 1.0,
                albedo_max: ph2d_light2d_ecs::ALBEDO_MAX,
            },
        );
        gpu.queue
            .write_buffer(&self.uniforme, 0, bytemuck::bytes_of(&u));
        let mut p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("luz2d C0 rebote"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: emissao,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: tempos.map(|t| t.escrita_render(tempos::C0_REBOTE)),
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if self.anterior.is_some() {
            p.set_pipeline(&self.pipeline);
            p.set_bind_group(0, &self.grupo, &[]);
            p.draw(0..3, 0..1);
        }
    }
}

/// Algum corpo da cena reflete (um oclusor ou um sprite — a camada do vetor incluída — com albedo)?
pub(super) fn ha_refletor(cena: &CenaDeLuz, sprites: &CapturaDeSprites<'_>) -> bool {
    let com_albedo = |a: &[f32; 3]| a.iter().any(|&x| x > 0.0);
    cena.oclusores.iter().any(|o| com_albedo(&o.albedo))
        || sprites.sprites.iter().any(|s| com_albedo(&s.albedo))
}

/// Sem o rebote, um passe vazio escreve os carimbos dele (valem 0; nenhum fica por escrever).
pub(super) fn carimbos_vazios(enc: &mut wgpu::CommandEncoder, tempos: Option<&Tempos>, par: u32) {
    if let Some(t) = tempos {
        enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("luz2d rebote (vazio)"),
            timestamp_writes: Some(t.escrita_compute(par)),
        });
    }
}
