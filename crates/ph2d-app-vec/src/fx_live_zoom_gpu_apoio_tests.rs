//! Corte F37 (teto de LOC) de `fx_live_zoom_gpu_tests.rs`: os apoios da placa (textura, disco,
//! upload, readback e a corrida da pilha) que os testes G5 partilham.

use super::super::{Cena, Job};
use ph2d_render::{FxStackPass, Origem};
use ph2d_vector::Affine;

pub(super) fn gpu() -> Option<ph2d_gpu::GpuContext> {
    ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
}

pub(super) fn tex(gpu: &ph2d_gpu::GpuContext, w: u32, h: u32) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("F34 paridade"),
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
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// O disco de raio `R·zoom` centrado em `centro` (tela), premultiplicado, na janela `[ex0, ey0]+w×h`.
pub(super) fn disco(w: u32, h: u32, ex0: f64, ey0: f64, centro: [f64; 2], raio: f64) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = (ex0 + f64::from(x) + 0.5, ey0 + f64::from(y) + 0.5);
            let d = (sx - centro[0]).hypot(sy - centro[1]);
            let a = (raio - d + 0.5).clamp(0.0, 1.0);
            let o = ((y * w + x) * 4) as usize;
            let c = |v: f64| (v * a).round() as u8;
            out[o..o + 4].copy_from_slice(&[c(235.0), c(175.0), c(60.0), c(255.0)]);
        }
    }
    out
}

pub(super) fn upload(gpu: &ph2d_gpu::GpuContext, t: &wgpu::Texture, w: u32, h: u32, bytes: &[u8]) {
    gpu.queue.write_texture(
        t.as_image_copy(),
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
}

pub(super) fn readback(gpu: &ph2d_gpu::GpuContext, t: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let row = (w * 4).div_ceil(256) * 256;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("F34 readback"),
        size: u64::from(row * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
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
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let data = buf.slice(..).get_mapped_range();
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let o = (y * row) as usize;
        out.extend_from_slice(&data[o..o + (w * 4) as usize]);
    }
    out
}

/// Corre a pilha do job sobre o disco, como o `cook_batch` (segmentos recortados, origem da forma).
pub(super) fn corre(
    gpu: &ph2d_gpu::GpuContext,
    pass: &mut FxStackPass,
    c: &Cena,
    cam: Affine,
    job: &Job,
    centro: [f64; 2],
    raio: f64,
) -> Vec<u8> {
    let (w, h) = (job.key.w, job.key.h);
    let src = tex(gpu, w, h);
    upload(
        gpu,
        &src,
        w,
        h,
        &disco(w, h, job.ex0, job.ey0, centro, raio),
    );
    let dst = tex(gpu, w, h);
    // Como o `cook_batch`: a geometria que o `job_for` já recortou, ou a de sempre.
    let geom = if let Some((o, segs)) = job.geom.as_ref() {
        let (dx, dy) = ((o[0] - job.ex0) as f32, (o[1] - job.ey0) as f32);
        segs.iter()
            .map(|&[a, b, c, d]| [a + dx, b + dy, c + dx, d + dy])
            .collect()
    } else {
        let alc = ph2d_render::stack_read(&job.key.ops) as f32;
        c.segs(cam, job, [-alc, -alc, w as f32 + alc, h as f32 + alc])
    };
    let origem = Origem::Ponto(job.ruido);
    pass.run_from_com_origem(gpu, &src, [0, 0], &dst, w, h, &job.key.ops, &geom, origem);
    readback(gpu, &dst, w, h)
}
