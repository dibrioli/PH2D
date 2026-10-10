//! ⭐ **A captura FINA dos opacos** (W5a, F1 + F2 da fila do dono) — uma 2.ª C0, só de cobertura e τ
//! dentro, na grade da captura ([`Config::grade_da_captura`]): o que tapa a luz (oclusores e corpos
//! que brilham — as lâmpadas são luz, não corpo) e os sprites por texel (a camada do vetor incluída),
//! com a MESMA lei do dono e os MESMOS shaders da C0. A luz direta (C3) marcha nela: a captura das
//! cascatas arredondava o canto do oclusor em ½–¾ do texel grosso e perdia o oclusor mais fino que
//! ~1 texel. Módulo-filho do [`super`].

use super::*;
use crate::captura::capturar_opacos;
use crate::config::Grade;
use gravar::subir_geometria;

pub(super) struct CapturaFina {
    uni: wgpu::Buffer,
    /// `[cobertura, τ dentro]` da captura fina.
    alvos: [wgpu::TextureView; 2],
    pipeline: wgpu::RenderPipeline,
    formatos: solver_sprite::FormatosC0,
    geometria: Option<Geometria>,
    sprites: Option<solver_sprite::C0Sprites>,
}

impl CapturaFina {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn novo(
        device: &wgpu::Device,
        modulo: &wgpu::ShaderModule,
        layout: &wgpu::PipelineLayout,
        uni: wgpu::Buffer,
        alvos: [wgpu::TextureView; 2],
        formatos: solver_sprite::FormatosC0,
        g: Grade,
        fila: &wgpu::Queue,
    ) -> Self {
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("luz2d c0 captura fina"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: modulo,
                entry_point: Some("vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: modulo,
                entry_point: Some("fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &solver_sprite::alvos_c0_opacos(formatos),
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
        fila.write_buffer(
            &uni,
            0,
            bytemuck::bytes_of(&UniformeC0 {
                tamanho: [g.largura as f32, g.altura as f32],
                _pad: [0.0; 2],
            }),
        );
        Self {
            uni,
            alvos,
            pipeline,
            formatos,
            geometria: None,
            sprites: None,
        }
    }

    /// Captura `cena` e `sprites` na grade `g` (a da captura). Devolve os bytes de geometria que
    /// subiram.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn gravar(
        &mut self,
        gpu: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        c0_layout: &wgpu::BindGroupLayout,
        cena: &CenaDeLuz,
        sprites: &CapturaDeSprites<'_>,
        g: Grade,
        tempos: Option<&Tempos>,
    ) -> u64 {
        let cap = capturar_opacos(cena, &g, &sprites.substituidos);
        subir_geometria(
            gpu,
            (c0_layout, &self.uni),
            &mut self.geometria,
            ["luz2d/buf/formas_fina", "luz2d/buf/vertices_fina"],
            &cap.formas,
            &cap.vertices,
        );
        let (formas_sprite, lotes) = captura_sprite::na_grade(sprites, &g);
        let lotes = if formas_sprite.is_empty() {
            Vec::new()
        } else {
            let formatos = self.formatos;
            self.sprites
                .get_or_insert_with(|| {
                    solver_sprite::C0Sprites::novo(
                        &gpu.device,
                        &solver_sprite::alvos_c0_opacos(formatos),
                        "luz2d/buf/sprites_fina",
                    )
                })
                .preparar(gpu, &self.uni, &formas_sprite, &lotes, &sprites.texturas)
        };
        let alvo = |v, cor| {
            Some(wgpu::RenderPassColorAttachment {
                view: v,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(cor),
                    store: wgpu::StoreOp::Store,
                },
            })
        };
        let mut p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("luz2d C0 captura fina"),
            color_attachments: &[
                None,
                alvo(&self.alvos[0], wgpu::Color::TRANSPARENT),
                alvo(&self.alvos[1], wgpu::Color::WHITE),
                None,
            ],
            depth_stencil_attachment: None,
            timestamp_writes: tempos.map(|t| t.escrita_render(tempos::C0_FINA)),
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let total = u32::try_from(cap.formas.len()).expect("formas cabem em u32");
        if total > 0 {
            let geo = self.geometria.as_ref().expect("subida acima");
            p.set_pipeline(&self.pipeline);
            p.set_bind_group(0, &geo.grupo, &[]);
            p.draw(0..4, 0..total);
        }
        if let Some(s) = &self.sprites {
            s.desenhar(&mut p, &lotes);
        }
        cap.relatorio.bytes_da_geometria
    }

    pub(super) fn bytes_residentes(&self) -> u64 {
        self.geometria.as_ref().map_or(0, |g| {
            (g.cap_formas * size_of::<Forma>() + g.cap_vertices * size_of::<[f32; 2]>()) as u64
        }) + self
            .sprites
            .as_ref()
            .map_or(0, solver_sprite::C0Sprites::bytes_residentes)
    }
}
