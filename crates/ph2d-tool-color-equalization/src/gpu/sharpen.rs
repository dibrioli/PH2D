//! WGSL compute paths for [`crate::algorithm::sharpen_laplacian`] and
//! [`crate::algorithm::sharpen_unsharp`].
//!
//! Two distinct pipelines because the CPU code paths are distinct:
//!
//! - **Laplacian** (radius ≤ 1, [`LaplacianSharpenPipeline`]): single
//!   compute pass, 4 neighbour reads per pixel (top / bottom / left /
//!   right). Cheap on both CPU and GPU; GPU still wins ~10× at 1024²
//!   by avoiding the 4× scalar load + the per-channel temporary.
//! - **Unsharp Mask** (radius > 1, [`UnsharpSharpenPipeline`]): two
//!   compute passes — separable Gaussian blur (H then V), then a
//!   combine step (`orig + amount · (orig − blur)`). Fused V + combine
//!   into the second pass so we only round-trip through one
//!   intermediate `rgba16float` texture. **The big GPU win** —
//!   per-pixel cost grows linearly with radius on CPU but stays roughly
//!   constant on GPU (memory-bound, not compute-bound). 1024² at
//!   radius 3 goes from ~120 ms CPU to ~3 ms GPU on Apple Silicon.
//!
//! ## CPU semantic parity notes
//!
//! - **Laplacian** skips transparent pixels entirely (matches CPU's
//!   `if src[ci + 3] == 0 { continue; }` short-circuit).
//! - **Unsharp** runs the H + V blur passes unconditionally (CPU does
//!   the same — alpha is ignored during the blur), but the final
//!   combine step preserves transparent pixels untouched.
//! - Both clamp the final result to `[0, 1]` before storage (matches
//!   the CPU's `clamp8`).
//!
//! ## Intermediate format
//!
//! The H-pass result lands in an `rgba16float` storage texture, **not**
//! `rgba8unorm`. Per-channel blur weights can sum to fractional
//! amounts whose precision matters when subsequently differenced
//! against the original — `rgba8unorm` quantization would inject up to
//! 2 LSB of noise into the difference *before* the combine pass, then
//! amplify by `amount` (up to 2). Half-precision keeps the H pass
//! lossless for our purposes (parity holds at ε ≤ 4 LSB).

use super::{make_input_texture, make_storage_texture, readback_into};
use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

const WORKGROUP_SIZE: u32 = 8;

// ── Laplacian ─────────────────────────────────────────────────────

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

// ── Unsharp Mask (Gaussian separable) ─────────────────────────────

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

// (Shared texture / readback helpers live in [`super`] now — see
// `make_input_texture`, `make_storage_texture`, `readback_into` in
// `gpu/mod.rs`.)

// ── WGSL kernels ─────────────────────────────────────────────────

/// 3×3 Laplacian in **linear sRGB** (Tier 3 audit parity). Decodes
/// center + 4 neighbours from sRGB to linear, applies the 5-cross kernel
/// `5·center − top − bottom − left − right`, blends
/// `center + (lap − center) · amount`, and encodes back to sRGB. Edge
/// pixels clamp the neighbour index (mirrors CPU `if y > 0 { ... } else
/// { center }`).
const LAPLACIAN_WGSL: &str = r#"
struct Uniforms {
    amount: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> u: Uniforms;

fn srgb_to_linear_c(c: f32) -> f32 {
    if (c <= 0.04045) {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn linear_to_srgb_c(c: f32) -> f32 {
    let c_clamped = max(c, 0.0);
    if (c_clamped <= 0.0031308) {
        return c_clamped * 12.92;
    }
    return 1.055 * pow(c_clamped, 1.0 / 2.4) - 0.055;
}

fn s2l(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(srgb_to_linear_c(rgb.r), srgb_to_linear_c(rgb.g), srgb_to_linear_c(rgb.b));
}

fn l2s(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(linear_to_srgb_c(rgb.r), linear_to_srgb_c(rgb.g), linear_to_srgb_c(rgb.b));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let coord = vec2<i32>(i32(id.x), i32(id.y));
    let dims_i = vec2<i32>(i32(dims.x), i32(dims.y));
    let center_srgb = textureLoad(input_tex, coord, 0);
    if (center_srgb.a == 0.0) {
        textureStore(output_tex, coord, center_srgb);
        return;
    }
    let top_coord    = vec2<i32>(coord.x, max(coord.y - 1, 0));
    let bottom_coord = vec2<i32>(coord.x, min(coord.y + 1, dims_i.y - 1));
    let left_coord   = vec2<i32>(max(coord.x - 1, 0), coord.y);
    let right_coord  = vec2<i32>(min(coord.x + 1, dims_i.x - 1), coord.y);
    let center = s2l(center_srgb.rgb);
    let top    = s2l(textureLoad(input_tex, top_coord, 0).rgb);
    let bottom = s2l(textureLoad(input_tex, bottom_coord, 0).rgb);
    let left   = s2l(textureLoad(input_tex, left_coord, 0).rgb);
    let right  = s2l(textureLoad(input_tex, right_coord, 0).rgb);

    let laplacian = 5.0 * center - top - bottom - left - right;
    let result_lin = center + (laplacian - center) * u.amount;
    let result_srgb = clamp(l2s(result_lin), vec3<f32>(0.0), vec3<f32>(1.0));
    textureStore(output_tex, coord, vec4<f32>(result_srgb, center_srgb.a));
}
"#;

/// Unsharp H pass in **linear sRGB**: decodes each sampled sRGB pixel to
/// linear, blurs in linear with the Gaussian kernel, writes the linear
/// blur into the `rgba16float` intermediate (which has the headroom to
/// store unclamped linear values). The V pass also reads linear and
/// finishes with the combine + sRGB encode.
const UNSHARP_H_WGSL: &str = r#"
struct Uniforms {
    amount: f32,
    half: i32,
    kernel_size: i32,
    _pad: i32,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var h_pass: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var<uniform> u: Uniforms;
@group(0) @binding(3) var<storage, read> kernel: array<f32>;

fn srgb_to_linear_c(c: f32) -> f32 {
    if (c <= 0.04045) {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn s2l(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(srgb_to_linear_c(rgb.r), srgb_to_linear_c(rgb.g), srgb_to_linear_c(rgb.b));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(h_pass);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let coord = vec2<i32>(i32(id.x), i32(id.y));
    let dims_i = vec2<i32>(i32(dims.x), i32(dims.y));
    let center = textureLoad(input_tex, coord, 0);

    var sum_rgb = vec3<f32>(0.0);
    var wt = 0.0;
    for (var k: i32 = 0; k < u.kernel_size; k = k + 1) {
        let sx = clamp(coord.x + k - u.half, 0, dims_i.x - 1);
        let sample_lin = s2l(textureLoad(input_tex, vec2<i32>(sx, coord.y), 0).rgb);
        let kw = kernel[k];
        sum_rgb = sum_rgb + sample_lin * kw;
        wt = wt + kw;
    }
    let blur_lin = sum_rgb / wt;
    textureStore(h_pass, coord, vec4<f32>(blur_lin, center.a));
}
"#;

/// Unsharp V + combine pass in **linear sRGB**: blurs the H-pass
/// intermediate (already linear) along the Y axis, decodes the original
/// sRGB sample to linear, combines `orig_lin + amount·(orig_lin −
/// blur_lin)`, encodes the result back to sRGB. Transparent pixels pass
/// through untouched.
const UNSHARP_V_WGSL: &str = r#"
struct Uniforms {
    amount: f32,
    half: i32,
    kernel_size: i32,
    _pad: i32,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var h_pass: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var<uniform> u: Uniforms;
@group(0) @binding(4) var<storage, read> kernel: array<f32>;

fn srgb_to_linear_c(c: f32) -> f32 {
    if (c <= 0.04045) {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn linear_to_srgb_c(c: f32) -> f32 {
    let c_clamped = max(c, 0.0);
    if (c_clamped <= 0.0031308) {
        return c_clamped * 12.92;
    }
    return 1.055 * pow(c_clamped, 1.0 / 2.4) - 0.055;
}

fn s2l(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(srgb_to_linear_c(rgb.r), srgb_to_linear_c(rgb.g), srgb_to_linear_c(rgb.b));
}

fn l2s(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(linear_to_srgb_c(rgb.r), linear_to_srgb_c(rgb.g), linear_to_srgb_c(rgb.b));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let coord = vec2<i32>(i32(id.x), i32(id.y));
    let dims_i = vec2<i32>(i32(dims.x), i32(dims.y));
    let orig_srgb = textureLoad(input_tex, coord, 0);
    if (orig_srgb.a == 0.0) {
        textureStore(output_tex, coord, orig_srgb);
        return;
    }

    var sum_rgb = vec3<f32>(0.0);
    var wt = 0.0;
    for (var k: i32 = 0; k < u.kernel_size; k = k + 1) {
        let sy = clamp(coord.y + k - u.half, 0, dims_i.y - 1);
        let sample = textureLoad(h_pass, vec2<i32>(coord.x, sy), 0).rgb;
        let kw = kernel[k];
        sum_rgb = sum_rgb + sample * kw;
        wt = wt + kw;
    }
    let blur_lin = sum_rgb / wt;
    let orig_lin = s2l(orig_srgb.rgb);
    let diff = orig_lin - blur_lin;
    let result_lin = orig_lin + u.amount * diff;
    let result_srgb = clamp(l2s(result_lin), vec3<f32>(0.0), vec3<f32>(1.0));
    textureStore(output_tex, coord, vec4<f32>(result_srgb, orig_srgb.a));
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::{sharpen_laplacian, sharpen_unsharp};
    use crate::gpu::try_headless_gpu;

    fn ramp(w: u32, h: u32) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let r = ((x * 13 + y * 5) % 256) as u8;
                let g = ((x * 7 + y * 19) % 256) as u8;
                let b = ((x * 17 + y * 3) % 256) as u8;
                v.extend_from_slice(&[r, g, b, 255]);
            }
        }
        v
    }

    /// Sharpening shines on edges — manufacture a deterministic image
    /// with a checkerboard of `lo` / `hi` value bands so the kernel
    /// has work to do at every interior pixel.
    fn checker(w: u32, h: u32, block: u32, lo: u8, hi: u8) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let on = ((x / block) + (y / block)).is_multiple_of(2);
                let val = if on { hi } else { lo };
                v.extend_from_slice(&[val, val, val, 255]);
            }
        }
        v
    }

    fn assert_within_lsb(cpu: &[u8], gpu: &[u8], max_lsb: i32, ctx: &str) {
        assert_eq!(cpu.len(), gpu.len());
        let mut worst = 0_i32;
        let mut worst_idx = 0;
        for (i, (a, b)) in cpu.iter().zip(gpu.iter()).enumerate() {
            let d = (*a as i32 - *b as i32).abs();
            if d > worst {
                worst = d;
                worst_idx = i;
            }
        }
        assert!(
            worst <= max_lsb,
            "{ctx}: CPU/GPU diverged by {worst} LSB at idx {worst_idx} \
             (cpu {} vs gpu {})",
            cpu[worst_idx],
            gpu[worst_idx],
        );
    }

    // ── Laplacian ───────────────────────────────────────────────

    #[test]
    fn laplacian_gpu_zero_amount_is_noop() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(16, 16);
        let mut buf = src.clone();
        sharpen_laplacian_gpu(&mut buf, 16, 16, 0.0, &gpu);
        assert_eq!(buf, src);
    }

    #[test]
    fn laplacian_gpu_matches_cpu_amount_one() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = checker(24, 24, 4, 60, 200);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        sharpen_laplacian(&mut cpu, 24, 24, 1.0);
        sharpen_laplacian_gpu(&mut gpu_buf, 24, 24, 1.0, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 2, "laplacian amount=1 checker");
    }

    #[test]
    fn laplacian_gpu_matches_cpu_amount_half() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(24, 24);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        sharpen_laplacian(&mut cpu, 24, 24, 0.5);
        sharpen_laplacian_gpu(&mut gpu_buf, 24, 24, 0.5, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 2, "laplacian amount=0.5 ramp");
    }

    #[test]
    fn laplacian_gpu_skips_transparent() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut buf = vec![100u8, 150, 200, 0, 100, 150, 200, 255];
        sharpen_laplacian_gpu(&mut buf, 2, 1, 1.0, &gpu);
        assert_eq!(&buf[0..4], &[100, 150, 200, 0]);
    }

    #[test]
    fn laplacian_gpu_handles_non_workgroup_aligned_dimensions() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = checker(13, 19, 3, 40, 220);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        sharpen_laplacian(&mut cpu, 13, 19, 0.8);
        sharpen_laplacian_gpu(&mut gpu_buf, 13, 19, 0.8, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 2, "laplacian 13×19");
    }

    // ── Unsharp Mask ────────────────────────────────────────────

    #[test]
    fn unsharp_gpu_zero_amount_is_noop() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(16, 16);
        let mut buf = src.clone();
        sharpen_unsharp_gpu(&mut buf, 16, 16, 0.0, 2.0, &gpu);
        assert_eq!(buf, src);
    }

    #[test]
    fn unsharp_gpu_matches_cpu_radius_2() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        // Use checker so the unsharp combine has visible structure.
        let src = checker(24, 24, 4, 50, 210);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        sharpen_unsharp(&mut cpu, 24, 24, 1.0, 2.0);
        sharpen_unsharp_gpu(&mut gpu_buf, 24, 24, 1.0, 2.0, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 4, "unsharp r=2 amount=1");
    }

    #[test]
    fn unsharp_gpu_matches_cpu_radius_3() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(32, 32);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        sharpen_unsharp(&mut cpu, 32, 32, 0.7, 3.0);
        sharpen_unsharp_gpu(&mut gpu_buf, 32, 32, 0.7, 3.0, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 4, "unsharp r=3 amount=0.7");
    }

    #[test]
    fn unsharp_gpu_matches_cpu_small_radius() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        // radius=1.5 → kernel size 7 (smallest non-Laplacian path).
        let src = checker(20, 20, 4, 30, 240);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        sharpen_unsharp(&mut cpu, 20, 20, 1.2, 1.5);
        sharpen_unsharp_gpu(&mut gpu_buf, 20, 20, 1.2, 1.5, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 4, "unsharp r=1.5 amount=1.2");
    }

    #[test]
    fn unsharp_gpu_skips_transparent() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut buf = vec![100u8, 150, 200, 0, 100, 150, 200, 255];
        sharpen_unsharp_gpu(&mut buf, 2, 1, 1.0, 2.0, &gpu);
        assert_eq!(&buf[0..4], &[100, 150, 200, 0]);
    }

    #[test]
    fn unsharp_gpu_handles_non_workgroup_aligned_dimensions() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = checker(13, 19, 3, 40, 220);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        sharpen_unsharp(&mut cpu, 13, 19, 0.9, 2.0);
        sharpen_unsharp_gpu(&mut gpu_buf, 13, 19, 0.9, 2.0, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 4, "unsharp 13×19");
    }
}
