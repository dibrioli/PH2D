//! WGSL compute path for [`crate::algorithm::nlm_denoise`].
//!
//! Non-Local Means (Buades 2005) is the texture-preserving counterpart
//! to bilateral filtering. For each pixel `(x, y)`, NLM walks an
//! `(2·search_half + 1)²` search window and for each candidate neighbour
//! `(nx, ny)` computes a `(2·patch_half + 1)²` patch-distance against
//! the center's patch; neighbours with similar patches contribute more
//! heavily. The result preserves fine texture (skin, fabric, foliage)
//! that bilateral would over-smooth, at substantially higher cost.
//!
//! **Cost:** `O(N · search² · patch²)`. For our defaults (`patch_half =
//! 3`, `search_half = clamp(w/8, 5, 10)`), that's ≈ `121 × 49 = 5929`
//! patch-pixel reads per output pixel (plus the centre + neighbour
//! sample) — roughly 10–30× the bilateral load. GPU brings this from
//! "seconds" to "tens of millis" on 1024² Apple Silicon.
//!
//! Operates in **linear sRGB** for parity with the CPU path
//! ([`crate::algorithm::nlm_denoise`]) — sRGB decode on every read,
//! sRGB encode on the final write. Alpha is preserved verbatim;
//! fully-transparent input passes through.

use super::{make_input_texture, make_storage_texture, readback_into};
use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

const WORKGROUP_SIZE: u32 = 8;

/// Compiled compute pipeline + bind-group layout for NLM denoise.
/// Build once per [`GpuContext`]; [`Self::dispatch`] is per-call.
pub struct NlmPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct NlmUniforms {
    /// `−1 / h²` where `h = (10 + 90·strength)/255` (linear-sRGB scale).
    /// Pre-computed host-side so the shader uses one MAD per neighbour
    /// instead of `exp(div)` ladders.
    inv_h_sq: f32,
    /// `patch_half = 3` (7×7 patch).
    patch_half: i32,
    /// `search_half = clamp(w/8, 5, 10)`.
    search_half: i32,
    _pad: i32,
}

impl NlmPipeline {
    /// Compile the shader and build pipeline + bind-group layout.
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ceq.nlm.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(NLM_WGSL)),
            });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ceq.nlm.bgl"),
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
                                    NlmUniforms,
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
                label: Some("ceq.nlm.layout"),
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ceq.nlm.pipeline"),
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

    /// Run NLM denoise over `rgba` in place. Wraps [`Self::encode_into`]
    /// with upload + readback; use `encode_into` directly when chaining
    /// stages.
    pub fn dispatch(&self, gpu: &GpuContext, rgba: &mut [u8], w: u32, h: u32, strength: f32) {
        let expected = (w as usize) * (h as usize) * 4;
        assert_eq!(rgba.len(), expected, "rgba length must match w*h*4");
        if strength <= 0.0 || w == 0 || h == 0 {
            return;
        }
        let input_tex = make_input_texture(gpu, "ceq.nlm.input", rgba, w, h);
        let output_tex = make_storage_texture(gpu, "ceq.nlm.output", w, h);
        let input_view = input_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ceq.nlm.encoder"),
            });
        self.encode_into(gpu, &mut encoder, &input_view, &output_view, w, h, strength);
        readback_into(&mut encoder, gpu, &output_tex, rgba, w, h);
    }

    /// Encode one NLM compute pass into `encoder`. Matches the bilateral
    /// shape so the [`super::ChainedPipelineCache`] can swap one for the
    /// other based on `params.denoise_method`.
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
        let patch_half: i32 = 3;
        let search_half: i32 = ((w as i32) / 8).clamp(5, 10);
        let h_param = (10.0 + strength * 90.0) / 255.0;
        let uniforms = NlmUniforms {
            inv_h_sq: -1.0 / (h_param * h_param),
            patch_half,
            search_half,
            _pad: 0,
        };
        let uniform_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ceq.nlm.uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ceq.nlm.bind_group"),
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
            label: Some("ceq.nlm.pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(w.div_ceil(WORKGROUP_SIZE), h.div_ceil(WORKGROUP_SIZE), 1);
    }
}

/// Convenience: build pipeline + dispatch. Use [`NlmPipeline::new`]
/// directly when applying many denoises (the pipeline build is the
/// expensive part — ~10–30 ms shader compile).
pub fn nlm_denoise_gpu(rgba: &mut [u8], w: u32, h: u32, strength: f32, gpu: &GpuContext) {
    let pipeline = NlmPipeline::new(gpu);
    pipeline.dispatch(gpu, rgba, w, h, strength);
}

/// WGSL compute kernel: per pixel, walk a `(2·search_half + 1)²` window
/// of neighbour candidates; for each candidate compute a `(2·patch_half +
/// 1)²` patch distance against the centre's patch; weight the neighbour
/// by `exp(-patch_distance² / h²)`. Operates in linear sRGB (decode each
/// `textureLoad`, encode the final `textureStore`).
const NLM_WGSL: &str = r#"
struct Uniforms {
    inv_h_sq: f32,
    patch_half: i32,
    search_half: i32,
    _pad: i32,
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

    let sh = u.search_half;
    let ph = u.patch_half;

    var sum_r: f32 = 0.0;
    var sum_g: f32 = 0.0;
    var sum_b: f32 = 0.0;
    var sum_w: f32 = 0.0;

    let sy_min = max(0, coord.y - sh);
    let sy_max = min(dims_i.y - 1, coord.y + sh);
    let sx_min = max(0, coord.x - sh);
    let sx_max = min(dims_i.x - 1, coord.x + sh);

    for (var ny: i32 = sy_min; ny <= sy_max; ny = ny + 1) {
        for (var nx: i32 = sx_min; nx <= sx_max; nx = nx + 1) {
            var patch_dist: f32 = 0.0;
            var patch_count: f32 = 0.0;
            for (var py: i32 = -ph; py <= ph; py = py + 1) {
                let py1 = coord.y + py;
                let py2 = ny + py;
                if (py1 < 0 || py1 >= dims_i.y || py2 < 0 || py2 >= dims_i.y) {
                    continue;
                }
                for (var px: i32 = -ph; px <= ph; px = px + 1) {
                    let px1 = coord.x + px;
                    let px2 = nx + px;
                    if (px1 < 0 || px1 >= dims_i.x || px2 < 0 || px2 >= dims_i.x) {
                        continue;
                    }
                    let p1 = s2l(textureLoad(input_tex, vec2<i32>(px1, py1), 0).rgb);
                    let p2 = s2l(textureLoad(input_tex, vec2<i32>(px2, py2), 0).rgb);
                    let d = p1 - p2;
                    patch_dist = patch_dist + dot(d, d);
                    patch_count = patch_count + 1.0;
                }
            }
            var norm_dist: f32 = 0.0;
            if (patch_count > 0.0) {
                norm_dist = patch_dist / (patch_count * 3.0);
            }
            let weight = exp(norm_dist * u.inv_h_sq);
            let n_lin = s2l(textureLoad(input_tex, vec2<i32>(nx, ny), 0).rgb);
            sum_r = sum_r + n_lin.r * weight;
            sum_g = sum_g + n_lin.g * weight;
            sum_b = sum_b + n_lin.b * weight;
            sum_w = sum_w + weight;
        }
    }

    var out_lin = s2l(center_srgb.rgb);
    if (sum_w > 0.0) {
        out_lin = vec3<f32>(sum_r, sum_g, sum_b) / sum_w;
    }
    let out_srgb = l2s(out_lin);
    textureStore(output_tex, coord, vec4<f32>(out_srgb, center_srgb.a));
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::nlm_denoise;
    use crate::gpu::try_headless_gpu;

    fn uniform_noisy(w: u32, h: u32) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let noise = ((x * 3 + y * 5) % 11) as i32 - 5;
                let val = (128 + noise).clamp(0, 255) as u8;
                v.extend_from_slice(&[val, val, val, 255]);
            }
        }
        v
    }

    fn max_diff(a: &[u8], b: &[u8]) -> i32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (*x as i32 - *y as i32).abs())
            .max()
            .unwrap_or(0)
    }

    #[test]
    fn nlm_gpu_matches_cpu_uniform_strength_half() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut cpu = uniform_noisy(24, 24);
        let mut gpu_buf = cpu.clone();
        nlm_denoise(&mut cpu, 24, 24, 0.5);
        nlm_denoise_gpu(&mut gpu_buf, 24, 24, 0.5, &gpu);
        let diff = max_diff(&cpu, &gpu_buf);
        assert!(
            diff <= 4,
            "NLM CPU↔GPU max channel diff = {diff} LSB on uniform-noisy 24² (tolerance 4)"
        );
    }

    #[test]
    fn nlm_gpu_matches_cpu_at_edge() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        // 32×32 with vertical edge at x=16.
        let mut buf = Vec::with_capacity(32 * 32 * 4);
        for y in 0..32u32 {
            for x in 0..32u32 {
                let v = if x < 16 { 50u8 } else { 200u8 };
                buf.extend_from_slice(&[v, v, v, 255]);
            }
        }
        let mut cpu = buf.clone();
        let mut gpu_buf = buf.clone();
        nlm_denoise(&mut cpu, 32, 32, 0.5);
        nlm_denoise_gpu(&mut gpu_buf, 32, 32, 0.5, &gpu);
        let diff = max_diff(&cpu, &gpu_buf);
        assert!(
            diff <= 4,
            "NLM CPU↔GPU max channel diff = {diff} LSB on edge 32² (tolerance 4)"
        );
    }

    #[test]
    fn nlm_gpu_zero_strength_is_noop() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut buf = uniform_noisy(16, 16);
        let original = buf.clone();
        nlm_denoise_gpu(&mut buf, 16, 16, 0.0, &gpu);
        assert_eq!(buf, original);
    }
}
