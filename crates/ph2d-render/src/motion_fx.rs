//! `MotionFx` — the Motion module's own HDR glow pass (ADR: doc 67).
//!
//! ## Why a pass Motion owns, and not the frame's
//!
//! Motion instances are fused into the sprite pass with no origin tag
//! ([`sprite_collect`](crate::sprite_collect)), so "post-process only the
//! Motion output" cannot be done downstream — by the time the frame reaches the
//! tonemap, nobody knows which pixels were Motion. This pass takes the other
//! road: the shell re-renders the Motion instances **in isolation** into
//! [`rt_view`](MotionFx::rt_view) (via
//! [`render_instances_only`](crate::SpriteRenderer::render_instances_only)), the
//! glow is computed from THAT, and only the glow is added back over the scene.
//! Blast radius is zero — the fused sprite+Motion pass and the tonemap are
//! untouched, so a frame with the effect off is byte-identical to today.
//!
//! ## Why HDR (the whole reason this is Motion's pass and not the compositor's)
//!
//! Every target here is `Rgba16Float`. Bloom lives on the values **above 1.0**:
//! a spark tinted `(6, 4, 2)` glows harder than one tinted white. The Painter's
//! 8-bit compositor would clip those to white on the way in — which is why
//! routing Motion glow through it was rejected (doc 66). The tonemap downstream
//! still clamps the summed result, so the brightest cores read as white and the
//! halo falls off through the mid-tones — a real bloom.
//!
//! ## The chain — Call of Duty / Jimenez mip bloom (round, not square)
//!
//! A single wide box blur keeps the SQUARE of the source quad. This is the
//! technique Unity/Unreal ship (SIGGRAPH 2014; reference: LearnOpenGL "Physically
//! Based Bloom"): the bright-passed image is progressively **downsampled** (13-tap)
//! into a mip chain, then **upsampled** back (9-tap tent) with additive
//! accumulation. The repeated bilinear halving dissolves the source's corners into
//! a ROUND falloff with energy at every scale — a tight core halo AND a soft wide
//! glow.
//!
//! ```text
//!   motion RT ──prefilter──▶ mip0 ─down─▶ mip1 ─down─▶ … ─down─▶ mipN
//!                             ▲            ▲                      │
//!                             └── +up ─────┴──── +up ─── … ── +up┘   (additive)
//!                             │
//!             game_rt ◀── additive composite (× intensity) ◀── mip0
//! ```
//!
//! All in `shaders/bloom.wgsl`; no transcendentals (HR-5).

/// **O que o artista autora**, e as derivações de cada número — irmão por HR-18.
#[path = "motion_fx_params.rs"]
mod params;
pub use params::{BloomParams, COMPOSITE_OPERATIONS};
// ⚠️ Os três números que as PROVAS leem — o raio-base da tenda, o piso da anamorfose e o maior
// finito do formato. Eles moram com a lei que os usa; o `use` aqui é o que mantém o
// `motion_fx_tests.rs` a chamá-los pelo nome, como sempre chamou.
#[cfg(test)]
use params::{BASE_FILTER_RADIUS, F16_MAX};

/// **O que se reconstrói a cada redimensionamento** — a cadeia de mips e os bind groups.
#[path = "motion_fx_targets.rs"]
mod targets;
use targets::{Shared, build_targets};

use ph2d_gpu::GpuContext;

#[path = "motion_fx_trig.rs"]
mod trig;

struct Tex {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: (u32, u32),
}

fn make_tex(gpu: &GpuContext, size: (u32, u32), label: &str) -> Tex {
    let size = (size.0.max(1), size.1.max(1));
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: crate::GameRt::FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Tex {
        texture,
        view,
        size,
    }
}

/// **A LUT DA RAMPA DO HALO** — `HALO_LUT_TEXELS × 1`, `Rgba16Float`.
///
/// ⚠️ **`Rgba16Float` e não `Rgba32Float`**, e a razão é a filtragem: o `rgba32float` **não é
/// filtrável** sem uma feature opcional do WebGPU, e sem filtragem linear a LUT devolveria
/// degraus — exactamente o artefacto que a resolução medida existe para não ter. E não
/// `Rgba8Unorm`, cuja quantização é `1/255`: ela **é** o passo do ecrã, ou seja gastaria o
/// orçamento inteiro do erro antes de a interpolação começar.
fn make_lut(gpu: &GpuContext) -> Tex {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render motion-fx halo LUT"),
        size: wgpu::Extent3d {
            width: HALO_LUT_TEXELS as u32,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: crate::GameRt::FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Tex {
        texture,
        view,
        size: (HALO_LUT_TEXELS as u32, 1),
    }
}

/// Quantos texels a LUT do halo carrega — **espelho local** de
/// `ph2d_node_fx_glow::HALO_LUT_TEXELS`.
///
/// ⚠️ **Dito aqui e não importado**: o renderer não depende de nó nenhum (é a fronteira que
/// mantém o `ph2d-render` utilizável fora do Motion). O preço do espelho é poder divergir, e é
/// por isso que ele tem gate na shell — o único sítio que vê os dois lados.
pub const HALO_LUT_TEXELS: usize = 512;

/// Cap on mip-chain depth (6 halvings reach a wide soft glow at any editor size).
const MAX_MIPS: usize = 6;

/// The mip resolutions: mip0 = half the RT, then halve while both dims stay ≥ 2,
/// capped at [`MAX_MIPS`]. Always at least one level.
fn mip_sizes(size: (u32, u32)) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    let mut s = (size.0.max(2) / 2, size.1.max(2) / 2);
    for _ in 0..MAX_MIPS {
        out.push(s);
        if s.0 <= 2 || s.1 <= 2 {
            break;
        }
        s = ((s.0 / 2).max(1), (s.1 / 2).max(1));
    }
    out
}

pub struct MotionFx {
    // Size-independent (built once):
    bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    prefilter_pipeline: wgpu::RenderPipeline,
    downsample_pipeline: wgpu::RenderPipeline,
    upsample_pipeline: wgpu::RenderPipeline,
    /// Um por OPERAÇÃO do halo, na ordem das tags — ver [`BloomParams::operation`].
    composite_pipelines: [wgpu::RenderPipeline; COMPOSITE_OPERATIONS],
    u_prefilter: wgpu::Buffer,
    u_up: wgpu::Buffer,
    u_composite: wgpu::Buffer,
    /// A tabela da rampa do halo. Construída uma vez; reescrita só quando a rampa muda.
    lut: Tex,
    /// A última LUT enviada — o que impede um `write_texture` por quadro sobre bytes iguais.
    lut_uploaded: Vec<[f32; 4]>,

    // Size-dependent (rebuilt on resize):
    rt: Tex,
    mips: Vec<Tex>,
    /// One per downsample pass (`mips.len() - 1`), holding that pass's source texel size.
    u_down: Vec<wgpu::Buffer>,
    bg_prefilter: wgpu::BindGroup,
    /// Downsample pass `i` reads `mips[i]` → writes `mips[i+1]`.
    bg_down: Vec<wgpu::BindGroup>,
    /// Upsample pass `i` reads `mips[i+1]` → writes `mips[i]` (additive).
    bg_up: Vec<wgpu::BindGroup>,
    bg_composite: wgpu::BindGroup,
    size: (u32, u32),
}

impl MotionFx {
    pub fn new(gpu: &GpuContext, size: (u32, u32)) -> Self {
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render motion-fx bgl"),
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
                    // **A LUT DA RAMPA DO HALO** (doc 89 folha 11).
                    //
                    // ⚠️ **Ela está no layout PARTILHADO, e por isso viaja em todos os quatro
                    // passes** — só o composite a lê. A alternativa seria um segundo layout e um
                    // segundo pipeline layout só para aquele passe, o que duplicaria a
                    // construção inteira por causa de uma textura de **4 KB**. O preço de a
                    // deixar amarrada nos outros três é um descritor por bind group.
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-render motion-fx layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render motion-fx bloom shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/bloom.wgsl").into()),
            });

        let pipeline = |label: &str, fs: &str, blend: Option<wgpu::BlendState>| {
            gpu.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some(fs),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: crate::GameRt::FORMAT,
                            blend,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
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

        // Additive: color One/One (light only brightens), alpha kept from the dst.
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let prefilter_pipeline = pipeline("ph2d-render motion-fx prefilter", "fs_prefilter", None);
        let downsample_pipeline =
            pipeline("ph2d-render motion-fx downsample", "fs_downsample", None);
        // Upsample accumulates onto the finer mip's existing downsample content.
        let upsample_pipeline = pipeline(
            "ph2d-render motion-fx upsample",
            "fs_upsample",
            Some(additive),
        );
        // **SCREEN** — `a + b − ab` (doc 89 folha 11, o *Glow Operation* do AE).
        //
        // ⚠️ **Ele é um par de FATORES, não um shader**: `src·(1−dst) + dst·1` é exactamente
        // a fórmula, e a máquina de mistura já sabe fazê-la. Escrevê-la no fragmento exigiria
        // LER o alvo, que um passe de fullscreen não pode — é essa a razão de a célula dizer
        // *"o `BlendState` é do pipeline, nenhum nó o alcança"*.
        //
        // ⚠️ **E é o único dos três modos que a navalha do §0 deixa passar**: `Screen` é
        // monótono e **nunca escurece**, enquanto o `Multiply` do AE tiraria luz de uma
        // composição sem profundidade — ele pintaria por cima do que estivesse à frente.
        let screen = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::OneMinusDst,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            // O alfa é o do aditivo, verbatim: a cena opaca continua opaca para o compositor.
            alpha: additive.alpha,
        };
        // ⚠️ **A ORDEM É A DAS TAGS** (`0 = Add`, `1 = Screen`) e o gate da shell liga esta
        // contagem à lista de rótulos do nó — um pipeline a menos aqui faria o último modo do
        // dropdown ser silenciosamente rebaixado.
        let composite_pipelines = [
            pipeline(
                "ph2d-render motion-fx composite (add)",
                "fs_composite",
                Some(additive),
            ),
            pipeline(
                "ph2d-render motion-fx composite (screen)",
                "fs_composite",
                Some(screen),
            ),
        ];

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render motion-fx sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let uniform = |label: &str| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: 32, // two vec4<f32> (Params.v + Params.v2; composite uses both)
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let u_prefilter = uniform("ph2d-render motion-fx u_prefilter");
        let u_up = uniform("ph2d-render motion-fx u_up");
        let u_composite = uniform("ph2d-render motion-fx u_composite");

        let lut = make_lut(gpu);
        let t = build_targets(
            gpu,
            &Shared {
                bgl: &bgl,
                sampler: &sampler,
                u_prefilter: &u_prefilter,
                u_up: &u_up,
                u_composite: &u_composite,
                lut: &lut,
            },
            size,
        );

        Self {
            bgl,
            sampler,
            prefilter_pipeline,
            downsample_pipeline,
            upsample_pipeline,
            composite_pipelines,
            u_prefilter,
            u_up,
            u_composite,
            lut,
            lut_uploaded: Vec::new(),
            rt: t.rt,
            mips: t.mips,
            u_down: t.u_down,
            bg_prefilter: t.bg_prefilter,
            bg_down: t.bg_down,
            bg_up: t.bg_up,
            bg_composite: t.bg_composite,
            size,
        }
    }

    /// The full-resolution HDR target the shell renders the Motion instances into
    /// (via [`render_instances_only`](crate::SpriteRenderer::render_instances_only))
    /// before calling [`bloom_over`](Self::bloom_over).
    pub fn rt_view(&self) -> &wgpu::TextureView {
        &self.rt.view
    }

    /// Recreate the RT + mip chain if the surface size changed. Call alongside
    /// `game_rt.ensure_size`.
    pub fn ensure_size(&mut self, gpu: &GpuContext, size: (u32, u32)) {
        if size == self.size || size.0 == 0 || size.1 == 0 {
            return;
        }
        let t = build_targets(
            gpu,
            &Shared {
                bgl: &self.bgl,
                sampler: &self.sampler,
                u_prefilter: &self.u_prefilter,
                u_up: &self.u_up,
                u_composite: &self.u_composite,
                lut: &self.lut,
            },
            size,
        );
        self.rt = t.rt;
        self.mips = t.mips;
        self.u_down = t.u_down;
        self.bg_prefilter = t.bg_prefilter;
        self.bg_down = t.bg_down;
        self.bg_up = t.bg_up;
        self.bg_composite = t.bg_composite;
        self.size = size;
    }

    /// Bright-pass, downsample, upsample and add the glow over `target` (the game
    /// RT). Assumes the shell already rendered the Motion instances into
    /// [`rt_view`](Self::rt_view) this frame, at the SAME sub-rect the scene used.
    /// `halo_lut` é a **rampa de cor do halo já assada** (`ph2d_node_fx_glow::bake_halo_lut`) —
    /// `None` quando o artista não autorou nenhuma, e aí o passe usa o `tint` constante de
    /// sempre, **ao bit**.
    ///
    /// ⚠️ **A tabela vem PRONTA de fora e não é calculada aqui**, e é a fronteira que importa: as
    /// cinco interpolações e os três espaços de cor que o editor oferece são semântica da
    /// biblioteca de cor, e reimplementá-los num shader seria a segunda porta que diverge da
    /// primeira. O renderer só sabe amostrar uma tabela.
    pub fn bloom_over(
        &mut self,
        gpu: &GpuContext,
        target: &wgpu::TextureView,
        params: &BloomParams,
        halo_lut: Option<&[[f32; 4]]>,
    ) {
        // ⚠️ **A LUT só sobe quando MUDA.** Um `write_texture` por quadro sobre bytes iguais é
        // 4 KB de banda e uma barreira de recurso por cada quadro em que nada aconteceu — e a
        // rampa muda quando o artista arrasta uma parada, não a 60 Hz.
        let live = match halo_lut {
            Some(lut) if lut.len() == HALO_LUT_TEXELS => {
                if self.lut_uploaded != lut {
                    self.upload_lut(gpu, lut);
                    self.lut_uploaded = lut.to_vec();
                }
                true
            }
            // ⚠️ Uma tabela de tamanho errado conta como AUSENTE, nunca como meia-tabela: ela só
            // pode vir de um espelho que divergiu, e desenhar metade dela seria o defeito a
            // aparecer como cor errada em vez de como nada.
            _ => false,
        };
        // Per-pass uniforms (distinct buffers → all queue writes land before the
        // single submit; no pass mutates another's value mid-encoder).
        // v = a curva do joelho; v2.x = o teto do bright-pass (ver `clamp_limit`).
        let curve = params.prefilter_curve();
        let pre: [f32; 8] = [
            curve[0],
            curve[1],
            curve[2],
            curve[3],
            params.clamp_limit(),
            params.source_flag(),
            0.0,
            0.0,
        ];
        gpu.queue
            .write_buffer(&self.u_prefilter, 0, bytemuck::cast_slice(&pre));
        for (i, buf) in self.u_down.iter().enumerate() {
            // Downsample pass i reads mips[i]; its taps step by that mip's texel.
            let s = self.mips[i].size;
            let v: [f32; 4] = [1.0 / s.0 as f32, 1.0 / s.1 as f32, 0.0, 0.0];
            gpu.queue.write_buffer(buf, 0, bytemuck::cast_slice(&v));
        }
        // A BASE da tenda em UV; o y leva o aspecto para o alcance ser redondo em
        // pixels. Ver `upsample_basis` — no neutro é `[fr, 0, 0, fr·aspect]`.
        let aspect = self.size.0.max(1) as f32 / self.size.1.max(1) as f32;
        let up = params.upsample_basis(aspect);
        gpu.queue
            .write_buffer(&self.u_up, 0, bytemuck::cast_slice(&up));
        // Composite reads both vec4s: v = (intensity, saturation, _, _),
        // v2 = tint rgba.
        let comp: [f32; 8] = [
            params.intensity,
            params.saturation.clamp(0.0, 1.0),
            // `v.z` diz ao shader se há LUT. `0` = o `tint` constante, o caminho literal.
            if live { 1.0 } else { 0.0 },
            0.0,
            params.tint[0],
            params.tint[1],
            params.tint[2],
            params.tint[3],
        ];
        gpu.queue
            .write_buffer(&self.u_composite, 0, bytemuck::cast_slice(&comp));

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render motion-fx encoder"),
            });

        // Prefilter: motion RT → mip0 (bright-pass + half-res downsample).
        fullscreen(
            &mut encoder,
            &self.prefilter_pipeline,
            &self.bg_prefilter,
            &self.mips[0].view,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            "render.motion_fx.prefilter",
        );
        // Downsample chain: mip[i] → mip[i+1].
        for (i, bg) in self.bg_down.iter().enumerate() {
            fullscreen(
                &mut encoder,
                &self.downsample_pipeline,
                bg,
                &self.mips[i + 1].view,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                "render.motion_fx.down",
            );
        }
        // Upsample chain, coarse → fine: add mip[i+1] onto mip[i] (Load + additive).
        for i in (0..self.bg_up.len()).rev() {
            fullscreen(
                &mut encoder,
                &self.upsample_pipeline,
                &self.bg_up[i],
                &self.mips[i].view,
                wgpu::LoadOp::Load,
                "render.motion_fx.up",
            );
        }
        // Composite: the accumulated glow (mip0) added over the scene.
        fullscreen(
            &mut encoder,
            &self.composite_pipelines[params.operation_tag()],
            &self.bg_composite,
            target,
            wgpu::LoadOp::Load,
            "render.motion_fx.composite",
        );
        gpu.queue.submit(Some(encoder.finish()));
    }
}

impl MotionFx {
    /// Escreve a tabela na textura, convertendo para meio-float — o formato que a torna
    /// **filtrável** (ver [`make_lut`]).
    fn upload_lut(&self, gpu: &GpuContext, lut: &[[f32; 4]]) {
        let mut bytes = Vec::with_capacity(HALO_LUT_TEXELS * 8);
        for texel in lut {
            for c in texel {
                bytes.extend_from_slice(&ph2d_color::f32_to_half(*c).to_le_bytes());
            }
        }
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.lut.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((HALO_LUT_TEXELS * 8) as u32),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: HALO_LUT_TEXELS as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// One fullscreen-triangle pass into `view`.
fn fullscreen(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::RenderPipeline,
    bind_group: &wgpu::BindGroup,
    view: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
    profile: &'static str,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("ph2d-render motion-fx pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: ph2d_gpu::pass_profiler::render_writes(profile),
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.draw(0..3, 0..1);
}

#[cfg(test)]
#[path = "motion_fx_tests.rs"]
mod tests;
