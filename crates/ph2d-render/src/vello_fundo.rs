//! ⭐⭐⭐ **O MUNDO por baixo da cena do Vello** (doc 118 do Motion, W2 — ordem do dono: *«todas as
//! opções possíveis sem excluir nenhuma»*, e o alcance `Scene`/`Everything` pede o cenário).
//!
//! # O problema
//!
//! Uma camada de mistura do Vello mistura-se com o que a MESMA cena já pintou. As sprites desta casa
//! vivem noutra textura (o `game_rt`, tonemapeado — ou o acumulador `WorldRt` quando há faixas), e o
//! compositor só as junta à cena do Vello no fim, por um `over`. ⇒ um grupo em `Multiply` sobre uma
//! imagem do cenário multiplicava-se com o VAZIO e a imagem aparecia intacta por baixo dele.
//!
//! # A cura
//!
//! Quando a cena o pede ([`ph2d_vector::VectorScene::quer_o_mundo_por_baixo`]), o passe copia a
//! textura do mundo para uma textura `Rgba8Unorm` registada no Vello e desenha-a como a PRIMEIRA
//! coisa de uma cena composta — o mundo por baixo, a cena de sempre por cima. O intermédio sai
//! opaco, e o compositor, que faz `over` dele sobre o mundo, devolve-o tal e qual.
//!
//! ⚠️⚠️ **A cópia existe porque os formatos não casam:** o mundo é `Bgra8`, e o Vello só aceita
//! texturas `Rgba8Unorm` (ele copia-as para o atlas dele). O passe lê pela vista sRGB da fonte e
//! escreve pela vista sRGB do destino — descodifica e volta a codificar os MESMOS 8 bits, logo os
//! bytes do mundo chegam ao Vello como estavam (há gate a medi-lo contra o caminho de sempre).
//!
//! ⛔ **Sem nenhuma camada a pedi-lo, nada disto corre** — nem a cópia, nem a cena composta.

use ph2d_gpu::GpuContext;
use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::{ImageBrush, ImageData, ImageQuality};

/// O formato que o Vello aceita num `register_texture`.
const FORMATO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// A vista de ESCRITA do destino — sRGB, para re-codificar o que a fonte descodificou.
const FORMATO_DE_ESCRITA: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

const SHADER: &str = r"
@group(0) @binding(0) var fonte: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let dim = vec2<i32>(textureDimensions(fonte));
    let xy = min(vec2<i32>(p.xy), dim - vec2<i32>(1, 1));
    return textureLoad(fonte, xy, 0);
}
";

/// A cópia do mundo para o Vello, e o registo dela.
pub(crate) struct Fundo {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    alvo: Option<Alvo>,
    /// A cena composta — reaproveitada entre quadros (o `reset` do Vello guarda as alocações).
    pub(crate) composta: Scene,
}

struct Alvo {
    escrita: wgpu::TextureView,
    imagem: ImageData,
    tamanho: (u32, u32),
}

impl Fundo {
    pub(crate) fn new(gpu: &GpuContext) -> Self {
        let device = &gpu.device;
        let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-render vello fundo"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-render vello fundo layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-render vello fundo pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-render vello fundo pipeline"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &modulo,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &modulo,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: FORMATO_DE_ESCRITA,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            layout,
            pipeline,
            alvo: None,
            composta: Scene::new(),
        }
    }

    /// Copia `fonte` (uma vista **sRGB** do mundo) para a textura registada e devolve a imagem.
    ///
    /// ⚠️ **Re-regista só quando o tamanho muda**, e marca a imagem SUJA a cada quadro: desde a
    /// `vello` 0.10 o atlas é persistente, e sem a marca ele serviria os pixels do registo
    /// (ver [`crate::VelloPass::mark_texture_dirty`]).
    pub(crate) fn copia(
        &mut self,
        gpu: &GpuContext,
        renderer: &mut vello::Renderer,
        fonte: &wgpu::TextureView,
        tamanho: (u32, u32),
    ) -> ImageData {
        let tamanho = (tamanho.0.max(1), tamanho.1.max(1));
        if self.alvo.as_ref().is_none_or(|a| a.tamanho != tamanho) {
            if let Some(velho) = self.alvo.take() {
                renderer.unregister_texture(velho.imagem);
            }
            let textura = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-render vello fundo"),
                size: wgpu::Extent3d {
                    width: tamanho.0,
                    height: tamanho.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: FORMATO,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[FORMATO_DE_ESCRITA],
            });
            let escrita = textura.create_view(&wgpu::TextureViewDescriptor {
                format: Some(FORMATO_DE_ESCRITA),
                ..Default::default()
            });
            // ⚠️ O Vello guarda a textura que regista — ela vive enquanto o registo viver.
            let imagem = renderer.register_texture(textura);
            self.alvo = Some(Alvo {
                escrita,
                imagem,
                tamanho,
            });
        }
        let alvo = self.alvo.as_ref().expect("acabou de ser criado");
        let grupo = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render vello fundo grupo"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(fonte),
            }],
        });
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render vello fundo encoder"),
            });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-render vello fundo pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &alvo.escrita,
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
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &grupo, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit([enc.finish()]);
        renderer.mark_override_image_dirty(&alvo.imagem);
        alvo.imagem.clone()
    }
}

/// Monta a cena composta: o mundo, pixel a pixel, e a `cena` por cima.
///
/// ⚠️ **Com o afim IDENTIDADE o texel `(x, y)` cai no centro do pixel `(x, y)`**, e aí QUALQUER
/// filtro devolve o texel tal e qual — medido: a mutação que troca `Low` por `Medium` sobrevive ao
/// gate de byte. ⇒ `Low` é escolhido por ser a amostragem mais BARATA, e não por ser a única
/// exacta; *uma escolha que a régua não distingue não se afirma como lei*.
pub(crate) fn compoe(composta: &mut Scene, mundo: ImageData, cena: &Scene) {
    composta.reset();
    composta.draw_image(
        &ImageBrush::new(mundo).with_quality(ImageQuality::Low),
        Affine::IDENTITY,
    );
    composta.append(cena, None);
}

#[cfg(test)]
#[path = "vello_fundo_tests.rs"]
mod tests;
