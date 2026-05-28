//! SpritePipeline — the wgpu RenderPipeline + bind group layouts for
//! the sprite shader (`shaders/sprite.wgsl`).
//!
//! Per LLM1 audit §10.5: `PipelineLayout` is **always** explicit. Two
//! BindGroupLayouts (frame/material) match the shader's `@group(0)`
//! and `@group(1)` declarations. Per-instance data is via the second
//! vertex buffer slot (instance step mode) — cheaper than a third
//! bind group for streaming data. Single triangle-strip draw call per
//! batch.

use crate::sprite::{QuadVertex, RenderInstance};
use ph2d_gpu::GpuContext;

pub struct SpritePipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub frame_bgl: wgpu::BindGroupLayout,
    pub material_bgl: wgpu::BindGroupLayout,
}

impl SpritePipeline {
    pub fn new(gpu: &GpuContext, color_format: wgpu::TextureFormat) -> Self {
        let frame_bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render frame bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let material_bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render material bgl"),
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
                ],
            });

        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-render sprite layout"),
                bind_group_layouts: &[&frame_bgl, &material_bgl],
                immediate_size: 0,
            });

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render sprite shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sprite.wgsl").into()),
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ph2d-render sprite pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[QuadVertex::buffer_layout(), RenderInstance::buffer_layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: color_format,
                        // M14.5: premultiplied alpha throughout the
                        // pipeline (game_rt → tonemap → compositor).
                        // The fragment shader is responsible for
                        // emitting `color.rgb *= color.a` before
                        // return so sprites with α<1 composite
                        // correctly when multiple draw on top of
                        // each other inside the RT.
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
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

        Self {
            pipeline,
            frame_bgl,
            material_bgl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    /// Shared headless GPU (mirrors `game_rt`/`atlas` test helpers). Each
    /// adapter+device pair costs ~30-50 s cold on Apple Silicon, so cache
    /// it per test binary. Returns `None` on adapter-less environments so
    /// the test skips (passes) where wgpu can't spin up a device.
    fn try_headless_gpu() -> Option<GpuContext> {
        static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
        SHARED
            .get_or_init(|| {
                let instance = GpuContext::default_instance();
                GpuContext::new(instance, None).ok()
            })
            .clone()
    }

    #[test]
    fn sprite_pipeline_v4_shader_compiles_and_binds_eleven_attrs() {
        // The ONLY automated coverage of `SpritePipeline::new` (W1.T1.11
        // closes the T1.7a "pipeline unverified by CI" gap). Building the
        // pipeline runs naga's WGSL front-end + validator on
        // `shaders/sprite.wgsl` AND binds `RenderInstance::buffer_layout()`'s
        // 11 vertex attributes (@location 2..14) against the v4
        // `InstanceInput`. A WGSL syntax/type error (e.g. the new
        // per-corner `mix`, the `& Nu` flag decode, `select`, the `flat`
        // interpolation) or a location/format mismatch raises an
        // uncaptured validation error → wgpu panics → this test fails.
        //
        // Skips gracefully (passes) on adapter-less CI; runs on dev Macs +
        // Mac CI, which is where the Enio smoke also runs.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let _pipeline = SpritePipeline::new(&gpu, wgpu::TextureFormat::Rgba8UnormSrgb);
        // Reaching here means naga validated the v4 shader and the vertex
        // layout bound without an uncaptured device error.
    }
}
