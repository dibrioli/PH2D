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
//! a spark tinted `(3, 2, 1)` glows three times as hard as one tinted white. The
//! Painter's 8-bit compositor would clip those to white on the way in — which is
//! exactly why routing Motion glow through it was rejected (doc 66). The tonemap
//! downstream still clamps the summed result, so the brightest cores read as
//! white and the halo falls off through the mid-tones — a real bloom.
//!
//! ## The chain
//!
//! ```text
//!   motion RT (full-res HDR) ──prefilter──▶ a (½-res) ─blur─▶ b ─blur─▶ a ─blur─▶ b ─blur─▶ a
//!                                                                                            │
//!                                    game_rt ◀────────────── additive composite ◀───────────┘
//! ```
//!
//! Prefilter (soft-knee bright-pass) also downsamples to half-res; four Kawase
//! iterations ping-pong `a`/`b` with a growing offset (a cheap wide Gaussian);
//! the composite adds the result over `game_rt` with One/One color blend. All in
//! `shaders/bloom.wgsl`; no transcendentals (HR-5).

use ph2d_gpu::GpuContext;

/// The three glow knobs the document carries (doc 67). Plain data — the panel
/// authors it, the shell hands it to [`bloom_over`](MotionFx::bloom_over).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BloomParams {
    /// Brightness above which a pixel starts to glow (premult `max(r,g,b)`).
    /// `1.0` = only genuinely HDR (emissive) pixels bloom.
    pub threshold: f32,
    /// Soft-knee width around the threshold — the glow ramps in over
    /// `[threshold-knee, threshold]` instead of switching on hard.
    pub knee: f32,
    /// Multiplier on the blurred glow before it is added to the scene.
    pub intensity: f32,
    /// Scales the blur offsets — a wider radius spreads the halo further.
    pub radius: f32,
}

impl Default for BloomParams {
    fn default() -> Self {
        Self {
            threshold: 1.0,
            knee: 0.6,
            intensity: 0.8,
            radius: 1.0,
        }
    }
}

impl BloomParams {
    /// Pack the soft-knee curve the prefilter shader expects:
    /// `(threshold, threshold-knee, 2·knee, 0.25/knee)` (COD/Karis).
    fn prefilter_curve(&self) -> [f32; 4] {
        let knee = self.knee.max(1e-4);
        [self.threshold, self.threshold - knee, 2.0 * knee, 0.25 / knee]
    }
}

/// Base blur offsets in half-res texels, scaled by [`BloomParams::radius`]. Four
/// growing steps ping-ponged `a→b→a→b→a` widen the halo.
const BLUR_OFFSETS: [f32; 4] = [1.0, 2.0, 3.0, 4.0];

struct Tex {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

fn make_tex(gpu: &GpuContext, size: (u32, u32), label: &str) -> Tex {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
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
    Tex { texture, view }
}

pub struct MotionFx {
    // Size-independent (built once):
    bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    prefilter_pipeline: wgpu::RenderPipeline,
    blur_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    u_prefilter: wgpu::Buffer,
    u_blur: [wgpu::Buffer; 4],
    u_composite: wgpu::Buffer,

    // Size-dependent (rebuilt on resize):
    rt: Tex,
    a: Tex,
    b: Tex,
    bg_prefilter: wgpu::BindGroup,
    /// One per blur iteration: source alternates `a,b,a,b`, each paired with its
    /// own offset uniform.
    bg_blur: [wgpu::BindGroup; 4],
    bg_composite: wgpu::BindGroup,
    size: (u32, u32),
    half: (u32, u32),
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

        // Additive over the scene: color One/One (glow only brightens), alpha
        // kept from the destination (the opaque scene stays opaque).
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
        let blur_pipeline = pipeline("ph2d-render motion-fx blur", "fs_blur", None);
        let composite_pipeline =
            pipeline("ph2d-render motion-fx composite", "fs_composite", Some(additive));

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
                size: 16, // one vec4<f32>
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let u_prefilter = uniform("ph2d-render motion-fx u_prefilter");
        let u_blur = [
            uniform("ph2d-render motion-fx u_blur0"),
            uniform("ph2d-render motion-fx u_blur1"),
            uniform("ph2d-render motion-fx u_blur2"),
            uniform("ph2d-render motion-fx u_blur3"),
        ];
        let u_composite = uniform("ph2d-render motion-fx u_composite");

        let (rt, a, b, bg_prefilter, bg_blur, bg_composite, half) =
            build_targets(gpu, &bgl, &sampler, &u_prefilter, &u_blur, &u_composite, size);

        Self {
            bgl,
            sampler,
            prefilter_pipeline,
            blur_pipeline,
            composite_pipeline,
            u_prefilter,
            u_blur,
            u_composite,
            rt,
            a,
            b,
            bg_prefilter,
            bg_blur,
            bg_composite,
            size,
            half,
        }
    }

    /// The full-resolution HDR target the shell renders the Motion instances
    /// into (via [`render_instances_only`](crate::SpriteRenderer::render_instances_only))
    /// before calling [`bloom_over`](Self::bloom_over).
    pub fn rt_view(&self) -> &wgpu::TextureView {
        &self.rt.view
    }

    /// Recreate the RT + blur chain if the surface size changed. Call alongside
    /// `game_rt.ensure_size`.
    pub fn ensure_size(&mut self, gpu: &GpuContext, size: (u32, u32)) {
        if size == self.size || size.0 == 0 || size.1 == 0 {
            return;
        }
        let (rt, a, b, bg_prefilter, bg_blur, bg_composite, half) = build_targets(
            gpu,
            &self.bgl,
            &self.sampler,
            &self.u_prefilter,
            &self.u_blur,
            &self.u_composite,
            size,
        );
        self.rt = rt;
        self.a = a;
        self.b = b;
        self.bg_prefilter = bg_prefilter;
        self.bg_blur = bg_blur;
        self.bg_composite = bg_composite;
        self.size = size;
        self.half = half;
    }

    /// Bright-pass + blur the Motion RT and add the glow over `target` (the game
    /// RT). Assumes the shell already rendered the Motion instances into
    /// [`rt_view`](Self::rt_view) this frame.
    pub fn bloom_over(&self, gpu: &GpuContext, target: &wgpu::TextureView, params: &BloomParams) {
        // Per-pass uniforms. Distinct buffers → all queue writes land before the
        // single submit, and no pass mutates another's value mid-encoder.
        gpu.queue.write_buffer(
            &self.u_prefilter,
            0,
            bytemuck::cast_slice(&params.prefilter_curve()),
        );
        let texel = (1.0 / self.half.0.max(1) as f32, 1.0 / self.half.1.max(1) as f32);
        for (off, buf) in BLUR_OFFSETS.iter().zip(&self.u_blur) {
            let s = off * params.radius;
            let v: [f32; 4] = [texel.0 * s, texel.1 * s, 0.0, 0.0];
            gpu.queue.write_buffer(buf, 0, bytemuck::cast_slice(&v));
        }
        let comp: [f32; 4] = [params.intensity, 0.0, 0.0, 0.0];
        gpu.queue
            .write_buffer(&self.u_composite, 0, bytemuck::cast_slice(&comp));

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render motion-fx encoder"),
            });

        // Prefilter: motion RT → a (½-res).
        fullscreen(
            &mut encoder,
            &self.prefilter_pipeline,
            &self.bg_prefilter,
            &self.a.view,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            "render.motion_fx.prefilter",
        );
        // Four Kawase iterations ping-ponging a→b→a→b→a.
        let dst = [&self.b.view, &self.a.view, &self.b.view, &self.a.view];
        for (bg, dst_view) in self.bg_blur.iter().zip(dst) {
            fullscreen(
                &mut encoder,
                &self.blur_pipeline,
                bg,
                dst_view,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                "render.motion_fx.blur",
            );
        }
        // Composite: final glow (in `a`) added over the scene.
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

/// (Re)create the full-res RT + the two half-res ping-pong textures and the
/// bind groups that reference them. The bind groups pair each pass's source
/// view with its uniform: prefilter reads the RT; blur `i` reads `a` on even
/// iterations and `b` on odd; composite reads `a` (the final blur target).
#[allow(clippy::type_complexity)]
fn build_targets(
    gpu: &GpuContext,
    bgl: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    u_prefilter: &wgpu::Buffer,
    u_blur: &[wgpu::Buffer; 4],
    u_composite: &wgpu::Buffer,
    size: (u32, u32),
) -> (
    Tex,
    Tex,
    Tex,
    wgpu::BindGroup,
    [wgpu::BindGroup; 4],
    wgpu::BindGroup,
    (u32, u32),
) {
    let half = (size.0.max(2) / 2, size.1.max(2) / 2);
    let rt = make_tex(gpu, size, "ph2d-render motion-fx RT (Rgba16Float HDR)");
    let a = make_tex(gpu, half, "ph2d-render motion-fx blur a");
    let b = make_tex(gpu, half, "ph2d-render motion-fx blur b");

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
    // Sources alternate a,b,a,b (matching the a→b→a→b→a ping-pong in bloom_over).
    let bg_blur = [
        bind(&a.view, &u_blur[0]),
        bind(&b.view, &u_blur[1]),
        bind(&a.view, &u_blur[2]),
        bind(&b.view, &u_blur[3]),
    ];
    let bg_composite = bind(&a.view, u_composite);
    (rt, a, b, bg_prefilter, bg_blur, bg_composite, half)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bloom_is_threshold_one() {
        // The neutral authored bloom only lights genuinely-HDR (emissive)
        // pixels: threshold 1.0 leaves an LDR scene untouched.
        assert_eq!(BloomParams::default().threshold, 1.0);
    }

    #[test]
    fn prefilter_curve_packs_the_soft_knee() {
        let p = BloomParams {
            threshold: 1.0,
            knee: 0.5,
            intensity: 1.0,
            radius: 1.0,
        };
        // (threshold, threshold-knee, 2·knee, 0.25/knee)
        assert_eq!(p.prefilter_curve(), [1.0, 0.5, 1.0, 0.5]);
    }

    #[test]
    fn zero_knee_does_not_divide_by_zero() {
        let p = BloomParams {
            threshold: 1.0,
            knee: 0.0,
            intensity: 1.0,
            radius: 1.0,
        };
        assert!(p.prefilter_curve().iter().all(|v| v.is_finite()));
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
    /// and builds the three pipelines + every bind group against a real device —
    /// a shader error, a layout mismatch, or a wrong texture format dies HERE, not
    /// as an empty glow at runtime. `ensure_size` exercises the resize rebuild, and
    /// `bloom_over` encodes + submits the whole chain (prefilter → 4 blur → additive
    /// composite) into a Motion-format target; `poll(Wait)` drains it so any
    /// deferred validation surfaces before the test returns.
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
