//! Compositor pass (M14.5) — final viewport composite.
//!
//! ```text
//!     game_rt_ldr  ──┐
//!                    ├─▶ Compositor ─▶ swap chain
//!     vello_rt    ──┘
//! ```
//!
//! Implements the **viewport / RT pattern** standard across Bevy,
//! Godot, Unity, and Unreal: the game renders to its own offscreen
//! texture, the editor UI renders to its own intermediate, and a
//! single fragment shader (see [`shaders/compositor.wgsl`](../shaders/compositor.wgsl))
//! composes them onto the swap chain. There is no shared backbuffer
//! between game and UI passes — they can't accidentally overdraw
//! each other.
//!
//! The output color format is the swap-chain format (sRGB on
//! desktop), so wgpu encodes linear-light blend results to sRGB
//! automatically on write.

use ph2d_gpu::GpuContext;

pub struct Compositor {
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    surface_format: wgpu::TextureFormat,
    game_view: wgpu::TextureView,
    vello_view: wgpu::TextureView,
}

impl Compositor {
    pub fn new(
        gpu: &GpuContext,
        surface_format: wgpu::TextureFormat,
        game_view: wgpu::TextureView,
        vello_view: wgpu::TextureView,
    ) -> Self {
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render compositor bgl"),
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
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
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
                label: Some("ph2d-render compositor layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render compositor shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/compositor.wgsl").into()),
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ph2d-render compositor pipeline"),
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
                        format: surface_format,
                        // Compositor is the sole writer of the swap
                        // chain in M14.5 — no blend, full overwrite.
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
            label: Some("ph2d-render compositor sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = make_bind_group(gpu, &bgl, &game_view, &vello_view, &sampler);

        Self {
            pipeline,
            bgl,
            sampler,
            bind_group,
            surface_format,
            game_view,
            vello_view,
        }
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    /// Rebuild the bind group with new input views. Called after any
    /// resize that produces new texture views for either input.
    pub fn rebind(
        &mut self,
        gpu: &GpuContext,
        game_view: wgpu::TextureView,
        vello_view: wgpu::TextureView,
    ) {
        self.game_view = game_view;
        self.vello_view = vello_view;
        self.bind_group = make_bind_group(
            gpu,
            &self.bgl,
            &self.game_view,
            &self.vello_view,
            &self.sampler,
        );
    }

    /// Composite both inputs onto `output_view` (typically the swap
    /// chain frame view). `output_view` must reference a texture of
    /// the `surface_format` passed at construction.
    pub fn run(&self, gpu: &GpuContext, output_view: &wgpu::TextureView) {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render compositor encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-render compositor pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
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
    game_view: &wgpu::TextureView,
    vello_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ph2d-render compositor bg"),
        layout: bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(game_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(vello_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    /// Software model of the WGSL composite formula (straight-alpha
    /// `over`, matching `bevy_vello` / vello 0.8 output convention).
    /// Used by host integration tests to verify a known game + vello
    /// pair produces the expected swap-chain pixel. `vello_rgb` here
    /// is expected to already be linear-light (the WGSL applies an
    /// sRGB→linear decode upstream; this helper takes the post-decode
    /// values so the math stays focused on the composite itself).
    pub fn composite_straight(game_rgb: [f32; 3], vello_rgb: [f32; 3], vello_a: f32) -> [f32; 3] {
        let inv_a = 1.0 - vello_a;
        [
            vello_rgb[0] * vello_a + game_rgb[0] * inv_a,
            vello_rgb[1] * vello_a + game_rgb[1] * inv_a,
            vello_rgb[2] * vello_a + game_rgb[2] * inv_a,
        ]
    }

    #[test]
    fn vello_opaque_occludes_game() {
        // α=1 → out = vello.
        let out = composite_straight([0.0, 0.5, 0.5], [1.0, 0.0, 0.0], 1.0);
        assert_eq!(out, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn vello_transparent_passes_game() {
        // α=0 → out = game (vello.rgb irrelevant).
        let out = composite_straight([0.3, 0.6, 0.9], [1.0, 1.0, 1.0], 0.0);
        assert_eq!(out, [0.3, 0.6, 0.9]);
    }

    #[test]
    fn vello_half_alpha_blends_50_50() {
        // Straight α=0.5 with full white vello over full green game:
        // out = (1,1,1)*0.5 + (0,1,0)*0.5 = (0.5, 1.0, 0.5).
        let out = composite_straight([0.0, 1.0, 0.0], [1.0, 1.0, 1.0], 0.5);
        assert_eq!(out, [0.5, 1.0, 0.5]);
    }

    #[test]
    fn straight_alpha_avoids_premul_halo() {
        // Regression: under the OLD premultiplied math, a straight
        // vello pixel `(1, 1, 1, 0.5)` would have produced
        // `1 + game * 0.5` → values > 1 → halos. Straight-alpha
        // composite stays in-range and matches Porter-Duff "over".
        let out = composite_straight([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.5);
        assert_eq!(out, [0.5, 0.5, 0.5]);
        // Sanity: every channel ≤ 1.0 → no clipping/halo.
        for c in out {
            assert!(c <= 1.0, "channel {c} exceeded 1.0 — halo regression");
        }
    }
}
