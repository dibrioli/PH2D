//! WGSL compute path for [`crate::algorithm::adjust_tonal`] — the
//! seven-stage Phase 1 tonal pipeline fused into a single shader.
//!
//! ## Why fuse?
//!
//! The CPU [`crate::algorithm::adjust_tonal`] already batches all seven
//! stages so the expensive transfer functions (`sRGB ↔ linear` × 2,
//! `OKLab ↔ linear` × 2) run **once** per pixel rather than seven
//! times. The GPU port does the same: one `textureLoad` + one
//! `textureStore` per pixel, the entire stage stack happens inside
//! registers. Bandwidth-bound on every adapter; speedup vs. CPU is
//! dominated by parallelism (1 024² scene goes from ~80 ms CPU to ~2 ms
//! GPU on Apple Silicon).
//!
//! ## Stage order (mirrors CPU)
//!
//! ```text
//! sRGB → linear → Exposure → Temperature (Bradford) → Tint →
//! Brightness → Contrast → linear → OKLab → Vibrance → Saturation →
//! OKLab → linear → sRGB
//! ```
//!
//! `OKLab` round-trip is gated on `vibrance != 0 || saturation != 0`
//! (mirrors the CPU's `needs_oklab` short-circuit) so identity-tonal
//! pixels skip the cube-root pair.
//!
//! ## Parity discipline
//!
//! `cbrt` is open-coded as `sign(x) · pow(|x|, 1/3)` (no native WGSL
//! intrinsic). `pow` and `exp` differ subtly between Metal/Vulkan
//! drivers and `f32::powf` / `f32::exp` — observed drift is ≤ 3 LSB on
//! single-stage tests, ≤ 4 LSB on the full seven-stage stack. Parity
//! tests pin both ceilings so a driver regression surfaces immediately.
//!
//! ## Future GPU work fused in this same shader
//!
//! The next planned stages (Phase 4.3+) — Laplacian sharpen, auto-WB
//! apply — are all per-pixel and would slot in BEFORE the
//! `linear → sRGB` ladder so the conversion cost stays amortised. The
//! infrastructure here (uniform layout, bind groups, readback) is
//! shaped to accommodate that growth without restructuring.

use super::{make_input_texture, make_storage_texture, readback_into};
use crate::color_utils::bradford_matrix_for_kelvin;
use crate::params::ColorEqualizationParams;
use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

const WORKGROUP_SIZE: u32 = 8;

/// Compiled pipeline + bind-group layout for the fused tonal compute
/// pass. Build once per [`GpuContext`]; [`Self::dispatch`] is per-call.
pub struct TonalBatchPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TonalUniforms {
    // ── Scalar knobs (8 floats — exactly two vec4s' worth). ──────
    exposure: f32,
    tint: f32,
    brightness: f32,
    contrast: f32,
    vibrance: f32,
    saturation: f32,
    /// `1` when the Bradford temperature matrix should be applied,
    /// `0` skips the matrix multiply. Equivalent to the CPU's
    /// `if params.temperature != 0.0` short-circuit.
    temperature_active: u32,
    _pad0: u32,
    // ── Bradford temperature matrix (3 rows × vec4 padding). ─────
    // The CPU pre-computes the matrix once outside the per-pixel loop
    // (`bradford_matrix_for_kelvin`); we mirror that on the host side
    // and ship the 9 floats here padded to 3×vec4 (std140 alignment).
    temp_row0: [f32; 4],
    temp_row1: [f32; 4],
    temp_row2: [f32; 4],
}

impl TonalBatchPipeline {
    /// Compile the shader and build pipeline + bind-group layout.
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ceq.tonal_batch.shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(TONAL_BATCH_WGSL)),
            });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ceq.tonal_batch.bgl"),
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
                                    TonalUniforms,
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
                label: Some("ceq.tonal_batch.layout"),
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ceq.tonal_batch.pipeline"),
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

    /// Wraps [`Self::encode_into`] with upload + readback. Skips
    /// entirely when [`ColorEqualizationParams::tonal_is_identity`] holds.
    pub fn dispatch(
        &self,
        gpu: &GpuContext,
        rgba: &mut [u8],
        w: u32,
        h: u32,
        params: &ColorEqualizationParams,
    ) {
        let expected = (w as usize) * (h as usize) * 4;
        assert_eq!(rgba.len(), expected, "rgba length must match w*h*4");
        if w == 0 || h == 0 || params.tonal_is_identity() {
            return;
        }
        let input_tex = make_input_texture(gpu, "ceq.tonal_batch.input", rgba, w, h);
        let output_tex = make_storage_texture(gpu, "ceq.tonal_batch.output", w, h);
        let input_view = input_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ceq.tonal_batch.encoder"),
            });
        self.encode_into(gpu, &mut encoder, &input_view, &output_view, w, h, params);
        readback_into(&mut encoder, gpu, &output_tex, rgba, w, h);
    }

    /// Encode the fused 7-stage tonal compute pass into `encoder`.
    /// Bradford matrix is pre-computed host-side; uniform buffer + bind
    /// group created here and retained via wgpu Arc semantics.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_into(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        w: u32,
        h: u32,
        params: &ColorEqualizationParams,
    ) {
        if w == 0 || h == 0 || params.tonal_is_identity() {
            return;
        }
        let (temperature_active, matrix) = if params.temperature != 0.0 {
            let kelvin = crate::algorithm::temperature01_to_kelvin(params.temperature);
            (1_u32, bradford_matrix_for_kelvin(kelvin))
        } else {
            (0_u32, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0_f32])
        };
        let uniforms = TonalUniforms {
            exposure: params.exposure,
            tint: params.tint,
            brightness: params.brightness,
            contrast: params.contrast,
            vibrance: params.vibrance,
            saturation: params.saturation,
            temperature_active,
            _pad0: 0,
            temp_row0: [matrix[0], matrix[1], matrix[2], 0.0],
            temp_row1: [matrix[3], matrix[4], matrix[5], 0.0],
            temp_row2: [matrix[6], matrix[7], matrix[8], 0.0],
        };
        let uniform_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ceq.tonal_batch.uniforms"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ceq.tonal_batch.bind_group"),
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
            label: Some("ceq.tonal_batch.pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(w.div_ceil(WORKGROUP_SIZE), h.div_ceil(WORKGROUP_SIZE), 1);
    }
}

/// Convenience: build pipeline + dispatch. Use
/// [`TonalBatchPipeline::new`] directly when chaining many calls.
pub fn adjust_tonal_gpu(
    rgba: &mut [u8],
    w: u32,
    h: u32,
    params: &ColorEqualizationParams,
    gpu: &GpuContext,
) {
    let pipeline = TonalBatchPipeline::new(gpu);
    pipeline.dispatch(gpu, rgba, w, h, params);
}

/// WGSL compute kernel: full Phase 1 tonal stack fused per-pixel.
/// Order mirrors [`crate::algorithm::adjust_tonal`] exactly so the
/// parity test holds within `ε ≤ 4 LSB` per channel.
const TONAL_BATCH_WGSL: &str = r#"
struct Uniforms {
    exposure: f32,
    tint: f32,
    brightness: f32,
    contrast: f32,
    vibrance: f32,
    saturation: f32,
    temperature_active: u32,
    _pad0: u32,
    temp_row0: vec4<f32>,
    temp_row1: vec4<f32>,
    temp_row2: vec4<f32>,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> u: Uniforms;

// ── sRGB transfer (IEC 61966-2-1) ────────────────────────────────
fn srgb_to_linear(c: f32) -> f32 {
    if (c <= 0.04045) {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn linear_to_srgb(c: f32) -> f32 {
    let cm = max(c, 0.0);
    if (cm <= 0.0031308) {
        return cm * 12.92;
    }
    return 1.055 * pow(cm, 1.0 / 2.4) - 0.055;
}

// ── Stage primitives ─────────────────────────────────────────────
fn soft_knee(v: f32) -> f32 {
    if (v <= 0.8) { return v; }
    return 0.8 + 0.2 * (1.0 - exp(-(v - 0.8) * 2.0));
}

fn s_curve_channel(c: f32, contrast: f32) -> f32 {
    let strength = (contrast - 1.0) * 2.0;
    let pivot = 0.18;
    let centered = c - pivot;
    let sig = select(-1.0, 1.0, centered >= 0.0);
    let abs_c = abs(centered);
    // c > 1: S-curve (steepens midtones); c < 1: flattens.
    let curved_above = abs_c * (1.0 + strength * (1.0 - abs_c));
    let curved_below = abs_c * (1.0 + strength * abs_c);
    let curved = select(curved_below, curved_above, contrast > 1.0);
    return clamp(pivot + sig * curved, 0.0, 1.0);
}

// `cbrt(x) = sign(x) · pow(|x|, 1/3)` — preserves sign for negatives
// (linear sRGB can dip below 0 mid-pipeline after temperature / tint).
fn cbrt_signed(x: f32) -> f32 {
    return sign(x) * pow(abs(x), 1.0 / 3.0);
}

// Björn Ottosson 2020 — same matrices as `color_utils::linear_rgb_to_oklab`.
fn linear_to_oklab(rgb: vec3<f32>) -> vec3<f32> {
    let l = 0.41222147 * rgb.r + 0.53633254 * rgb.g + 0.05144599 * rgb.b;
    let m = 0.21190350 * rgb.r + 0.68069955 * rgb.g + 0.10739696 * rgb.b;
    let s = 0.08830246 * rgb.r + 0.28171884 * rgb.g + 0.62997870 * rgb.b;
    let lp = cbrt_signed(l);
    let mp = cbrt_signed(m);
    let sp = cbrt_signed(s);
    return vec3<f32>(
        0.21045425 * lp + 0.79361780 * mp - 0.00407205 * sp,
        1.97799850 * lp - 2.42859220 * mp + 0.45059371 * sp,
        0.02590404 * lp + 0.78277177 * mp - 0.80867577 * sp,
    );
}

fn oklab_to_linear(lab: vec3<f32>) -> vec3<f32> {
    let lp = lab.x + 0.39633778 * lab.y + 0.21580376 * lab.z;
    let mp = lab.x - 0.10556135 * lab.y - 0.06385417 * lab.z;
    let sp = lab.x - 0.08948418 * lab.y - 1.29148550 * lab.z;
    let l = lp * lp * lp;
    let m = mp * mp * mp;
    let s = sp * sp * sp;
    return vec3<f32>(
         4.07674170 * l - 3.30771160 * m + 0.23096994 * s,
        -1.26843800 * l + 2.60975740 * m - 0.34131938 * s,
        -0.00419609 * l - 0.70341860 * m + 1.70761470 * s,
    );
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let coord = vec2<i32>(i32(id.x), i32(id.y));
    let pixel = textureLoad(input_tex, coord, 0);

    if (pixel.a == 0.0) {
        // Match CPU: transparent passes through untouched (RGB
        // undefined per straight-alpha contract).
        textureStore(output_tex, coord, pixel);
        return;
    }

    // sRGB → linear.
    var rgb = vec3<f32>(
        srgb_to_linear(pixel.r),
        srgb_to_linear(pixel.g),
        srgb_to_linear(pixel.b),
    );

    // ── Exposure (EV stops + soft-knee). ─────────────────────────
    if (u.exposure != 0.0) {
        let mul = pow(2.0, u.exposure);
        rgb = vec3<f32>(
            soft_knee(rgb.r * mul),
            soft_knee(rgb.g * mul),
            soft_knee(rgb.b * mul),
        );
    }

    // ── Temperature (Bradford 3×3 matrix). ───────────────────────
    if (u.temperature_active != 0u) {
        let new_r = u.temp_row0.x * rgb.r + u.temp_row0.y * rgb.g + u.temp_row0.z * rgb.b;
        let new_g = u.temp_row1.x * rgb.r + u.temp_row1.y * rgb.g + u.temp_row1.z * rgb.b;
        let new_b = u.temp_row2.x * rgb.r + u.temp_row2.y * rgb.g + u.temp_row2.z * rgb.b;
        rgb = vec3<f32>(new_r, new_g, new_b);
    }

    // ── Tint (luminance-preserving green ↔ magenta). ─────────────
    if (u.tint != 0.0) {
        let t = clamp(u.tint, -1.0, 1.0);
        let g_shift = -t * 0.05;
        let r_comp = t * 0.05 * 0.7152 / 0.2126;
        let b_comp = t * 0.05 * 0.7152 / 0.0722;
        rgb = vec3<f32>(
            rgb.r * (1.0 + r_comp),
            rgb.g * (1.0 + g_shift),
            rgb.b * (1.0 + b_comp),
        );
    }

    // ── Brightness (multiplicative — preserves blacks). ──────────
    if (u.brightness != 0.0) {
        let m = 1.0 + clamp(u.brightness, -1.0, 1.0);
        rgb = rgb * m;
    }

    // ── Contrast (S-curve around 0.18 perceptual midpoint). ──────
    if (u.contrast != 1.0) {
        rgb = vec3<f32>(
            s_curve_channel(rgb.r, u.contrast),
            s_curve_channel(rgb.g, u.contrast),
            s_curve_channel(rgb.b, u.contrast),
        );
    }

    // ── OKLab stages (Vibrance + Saturation). ────────────────────
    // Gated together so identity-tonal pixels skip the cbrt+cube
    // round-trip — matches CPU `needs_oklab` short-circuit.
    if (u.vibrance != 0.0 || u.saturation != 0.0) {
        var lab = linear_to_oklab(rgb);
        if (u.vibrance != 0.0) {
            let vn = clamp(u.vibrance, -1.0, 1.0);
            let chroma = sqrt(lab.y * lab.y + lab.z * lab.z);
            if (chroma > 0.0) {
                let chroma_norm = min(chroma / 0.15, 1.0);
                let boost = vn * (1.0 - chroma_norm * chroma_norm);
                let factor = max(1.0 + boost, 0.0);
                lab = vec3<f32>(lab.x, lab.y * factor, lab.z * factor);
            }
        }
        if (u.saturation != 0.0) {
            let sat_mult = max(1.0 + clamp(u.saturation, -1.0, 1.0), 0.0);
            lab = vec3<f32>(lab.x, lab.y * sat_mult, lab.z * sat_mult);
        }
        rgb = oklab_to_linear(lab);
    }

    // linear → sRGB.
    let out_rgb = vec3<f32>(
        linear_to_srgb(rgb.r),
        linear_to_srgb(rgb.g),
        linear_to_srgb(rgb.b),
    );
    textureStore(output_tex, coord, vec4<f32>(out_rgb, pixel.a));
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::adjust_tonal;
    use crate::gpu::try_headless_gpu;

    /// 24×24 diverse-colour ramp (varies every component independently
    /// so each stage exercises a fresh distribution).
    fn ramp(w: u32, h: u32) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let r = ((x * 17 + y * 5) % 256) as u8;
                let g = ((x * 7 + y * 23) % 256) as u8;
                let b = ((x * 13 + y * 41) % 256) as u8;
                v.extend_from_slice(&[r, g, b, 255]);
            }
        }
        v
    }

    fn assert_within_lsb(cpu: &[u8], gpu: &[u8], max_lsb: i32, ctx: &str) {
        assert_eq!(cpu.len(), gpu.len());
        let mut worst = 0_i32;
        let mut worst_idx = 0;
        let mut diff_count = 0_u64;
        let mut sum_diff = 0_u64;
        for (i, (a, b)) in cpu.iter().zip(gpu.iter()).enumerate() {
            let d = (*a as i32 - *b as i32).abs();
            if d > worst {
                worst = d;
                worst_idx = i;
            }
            if d > 0 {
                diff_count += 1;
                sum_diff += d as u64;
            }
        }
        assert!(
            worst <= max_lsb,
            "{ctx}: CPU/GPU diverged by {worst} LSB at idx {worst_idx} \
             (cpu {} vs gpu {}); mean delta {:.2} over {} pixels",
            cpu[worst_idx],
            gpu[worst_idx],
            sum_diff as f64 / diff_count.max(1) as f64,
            diff_count,
        );
    }

    #[test]
    fn tonal_gpu_identity_is_noop() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(16, 16);
        let mut buf = src.clone();
        let params = ColorEqualizationParams::default();
        adjust_tonal_gpu(&mut buf, 16, 16, &params, &gpu);
        // tonal_is_identity short-circuits before dispatch.
        assert_eq!(buf, src);
    }

    #[test]
    fn tonal_gpu_matches_cpu_brightness_only() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(24, 24);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        let params = ColorEqualizationParams {
            brightness: 0.4,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut cpu, &params);
        adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3, "brightness=0.4");
    }

    #[test]
    fn tonal_gpu_matches_cpu_contrast_above_one() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(24, 24);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        let params = ColorEqualizationParams {
            contrast: 1.5,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut cpu, &params);
        adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3, "contrast=1.5");
    }

    #[test]
    fn tonal_gpu_matches_cpu_contrast_below_one() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(24, 24);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        let params = ColorEqualizationParams {
            contrast: 0.7,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut cpu, &params);
        adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3, "contrast=0.7");
    }

    #[test]
    fn tonal_gpu_matches_cpu_temperature_warm() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(24, 24);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        let params = ColorEqualizationParams {
            temperature: 0.6,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut cpu, &params);
        adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3, "temperature=+0.6 (warm)");
    }

    #[test]
    fn tonal_gpu_matches_cpu_temperature_cool() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(24, 24);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        let params = ColorEqualizationParams {
            temperature: -0.6,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut cpu, &params);
        adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3, "temperature=-0.6 (cool)");
    }

    #[test]
    fn tonal_gpu_matches_cpu_tint() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(24, 24);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        let params = ColorEqualizationParams {
            tint: 0.5,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut cpu, &params);
        adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3, "tint=+0.5 (magenta)");
    }

    #[test]
    fn tonal_gpu_matches_cpu_exposure() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(24, 24);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        let params = ColorEqualizationParams {
            exposure: 1.0, // +1 EV
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut cpu, &params);
        adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3, "exposure=+1 EV");
    }

    #[test]
    fn tonal_gpu_matches_cpu_vibrance() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(24, 24);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        let params = ColorEqualizationParams {
            vibrance: 0.5,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut cpu, &params);
        adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
        // OKLab cbrt + cube introduces a touch more drift than the
        // linear-sRGB stages — observed worst-case is 3 LSB.
        assert_within_lsb(&cpu, &gpu_buf, 3, "vibrance=0.5");
    }

    #[test]
    fn tonal_gpu_matches_cpu_saturation_grayscale() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(24, 24);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        let params = ColorEqualizationParams {
            saturation: -1.0,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut cpu, &params);
        adjust_tonal_gpu(&mut gpu_buf, 24, 24, &params, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 3, "saturation=-1 (full desat)");
    }

    #[test]
    fn tonal_gpu_matches_cpu_full_stack() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(32, 32);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        let params = ColorEqualizationParams {
            exposure: 0.5,
            temperature: 0.3,
            tint: -0.2,
            brightness: 0.15,
            contrast: 1.2,
            vibrance: 0.3,
            saturation: 0.4,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut cpu, &params);
        adjust_tonal_gpu(&mut gpu_buf, 32, 32, &params, &gpu);
        // 7 stages compose → accumulated rounding peaks around 4 LSB
        // in observed runs.
        assert_within_lsb(&cpu, &gpu_buf, 4, "full Phase 1 stack");
    }

    #[test]
    fn tonal_gpu_skips_transparent_pixels() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut buf = vec![100u8, 150, 200, 0, 100, 150, 200, 255];
        let params = ColorEqualizationParams {
            brightness: 0.8,
            saturation: -1.0,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal_gpu(&mut buf, 2, 1, &params, &gpu);
        // Transparent pixel: passthrough (matches CPU).
        assert_eq!(&buf[0..4], &[100, 150, 200, 0]);
        // Opaque pixel must have been adjusted.
        assert_ne!(&buf[4..7], &[100, 150, 200]);
    }

    #[test]
    fn tonal_gpu_handles_non_workgroup_aligned_dimensions() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let src = ramp(13, 19);
        let mut cpu = src.clone();
        let mut gpu_buf = src.clone();
        let params = ColorEqualizationParams {
            brightness: 0.2,
            contrast: 1.3,
            saturation: 0.5,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut cpu, &params);
        adjust_tonal_gpu(&mut gpu_buf, 13, 19, &params, &gpu);
        assert_within_lsb(&cpu, &gpu_buf, 4, "13×19 non-aligned");
    }
}
