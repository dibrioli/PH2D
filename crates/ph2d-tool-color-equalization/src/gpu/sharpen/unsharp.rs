//! Unsharp Mask compute pipeline (radius > 1) — separable Gaussian blur
//! (H pass) + fused V-and-combine pass through an `rgba16float`
//! intermediate. Split out of `gpu/sharpen` — mechanical move, no
//! behaviour change.

use super::WORKGROUP_SIZE;
use super::wgsl::{UNSHARP_H_WGSL, UNSHARP_V_WGSL};
use crate::gpu::{make_input_texture, make_storage_texture, readback_into};
use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

/// Compiled pipelines (H pass + V-and-combine pass) for the Unsharp
/// Mask kernel. The blur intermediate lives in an `rgba16float`
/// storage texture to keep enough precision for the subsequent
/// `orig - blur` differencing.
pub struct UnsharpSharpenPipeline {
    h_pipeline: wgpu::ComputePipeline,
    h_bind_group_layout: wgpu::BindGroupLayout,
    v_pipeline: wgpu::ComputePipeline,
    v_bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct UnsharpUniforms {
    amount: f32,
    /// Half-side of the kernel (`(kernel_size - 1) / 2`). Used to map
    /// `k ∈ [0, kernel_size)` back to a signed offset `k − half`.
    half: i32,
    kernel_size: i32,
    _pad: i32,
}

impl UnsharpSharpenPipeline {
    pub fn new(gpu: &GpuContext) -> Self {
        let h_shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ceq.sharpen_unsharp_h.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(UNSHARP_H_WGSL)),
            });
        let v_shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ceq.sharpen_unsharp_v.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(UNSHARP_V_WGSL)),
            });

        let h_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ceq.sharpen_unsharp_h.bgl"),
                    entries: &[
                        // 0: input rgba8unorm.
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
                        // 1: H-pass output (rgba16float storage).
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: wgpu::TextureFormat::Rgba16Float,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        // 2: uniforms.
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                    UnsharpUniforms,
                                >(
                                )
                                    as u64),
                            },
                            count: None,
                        },
                        // 3: kernel weights (storage, read-only — tight
                        //    packing avoids uniform-array 16-byte stride).
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let v_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ceq.sharpen_unsharp_v.bgl"),
                    entries: &[
                        // 0: original rgba8unorm input (for combine).
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
                        // 1: H-pass intermediate (rgba16float).
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // 2: final output rgba8unorm.
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: wgpu::TextureFormat::Rgba8Unorm,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        // 3: uniforms.
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                    UnsharpUniforms,
                                >(
                                )
                                    as u64),
                            },
                            count: None,
                        },
                        // 4: kernel.
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let h_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ceq.sharpen_unsharp_h.layout"),
                    bind_group_layouts: &[&h_bind_group_layout],
                    immediate_size: 0,
                });
        let v_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ceq.sharpen_unsharp_v.layout"),
                    bind_group_layouts: &[&v_bind_group_layout],
                    immediate_size: 0,
                });

        let h_pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ceq.sharpen_unsharp_h.pipeline"),
                layout: Some(&h_pipeline_layout),
                module: &h_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });
        let v_pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ceq.sharpen_unsharp_v.pipeline"),
                layout: Some(&v_pipeline_layout),
                module: &v_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            h_pipeline,
            h_bind_group_layout,
            v_pipeline,
            v_bind_group_layout,
        }
    }

    /// Wraps [`Self::encode_into`] with upload + readback.
    pub fn dispatch(
        &self,
        gpu: &GpuContext,
        rgba: &mut [u8],
        w: u32,
        h: u32,
        amount: f32,
        radius: f32,
    ) {
        let expected = (w as usize) * (h as usize) * 4;
        assert_eq!(rgba.len(), expected, "rgba length must match w*h*4");
        if amount <= 0.0 || radius <= 0.0 || w == 0 || h == 0 {
            return;
        }
        let input_tex = make_input_texture(gpu, "ceq.sharpen_unsharp.input", rgba, w, h);
        let output_tex = make_storage_texture(gpu, "ceq.sharpen_unsharp.output", w, h);
        let input_view = input_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ceq.sharpen_unsharp.encoder"),
            });
        self.encode_into(
            gpu,
            &mut encoder,
            &input_view,
            &output_view,
            w,
            h,
            amount,
            radius,
        );
        readback_into(&mut encoder, gpu, &output_tex, rgba, w, h);
    }

    /// Encode the H + V combine compute passes into `encoder`. The
    /// rgba16float intermediate texture is created here and kept alive
    /// via the V bind group's Arc retention. `input_view` is sampled by
    /// BOTH passes (H reads original RGB; V reads original alpha + diffs
    /// against original RGB), so the chain caller must keep that texture
    /// alive for the entire encoder lifetime — which it would anyway,
    /// since the chain's input ping-pong texture is the same.
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
        radius: f32,
    ) {
        if amount <= 0.0 || radius <= 0.0 || w == 0 || h == 0 {
            return;
        }
        let kernel = crate::algorithm::gaussian_kernel_1d(radius);
        let kernel_size = kernel.len() as i32;
        let half = (kernel.len() / 2) as i32;
        let uniforms = UnsharpUniforms {
            amount,
            half,
            kernel_size,
            _pad: 0,
        };
        let intermediate_tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ceq.sharpen_unsharp.intermediate"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let uniform_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ceq.sharpen_unsharp.uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let kernel_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ceq.sharpen_unsharp.kernel"),
                contents: bytemuck::cast_slice(&kernel),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let intermediate_view =
            intermediate_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let h_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ceq.sharpen_unsharp_h.bind_group"),
            layout: &self.h_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&intermediate_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: kernel_buf.as_entire_binding(),
                },
            ],
        });
        let v_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ceq.sharpen_unsharp_v.bind_group"),
            layout: &self.v_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&intermediate_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(output_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: kernel_buf.as_entire_binding(),
                },
            ],
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ceq.sharpen_unsharp.h_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.h_pipeline);
            pass.set_bind_group(0, &h_bind_group, &[]);
            pass.dispatch_workgroups(w.div_ceil(WORKGROUP_SIZE), h.div_ceil(WORKGROUP_SIZE), 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ceq.sharpen_unsharp.v_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.v_pipeline);
            pass.set_bind_group(0, &v_bind_group, &[]);
            pass.dispatch_workgroups(w.div_ceil(WORKGROUP_SIZE), h.div_ceil(WORKGROUP_SIZE), 1);
        }
    }
}

/// Convenience: build pipelines + dispatch.
pub fn sharpen_unsharp_gpu(
    rgba: &mut [u8],
    w: u32,
    h: u32,
    amount: f32,
    radius: f32,
    gpu: &GpuContext,
) {
    let pipeline = UnsharpSharpenPipeline::new(gpu);
    pipeline.dispatch(gpu, rgba, w, h, amount, radius);
}
