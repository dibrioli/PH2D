//! Painter GPU preview premultiply pass — parity gate (ADR-0045 Phase 3 step 2).
//!
//! `#[ignore]`d: needs a real device, so it runs on a developer machine / the
//! GPU CI lane, not the headless CPU runners (which `None` out of
//! `try_headless_gpu`). It proves the `PreviewPremul` compute blit agrees with
//! the CPU `premultiply_rgba8` (the byte-space premultiply the CPU preview path
//! uploads) to 0 levels per channel (B20.8) — the bound that makes the GPU live preview
//! byte-identical to the committed result on the shared `Rgba8UnormSrgb` +
//! premul-blend sprite shader.
//!
//! Run with:
//!   cargo test -p ph2d-render --test preview_premul_gpu -- --ignored

use ph2d_gpu::GpuContext;
use ph2d_render::{PreviewPremul, premultiply_rgba8};

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// A straight-sRGB8 `Rgba8Unorm` source texture holding `bytes`.
fn make_src(gpu: &GpuContext, w: u32, h: u32, bytes: &[u8]) -> wgpu::Texture {
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test premul src"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    tex
}

/// Block-on-GPU readback of an `rgba8unorm` texture to a tight `w*h*4` buffer.
fn readback(gpu: &GpuContext, tex: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let unpadded = w * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test premul readback"),
        size: (padded as u64) * (h as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().expect("recv").expect("map");
    let mapped = staging.slice(..).get_mapped_range().expect("mapped range");
    let mut out = Vec::with_capacity((unpadded as usize) * (h as usize));
    for row in 0..h as usize {
        let start = row * padded as usize;
        out.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}

#[test]
#[ignore = "needs a GPU device (GPU CI lane)"]
fn preview_premul_matches_cpu_reference() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no headless GPU; skipping preview_premul_matches_cpu_reference");
        return;
    };
    // Straight-sRGB8 pixels spanning the alpha range + varied colours, incl.
    // the identity (opaque), the collapse (transparent) and near-zero alphas.
    let straight: Vec<u8> = vec![
        255, 0, 0, 255, // opaque red — identity
        0, 255, 0, 128, // half-alpha green
        10, 200, 30, 64, // quarter-ish
        123, 45, 200, 0, // transparent → rgb*0
        200, 200, 200, 200, // grey
        7, 99, 250, 1, // near-zero alpha
        255, 255, 255, 17, // bright, low alpha
        64, 32, 16, 250, // near-opaque
    ];
    let w = (straight.len() / 4) as u32;
    let h = 1;

    let src = make_src(&gpu, w, h, &straight);
    let mut premul = PreviewPremul::new(&gpu);
    let out_tex = premul.run(&gpu, &src, w, h).expect("premul ran");
    let gpu_bytes = readback(&gpu, out_tex, w, h);

    let mut cpu = straight.clone();
    premultiply_rgba8(&mut cpu);

    assert_eq!(gpu_bytes.len(), cpu.len(), "readback size");
    for (i, (g, c)) in gpu_bytes.iter().zip(cpu.iter()).enumerate() {
        let d = (*g as i32 - *c as i32).abs();
        assert!(
            d == 0,
            "byte {i} (px {}, ch {}): GPU {g} vs CPU {c} (delta {d}; B20.8: 0 levels)",
            i / 4,
            i % 4,
        );
    }
}

#[test]
#[ignore = "needs a GPU device (GPU CI lane)"]
fn rgba8unorm_copies_byte_identical_into_srgb_slot() {
    // The bridge copies the premultiplied `rgba8unorm` output straight into the
    // `Rgba8UnormSrgb` preview slot (`IndividualTextureStore::copy_from_texture`).
    // Prove on real hardware that `copy_texture_to_texture` across the sRGB-suffix
    // boundary is byte-identical (the formats are copy-compatible — same texel
    // block; the sRGB-ness only changes how the sprite shader DECODES at sample
    // time, which is exactly the premultiplied→linear step the shader wants).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no headless GPU; skipping rgba8unorm_copies_byte_identical_into_srgb_slot");
        return;
    };
    let w = 4;
    let h = 4;
    let bytes: Vec<u8> = (0..(w * h * 4)).map(|i| (i % 251) as u8).collect();
    let src = make_src(&gpu, w, h, &bytes); // rgba8unorm, COPY_SRC

    let dst = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test premul srgb dst (mirror of the individual slot)"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &src,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: &dst,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);

    let got = readback(&gpu, &dst, w, h);
    assert_eq!(
        got, bytes,
        "rgba8unorm → Rgba8UnormSrgb copy must be byte-identical"
    );
}

/// B20.1: the premultiply of a dirty rect writes ONLY that rect (the rest keeps the last frame) and,
/// inside it, the same bytes as the whole-canvas pass. Odd sizes and a rect that touches no workgroup edge.
#[test]
#[ignore = "needs a GPU device (GPU CI lane)"]
fn o_premul_da_regiao_so_escreve_a_regiao() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no headless GPU; skipping o_premul_da_regiao_so_escreve_a_regiao");
        return;
    };
    let (w, h) = (37u32, 29u32);
    let px = |s: u32| -> Vec<u8> {
        (0..w * h * 4)
            .map(|i| ((i.wrapping_mul(2_654_435_761) >> 7).wrapping_add(s) % 256) as u8)
            .collect()
    };
    let (a, b) = (px(0), px(91));
    let (src_a, src_b) = (make_src(&gpu, w, h, &a), make_src(&gpu, w, h, &b));
    let mut todo = PreviewPremul::new(&gpu);
    let esperado_b = readback(&gpu, todo.run(&gpu, &src_b, w, h).expect("premul"), w, h);
    let mut premul = PreviewPremul::new(&gpu);
    let esperado_a = readback(&gpu, premul.run(&gpu, &src_a, w, h).expect("premul"), w, h);
    for rect in [
        (5u32, 3u32, 11u32, 9u32),
        (0, 0, w, 1),
        (36, 28, 1, 1),
        (0, 0, w, h),
    ] {
        let _ = premul.run(&gpu, &src_a, w, h);
        let got = readback(
            &gpu,
            premul.run_region(&gpu, &src_b, w, h, rect).expect("premul"),
            w,
            h,
        );
        let (rx, ry, rw, rh) = rect;
        for y in 0..h {
            for x in 0..w {
                let dentro = x >= rx && x < rx + rw && y >= ry && y < ry + rh;
                let i = ((y * w + x) * 4) as usize;
                let quer = if dentro { &esperado_b } else { &esperado_a };
                assert_eq!(
                    &got[i..i + 4],
                    &quer[i..i + 4],
                    "rect {rect:?}, px ({x},{y}), dentro={dentro}"
                );
            }
        }
    }
}

/// n.º 10 (B20.8): a prévia da placa pré-multiplica com a lei INTEIRA da CPU, `(c·a + 127) / 255`, e dá o byte
/// dela em TODOS os 65 536 pares (c, α) — 0 níveis, nunca o ±1 do produto em `f32` arredondado pelo driver.
/// Cada canal varre os 256 c em ordens diferentes (r = c, g = 255 − c, b = 37·c mod 256) contra os 256 α.
#[test]
#[ignore = "needs a GPU device (GPU CI lane)"]
fn o_premul_da_placa_e_o_byte_da_cpu_nos_65536_pares() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no headless GPU; skipping o_premul_da_placa_e_o_byte_da_cpu_nos_65536_pares");
        return;
    };
    let (w, h) = (256u32, 256u32);
    let mut straight = Vec::with_capacity(256 * 256 * 4);
    for a in 0..=255u8 {
        for c in 0..=255u8 {
            straight.extend_from_slice(&[c, 255 - c, c.wrapping_mul(37), a]);
        }
    }
    let src = make_src(&gpu, w, h, &straight);
    let mut premul = PreviewPremul::new(&gpu);
    let out_tex = premul.run(&gpu, &src, w, h).expect("premul ran");
    let gpu_bytes = readback(&gpu, out_tex, w, h);
    let mut cpu = straight.clone();
    premultiply_rgba8(&mut cpu);
    let diferentes = gpu_bytes.iter().zip(&cpu).filter(|(g, c)| g != c).count();
    let maximo = (gpu_bytes.iter().zip(&cpu))
        .map(|(g, c)| (*g as i32 - *c as i32).abs())
        .max()
        .unwrap_or(0);
    eprintln!("B20.8\t65 536 pares\tbytes diferentes {diferentes}\tmáx {maximo}");
    assert_eq!(
        diferentes, 0,
        "a placa dá o byte da CPU (máx {maximo} níveis)"
    );
}
