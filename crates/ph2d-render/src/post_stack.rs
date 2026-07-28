//! `PostStack` — the app's HDR colour-grade post-process (ADR-0145).
//!
//! ## Why a frame-wide pass, and not a Motion node
//!
//! A vignette darkens the **frame** edges; the Motion instances are fused into the
//! sprite pass with no origin tag, so "vignette only the Motion layer" is not a real
//! operation (doc 66/67). The glow could be a Motion node because bloom is **additive,
//! z-agnostic** light; a vignette is **frame-anchored and subtractive** — it is
//! intrinsically Option A. This pass grades the whole `game_rt`.
//!
//! ## The self-contained pass — the tonemap is untouched
//!
//! ```text
//!   game_rt (Rgba16Float) ──copy──▶ scratch ──[grade fullscreen]──▶ game_rt
//! ```
//!
//! `copy_texture_to_texture(game_rt → scratch)` (`game_rt` already has `COPY_SRC`, the
//! scratch owns `COPY_DST`), then a fullscreen pass samples the scratch, applies the
//! grade in HDR **linear** light, and rewrites `game_rt`. The downstream tonemap keeps
//! reading `game_rt` — blast radius zero, exactly like [`MotionFx::bloom_over`] adds the
//! glow over `game_rt` in place.
//!
//! **Byte-identity at neutral is a SKIP, not shader math** (ADR-0145): the shell only
//! encodes this pass when [`GradeParams::is_neutral`] is false — the glow's discipline
//! (`intensity > 0`). So the grade shader never runs at the exact neutral point.
//!
//! The grade order is scene-referred (a colour corrector before the display transform):
//! exposure → tint → contrast → saturation → vignette. The one source of truth for the
//! maths is [`grade_pixel`]; [`shaders/post_stack.wgsl`](../shaders/post_stack.wgsl)
//! mirrors it line for line, and the `#[ignore]` GPU parity test reads the device back
//! and compares.

use ph2d_gpu::GpuContext;

/// The colour-grade knobs (ADR-0145). Plain data — the shell authors it (fatia 2:
/// `ProjectSettings.grade`), the pass consumes it. `Default` is the **neutral** grade
/// (a no-op the shell skips).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GradeParams {
    /// Exposure in **stops**. `0` = neutral. `c *= 2^exposure` (the CPU pre-computes
    /// the multiplier, so the shader runs no transcendental).
    pub exposure: f32,
    /// Contrast around the scene-referred middle-grey pivot `0.18`. `1` = neutral.
    pub contrast: f32,
    /// Saturation via `mix(luma, c, sat)`. `1` = neutral, `0` = greyscale.
    pub saturation: f32,
    /// White-balance / tint multiply. `[1,1,1]` = neutral.
    pub tint: [f32; 3],
    /// Vignette **amount** `0..1`. `0` = neutral (the frame is untouched).
    pub vignette: f32,
    /// Where the darkening starts, as a fraction of the half-diagonal (`0` centre,
    /// `1` corner).
    pub vignette_radius: f32,
    /// Falloff width of the vignette band (added to `vignette_radius`).
    pub vignette_softness: f32,
}

impl Default for GradeParams {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

impl GradeParams {
    /// The neutral grade — the pass is a no-op and the shell skips it (byte-identical).
    pub const NEUTRAL: Self = Self {
        exposure: 0.0,
        contrast: 1.0,
        saturation: 1.0,
        tint: [1.0, 1.0, 1.0],
        vignette: 0.0,
        // Sensible defaults; irrelevant while `vignette == 0` (factor is 1 regardless).
        vignette_radius: 0.5,
        vignette_softness: 0.4,
    };

    /// The neutral point the shell tests to SKIP the pass (byte-identity, ADR-0145).
    /// `vignette_radius`/`vignette_softness` are excluded on purpose: with
    /// `vignette == 0` the factor is exactly 1 whatever they are, so a grade that only
    /// differs in those is still neutral.
    pub fn is_neutral(&self) -> bool {
        self.exposure == 0.0
            && self.contrast == 1.0
            && self.saturation == 1.0
            && self.tint == [1.0, 1.0, 1.0]
            && self.vignette == 0.0
    }

    /// `2^exposure` — the linear multiplier the exposure stop maps to.
    pub fn exposure_mul(&self) -> f32 {
        self.exposure.exp2()
    }

    /// Pack into the three `vec4<f32>` the shader's uniform expects:
    /// `v0 = (exposure_mul, contrast, saturation, vignette)`,
    /// `v1 = (tint.rgb, vignette_radius)`, `v2 = (vignette_softness, aspect, 0, 0)`.
    fn pack(&self, aspect: f32) -> [f32; 12] {
        [
            self.exposure_mul(),
            self.contrast,
            self.saturation,
            self.vignette,
            self.tint[0],
            self.tint[1],
            self.tint[2],
            self.vignette_radius,
            self.vignette_softness,
            aspect,
            0.0,
            0.0,
        ]
    }
}

/// Rec.709 luma weights (matches the shader's `dot`).
const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];
/// Scene-referred middle-grey pivot for the contrast op (matches the shader).
const PIVOT: f32 = 0.18;

/// Hermite smoothstep, defined identically here and in the shader so CPU↔GPU parity
/// holds on the vignette falloff (never `std`'s or a builtin whose edges might differ).
fn vig_smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The vignette factor at `uv` (top-left `(0,0)`, `[0,1]`). `1` at the centre, dips to
/// `1 - amount` toward the corners; the distance is aspect-corrected so the falloff is
/// round in **pixels**, not stretched by a wide frame.
fn vignette_factor(p: &GradeParams, uv: [f32; 2], aspect: f32) -> f32 {
    let dx = (uv[0] - 0.5) * aspect;
    let dy = uv[1] - 0.5;
    // Corner distance for the aspect-scaled centre offset; normalise so `d` is 1 there.
    let norm = 0.5 * (aspect * aspect + 1.0).sqrt();
    let d = (dx * dx + dy * dy).sqrt() / norm;
    let t = vig_smoothstep(
        p.vignette_radius,
        p.vignette_radius + p.vignette_softness.max(1e-4),
        d,
    );
    1.0 - p.vignette * t
}

/// **The one source of truth for the grade maths.** The shader is a line-for-line
/// mirror; the `#[ignore]` GPU parity test compares the two. `rgb` is scene-referred
/// linear HDR, `uv` is the fullscreen coord (top-left origin), `aspect = w/h`.
pub fn grade_pixel(p: &GradeParams, rgb: [f32; 3], uv: [f32; 2], aspect: f32) -> [f32; 3] {
    let e = p.exposure_mul();
    // 1. exposure + 2. tint (one multiply chain).
    let mut c = [
        rgb[0] * e * p.tint[0],
        rgb[1] * e * p.tint[1],
        rgb[2] * e * p.tint[2],
    ];
    // 3. contrast around the pivot, clamped non-negative (contrast can push below 0).
    for x in &mut c {
        *x = ((*x - PIVOT) * p.contrast + PIVOT).max(0.0);
    }
    // 4. saturation via the `mix` form — EXACT at sat == 1 (`luma*0 + c*1 == c`), which
    //    is why the neutral point stays bit-identical CPU↔GPU on this axis.
    let luma = LUMA[0] * c[0] + LUMA[1] * c[1] + LUMA[2] * c[2];
    for x in &mut c {
        *x = (luma * (1.0 - p.saturation) + *x * p.saturation).max(0.0);
    }
    // 5. vignette (last — an optical lens falloff on radiance before the display xform).
    let v = vignette_factor(p, uv, aspect);
    for x in &mut c {
        *x *= v;
    }
    c
}

struct Tex {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

fn make_scratch(gpu: &GpuContext, size: (u32, u32)) -> Tex {
    let size = (size.0.max(1), size.1.max(1));
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render post-stack scratch (Rgba16Float HDR)"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: crate::GameRt::FORMAT,
        // COPY_DST: target of the copy from game_rt. TEXTURE_BINDING: sampled by the
        // grade fragment.
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Tex { texture, view }
}

/// The app HDR colour-grade pass (ADR-0145). Grades `game_rt` **in place** (copy to a
/// scratch, grade back) so the tonemap is untouched. Sized alongside `game_rt`.
pub struct PostStack {
    // Size-independent (built once):
    bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    // Size-dependent (rebuilt on resize):
    scratch: Tex,
    bind_group: wgpu::BindGroup,
    size: (u32, u32),
}

impl PostStack {
    pub fn new(gpu: &GpuContext, size: (u32, u32)) -> Self {
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render post-stack bgl"),
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
                label: Some("ph2d-render post-stack layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render post-stack grade shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/post_stack.wgsl").into()),
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ph2d-render post-stack pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: crate::GameRt::FORMAT,
                        // Full overwrite of game_rt with the graded scratch — no blend.
                        blend: None,
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
            });

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render post-stack sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let uniform = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render post-stack uniform"),
            size: 48, // three vec4<f32>
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let scratch = make_scratch(gpu, size);
        let bind_group = make_bind_group(gpu, &bgl, &scratch.view, &sampler, &uniform);

        Self {
            bgl,
            sampler,
            pipeline,
            uniform,
            scratch,
            bind_group,
            size,
        }
    }

    /// Recreate the scratch RT + bind group if the surface size changed. Call alongside
    /// `game_rt.ensure_size`.
    pub fn ensure_size(&mut self, gpu: &GpuContext, size: (u32, u32)) {
        if size == self.size || size.0 == 0 || size.1 == 0 {
            return;
        }
        self.scratch = make_scratch(gpu, size);
        self.bind_group = make_bind_group(
            gpu,
            &self.bgl,
            &self.scratch.view,
            &self.sampler,
            &self.uniform,
        );
        self.size = size;
    }

    /// Grade `game_rt` in place: copy it to the scratch, then a fullscreen pass reads
    /// the scratch, applies `params` in HDR, and rewrites `game_rt`. `game_tex` must be
    /// the same-format (`Rgba16Float`), same-size texture backing `game_view`, and carry
    /// `COPY_SRC` (the shell's `game_rt` does).
    ///
    /// The shell only calls this when `!params.is_neutral()` — at neutral the whole
    /// block is skipped (ADR-0145, byte-identical).
    pub fn grade(
        &self,
        gpu: &GpuContext,
        game_tex: &wgpu::Texture,
        game_view: &wgpu::TextureView,
        params: &GradeParams,
    ) {
        let aspect = self.size.0.max(1) as f32 / self.size.1.max(1) as f32;
        gpu.queue
            .write_buffer(&self.uniform, 0, bytemuck::cast_slice(&params.pack(aspect)));

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render post-stack encoder"),
            });

        // 1. copy game_rt → scratch (the frozen input the grade reads; game_rt is both
        //    read and written this frame, so it must be copied out first — no hazard).
        let extent = wgpu::Extent3d {
            width: self.size.0.max(1),
            height: self.size.1.max(1),
            depth_or_array_layers: 1,
        };
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: game_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.scratch.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            extent,
        );

        // 2. fullscreen grade: scratch → game_rt.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-render post-stack pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: game_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Full overwrite — the grade writes every pixel.
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: ph2d_gpu::pass_profiler::render_writes("render.post_stack"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit(Some(encoder.finish()));
    }
}

fn make_bind_group(
    gpu: &GpuContext,
    bgl: &wgpu::BindGroupLayout,
    src: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    uniform: &wgpu::Buffer,
) -> wgpu::BindGroup {
    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ph2d-render post-stack bg"),
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
                resource: uniform.as_entire_binding(),
            },
        ],
    })
}

#[cfg(test)]
#[path = "post_stack_tests.rs"]
mod tests;
