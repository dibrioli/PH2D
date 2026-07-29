//! T1.7 — composição por-camada do Flip reusando o **compositor 22-modos do
//! Painter** (`ph2d_render::layer_compositor::LayerCompositor`), sem nunca ler
//! pixel de volta pra CPU.
//!
//! ## O caminho
//!
//! O compositor trabalha em fatias `Rgba8Unorm` **straight sRGB8**; o Flip
//! rasteriza **premultiplicado, linear, em 16F** (o mesmo espaço do `game_rt`).
//! Esta struct é a ponte de espaço-de-cor, em duas passagens fullscreen 1:1:
//!
//! 1. **`stage_layer`** — rasteriza UMA camada (o desenho ativo dela) no scratch
//!    16F (premult-over + depth, via [`crate::FlipRenderer`]) e **resolve** pra
//!    uma textura `Rgba8Unorm` straight sRGB8 (`fs_resolve`: un-premultiply +
//!    encode). Essa textura é o que a orquestração injeta no compositor
//!    (`inject_slice_from_texture`, GPU→GPU).
//! 2. **`blit`** — pega a saída do compositor (straight sRGB8) e a compõe no
//!    `game_rt` 16F (`fs_blit`: decode + premultiply, blend premult-over).
//!
//! Os dois scratch (16F + 8-bit) são reusados entre as camadas do frame: a
//! ordem de submissão da fila garante que a cópia do `inject` (submissão *k*)
//! roda antes do próximo `stage_layer` sobrescrever o scratch (submissão *k+1*).

use crate::pack::FlipGpuData;
use crate::pipeline::{CameraRaw, FlipRenderer, no_msaa, premult_over, tri_list};

/// Formato das fatias/saída do compositor do Painter — o alvo do `fs_resolve` e
/// a origem do `fs_blit`. `inject_slice_from_texture` exige exatamente esta
/// família (`remove_srgb_suffix() == Rgba8Unorm`).
pub const SLICE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// As duas passagens de espaço-de-cor + os scratch reusados. Uma por (device,
/// formato do `game_rt`).
pub struct FlipCompose {
    /// `fs_resolve`: 16F premult → `Rgba8Unorm` straight (sem blend, replace).
    resolve_pipeline: wgpu::RenderPipeline,
    /// `fs_blit`: `Rgba8Unorm` straight → `game_rt` 16F (premult-over).
    blit_pipeline: wgpu::RenderPipeline,
    /// Uma textura `texture_2d<f32>` no binding 0 (serve resolve e blit).
    bgl: wgpu::BindGroupLayout,
    /// Scratch 16F onde cada camada é rasterizada (premult, linear).
    hdr: Option<wgpu::Texture>,
    hdr_view: Option<wgpu::TextureView>,
    /// Bind group que liga o `hdr_view` ao `fs_resolve` (recriado no resize).
    resolve_bg: Option<wgpu::BindGroup>,
    /// Textura straight sRGB8 da camada resolvida (origem do `inject`).
    straight: Option<wgpu::Texture>,
    straight_view: Option<wgpu::TextureView>,
    size: (u32, u32),
    /// O MOTOR NOVO ([doc 12](../../../docs/Flip/12_novo_motor_pesquisa.md)), quando armado.
    ///
    /// ⚠️ **Ele substitui o Pass A e SÓ ele.** O Pass B (premult → straight), o compositor
    /// 22-modos, o multiplano, o tint de fantasma e o `inject` não sabem que a tinta mudou de
    /// produtor — é isso que torna a troca uma decisão sobre *quais pixels o traço acende*, e não
    /// um segundo pipeline de Flip.
    walk: Option<crate::WalkPass>,
}

impl FlipCompose {
    /// Cria os pipelines. `game_rt_format` é o formato do alvo final do `blit`
    /// (o `game_rt`, `Rgba16Float`).
    #[must_use]
    pub fn new(device: &wgpu::Device, game_rt_format: wgpu::TextureFormat) -> Self {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-flip compose bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    // `textureLoad` (sem filtro) — `filterable:false` serve tanto o
                    // 16F (resolve) quanto o 8-bit (blit).
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-flip compose layout"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-flip compose shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/composite.wgsl").into()),
        });

        let resolve_pipeline = fullscreen_pipeline(
            device,
            &layout,
            &shader,
            "fs_resolve",
            SLICE_FORMAT,
            None, // replace: escreve a fatia straight inteira (rgb + cobertura)
        );
        let blit_pipeline = fullscreen_pipeline(
            device,
            &layout,
            &shader,
            "fs_blit",
            game_rt_format,
            Some(premult_over()),
        );

        Self {
            resolve_pipeline,
            blit_pipeline,
            bgl,
            hdr: None,
            hdr_view: None,
            resolve_bg: None,
            straight: None,
            straight_view: None,
            size: (0, 0),
            walk: None,
        }
    }

    /// Garante os scratch do tamanho `size` (recria no resize). O `resolve_bg`
    /// re-liga o novo `hdr_view`.
    fn ensure(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        let size = (size.0.max(1), size.1.max(1));
        if self.hdr.is_some() && self.size == size {
            return;
        }
        let extent = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let hdr = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-flip compose hdr scratch"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            // ⚠️ `STORAGE_BINDING` é o que permite o percurso do doc 12 escrever aqui; sem ele o
            // motor novo não tem alvo. Custo: nenhum para o rasterizador (a usage é aditiva).
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let hdr_view = hdr.create_view(&wgpu::TextureViewDescriptor::default());
        let straight = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-flip compose straight slice"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SLICE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let straight_view = straight.create_view(&wgpu::TextureViewDescriptor::default());
        let resolve_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-flip compose resolve bg"),
            layout: &self.bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&hdr_view),
            }],
        });
        self.hdr = Some(hdr);
        self.hdr_view = Some(hdr_view);
        self.straight = Some(straight);
        self.straight_view = Some(straight_view);
        self.resolve_bg = Some(resolve_bg);
        self.size = size;
    }

    /// Rasteriza UMA camada (`data` = o desenho ativo dela) no scratch 16F e a
    /// resolve pra straight sRGB8; devolve essa textura (pronta pro `inject`). O
    /// scratch é reusado no próximo `stage_layer` do frame — a cópia do `inject`
    /// que o chamador faz em seguida roda antes (ordem de submissão da fila).
    /// Arma (ou desarma) o motor novo. **Compila o kernel na 1ª chamada** e depois é idempotente.
    ///
    /// ⚠️ Quem decide é o SHELL (uma variável de ambiente hoje) — a crate não lê o ambiente, senão
    /// a escolha ficaria em dois lugares e o gate de um deles seria verde sobre o outro.
    pub fn set_walk_engine(&mut self, device: &wgpu::Device, on: bool) {
        if on {
            if self.walk.is_none() {
                self.walk = Some(crate::WalkPass::new(device));
            }
        } else {
            self.walk = None;
        }
    }

    /// O motor novo está armado?
    #[must_use]
    pub fn walk_engine_armed(&self) -> bool {
        self.walk.is_some()
    }

    pub fn stage_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        flip: &mut FlipRenderer,
        camera: &CameraRaw,
        data: &FlipGpuData,
        size: (u32, u32),
    ) -> &wgpu::Texture {
        self.ensure(device, size);
        flip.upload(device, queue, camera, data);
        flip.ensure_depth(device, size);

        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-flip stage layer"),
        });
        // Passe A: a camada no scratch 16F (premult, linear).
        //
        // ⚠️ **Dois produtores, UM alvo.** O rasterizador (`flip.wgsl` + o depth 2D que elege um
        // fragmento por pixel) e o **percurso por ladrilho** (doc 12) escrevem o MESMO `hdr`, e o
        // Passe B abaixo não sabe qual dos dois correu — é isso que mantém a troca honesta: o
        // motor novo é medido contra o antigo dentro do produto REAL, não num harness ao lado.
        if let Some(walk) = self.walk.as_ref() {
            let screen = crate::ScreenSpace::from_camera(camera);
            let bins = crate::bin_segments(data, &screen, crate::DEFAULT_TILE);
            let target = self.hdr_view.as_ref().expect("ensure criou o hdr");
            // ⚠️ O percurso escreve TODO pixel (o `acc` nasce zerado), então ele **não precisa** de
            // um clear — mas uma cena sem segmento nenhum retorna `None` do `prepare`, e aí o `hdr`
            // ficaria com a camada ANTERIOR. O clear cobre exatamente esse caso.
            match walk.prepare(device, data, &screen, &bins, target) {
                Some(job) => walk.record(&mut enc, &job),
                None => {
                    let _ = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("ph2d-flip stage: clear (walk, cena vazia)"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: target,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                }
            }
        } else {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-flip stage: rasterize layer"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.hdr_view.as_ref().expect("ensure criou o hdr"),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: flip.depth_view().map(|v| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: v,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0.0),
                            store: wgpu::StoreOp::Discard,
                        }),
                        stencil_ops: None,
                    }
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            flip.draw(&mut pass);
        }
        // Passe B: resolve 16F premult → straight sRGB8 (replace, sem depth).
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-flip stage: resolve to straight"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self
                        .straight_view
                        .as_ref()
                        .expect("ensure criou o straight"),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.resolve_pipeline);
            pass.set_bind_group(0, self.resolve_bg.as_ref().expect("ensure criou o bg"), &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([enc.finish()]);
        self.straight.as_ref().expect("ensure criou o straight")
    }

    /// Compõe a saída do compositor (`src`, straight sRGB8) no `game_rt` (`target`,
    /// 16F) com premult-over, preservando o que já está lá (`LoadOp::Load`).
    pub fn blit(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        src: &wgpu::Texture,
        target: &wgpu::TextureView,
    ) {
        let src_view = src.create_view(&wgpu::TextureViewDescriptor::default());
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-flip compose blit bg"),
            layout: &self.bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&src_view),
            }],
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-flip compose blit"),
        });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-flip compose blit pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // preserva sprites + camadas prévias
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.blit_pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([enc.finish()]);
    }
}

/// Um pipeline fullscreen (triângulo único) que amostra `binding 0` e escreve
/// `target_format`, com `blend` opcional (None = replace).
fn fullscreen_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    fs_entry: &str,
    target_format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ph2d-flip compose pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_fullscreen"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fs_entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: tri_list(),
        depth_stencil: None,
        multisample: no_msaa(),
        multiview_mask: None,
        cache: None,
    })
}
