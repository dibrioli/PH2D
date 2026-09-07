//! ⭐⭐⭐ **`FrostPass` — o VIDRO JATEADO por trás da receita aberta** (Enio, 2026-09-07: *«o canvas
//! deve ser borrado levemente assim como todos os objetos nele, e o Prefab aparece no centro do
//! canvas acima de tudo, livre do blur»* / *«crie a feature de borrar discretamente o que está por
//! trás do prefab como um vidro jateado»*).
//!
//! # O que ele borra, e porque não é a tela
//!
//! Ele borra **uma textura**, e quem lhe entrega o acumulador do mundo
//! ([`crate::world_rt::WorldRt`]) é o presente. Isso é a metade que faz a feature ser exprimível:
//! naquele acumulador estão o fundo do canvas, as sprites e o documento vectorial — e **não** estão
//! os painéis, que vivem na cena de chrome e são compostos DEPOIS. ⇒ *«borrar o canvas e tudo o que
//! está nele, e mais nada»* não precisa de máscara nenhuma: é o conteúdo daquela textura.
//!
//! ⛔ **A tentação era borrar o resultado final dentro de um rectângulo.** Ela custa uma máscara
//! que tem de conhecer o layout, e erra em toda janela flutuante que passe por cima do canvas.
//!
//! # A cadeia
//!
//! ```text
//! mundo (tela cheia) ─down→ a (½) ─blur H→ b (½) ─blur V→ a (½) ─up+véu→ mundo
//! ```
//!
//! Quatro passagens, três delas a um quarto dos pixels. ⚠️ **Ler e escrever a MESMA textura é
//! ilegal numa passagem** — é essa a razão de haver duas texturas de meia resolução e de o
//! resultado só voltar ao mundo no fim, e não uma preferência de estilo.
//!
//! ⚠️ **Ele NÃO desenha a receita.** O que fica por cima do vidro é responsabilidade de quem
//! chama: a receita entra depois, pelos motores dela (o vector pelo Vello, as peças raster pelo
//! passe de sprites), e é isso que a deixa **nítida acima de tudo**.

use ph2d_gpu::GpuContext;

/// **A força do véu** — quanto o mundo recua por trás do vidro.
///
/// ⚠️ Não é um teto de recurso: é o degrau de leitura entre *«desfocado»* e *«atrás do vidro»*. As
/// duas falhas que ele evita têm nome — a `0,0` deixa um fundo de contraste baixo apenas mole (a
/// receita não ganha frente), e a `1,0` apaga a cena (o artista perde a referência de onde a
/// receita está no mundo, que é metade do que o modo serve).
///
/// ⚠️ Ele herda o valor da camada que esta feature usava antes do vidro existir
/// (`ISOLATION_BACKDROP_ALPHA`, 2026-09-07), e por isso o recuo do mundo **não muda de força** — o
/// que muda é ele passar a ser um vidro em vez de uma cortina.
pub const FROST_VEIL_ALPHA: f64 = 0.25;

/// O divisor da resolução de trabalho. Ver o cabeçalho do WGSL: ele **duplica o alcance** do mesmo
/// kernel de 5 taps, e o quarto do custo é consequência.
const SHRINK: u32 = 2;

/// As duas texturas de meia resolução e as vistas delas.
struct Half {
    a: wgpu::TextureView,
    b: wgpu::TextureView,
    size: (u32, u32),
}

pub struct FrostPass {
    down: wgpu::RenderPipeline,
    blur: wgpu::RenderPipeline,
    up: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// Um uniforme por passagem — `down`, `blur H`, `blur V`, `up`.
    ///
    /// ⚠️ **Quatro buffers e não um**: as quatro passagens são gravadas no MESMO encoder e
    /// submetidas juntas, então um `write_buffer` entre elas não as separa — todas leriam o último
    /// valor escrito. *Um uniforme partilhado por passagens de um só `submit` é uma corrida
    /// escrita à mão.*
    ubo: [wgpu::Buffer; 4],
    half: Option<Half>,
    format: wgpu::TextureFormat,
}

/// O que o WGSL lê. `repr(C)` + `Pod` porque atravessa a fronteira.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    step: [f32; 2],
    _pad: [f32; 2],
    veil: [f32; 4],
}

impl FrostPass {
    /// `format` é o da textura que ele borra — as intermediárias nascem no mesmo, senão a última
    /// passagem quantizaria numa convenção e escreveria noutra.
    pub fn new(gpu: &GpuContext, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-render frost bgl"),
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
            label: Some("ph2d-render frost layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-render frost shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/frost.wgsl").into()),
        });
        let pipe = |entry: &str, label: &str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        // ⚠️ **Sem mistura, de propósito**: cada passagem SUBSTITUI o alvo. Um
                        // `over` aqui somaria o borrão sobre o que já lá está, e o mundo apareceria
                        // duas vezes — nítido por baixo do próprio borrão.
                        blend: None,
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
            })
        };
        let down = pipe("fs_down", "ph2d-render frost down");
        let blur = pipe("fs_blur", "ph2d-render frost blur");
        let up = pipe("fs_up", "ph2d-render frost up");
        // ⚠️ **Linear e `ClampToEdge`**: o kernel de 5 taps DEPENDE da interpolação bilinear (dois
        // vizinhos por leitura), e o clamp é o que impede a borda da tela de amostrar o outro lado.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render frost sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let ubo = std::array::from_fn(|_| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-render frost uniform"),
                size: std::mem::size_of::<Uniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
        Self {
            down,
            blur,
            up,
            bgl,
            sampler,
            ubo,
            half: None,
            format,
        }
    }

    /// A resolução de trabalho para uma tela de `size`. **Nunca zero** — uma textura de lado zero é
    /// erro de validação, e o modo de falha seria a janela minimizada matar o app.
    #[must_use]
    pub fn work_size(size: (u32, u32)) -> (u32, u32) {
        ((size.0 / SHRINK).max(1), (size.1 / SHRINK).max(1))
    }

    /// Recria as intermediárias se a tela mudou de tamanho. No-op quando bate.
    pub fn ensure_size(&mut self, gpu: &GpuContext, size: (u32, u32)) {
        let want = Self::work_size(size);
        if self.half.as_ref().is_some_and(|h| h.size == want) {
            return;
        }
        let make = |label: &str| {
            gpu.device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: want.0,
                        height: want.1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };
        self.half = Some(Half {
            a: make("ph2d-render frost half A"),
            b: make("ph2d-render frost half B"),
            size: want,
        });
    }

    /// ⭐⭐⭐ **Borra `view` no lugar** — a textura entra nítida e sai jateada.
    ///
    /// `view` tem de ser uma vista **crua** (sem sRGB) da textura de `size`, e serve de fonte e de
    /// alvo; `veil_linear` é a cor do véu em **luz linear** com a força no alfa (a mesma convenção
    /// do [`crate::world_rt::WorldRt::clear_linear`] — quem chama tem a cor do token, não a
    /// codificada).
    pub fn run(
        &mut self,
        gpu: &GpuContext,
        view: &wgpu::TextureView,
        size: (u32, u32),
        veil_linear: wgpu::Color,
    ) {
        self.ensure_size(gpu, size);
        let Some(half) = self.half.as_ref() else {
            return;
        };
        let (hw, hh) = (half.size.0 as f32, half.size.1 as f32);
        let veil = [
            veil_linear.r as f32,
            veil_linear.g as f32,
            veil_linear.b as f32,
            veil_linear.a as f32,
        ];
        // O `down` amostra a tela CHEIA (o texel dela é o passo); os borrões andam no texel da
        // metade; o `up` não amostra vizinho nenhum e só carrega o véu.
        let uniforms = [
            Uniform {
                step: [1.0 / size.0.max(1) as f32, 1.0 / size.1.max(1) as f32],
                _pad: [0.0; 2],
                veil,
            },
            Uniform {
                step: [1.0 / hw, 0.0],
                _pad: [0.0; 2],
                veil,
            },
            Uniform {
                step: [0.0, 1.0 / hh],
                _pad: [0.0; 2],
                veil,
            },
            Uniform {
                step: [0.0, 0.0],
                _pad: [0.0; 2],
                veil,
            },
        ];
        for (buf, u) in self.ubo.iter().zip(uniforms.iter()) {
            gpu.queue.write_buffer(buf, 0, bytemuck::bytes_of(u));
        }
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render frost encoder"),
            });
        // ⚠️ **A ordem das quatro é a lei do módulo**, e a última é a única que toca no mundo.
        let steps: [(
            &wgpu::RenderPipeline,
            &wgpu::TextureView,
            &wgpu::TextureView,
            usize,
        ); 4] = [
            (&self.down, view, &half.a, 0),
            (&self.blur, &half.a, &half.b, 1),
            (&self.blur, &half.b, &half.a, 2),
            (&self.up, &half.a, view, 3),
        ];
        for (pipeline, src, dst, ubo) in steps {
            let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render frost bg"),
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
                        resource: self.ubo[ubo].as_entire_binding(),
                    },
                ],
            });
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-render frost pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: dst,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Cada passagem escreve o alvo inteiro — `Load` evita a limpeza que seria
                        // sobrescrita no mesmo instante.
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: ph2d_gpu::pass_profiler::render_writes("render.frost"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit(Some(enc.finish()));
    }
}

#[cfg(test)]
#[path = "frost_tests.rs"]
mod tests;
