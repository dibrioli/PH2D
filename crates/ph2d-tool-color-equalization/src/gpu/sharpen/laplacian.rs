//! Laplacian sharpen compute pipeline (radius ≤ 1). Single compute pass,
//! 4 neighbour reads per pixel. Split out of `gpu/sharpen` — mechanical
//! move, no behaviour change.

use super::WORKGROUP_SIZE;
use super::wgsl::LAPLACIAN_WGSL;
use crate::gpu::{make_input_texture, make_storage_texture, readback_into};
use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

/// Compiled pipeline + bind-group layout for the Laplacian sharpen
/// kernel (3×3, 4-neighbour Laplacian + identity blend by `amount`).
pub struct LaplacianSharpenPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct LaplacianUniforms {
    amount: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

impl LaplacianSharpenPipeline {
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ceq.sharpen_laplacian.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(LAPLACIAN_WGSL)),
            });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ceq.sharpen_laplacian.bgl"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: wgpu::TextureFormat::Rgba8Unorm,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                    LaplacianUniforms,
                                >(
                                )
                                    as u64),
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ceq.sharpen_laplacian.layout"),
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ceq.sharpen_laplacian.pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    /// Wraps [`Self::encode_into`] with upload + readback.
    pub fn dispatch(&self, gpu: &GpuContext, rgba: &mut [u8], w: u32, h: u32, amount: f32) {
        let expected = (w as usize) * (h as usize) * 4;
        assert_eq!(rgba.len(), expected, "rgba length must match w*h*4");
        if amount <= 0.0 || w == 0 || h == 0 {
            return;
        }
        let input_tex = make_input_texture(gpu, "ceq.sharpen_laplacian.input", rgba, w, h);
        let output_tex = make_storage_texture(gpu, "ceq.sharpen_laplacian.output", w, h);
        let input_view = input_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ceq.sharpen_laplacian.encoder"),
            });
        self.encode_into(gpu, &mut encoder, &input_view, &output_view, w, h, amount);
        readback_into(&mut encoder, gpu, &output_tex, rgba, w, h);
    }

    /// Encode the Laplacian compute pass into `encoder`.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_into(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        w: u32,
        h: u32,
        amount: f32,
    ) {
        if amount <= 0.0 || w == 0 || h == 0 {
            return;
        }
        let uniforms = LaplacianUniforms {
            amount,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        let uniform_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ceq.sharpen_laplacian.uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ceq.sharpen_laplacian.bind_group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(output_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buf.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ceq.sharpen_laplacian.pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(w.div_ceil(WORKGROUP_SIZE), h.div_ceil(WORKGROUP_SIZE), 1);
    }
}

/// Convenience wrapper: build pipeline + dispatch.
pub fn sharpen_laplacian_gpu(rgba: &mut [u8], w: u32, h: u32, amount: f32, gpu: &GpuContext) {
    let pipeline = LaplacianSharpenPipeline::new(gpu);
    pipeline.dispatch(gpu, rgba, w, h, amount);
}
