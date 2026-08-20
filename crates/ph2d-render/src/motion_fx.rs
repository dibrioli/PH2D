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

use ph2d_gpu::GpuContext;

#[path = "motion_fx_trig.rs"]
mod trig;

/// The three glow knobs the document carries (doc 67). Plain data — the `fx.glow`
/// node authors it, the shell hands it to [`bloom_over`](MotionFx::bloom_over).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BloomParams {
    /// Brightness above which a pixel starts to glow (premult `max(r,g,b)`).
    /// `1.0` = only genuinely HDR (emissive) pixels bloom.
    pub threshold: f32,
    /// Soft-knee width around the threshold — the glow ramps in over
    /// `[threshold-knee, threshold]` instead of switching on hard.
    pub knee: f32,
    /// Multiplier on the accumulated glow before it is added to the scene.
    pub intensity: f32,
    /// Scales the upsample tent radius — a wider radius spreads the halo further.
    pub radius: f32,
    /// `0` pulls the glow to grey (a white bloom), `1` keeps the source colour.
    pub saturation: f32,
    /// Multiplies the (desaturated) glow — default white `[1,1,1,1]` is a no-op.
    pub tint: [f32; 4],
    /// **A ANAMORFOSE** — a razão entre o alcance do halo ao longo de
    /// [`Self::angle`] e o alcance perpendicular a ele (doc 89 folha 11). `1` é o
    /// halo redondo que sempre shipou; `>1` estica na direção do ângulo e aperta na
    /// outra, que é o *streak* anamórfico do cinema (o `Glow Dimensions H/V` do AE,
    /// o *Anamorphic Ratio* do Unity, a *Bloom Convolution* do Unreal).
    ///
    /// ⚠️ **Ela mora na TENDA do upsample, não na cadeia de mips.** Os mips são a
    /// máquina que arredonda as quinas da fonte (é literalmente o que os torna
    /// melhores que um box blur largo); torcê-los tornaria a queda direcional em
    /// TODAS as escalas e o halo perderia o miolo. A tenda de 9 taps é onde a
    /// referência põe a razão, e é o passe que corre uma vez por nível.
    pub stretch: f32,
    /// A direção do *streak*, em GRAUS — a unidade autorada única do app. Sem
    /// efeito em [`Self::stretch`] `= 1` (um círculo rodado é o mesmo círculo), e o
    /// `ParamGate` do nó esconde-a ali.
    pub angle: f32,
    /// **O TETO do bright-pass** — o antídoto dos *fireflies* (o `Clamp` do Bloom do
    /// Unity URP). `0` = **desligado**, o caminho literal que sempre shipou.
    ///
    /// ⚠️ **O recurso é a REPRESENTAÇÃO, e o número está medido.** O `tint` de uma
    /// instância é `[f32; 4]` **sem clamp** (doc 67 §4), e o bright-pass não limita
    /// nada: para `brightness → ∞` a contribuição tende a `1` e a saída tende ao
    /// próprio `c`. Então um único elemento com `tint = 5000` entra inteiro na
    /// cadeia, espalha-se por seis níveis de mip e lava a tela. O único teto que
    /// existe hoje é o do FORMATO — `Rgba16Float` guarda até **65 504** e depois é
    /// `inf`, que envenena a soma de toda a cadeia. Este param é o teto AUTORADO,
    /// que é o que a referência expõe.
    pub clamp: f32,
}

impl Default for BloomParams {
    fn default() -> Self {
        Self {
            threshold: 1.0,
            knee: 0.6,
            intensity: 0.8,
            radius: 1.0,
            saturation: 1.0,
            tint: [1.0, 1.0, 1.0, 1.0],
            stretch: 1.0,
            angle: 0.0,
            clamp: 0.0,
        }
    }
}

impl BloomParams {
    /// Pack the soft-knee curve the prefilter shader expects:
    /// `(threshold, threshold-knee, 2·knee, 0.25/knee)` (COD/Karis).
    fn prefilter_curve(&self) -> [f32; 4] {
        let knee = self.knee.max(1e-4);
        [
            self.threshold,
            self.threshold - knee,
            2.0 * knee,
            0.25 / knee,
        ]
    }
}

impl BloomParams {
    /// **A BASE 2×2 da tenda do upsample**, em UV: `[du.x, du.y, dv.x, dv.y]`.
    ///
    /// Os 9 taps deixam de ser `(±x, ±y)` e passam a ser `(±du ±dv)`. No neutro
    /// (`stretch = 1`) o caminho é **LITERAL** e devolve `[fr, 0, 0, fr·aspect]`,
    /// que reconstrói tap a tap os offsets de sempre — `uv + (−du + dv)` é
    /// `uv + (−fr, fr·aspect)`, a mesma soma, ao bit.
    ///
    /// ⚠️ **A anisotropia é calculada em PIXELS e convertida no fim.** O `aspect`
    /// existe para o halo sair redondo na tela; aplicá-lo antes da rotação faria o
    /// ângulo significar coisas diferentes em janelas diferentes — o mesmo `45°`
    /// apontaria para outro sítio ao redimensionar.
    fn upsample_basis(&self, aspect: f32) -> [f32; 4] {
        let fr = BASE_FILTER_RADIUS * self.radius.max(0.0);
        let s = self.stretch.max(MIN_STRETCH);
        if s == 1.0 {
            return [fr, 0.0, 0.0, fr * aspect];
        }
        let (c, sn) = trig::cos_sin_cycles(self.angle / 360.0);
        // Ao longo do ângulo alarga por `s`; perpendicular aperta por `1/s`, para o
        // «raio» continuar a ser a média geométrica dos dois e o knob não mudar a
        // ENERGIA do halo, só a forma dele.
        let (ax, ay) = (c * fr * s, sn * fr * s);
        let (bx, by) = (-sn * fr / s, c * fr / s);
        [ax, ay * aspect, bx, by * aspect]
    }

    /// O teto do bright-pass, como o shader o quer: `0` (desligado) vira o maior
    /// finito do `Rgba16Float`, que é o teto que a REPRESENTAÇÃO já impunha — então
    /// o `min` do shader é um no-op sobre qualquer valor que o RT consiga guardar.
    fn clamp_limit(&self) -> f32 {
        if self.clamp > 0.0 {
            self.clamp
        } else {
            F16_MAX
        }
    }
}

/// Upsample tent radius in UV at `radius = 1` (the mip chain does the heavy
/// spreading; this is the per-level tent overlap). Scaled by `BloomParams::radius`.
const BASE_FILTER_RADIUS: f32 = 0.006;
/// Piso da anamorfose: abaixo disto o eixo estreito colapsa e a tenda deixa de
/// cobrir o próprio texel (o `1/s` explodiria o outro eixo).
const MIN_STRETCH: f32 = 0.05;
/// O maior finito representável em `Rgba16Float` — o teto que o formato do RT já
/// impõe, e o valor com que o clamp desligado passa pelo `min` sem morder.
const F16_MAX: f32 = 65_504.0;
/// Cap on mip-chain depth (6 halvings reach a wide soft glow at any editor size).
const MAX_MIPS: usize = 6;

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
    composite_pipeline: wgpu::RenderPipeline,
    u_prefilter: wgpu::Buffer,
    u_up: wgpu::Buffer,
    u_composite: wgpu::Buffer,

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
        let composite_pipeline = pipeline(
            "ph2d-render motion-fx composite",
            "fs_composite",
            Some(additive),
        );

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

        let t = build_targets(gpu, &bgl, &sampler, &u_prefilter, &u_up, &u_composite, size);

        Self {
            bgl,
            sampler,
            prefilter_pipeline,
            downsample_pipeline,
            upsample_pipeline,
            composite_pipeline,
            u_prefilter,
            u_up,
            u_composite,
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
            &self.bgl,
            &self.sampler,
            &self.u_prefilter,
            &self.u_up,
            &self.u_composite,
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
    pub fn bloom_over(&self, gpu: &GpuContext, target: &wgpu::TextureView, params: &BloomParams) {
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
            0.0,
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
            0.0,
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
            &self.composite_pipeline,
            &self.bg_composite,
            target,
            wgpu::LoadOp::Load,
            "render.motion_fx.composite",
        );
        gpu.queue.submit(Some(encoder.finish()));
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

/// Everything size-dependent: the full-res RT, the mip chain, the per-pass
/// downsample uniforms, and all bind groups.
struct Targets {
    rt: Tex,
    mips: Vec<Tex>,
    u_down: Vec<wgpu::Buffer>,
    bg_prefilter: wgpu::BindGroup,
    bg_down: Vec<wgpu::BindGroup>,
    bg_up: Vec<wgpu::BindGroup>,
    bg_composite: wgpu::BindGroup,
}

fn build_targets(
    gpu: &GpuContext,
    bgl: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    u_prefilter: &wgpu::Buffer,
    u_up: &wgpu::Buffer,
    u_composite: &wgpu::Buffer,
    size: (u32, u32),
) -> Targets {
    let rt = make_tex(gpu, size, "ph2d-render motion-fx RT (Rgba16Float HDR)");
    let mips: Vec<Tex> = mip_sizes(size)
        .into_iter()
        .map(|d| make_tex(gpu, d, "ph2d-render motion-fx mip"))
        .collect();
    let passes = mips.len().saturating_sub(1);

    let u_down: Vec<wgpu::Buffer> = (0..passes)
        .map(|_| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-render motion-fx u_down"),
                size: 32,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        })
        .collect();

    let bind = |src: &wgpu::TextureView, u: &wgpu::Buffer| {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render motion-fx bg"),
            layout: bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(src),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: u.as_entire_binding(),
                },
            ],
        })
    };

    let bg_prefilter = bind(&rt.view, u_prefilter);
    let bg_down: Vec<_> = (0..passes)
        .map(|i| bind(&mips[i].view, &u_down[i]))
        .collect();
    let bg_up: Vec<_> = (0..passes).map(|i| bind(&mips[i + 1].view, u_up)).collect();
    let bg_composite = bind(&mips[0].view, u_composite);

    Targets {
        rt,
        mips,
        u_down,
        bg_prefilter,
        bg_down,
        bg_up,
        bg_composite,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bloom_is_threshold_one() {
        // The neutral authored bloom only lights genuinely-HDR (emissive) pixels:
        // threshold 1.0 leaves an LDR scene untouched.
        assert_eq!(BloomParams::default().threshold, 1.0);
    }

    #[test]
    /// **O NEUTRO DA TENDA RECONSTRÓI OS OFFSETS DE SEMPRE** — a base 2×2 em
    /// `stretch = 1` é `[fr, 0, 0, fr·aspect]`, e os nove taps `(±du ±dv)` são,
    /// termo a termo, os `(±x, ±y)` de antes.
    ///
    /// ⚠️ A igualdade tem de ser EXACTA: este é o gate que autoriza a troca do
    /// shader sem um passe de paridade na GPU.
    fn the_neutral_basis_is_the_two_radii_that_shipped() {
        let p = BloomParams::default();
        let fr = BASE_FILTER_RADIUS * p.radius;
        for aspect in [1.0f32, 16.0 / 9.0, 0.5] {
            assert_eq!(p.upsample_basis(aspect), [fr, 0.0, 0.0, fr * aspect]);
        }
    }

    #[test]
    /// **A ANAMORFOSE ESTICA AO LONGO DO ÂNGULO E APERTA NA PERPENDICULAR.**
    ///
    /// ⚠️ O oráculo é a RAZÃO dos dois eixos e não o comprimento de um deles: um
    /// `stretch` que só alargasse `du` mudaria a energia do halo em vez da forma.
    /// A `0°` a base fica alinhada aos eixos, então os comprimentos são lidos
    /// directamente (com o `aspect` desfeito no eixo y).
    fn the_anamorphic_basis_trades_one_axis_for_the_other() {
        let p = BloomParams {
            stretch: 4.0,
            angle: 0.0,
            ..BloomParams::default()
        };
        let b = p.upsample_basis(1.0);
        let (du, dv) = (b[0].hypot(b[1]), b[2].hypot(b[3]));
        let fr = BASE_FILTER_RADIUS * p.radius;
        assert!((du - fr * 4.0).abs() < 1e-7, "du = {du}");
        assert!((dv - fr / 4.0).abs() < 1e-7, "dv = {dv}");
        // A média geométrica é o raio: a forma muda, a energia não.
        assert!(((du * dv).sqrt() - fr).abs() < 1e-7);
    }

    #[test]
    /// **A `stretch = 1` O ÂNGULO NÃO PODE RODAR NADA** — um círculo rodado é o
    /// mesmo círculo, e é essa a lei que o `ParamGate` do nó espelha ao esconder o
    /// controle ali.
    ///
    /// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE.** Apagar o braço literal
    /// do neutro passava pelo gate do neutro, porque com `angle = 0` a senoide
    /// parabólica devolve `(1, 0)` EXACTO e o `-0.0` que sobra compara igual a
    /// `0.0`. A propriedade que se perdia só aparece com um ângulo **não-nulo**:
    /// sem o braço literal a base roda e o halo redondo passa a depender de um
    /// controle que não devia mordê-lo.
    fn at_stretch_one_the_angle_cannot_turn_the_round_halo() {
        let round = BloomParams::default();
        for angle in [0.0f32, 37.0, 90.0, 213.5] {
            let p = BloomParams { angle, ..round };
            assert_eq!(
                p.upsample_basis(1.6),
                round.upsample_basis(1.6),
                "a {angle}° o halo redondo tem de ficar exactamente onde estava"
            );
        }
    }

    #[test]
    /// **O ÂNGULO RODA A BASE, e a 90° os dois eixos trocam de papel.**
    fn the_streak_angle_turns_the_basis() {
        let p = BloomParams {
            stretch: 3.0,
            angle: 90.0,
            ..BloomParams::default()
        };
        let b = p.upsample_basis(1.0);
        let fr = BASE_FILTER_RADIUS * p.radius;
        // A 90° o eixo LARGO aponta para +y.
        assert!(b[0].abs() < 1e-5, "du.x = {}", b[0]);
        assert!((b[1] - fr * 3.0).abs() < 1e-5, "du.y = {}", b[1]);
    }

    #[test]
    /// **O CLAMP NASCE DESLIGADO, e desligado ele é o teto do FORMATO.**
    ///
    /// ⚠️ É isso que faz o `min` do shader não precisar de um ramo: `65 504` é o
    /// maior finito que o `Rgba16Float` guarda, então o limite não pode morder
    /// nada que o RT consiga representar. Um `0` a chegar cru ao shader apagaria
    /// o glow inteiro — é a inversão que este gate impede.
    fn the_clamp_is_off_by_default_and_off_means_the_formats_own_ceiling() {
        assert_eq!(BloomParams::default().clamp, 0.0);
        assert_eq!(BloomParams::default().clamp_limit(), F16_MAX);
        let p = BloomParams {
            clamp: 2.5,
            ..BloomParams::default()
        };
        assert_eq!(p.clamp_limit(), 2.5);
        // Um valor absurdo continua a ser o do artista — quem decide o teto do
        // teto é a `ParamHardMax` do nó, não este conversor.
        let big = BloomParams {
            clamp: 1e9,
            ..BloomParams::default()
        };
        assert_eq!(big.clamp_limit(), 1e9);
    }

    #[test]
    /// **UM `stretch` DEGENERADO NÃO EXPLODE O EIXO ESTREITO.**
    fn a_degenerate_stretch_is_floored() {
        for s in [0.0f32, -3.0, 1e-9] {
            let p = BloomParams {
                stretch: s,
                ..BloomParams::default()
            };
            let b = p.upsample_basis(1.0);
            assert!(b.iter().all(|v| v.is_finite()), "stretch {s}: {b:?}");
        }
    }

    #[test]
    fn prefilter_curve_packs_the_soft_knee() {
        let p = BloomParams {
            threshold: 1.0,
            knee: 0.5,
            ..BloomParams::default()
        };
        // (threshold, threshold-knee, 2·knee, 0.25/knee)
        assert_eq!(p.prefilter_curve(), [1.0, 0.5, 1.0, 0.5]);
    }

    #[test]
    fn zero_knee_does_not_divide_by_zero() {
        let p = BloomParams {
            knee: 0.0,
            ..BloomParams::default()
        };
        assert!(p.prefilter_curve().iter().all(|v| v.is_finite()));
    }

    #[test]
    fn mip_chain_halves_and_is_capped() {
        // Half-res start, then halving, capped at MAX_MIPS, always ≥ 1 level.
        let m = mip_sizes((1024, 768));
        assert_eq!(m[0], (512, 384));
        assert!(m.len() <= MAX_MIPS);
        assert!(m.windows(2).all(|w| w[1].0 <= w[0].0 && w[1].1 <= w[0].1));
        // A tiny surface still yields a usable single mip, never an empty chain.
        assert!(!mip_sizes((2, 2)).is_empty());
    }

    /// Build a headless GpuContext (see `game_rt` tests). `None` on an
    /// adapter-less runner → the test no-ops there.
    fn try_headless_gpu() -> Option<GpuContext> {
        use std::sync::OnceLock;
        static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
        SHARED
            .get_or_init(|| {
                let instance = GpuContext::default_instance();
                GpuContext::new(instance, None).ok()
            })
            .clone()
    }

    /// **The blank-screen guard.** Constructing `MotionFx` compiles `bloom.wgsl`
    /// and builds the four pipelines + every bind group against a real device — a
    /// shader error, a layout mismatch, or a wrong texture format dies HERE, not
    /// as an empty glow at runtime. `ensure_size` exercises the resize rebuild
    /// (and a mip count change), and `bloom_over` encodes + submits the whole
    /// chain (prefilter → downsample → upsample → composite); `poll(Wait)` drains
    /// it so any deferred validation surfaces before the test returns.
    #[test]
    fn the_bloom_chain_is_a_valid_pipeline_on_a_real_device() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut fx = MotionFx::new(&gpu, (256, 256));
        fx.ensure_size(&gpu, (320, 200));
        let target = crate::GameRt::new(&gpu, (320, 200));
        fx.bloom_over(&gpu, target.view(), &BloomParams::default());
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
    }
}
