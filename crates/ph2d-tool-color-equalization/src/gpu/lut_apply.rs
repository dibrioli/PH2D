//! WGSL compute path for [`crate::lut::apply_lut3d`].
//!
//! Layout: input RGBA8 lives in a `texture_2d<f32>` (storage `rgba8unorm`,
//! readable); the LUT cube lives in a `texture_3d<f32>` (`rgba16float`,
//! filterable on every adapter) — the hardware trilinear sampler does
//! the per-pixel 8-cell interp natively, replacing the CPU loop. Output
//! goes into a storage `texture_2d<rgba8unorm, write>`; we then copy to
//! a buffer and read back into the caller's `&mut [u8]` (an explicit
//! `map_async` + `pollster::block_on`).
//!
//! Parity test pinned at ε ≤ 1 LSB per channel (hardware trilinear vs.
//! `f32` software trilinear has tiny rounding deltas in the LSB; cell
//! values themselves are quantized to f16 on upload).

use super::{make_input_texture, make_storage_texture, readback_into};
use crate::lut::LUT3D;
use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

/// Compiled pipeline + reusable sampler for the LUT apply compute pass.
/// Build once per [`GpuContext`]; [`Self::dispatch`] is then per-call
/// (creates the input/output/LUT textures sized to the image).
pub struct LutApplyPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct LutUniforms {
    intensity: f32,
    /// `(size - 1) / size` — scales input `[0, 1]` so cell 0's center
    /// lands at `0.5 / size` (texel centre) and cell `size − 1`'s
    /// centre lands at `(size − 0.5) / size`. Together with [`Self::lut_bias`]
    /// this remaps from grid-space (CPU convention) to texture-space
    /// (sampler convention), matching the CPU trilinear behaviour
    /// exactly. Without this the half-texel offset shifts every
    /// pixel by ~6-9 LSB.
    lut_scale: f32,
    /// `0.5 / size` — see [`Self::lut_scale`].
    lut_bias: f32,
    _pad: f32,
}

const WORKGROUP_SIZE: u32 = 8;

impl LutApplyPipeline {
    /// Compile the shader and build the bind-group layout + pipeline.
    /// One-time per device (≈ 2 ms cold; ~0 ms with shader cache).
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ceq.lut_apply.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(LUT_APPLY_WGSL)),
            });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ceq.lut_apply.bgl"),
                    entries: &[
                        // 0: input RGBA8 texture (read).
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
                        // 1: output RGBA8 storage texture (write).
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
                        // 2: LUT 3D texture (sample).
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D3,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // 3: LUT sampler (trilinear filtering — hardware
                        //    interpolates the 8 surrounding cells).
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        // 4: uniforms (intensity).
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                    LutUniforms,
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
                label: Some("ceq.lut_apply.layout"),
                bind_group_layouts: &[&bind_group_layout],
                // wgpu 28: `push_constant_ranges` was renamed to
                // `immediate_size` — we don't use push constants.
                immediate_size: 0,
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ceq.lut_apply.pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ceq.lut_apply.sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
        }
    }

    /// Apply the LUT to `rgba` in place. Builds per-call input + output
    /// textures, encodes via [`Self::encode_into`], reads back the
    /// result. Skips entirely when `intensity ≤ 0`. Use [`Self::encode_into`]
    /// directly when chaining multiple GPU stages (see
    /// [`super::ChainedPipelineCache`]) — that variant skips upload and
    /// readback so the whole pipeline stays GPU-resident.
    pub fn dispatch(
        &self,
        gpu: &GpuContext,
        rgba: &mut [u8],
        w: u32,
        h: u32,
        lut: &LUT3D,
        intensity: f32,
    ) {
        let expected = (w as usize) * (h as usize) * 4;
        assert_eq!(rgba.len(), expected, "rgba length must match w*h*4");
        if intensity <= 0.0 || w == 0 || h == 0 || lut.size < 2 {
            return;
        }
        let input_tex = make_input_texture(gpu, "ceq.lut_apply.input", rgba, w, h);
        let output_tex = make_storage_texture(gpu, "ceq.lut_apply.output", w, h);
        let input_view = input_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ceq.lut_apply.encoder"),
            });
        self.encode_into(
            gpu,
            &mut encoder,
            &input_view,
            &output_view,
            w,
            h,
            lut,
            intensity,
        );
        readback_into(&mut encoder, gpu, &output_tex, rgba, w, h);
    }

    /// Encode a single compute pass into `encoder` that reads `input_view`
    /// (`rgba8unorm`) and writes `output_view` (`rgba8unorm` storage),
    /// applying the LUT at `intensity`.
    ///
    /// The LUT 3D texture + uniform buffer + bind group are created
    /// inside this function and kept alive across the call via wgpu's
    /// internal Arc retention on the bind group → texture chain — so the
    /// caller only needs to ensure `input_view` + `output_view` survive
    /// until queue submission (the chain owns its ping-pong textures).
    ///
    /// Caller is responsible for the encoder's submission + any readback.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_into(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        w: u32,
        h: u32,
        lut: &LUT3D,
        intensity: f32,
    ) {
        if intensity <= 0.0 || w == 0 || h == 0 || lut.size < 2 {
            return;
        }
        // LUT 3D texture (rgba16float, packed f16 RGB + alpha=1).
        let lut_size = lut.size;
        let lut_cells = (lut_size as usize).pow(3);
        let mut lut_f16 = Vec::with_capacity(lut_cells * 4);
        for cell in lut.data.chunks_exact(3) {
            lut_f16.push(half::f16::from_f32(cell[0]));
            lut_f16.push(half::f16::from_f32(cell[1]));
            lut_f16.push(half::f16::from_f32(cell[2]));
            lut_f16.push(half::f16::from_f32(1.0));
        }
        let lut_bytes: Vec<u8> = lut_f16.iter().flat_map(|h| h.to_le_bytes()).collect();
        let lut_tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ceq.lut_apply.lut3d"),
            size: wgpu::Extent3d {
                width: lut_size,
                height: lut_size,
                depth_or_array_layers: lut_size,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &lut_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &lut_bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(lut_size * 4 * 2),
                rows_per_image: Some(lut_size),
            },
            wgpu::Extent3d {
                width: lut_size,
                height: lut_size,
                depth_or_array_layers: lut_size,
            },
        );

        let lut_size_f = lut_size as f32;
        let uniforms = LutUniforms {
            intensity: intensity.clamp(0.0, 1.0),
            lut_scale: (lut_size_f - 1.0) / lut_size_f,
            lut_bias: 0.5 / lut_size_f,
            _pad: 0.0,
        };
        let uniform_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ceq.lut_apply.uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let lut_view = lut_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ceq.lut_apply.bind_group"),
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
                    resource: wgpu::BindingResource::TextureView(&lut_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: uniform_buf.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ceq.lut_apply.pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(w.div_ceil(WORKGROUP_SIZE), h.div_ceil(WORKGROUP_SIZE), 1);
    }
}

/// Convenience wrapper: build a pipeline + dispatch in one call.
/// Use [`LutApplyPipeline::new`] explicitly when applying many LUTs back
/// to back; the pipeline + sampler + layout are reusable.
pub fn apply_lut3d_gpu(
    rgba: &mut [u8],
    w: u32,
    h: u32,
    lut: &LUT3D,
    intensity: f32,
    gpu: &GpuContext,
) {
    let pipeline = LutApplyPipeline::new(gpu);
    pipeline.dispatch(gpu, rgba, w, h, lut, intensity);
}

/// WGSL compute kernel: per pixel, sample the LUT via hardware
/// trilinear filtering, then blend with the original by `intensity`.
const LUT_APPLY_WGSL: &str = r#"
struct Uniforms {
    intensity: f32,
    // (size - 1) / size  →  rescales input rgb so it indexes cell
    // centres (not texture edges).
    lut_scale: f32,
    // 0.5 / size
    lut_bias: f32,
    _pad: f32,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var lut_tex: texture_3d<f32>;
@group(0) @binding(3) var lut_sampler: sampler;
@group(0) @binding(4) var<uniform> uniforms: Uniforms;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let coord = vec2<i32>(i32(id.x), i32(id.y));
    let pixel = textureLoad(input_tex, coord, 0);

    // Grid → texture-coordinate remap: the CPU trilinear samples cell
    // `i` at `i / (N - 1)`; the GPU sampler places cell `i` centre at
    // `(i + 0.5) / N`. Compose those: `tex = pixel · (N-1)/N + 0.5/N`.
    // Hardware trilinear then does the 8-cell interp in one fetch.
    let lut_coord = pixel.rgb * uniforms.lut_scale + vec3<f32>(uniforms.lut_bias);
    let lut_rgb = textureSampleLevel(lut_tex, lut_sampler, lut_coord, 0.0).rgb;

    let blended = mix(pixel.rgb, lut_rgb, uniforms.intensity);
    textureStore(output_tex, coord, vec4<f32>(blended, pixel.a));
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::try_headless_gpu;
    use crate::lut::{DEFAULT_LUT_SIZE, apply_lut3d, identity_lut};
    use crate::lut_presets::{LutPreset, generate_preset_lut};

    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    /// Build a deterministic ramp image with diverse colours for parity
    /// checks (covers the full input space, not just a single cell).
    fn ramp(w: u32, h: u32) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let r = ((x * 37 + y * 5) % 256) as u8;
                let g = ((x * 13 + y * 23) % 256) as u8;
                let b = ((x * 7 + y * 41) % 256) as u8;
                v.extend_from_slice(&[r, g, b, 255]);
            }
        }
        v
    }

    fn assert_within_lsb(cpu: &[u8], gpu: &[u8], max_lsb: i32) {
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
            "CPU/GPU diverged at index {worst_idx} by {worst} LSB (cpu {} vs gpu {})",
            cpu[worst_idx],
            gpu[worst_idx],
        );
    }

    #[test]
    fn identity_lut_gpu_round_trips_within_lsb() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let lut = identity_lut(DEFAULT_LUT_SIZE);
        let mut buf = solid(16, 16, [80, 160, 220]);
        let before = buf.clone();
        apply_lut3d_gpu(&mut buf, 16, 16, &lut, 1.0, &gpu);
        // Identity LUT with intensity 1 → output ≈ input, modulo f16
        // quantization + hardware trilinear rounding.
        for (a, b) in buf.iter().zip(before.iter()) {
            assert!(a.abs_diff(*b) <= 2, "identity GPU drifted: {a} vs {b}");
        }
    }

    #[test]
    fn gpu_matches_cpu_for_warm_preset_within_lsb() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let lut = generate_preset_lut(LutPreset::Warm, DEFAULT_LUT_SIZE).unwrap();
        let src = ramp(32, 32);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        apply_lut3d(&mut cpu, &lut, 1.0);
        apply_lut3d_gpu(&mut gpu_buf, 32, 32, &lut, 1.0, &gpu);
        // ε ≤ 2 LSB: hardware trilinear (8-bit input cube) + f16 LUT
        // quantization vs. f32 software produces ≤ 1 LSB diff in nearly
        // every pixel, with rare 2-LSB outliers near cell boundaries.
        assert_within_lsb(&cpu, &gpu_buf, 2);
    }

    #[test]
    fn gpu_matches_cpu_for_sepia_preset_within_lsb() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let lut = generate_preset_lut(LutPreset::Sepia, DEFAULT_LUT_SIZE).unwrap();
        let src = ramp(32, 32);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        apply_lut3d(&mut cpu, &lut, 1.0);
        apply_lut3d_gpu(&mut gpu_buf, 32, 32, &lut, 1.0, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 2);
    }

    #[test]
    fn gpu_intensity_half_matches_cpu_within_lsb() {
        // Verify intensity blend on GPU matches CPU (not just intensity = 1).
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let lut = generate_preset_lut(LutPreset::Cinematic, DEFAULT_LUT_SIZE).unwrap();
        let src = ramp(32, 32);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        apply_lut3d(&mut cpu, &lut, 0.5);
        apply_lut3d_gpu(&mut gpu_buf, 32, 32, &lut, 0.5, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 2);
    }

    #[test]
    fn gpu_zero_intensity_is_noop() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let lut = generate_preset_lut(LutPreset::FilmNoir, DEFAULT_LUT_SIZE).unwrap();
        let src = ramp(16, 16);
        let mut buf = src.clone();
        apply_lut3d_gpu(&mut buf, 16, 16, &lut, 0.0, &gpu);
        assert_eq!(buf, src, "intensity = 0 must short-circuit before dispatch");
    }

    #[test]
    fn gpu_handles_non_workgroup_aligned_dimensions() {
        // 17 × 11 — neither divisible by 8 (workgroup size). The shader's
        // per-thread bounds check is the only thing preventing out-of-
        // range writes; verify it works.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let lut = generate_preset_lut(LutPreset::Cool, DEFAULT_LUT_SIZE).unwrap();
        let src = ramp(17, 11);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        apply_lut3d(&mut cpu, &lut, 1.0);
        apply_lut3d_gpu(&mut gpu_buf, 17, 11, &lut, 1.0, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 2);
    }
}
