//! WGSL compute path for [`crate::algorithm::bilateral_denoise`].
//!
//! Bilateral filtering is the heaviest CPU stage in the whole pipeline:
//! `O(N · (2r + 1)²)` where `r` grows with strength (≈ 6 px at strength
//! 0.2, ≈ 16 px at strength 1.0 — so 169 to 1 089 texture reads per
//! pixel). The CPU version saturates a core at ≈ 10 s on a 1 024² image
//! at strength 0.8; the GPU version brings that into the tens of
//! milliseconds on Apple Silicon (every workgroup runs 64 threads in
//! parallel, and each thread's inner loop is exactly the same math).
//!
//! ## Algorithm
//!
//! Per pixel: walk the `(2r + 1)²` neighbourhood; compute the spatial
//! Gaussian (over pixel-distance) and the range Gaussian (over RGB
//! colour-distance); combine into a weight; weighted sum the
//! neighbourhood and divide by total weight. Border pixels iterate a
//! smaller window (out-of-bounds neighbours are skipped, **not**
//! clamped) — exactly matching the CPU semantics so the parity test
//! holds at `ε ≤ 3 LSB`.
//!
//! ## Future optimisation (out of scope for this pass)
//!
//! - **Workgroup-shared neighbourhood tiles**: load a `(WG + 2r) ×
//!   (WG + 2r)` tile into `workgroup` memory once per workgroup; each
//!   thread reads only from shared memory. ~3-5× extra speedup at
//!   `r ≥ 8`. Adds ~100 LOC + a separate "small radius" path.
//! - **Separable approximation** (joint bilateral): radius decomposes
//!   into H + V passes when the range term is approximated. Pixar
//!   "Edge-Avoiding À-Trous Wavelet" papers cover this. Quality drops
//!   at high range-σ, so we keep the dense kernel here.

use super::{make_input_texture, make_storage_texture, readback_into};
use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

const WORKGROUP_SIZE: u32 = 8;

/// Compiled compute pipeline + bind-group layout for bilateral
/// denoise. Build once per [`GpuContext`]; [`Self::dispatch`] is
/// per-call.
pub struct BilateralPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct BilateralUniforms {
    /// `−1 / (2·σ²)` for the spatial term (Gaussian in pixel distance).
    /// Pre-computed host-side so the shader uses one MAD per neighbour
    /// instead of an `exp(div)` ladder.
    inv_spatial_sq: f32,
    /// `−1 / (2·σ²)` for the range term (Gaussian in colour distance).
    inv_range_sq: f32,
    /// `r` — half-side of the square neighbourhood window.
    radius: i32,
    _pad: i32,
}

impl BilateralPipeline {
    /// Compile the shader and build pipeline + bind-group layout.
    /// Idempotent across calls (cache by `GpuContext` on the caller).
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ceq.bilateral.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(BILATERAL_WGSL)),
            });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ceq.bilateral.bgl"),
                    entries: &[
                        // 0: input RGBA8 (read).
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
                        // 1: output RGBA8 storage (write).
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
                        // 2: uniforms.
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                    BilateralUniforms,
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
                label: Some("ceq.bilateral.layout"),
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ceq.bilateral.pipeline"),
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

    /// Run bilateral denoise over `rgba` in place. Wraps
    /// [`Self::encode_into`] with upload + readback; use `encode_into`
    /// directly when chaining stages (see
    /// [`super::ChainedPipelineCache`]).
    pub fn dispatch(&self, gpu: &GpuContext, rgba: &mut [u8], w: u32, h: u32, strength: f32) {
        let expected = (w as usize) * (h as usize) * 4;
        assert_eq!(rgba.len(), expected, "rgba length must match w*h*4");
        if strength <= 0.0 || w == 0 || h == 0 {
            return;
        }
        let input_tex = make_input_texture(gpu, "ceq.bilateral.input", rgba, w, h);
        let output_tex = make_storage_texture(gpu, "ceq.bilateral.output", w, h);
        let input_view = input_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ceq.bilateral.encoder"),
            });
        self.encode_into(gpu, &mut encoder, &input_view, &output_view, w, h, strength);
        readback_into(&mut encoder, gpu, &output_tex, rgba, w, h);
    }

    /// Encode one bilateral compute pass into `encoder`. Creates the
    /// uniform buffer + bind group; both kept alive across the call via
    /// wgpu's internal Arc retention on the bind group. Caller manages
    /// `input_view` + `output_view` lifetime and the encoder's submit.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_into(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        w: u32,
        h: u32,
        strength: f32,
    ) {
        if strength <= 0.0 || w == 0 || h == 0 {
            return;
        }
        let strength = strength.clamp(0.0, 1.0);
        let sigma_space = 3.0 + strength * 5.0;
        // Linear-sRGB domain (Tier 2 audit, parity with CPU
        // `algorithm::bilateral_denoise`). σ_range stays numerically
        // identical to the legacy [0,255] tuning when divided by 255 —
        // the texture reads land in [0,1] after sRGB decode and the
        // pixel diffs operate in that same scale.
        let sigma_range = (20.0 + strength * 50.0) / 255.0;
        let radius = (sigma_space * 2.0).ceil() as i32;
        let uniforms = BilateralUniforms {
            inv_spatial_sq: -1.0 / (2.0 * sigma_space * sigma_space),
            inv_range_sq: -1.0 / (2.0 * sigma_range * sigma_range),
            radius,
            _pad: 0,
        };
        let uniform_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ceq.bilateral.uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ceq.bilateral.bind_group"),
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
            label: Some("ceq.bilateral.pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(w.div_ceil(WORKGROUP_SIZE), h.div_ceil(WORKGROUP_SIZE), 1);
    }
}

/// Convenience: build pipeline + dispatch. Use
/// [`BilateralPipeline::new`] directly when applying many denoises.
pub fn bilateral_denoise_gpu(rgba: &mut [u8], w: u32, h: u32, strength: f32, gpu: &GpuContext) {
    let pipeline = BilateralPipeline::new(gpu);
    pipeline.dispatch(gpu, rgba, w, h, strength);
}

/// WGSL compute kernel: per pixel, sum the bilateral-weighted RGB over
/// the `(2r + 1)²` window centred on the pixel. Out-of-bounds neighbours
/// are skipped (matching the CPU which iterates a clamped window).
///
/// Operates in **linear sRGB** (Tier 2 audit parity with CPU
/// `algorithm::bilateral_denoise`). Each `textureLoad` decodes sRGB →
/// linear; range diffs + weighted sums live in `[0, 1]` linear; the
/// final `textureStore` re-encodes linear → sRGB. σ_range is calibrated
/// on the host (`(20 + 50·strength) / 255`) to match the same numeric
/// weight in linear-[0,1] space.
const BILATERAL_WGSL: &str = r#"
struct Uniforms {
    inv_spatial_sq: f32,
    inv_range_sq: f32,
    radius: i32,
    _pad: i32,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

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

fn srgb_to_linear_rgb(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(srgb_to_linear_c(rgb.r), srgb_to_linear_c(rgb.g), srgb_to_linear_c(rgb.b));
}

fn linear_to_srgb_rgb(rgb: vec3<f32>) -> vec3<f32> {
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

    let center_lin = srgb_to_linear_rgb(center_srgb.rgb);

    var sum_r = 0.0;
    var sum_g = 0.0;
    var sum_b = 0.0;
    var sum_w = 0.0;

    let r = uniforms.radius;
    for (var dy = -r; dy <= r; dy = dy + 1) {
        let ny = coord.y + dy;
        if (ny < 0 || ny >= dims_i.y) { continue; }
        for (var dx = -r; dx <= r; dx = dx + 1) {
            let nx = coord.x + dx;
            if (nx < 0 || nx >= dims_i.x) { continue; }

            let neighbor_srgb = textureLoad(input_tex, vec2<i32>(nx, ny), 0);
            let n_lin = srgb_to_linear_rgb(neighbor_srgb.rgb);

            let spatial = f32(dx * dx + dy * dy);
            let d = n_lin - center_lin;
            let color = dot(d, d);
            let weight = exp(spatial * uniforms.inv_spatial_sq + color * uniforms.inv_range_sq);

            sum_r = sum_r + n_lin.r * weight;
            sum_g = sum_g + n_lin.g * weight;
            sum_b = sum_b + n_lin.b * weight;
            sum_w = sum_w + weight;
        }
    }

    var out_lin = center_lin;
    if (sum_w > 0.0) {
        out_lin = vec3<f32>(sum_r, sum_g, sum_b) / sum_w;
    }
    let out_srgb = linear_to_srgb_rgb(out_lin);
    textureStore(output_tex, coord, vec4<f32>(out_srgb, center_srgb.a));
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::bilateral_denoise;
    use crate::gpu::try_headless_gpu;

    /// Deterministic noisy ramp — diverse colours + spatial structure
    /// so the parity check exercises both the spatial and range terms.
    fn noisy_image(w: u32, h: u32) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let base_r = ((x * 11 + y * 5) % 256) as i32;
                let base_g = ((x * 7 + y * 13) % 256) as i32;
                let base_b = ((x * 17 + y * 3) % 256) as i32;
                // Layer in pseudo-random noise so range term has work.
                let noise = ((x * 31 + y * 47) % 13) as i32 - 6;
                let r = (base_r + noise).clamp(0, 255) as u8;
                let g = (base_g + noise).clamp(0, 255) as u8;
                let b = (base_b + noise).clamp(0, 255) as u8;
                v.extend_from_slice(&[r, g, b, 255]);
            }
        }
        v
    }

    fn assert_within_lsb(cpu: &[u8], gpu: &[u8], max_lsb: i32) {
        assert_eq!(cpu.len(), gpu.len());
        let mut worst = 0_i32;
        let mut worst_idx = 0;
        let mut sum_diff = 0_u64;
        let mut diff_count = 0_u64;
        for (i, (a, b)) in cpu.iter().zip(gpu.iter()).enumerate() {
            let d = (*a as i32 - *b as i32).abs();
            if d > worst {
                worst = d;
                worst_idx = i;
            }
            if d > 0 {
                sum_diff += d as u64;
                diff_count += 1;
            }
        }
        assert!(
            worst <= max_lsb,
            "Bilateral CPU/GPU diverged: worst {worst} LSB at idx {worst_idx} \
             (cpu {} vs gpu {}); mean delta {:.2} over {} pixels",
            cpu[worst_idx],
            gpu[worst_idx],
            sum_diff as f64 / diff_count.max(1) as f64,
            diff_count
        );
    }

    #[test]
    fn bilateral_gpu_zero_strength_is_noop() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = noisy_image(16, 16);
        let mut buf = src.clone();
        bilateral_denoise_gpu(&mut buf, 16, 16, 0.0, &gpu);
        assert_eq!(buf, src);
    }

    #[test]
    fn bilateral_gpu_matches_cpu_low_strength() {
        // Small radius (≈ 6) — easy on both paths, parity is tight.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = noisy_image(24, 24);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        bilateral_denoise(&mut cpu, 24, 24, 0.2);
        bilateral_denoise_gpu(&mut gpu_buf, 24, 24, 0.2, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3);
    }

    #[test]
    fn bilateral_gpu_matches_cpu_medium_strength() {
        // radius ≈ 11 — larger window, more accumulated rounding.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = noisy_image(32, 32);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        bilateral_denoise(&mut cpu, 32, 32, 0.5);
        bilateral_denoise_gpu(&mut gpu_buf, 32, 32, 0.5, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3);
    }

    #[test]
    fn bilateral_gpu_handles_edges_correctly() {
        // The border-skip semantics differ between naive clamp-to-edge
        // (would inflate weights at corners) and the CPU's
        // skip-out-of-bounds. Use a 12×12 with strength 0.4 (radius ≈
        // 10) so EVERY pixel touches a border on at least one side.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = noisy_image(12, 12);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        bilateral_denoise(&mut cpu, 12, 12, 0.4);
        bilateral_denoise_gpu(&mut gpu_buf, 12, 12, 0.4, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3);
    }

    #[test]
    fn bilateral_gpu_preserves_edges_just_like_cpu() {
        // Half-and-half edge image (left = 50, right = 200) with no
        // noise. Both CPU and GPU should preserve the edge (range
        // Gaussian rejects the cross-edge neighbours). Output near
        // x = 6 should still show the sharp ~150-unit jump.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut src = Vec::with_capacity(16 * 16 * 4);
        for _y in 0..16u32 {
            for x in 0..16u32 {
                let v = if x < 8 { 50u8 } else { 200u8 };
                src.extend_from_slice(&[v, v, v, 255]);
            }
        }
        let mut gpu_buf = src.clone();
        bilateral_denoise_gpu(&mut gpu_buf, 16, 16, 0.5, &gpu);
        // Edge at column 7/8 — sample row 8 to be deep inside.
        let left = gpu_buf[((8 * 16 + 6) * 4) as usize] as i32;
        let right = gpu_buf[((8 * 16 + 9) * 4) as usize] as i32;
        assert!(
            (right - left) > 100,
            "GPU bilateral collapsed the edge (left {left} vs right {right})"
        );
    }

    #[test]
    fn bilateral_gpu_handles_non_workgroup_aligned_dimensions() {
        // 13 × 19 — neither dimension divisible by 8.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = noisy_image(13, 19);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        bilateral_denoise(&mut cpu, 13, 19, 0.3);
        bilateral_denoise_gpu(&mut gpu_buf, 13, 19, 0.3, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3);
    }
}
