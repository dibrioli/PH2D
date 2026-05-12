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
