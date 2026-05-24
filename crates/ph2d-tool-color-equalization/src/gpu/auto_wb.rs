//! WGSL compute path for [`crate::algorithm::auto_white_balance`] —
//! Gray-World auto white balance via a two-pass dispatch.
//!
//! Auto-WB is the only stage in the pipeline that needs a **reduction**
//! across the whole image (per-channel mean over opaque pixels) before
//! the per-pixel rescale can run. Two clean choices for the reduce:
//!
//! 1. **Global atomics** — each thread does 4 `atomicAdd` against the
//!    same handful of buffer slots. Simple, but contention scales with
//!    image size and serialises the writes through the memory system.
//! 2. **Workgroup-shared reduction** — each workgroup (8×8 = 64
//!    threads) accumulates its tile into `var<workgroup>` atomic
//!    counters; one thread per workgroup then commits the partial sum
//!    to global atomics. 64× fewer global atomics, single cache-line
//!    contention domain per workgroup.
//!
//! Option (2) is the canonical pattern and what this module
//! implements. The CPU↔GPU bounce in the middle (host reads sums,
//! computes gains, ships them to the apply shader) is acceptable for
//! the one-shot Apply path; for a real-time slider preview we'd add a
//! tiny third compute pass to compute gains GPU-side, but that is
//! premature here.
//!
//! ## Precision
//!
//! Per-pixel R / G / B are summed as `u32` after rounding the
//! normalised `[0, 1]` texture sample back to `u8` (`round(v · 255)`).
//! Total per channel ≤ `pixel_count · 255` — `u32` holds up to
//! `≈ 16M pixels · 255` before overflow, well above any sprite this
//! tool targets. The mean is computed host-side in `f32` (matching
//! CPU). Final per-pixel multiply + `clamp(0, 1)` matches CPU's
//! `clamp8(v as f32 * gain)`.
//!
//! Parity test ε ≤ 2 LSB (only one multiply + clamp per channel, no
//! transcendentals).

use super::{make_input_texture, make_storage_texture, readback_into};
use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

const WORKGROUP_SIZE: u32 = 8;
/// `[sum_r, sum_g, sum_b, count]` — four `atomic<u32>` slots.
const SUMS_SLOT_COUNT: u64 = 4;

/// Compiled pipelines + bind-group layouts for the reduce + apply
/// passes. Build once per [`GpuContext`]; [`Self::dispatch`] is
/// per-call.
pub struct AutoWbPipelines {
    reduce_pipeline: wgpu::ComputePipeline,
    reduce_bind_group_layout: wgpu::BindGroupLayout,
    apply_pipeline: wgpu::ComputePipeline,
    apply_bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct ReduceUniforms {
    width: u32,
    height: u32,
    _pad0: u32,
    _pad1: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct ApplyUniforms {
    gain_r: f32,
    gain_g: f32,
    gain_b: f32,
    _pad: f32,
}

impl AutoWbPipelines {
    pub fn new(gpu: &GpuContext) -> Self {
        let reduce_shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ceq.auto_wb.reduce.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(AUTO_WB_REDUCE_WGSL)),
            });
        let apply_shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ceq.auto_wb.apply.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(AUTO_WB_APPLY_WGSL)),
            });

        let reduce_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ceq.auto_wb.reduce.bgl"),
                    entries: &[
                        // 0: input rgba8unorm texture.
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
                        // 1: sums storage buffer (4× atomic u32).
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(SUMS_SLOT_COUNT * 4),
                            },
                            count: None,
                        },
                        // 2: dims uniform.
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                    ReduceUniforms,
                                >(
                                )
                                    as u64),
                            },
                            count: None,
                        },
                    ],
                });

        let apply_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ceq.auto_wb.apply.bgl"),
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
                        // 1: output rgba8unorm storage.
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
                        // 2: gains uniform.
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                    ApplyUniforms,
                                >(
                                )
                                    as u64),
                            },
                            count: None,
                        },
                    ],
                });

        let reduce_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ceq.auto_wb.reduce.layout"),
                    bind_group_layouts: &[&reduce_bind_group_layout],
                    immediate_size: 0,
                });
        let apply_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ceq.auto_wb.apply.layout"),
                    bind_group_layouts: &[&apply_bind_group_layout],
                    immediate_size: 0,
                });

        let reduce_pipeline =
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("ceq.auto_wb.reduce.pipeline"),
                    layout: Some(&reduce_pipeline_layout),
                    module: &reduce_shader,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    cache: None,
                });
        let apply_pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ceq.auto_wb.apply.pipeline"),
                layout: Some(&apply_pipeline_layout),
                module: &apply_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            reduce_pipeline,
            reduce_bind_group_layout,
            apply_pipeline,
            apply_bind_group_layout,
        }
    }

    /// Run Gray-World auto white balance over `rgba` in place. Same
    /// semantics as the CPU [`crate::algorithm::auto_white_balance`]:
    /// short-circuits when no opaque pixels OR any channel mean is
    /// zero. Wraps [`Self::encode_reduce_into`] +
    /// [`Self::compute_gains_from_sums`] + [`Self::encode_apply_into`]
    /// for the standalone single-stage path; chains call those three
    /// separately around their own ping-pong textures + CPU bounce
    /// between the two encoders.
    pub fn dispatch(&self, gpu: &GpuContext, rgba: &mut [u8], w: u32, h: u32) {
        let expected = (w as usize) * (h as usize) * 4;
        assert_eq!(rgba.len(), expected, "rgba length must match w*h*4");
        if w == 0 || h == 0 {
            return;
        }
        // Reduce + sums readback in encoder 1.
        let input_tex = make_input_texture(gpu, "ceq.auto_wb.input", rgba, w, h);
        let input_view = input_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let sums_buf = make_sums_buffer(gpu);
        let sums_readback = make_sums_readback_buffer(gpu);
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ceq.auto_wb.reduce.encoder"),
            });
        self.encode_reduce_into(gpu, &mut encoder, &input_view, &sums_buf, w, h);
        encoder.copy_buffer_to_buffer(&sums_buf, 0, &sums_readback, 0, SUMS_SLOT_COUNT * 4);
        gpu.queue.submit(std::iter::once(encoder.finish()));

        let sums = read_sums(gpu, &sums_readback);
        let Some(gains) = Self::compute_gains_from_sums(sums) else {
            return;
        };

        // Apply + final readback in encoder 2.
        let output_tex = make_storage_texture(gpu, "ceq.auto_wb.output", w, h);
        let output_view = output_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut apply_encoder =
            gpu.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("ceq.auto_wb.apply.encoder"),
                });
        self.encode_apply_into(
            gpu,
            &mut apply_encoder,
            &input_view,
            &output_view,
            w,
            h,
            gains,
        );
        readback_into(&mut apply_encoder, gpu, &output_tex, rgba, w, h);
    }

    /// Encode the reduce compute pass into `encoder`. `sums_buf` must
    /// be a 4×u32 storage buffer (use [`make_sums_buffer`]); after the
    /// pass it will hold `[sum_r, sum_g, sum_b, count]`. Caller is
    /// responsible for copying + reading back the sums to compute gains
    /// (via [`Self::compute_gains_from_sums`]) before the apply pass.
    pub fn encode_reduce_into(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        sums_buf: &wgpu::Buffer,
        w: u32,
        h: u32,
    ) {
        if w == 0 || h == 0 {
            return;
        }
        let reduce_uniforms = ReduceUniforms {
            width: w,
            height: h,
            _pad0: 0,
            _pad1: 0,
        };
        let reduce_uniform_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ceq.auto_wb.reduce.uniforms"),
                contents: bytemuck::bytes_of(&reduce_uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let reduce_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ceq.auto_wb.reduce.bg"),
            layout: &self.reduce_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sums_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: reduce_uniform_buf.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ceq.auto_wb.reduce.pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.reduce_pipeline);
        pass.set_bind_group(0, &reduce_bg, &[]);
        pass.dispatch_workgroups(w.div_ceil(WORKGROUP_SIZE), h.div_ceil(WORKGROUP_SIZE), 1);
    }

    /// Project the raw `[sum_r, sum_g, sum_b, count]` from the reduce
    /// pass into `[gain_r, gain_g, gain_b]`. Returns `None` when CPU
    /// auto-WB would short-circuit (zero opaque count or any
    /// zero-mean channel).
    pub fn compute_gains_from_sums(sums: [u32; 4]) -> Option<[f32; 3]> {
        let (sum_r, sum_g, sum_b, count) = (sums[0], sums[1], sums[2], sums[3]);
        if count == 0 {
            return None;
        }
        let mean_r = sum_r as f32 / count as f32;
        let mean_g = sum_g as f32 / count as f32;
        let mean_b = sum_b as f32 / count as f32;
        if mean_r == 0.0 || mean_g == 0.0 || mean_b == 0.0 {
            return None;
        }
        let mean_gray = (mean_r + mean_g + mean_b) / 3.0;
        Some([mean_gray / mean_r, mean_gray / mean_g, mean_gray / mean_b])
    }

    /// Encode the apply compute pass into `encoder`. `gains` come from
    /// [`Self::compute_gains_from_sums`] (or any equivalent CPU
    /// projection).
    #[allow(clippy::too_many_arguments)]
    pub fn encode_apply_into(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        w: u32,
        h: u32,
        gains: [f32; 3],
    ) {
        if w == 0 || h == 0 {
            return;
        }
        let apply_uniforms = ApplyUniforms {
            gain_r: gains[0],
            gain_g: gains[1],
            gain_b: gains[2],
            _pad: 0.0,
        };
        let apply_uniform_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ceq.auto_wb.apply.uniforms"),
                contents: bytemuck::bytes_of(&apply_uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let apply_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ceq.auto_wb.apply.bg"),
            layout: &self.apply_bind_group_layout,
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
                    resource: apply_uniform_buf.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ceq.auto_wb.apply.pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.apply_pipeline);
        pass.set_bind_group(0, &apply_bg, &[]);
        pass.dispatch_workgroups(w.div_ceil(WORKGROUP_SIZE), h.div_ceil(WORKGROUP_SIZE), 1);
    }
}

/// Build the zeroed `[sum_r, sum_g, sum_b, count]` storage buffer used
/// by the reduce pass. Sized for `SUMS_SLOT_COUNT = 4` u32s.
pub fn make_sums_buffer(gpu: &GpuContext) -> wgpu::Buffer {
    let zero_sums = [0_u32; SUMS_SLOT_COUNT as usize];
    gpu.device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ceq.auto_wb.sums"),
            contents: bytemuck::cast_slice(&zero_sums),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        })
}

/// Build the host-mappable readback buffer for the sums (4 × u32).
pub fn make_sums_readback_buffer(gpu: &GpuContext) -> wgpu::Buffer {
    gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ceq.auto_wb.sums_readback"),
        size: SUMS_SLOT_COUNT * 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

/// Block on the queue's pending submissions, map the sums readback
/// buffer, and decode the 4 u32s. Caller must have already submitted
/// an encoder that wrote into `sums_readback` via
/// `copy_buffer_to_buffer`.
pub fn read_sums(gpu: &GpuContext, sums_readback: &wgpu::Buffer) -> [u32; 4] {
    let slice = sums_readback.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = sender.send(res);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll sums readback");
    match receiver.recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => panic!("sums readback failed: {e}"),
        Err(_) => panic!("sums readback channel closed"),
    }
    let bytes = slice.get_mapped_range();
    let sums: [u32; 4] = bytemuck::cast_slice(&bytes)[..4].try_into().unwrap();
    drop(bytes);
    sums_readback.unmap();
    sums
}

/// Convenience: build pipelines + dispatch in one call.
pub fn auto_white_balance_gpu(rgba: &mut [u8], w: u32, h: u32, gpu: &GpuContext) {
    let pipelines = AutoWbPipelines::new(gpu);
    pipelines.dispatch(gpu, rgba, w, h);
}

/// WGSL reduce kernel: workgroup-shared atomic accumulators (4 slots)
/// merged into 4 global atomics. Each opaque pixel contributes
/// `round(channel · 255)` to its channel sum and `1` to the count.
///
/// `var<workgroup> atomic<u32>` is the canonical pattern: 64 threads of
/// a workgroup hit the same 4 shared slots (single cache line) instead
/// of 64 threads competing on the global slots; then one thread per
/// workgroup commits the partial sum.
const AUTO_WB_REDUCE_WGSL: &str = r#"
struct Uniforms {
    width: u32,
    height: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> sums: array<atomic<u32>, 4>;
@group(0) @binding(2) var<uniform> u: Uniforms;

var<workgroup> local_sum_r: atomic<u32>;
var<workgroup> local_sum_g: atomic<u32>;
var<workgroup> local_sum_b: atomic<u32>;
var<workgroup> local_count: atomic<u32>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>,
        @builtin(local_invocation_index) lid: u32) {
    // Thread 0 zeroes the workgroup-shared slots before accumulation.
    if (lid == 0u) {
        atomicStore(&local_sum_r, 0u);
        atomicStore(&local_sum_g, 0u);
        atomicStore(&local_sum_b, 0u);
        atomicStore(&local_count, 0u);
    }
    workgroupBarrier();

    if (id.x < u.width && id.y < u.height) {
        let coord = vec2<i32>(i32(id.x), i32(id.y));
        let pixel = textureLoad(input_tex, coord, 0);
        if (pixel.a > 0.0) {
            // Convert normalised [0, 1] back to byte [0, 255] so the
            // sums match the CPU semantics exactly (CPU sums u8 raw).
            let r = u32(round(pixel.r * 255.0));
            let g = u32(round(pixel.g * 255.0));
            let b = u32(round(pixel.b * 255.0));
            atomicAdd(&local_sum_r, r);
            atomicAdd(&local_sum_g, g);
            atomicAdd(&local_sum_b, b);
            atomicAdd(&local_count, 1u);
        }
    }
    workgroupBarrier();

    // One thread per workgroup commits its partial sum to the global
    // 4-slot atomic buffer. 64× fewer global atomics than naive.
    if (lid == 0u) {
        atomicAdd(&sums[0], atomicLoad(&local_sum_r));
        atomicAdd(&sums[1], atomicLoad(&local_sum_g));
        atomicAdd(&sums[2], atomicLoad(&local_sum_b));
        atomicAdd(&sums[3], atomicLoad(&local_count));
    }
}
"#;

/// WGSL apply kernel: per-pixel `clamp(channel · gain, 0, 1)`,
/// preserves alpha + skips transparent pixels (CPU parity).
const AUTO_WB_APPLY_WGSL: &str = r#"
struct Uniforms {
    gain_r: f32,
    gain_g: f32,
    gain_b: f32,
    _pad: f32,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> u: Uniforms;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let coord = vec2<i32>(i32(id.x), i32(id.y));
    let pixel = textureLoad(input_tex, coord, 0);
    if (pixel.a == 0.0) {
        textureStore(output_tex, coord, pixel);
        return;
    }
    let out_r = clamp(pixel.r * u.gain_r, 0.0, 1.0);
    let out_g = clamp(pixel.g * u.gain_g, 0.0, 1.0);
    let out_b = clamp(pixel.b * u.gain_b, 0.0, 1.0);
    textureStore(output_tex, coord, vec4<f32>(out_r, out_g, out_b, pixel.a));
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::auto_white_balance;
    use crate::gpu::try_headless_gpu;

    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
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

    #[test]
    fn auto_wb_gpu_all_transparent_is_noop() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut src = vec![100_u8, 150, 200, 0]; // alpha 0
        src.extend_from_slice(&[200, 80, 40, 0]);
        let mut buf = src.clone();
        auto_white_balance_gpu(&mut buf, 2, 1, &gpu);
        assert_eq!(buf, src);
    }

    #[test]
    fn auto_wb_gpu_pure_grey_is_near_noop() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = solid(16, 16, [128, 128, 128]);
        let mut buf = src.clone();
        auto_white_balance_gpu(&mut buf, 16, 16, &gpu);
        // gains all equal 1 → identity (within rounding).
        for (a, b) in buf.iter().zip(src.iter()) {
            assert!(a.abs_diff(*b) <= 1);
        }
    }

    #[test]
    fn auto_wb_gpu_matches_cpu_solid_red_cast() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = solid(16, 16, [200, 100, 100]);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        auto_white_balance(&mut cpu);
        auto_white_balance_gpu(&mut gpu_buf, 16, 16, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 2, "solid red cast 200/100/100");
    }

    #[test]
    fn auto_wb_gpu_matches_cpu_solid_blue_cast() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = solid(16, 16, [80, 100, 220]);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        auto_white_balance(&mut cpu);
        auto_white_balance_gpu(&mut gpu_buf, 16, 16, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 2, "solid blue cast 80/100/220");
    }

    #[test]
    fn auto_wb_gpu_matches_cpu_diverse_input() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        // Varied per-pixel input — exercises the reduce over many bins.
        let mut src = Vec::with_capacity(32 * 32 * 4);
        for y in 0..32_u32 {
            for x in 0..32_u32 {
                let r = ((x * 11 + y * 5) % 256) as u8;
                let g = ((x * 7 + y * 13) % 256) as u8;
                let b = ((x * 3 + y * 17) % 256) as u8;
                src.extend_from_slice(&[r, g, b, 255]);
            }
        }
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        auto_white_balance(&mut cpu);
        auto_white_balance_gpu(&mut gpu_buf, 32, 32, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 2, "diverse 32×32 ramp");
    }

    #[test]
    fn auto_wb_gpu_matches_cpu_with_transparent_border() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        // 16×16 with a transparent 2-px border and a red-tinted core —
        // the CPU reduce skips alpha=0 pixels; GPU must do the same so
        // the gains land on the opaque-pixel mean only.
        let mut src = Vec::with_capacity(16 * 16 * 4);
        for y in 0..16_u32 {
            for x in 0..16_u32 {
                let opaque = (2..14).contains(&x) && (2..14).contains(&y);
                let alpha: u8 = if opaque { 255 } else { 0 };
                src.extend_from_slice(&[200, 100, 100, alpha]);
            }
        }
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        auto_white_balance(&mut cpu);
        auto_white_balance_gpu(&mut gpu_buf, 16, 16, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 2, "transparent border + red core");
    }

    #[test]
    fn auto_wb_gpu_skips_when_channel_mean_is_zero() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        // Pure red channel — green + blue means are zero → both CPU
        // and GPU short-circuit (no rescale).
        let src = solid(8, 8, [200, 0, 0]);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        auto_white_balance(&mut cpu);
        auto_white_balance_gpu(&mut gpu_buf, 8, 8, &gpu);
        // Both should pass through unchanged.
        assert_eq!(cpu, src);
        assert_eq!(gpu_buf, src);
    }

    #[test]
    fn auto_wb_gpu_handles_non_workgroup_aligned_dimensions() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = solid(13, 19, [180, 110, 90]);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        auto_white_balance(&mut cpu);
        auto_white_balance_gpu(&mut gpu_buf, 13, 19, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 2, "13×19 red cast");
    }
}
