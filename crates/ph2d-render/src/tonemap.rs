//! Tonemap pass (M14.5) — HDR (Rgba16Float) → LDR (Bgra8UnormSrgb)
//! via a 3D LUT.
//!
//! # Pipeline placement
//!
//! ```text
//!     game_rt (Rgba16Float HDR)
//!           ▼
//!     Tonemap (this pass) ─── samples 3D LUT 33³
//!           ▼
//!     game_rt_ldr (Bgra8UnormSrgb)
//! ```
//!
//! The downstream [`Compositor`](crate::compositor::Compositor)
//! consumes `game_rt_ldr`, so the LDR target's format MUST equal
//! [`Tonemap::OUTPUT_FORMAT`].
//!
//! # AgX
//!
//! The shader expects an AgX Log2-encoded input domain (see
//! [`shaders/tonemap.wgsl`](../shaders/tonemap.wgsl)). The LUT
//! itself is loaded from raw bytes via [`Tonemap::new`]; supply
//! `None` to get an **identity LUT** (2³ trilinear identity volume).
//!
//! ## Current state: bypassed
//!
//! The shader sets `BYPASS_LUT = true` and returns
//! `clamp(hdr, 0, 1)` directly — see the long comment block at the
//! top of [`shaders/tonemap.wgsl`](../shaders/tonemap.wgsl) for the
//! rationale and the **migration trigger** that flips bypass off.
//! All the bind-group / sampler / pipeline wiring is kept live so
//! that flipping the const back to `false` (plus loading a real AgX
//! LUT via [`Tonemap::set_lut`]) is a one-line activation when HDR
//! content arrives.

use ph2d_gpu::GpuContext;

pub struct Tonemap {
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    sampler_2d: wgpu::Sampler,
    sampler_3d: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    /// Currently-bound LUT texture + view. The texture must stay
    /// alive for the view to remain valid; both are owned here.
    /// Replaced together via `set_lut`.
    #[allow(dead_code)]
    lut_texture: wgpu::Texture,
    lut_view: wgpu::TextureView,
    /// Most recently sampled game RT view — needed to rebuild the
    /// bind group whenever the host resizes its game RT and we
    /// receive a new view.
    game_view: wgpu::TextureView,
    /// LDR output texture owned by this pass. Sized to match the
    /// bound `game_view`; the host calls `ensure_size` whenever the
    /// surface resizes.
    output_texture: wgpu::Texture,
    output_view: wgpu::TextureView,
    output_size: (u32, u32),
}

impl Tonemap {
    /// LDR output format. **Must match** the texture you bind as
    /// the render target when calling [`run`](Self::run).
    pub const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

    pub fn new(gpu: &GpuContext, game_view: wgpu::TextureView, size: (u32, u32)) -> Self {
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render tonemap bgl"),
                entries: &[
                    // game_rt
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
                    // game_sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // agx_lut
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D3,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // lut_sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-render tonemap layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render tonemap shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/tonemap.wgsl").into()),
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ph2d-render tonemap pipeline"),
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
                        format: Self::OUTPUT_FORMAT,
                        // Tonemap is the SOLE writer of game_rt_ldr;
                        // no blend needed (full overwrite).
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

        let sampler_2d = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render tonemap 2D sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let sampler_3d = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-render tonemap 3D LUT sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let (lut_texture, lut_view) = build_identity_lut(gpu);
        let (output_texture, output_view) = build_ldr_output(gpu, size);

        let bind_group =
            make_bind_group(gpu, &bgl, &game_view, &sampler_2d, &lut_view, &sampler_3d);

        Self {
            pipeline,
            bgl,
            sampler_2d,
            sampler_3d,
            bind_group,
            lut_texture,
            lut_view,
            game_view,
            output_texture,
            output_view,
            output_size: size,
        }
    }

    /// View of the LDR output texture. Bound by the
    /// [`Compositor`](crate::Compositor) as its "game" input.
    pub fn output_view(&self) -> &wgpu::TextureView {
        &self.output_view
    }

    /// Underlying LDR texture. Exposed so callers (typically the
    /// [`Compositor`](crate::Compositor)) can create additional
    /// views — `wgpu::TextureView` is not `Clone` in wgpu 28.
    pub fn output_texture(&self) -> &wgpu::Texture {
        &self.output_texture
    }

    /// Recreate the LDR output texture if the requested size changes.
    /// Must be called BEFORE [`rebind_game_view`](Self::rebind_game_view)
    /// when the host resizes the surface.
    pub fn ensure_size(&mut self, gpu: &GpuContext, size: (u32, u32)) {
        if size == self.output_size || size.0 == 0 || size.1 == 0 {
            return;
        }
        let (tex, view) = build_ldr_output(gpu, size);
        self.output_texture = tex;
        self.output_view = view;
        self.output_size = size;
    }

    /// Rebuild the bind group with a new game RT view (called by the
    /// host whenever it resizes the game RT and gets a new view).
    pub fn rebind_game_view(&mut self, gpu: &GpuContext, game_view: wgpu::TextureView) {
        self.game_view = game_view;
        self.bind_group = make_bind_group(
            gpu,
            &self.bgl,
            &self.game_view,
            &self.sampler_2d,
            &self.lut_view,
            &self.sampler_3d,
        );
    }

    /// Replace the AgX LUT. `bytes` must contain `33×33×33×4` bytes
    /// of `Rgba8Unorm` (R, G, B, A each 8-bit), row-major (R fastest,
    /// B slowest). Linear-light values in [0, 1] — the shader feeds
    /// them straight into the sRGB target which wgpu encodes on
    /// write. The host bakes this offline from Blender's AgX
    /// configuration; see `tools/lut-bake/`.
    pub fn set_lut(&mut self, gpu: &GpuContext, bytes: &[u8]) -> Result<(), String> {
        const N: u32 = 33;
        const EXPECTED: usize = (N * N * N) as usize * 4; // rgba8 unorm
        if bytes.len() != EXPECTED {
            return Err(format!(
                "AgX LUT size mismatch: expected {EXPECTED} bytes ({N}³ rgba8), got {}",
                bytes.len()
            ));
        }
        let (texture, view) = build_lut_from_bytes(gpu, N, bytes);
        self.lut_texture = texture;
        self.lut_view = view;
        self.bind_group = make_bind_group(
            gpu,
            &self.bgl,
            &self.game_view,
            &self.sampler_2d,
            &self.lut_view,
            &self.sampler_3d,
        );
        Ok(())
    }

    /// Run the tonemap pass: sample `game_rt` (the view bound at
    /// construction or via `rebind_game_view`), apply AgX, write to
    /// the owned LDR output (see [`output_view`](Self::output_view)).
    pub fn run(&self, gpu: &GpuContext) {
        let output_view = &self.output_view;
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render tonemap encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-render tonemap pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: ph2d_gpu::pass_profiler::render_writes("render.tonemap"),
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

fn build_ldr_output(gpu: &GpuContext, size: (u32, u32)) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render game_rt_ldr (tonemap output)"),
        size: wgpu::Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Tonemap::OUTPUT_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn make_bind_group(
    gpu: &GpuContext,
    bgl: &wgpu::BindGroupLayout,
    game_view: &wgpu::TextureView,
    sampler_2d: &wgpu::Sampler,
    lut_view: &wgpu::TextureView,
    sampler_3d: &wgpu::Sampler,
) -> wgpu::BindGroup {
    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ph2d-render tonemap bg"),
        layout: bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(game_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler_2d),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(lut_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(sampler_3d),
            },
        ],
    })
}

/// Build a 2×2×2 identity 3D LUT (Rgba8Unorm). Each corner stores
/// its own coordinate as a (R, G, B) triple — trilinear sampling
/// reconstructs the input domain. 32 bytes upload.
fn build_identity_lut(gpu: &GpuContext) -> (wgpu::Texture, wgpu::TextureView) {
    const N: u32 = 2;
    let mut bytes = Vec::<u8>::with_capacity((N * N * N) as usize * 4);
    for z in 0..N {
        for y in 0..N {
            for x in 0..N {
                let denom = (N - 1) as f32;
                bytes.push((x as f32 / denom * 255.0).round() as u8);
                bytes.push((y as f32 / denom * 255.0).round() as u8);
                bytes.push((z as f32 / denom * 255.0).round() as u8);
                bytes.push(255);
            }
        }
    }
    build_lut_from_bytes(gpu, N, &bytes)
}

fn build_lut_from_bytes(
    gpu: &GpuContext,
    n: u32,
    bytes: &[u8],
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render AgX LUT"),
        size: wgpu::Extent3d {
            width: n,
            height: n,
            depth_or_array_layers: n,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        // M14.5 (revised): Rgba8Unorm is enough for the AgX output
        // curve — trilinear interpolation in a 33³ LUT is visually
        // indistinguishable from 65³ to the human eye, and 8-bit
        // precision is below display contrast threshold. Values are
        // linear-light [0, 1]; the downstream tonemap output is
        // *Srgb so wgpu encodes on write.
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(n * 4),
            rows_per_image: Some(n),
        },
        wgpu::Extent3d {
            width: n,
            height: n,
            depth_or_array_layers: n,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D3),
        ..Default::default()
    });
    (texture, view)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_format_is_srgb() {
        assert_eq!(Tonemap::OUTPUT_FORMAT, wgpu::TextureFormat::Bgra8UnormSrgb);
    }

    /// AgX log-encoding pinned constants from Blender's reference
    /// config (`AgX-default_contrast.cube` header). Regression on
    /// these stops the LUT bake tool from drifting silently.
    #[test]
    fn agx_log_constants_match_blender_reference() {
        // We don't expose the WGSL constants, but the shader source
        // is plain text — assert by string match.
        let src = include_str!("shaders/tonemap.wgsl");
        assert!(
            src.contains("AGX_MIN_EV: f32 = -12.47393"),
            "AGX_MIN_EV constant drifted from Blender 4.0+ reference"
        );
        assert!(
            src.contains("AGX_MAX_EV: f32 = 4.026069"),
            "AGX_MAX_EV constant drifted from Blender 4.0+ reference"
        );
    }
}
