//! Phase 4 — wgpu compute paths for the Color Equalization pipeline.
//!
//! Each stage in the CPU pipeline ([`crate::algorithm`]) that is
//! embarrassingly parallel per pixel has (or will have) a WGSL compute
//! twin here. Every twin is pinned by a **parity test** that asserts
//! `|cpu − gpu| ≤ 1 LSB` per channel on identical input — so the shell
//! can flip a stage from CPU to GPU without diff'ing pixels.
//!
//! Current status (vs. the GPU analysis matrix in
//! [`crate::algorithm`]'s module docs):
//!
//! | Stage                  | CPU | GPU | Status |
//! |------------------------|-----|-----|--------|
//! | LUT3D apply (trilinear)| ✓   | ✓   | [`lut_apply`] — Phase 4 |
//! | Bilateral denoise      | ✓   | ✓   | [`bilateral`] — Phase 4.1 (heaviest CPU stage) |
//! | Phase 1 tonal batch    | ✓   | ✓   | [`tonal_batch`] — Phase 4.2 (7 stages fused, single sRGB ↔ linear ↔ OKLab round-trip) |
//! | Sharpen Laplacian      | ✓   | ✓   | [`sharpen`] — Phase 4.3 (3×3, 1 pass, 4-neighbour) |
//! | Sharpen Unsharp (Gauss)| ✓   | ✓   | [`sharpen`] — Phase 4.3 (separable H + V combine; the big GPU win for radius > 3) |
//! | Auto-WB reduce + apply | ✓   | ✓   | [`auto_wb`] — Phase 4.4 (workgroup-shared atomic reduce + per-pixel rescale) |
//! | CLAHE histogram        | ✓   |     | CPU stays (atomic contention) — only LUT apply needs GPU |
//!
//! ## Architecture
//!
//! Each GPU primitive owns a `Pipeline` struct holding the compiled
//! [`wgpu::ComputePipeline`] + bind-group layout + reusable sampler.
//! Construction is one-time per [`GpuContext`]; dispatch creates the
//! per-call textures + buffers, runs the compute pass, then reads back
//! the result into a CPU `&mut [u8]`. The shell will eventually keep
//! pipelines + intermediate textures cached on the active tool — for
//! now `apply_*_gpu` convenience wrappers build the pipeline each call
//! (≈ 2 ms one-time cost; dwarfed by the multi-ms dispatch for any
//! useful image size).
//!
//! ## Testing
//!
//! [`try_headless_gpu`] mirrors `ph2d-render`'s pattern: lazy
//! `OnceLock<Option<GpuContext>>` so the 30-50 s Metal driver cold load
//! is paid once per test binary. Tests use `let Some(gpu) = … else {
//! return; };` so adapter-less CI runners skip cleanly.

pub mod auto_wb;
pub mod bilateral;
pub mod chain;
pub mod lut_apply;
pub mod sharpen;
pub mod tonal_batch;

pub use auto_wb::{AutoWbPipelines, auto_white_balance_gpu};
pub use bilateral::{BilateralPipeline, bilateral_denoise_gpu};
pub use chain::ChainedPipelineCache;
pub use lut_apply::{LutApplyPipeline, apply_lut3d_gpu};
pub use sharpen::{
    LaplacianSharpenPipeline, UnsharpSharpenPipeline, sharpen_laplacian_gpu, sharpen_unsharp_gpu,
};
pub use tonal_batch::{TonalBatchPipeline, adjust_tonal_gpu};

use ph2d_gpu::GpuContext;
use std::sync::OnceLock;

/// Lazily build (and cache) a headless [`GpuContext`] for tests. Same
/// pattern as `ph2d_render::game_rt::try_headless_gpu` — `request_device`
/// is ~30-50 s on Apple Silicon cold; we pay it once per test binary.
/// Returns `None` when no adapter is available (adapter-less CI), letting
/// callers `let Some(gpu) = … else { return; };` and skip.
pub fn try_headless_gpu() -> Option<GpuContext> {
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            let instance = GpuContext::default_instance();
            GpuContext::new(instance, None).ok()
        })
        .clone()
}

/// Process-wide cached [`chain::ChainedPipelineCache`] for the preview
/// path. Pipeline compile (~10-20 ms total) is paid once; subsequent
/// `run_chained` calls are pure dispatch + readback. Returns `None`
/// when no GPU is available. Distinct from the per-Apply
/// `ChainedPipelineCache::new` call site (which rebuilds the cache
/// each Apply — only matters at full-res, where the overhead is
/// dwarfed by the dispatch itself). The preview, by contrast, fires
/// on every slider release across N selected sprites; amortising the
/// compile is what makes the GPU preview viable.
pub fn try_preview_chain() -> Option<&'static chain::ChainedPipelineCache> {
    static CHAIN: OnceLock<Option<chain::ChainedPipelineCache>> = OnceLock::new();
    CHAIN
        .get_or_init(|| {
            let gpu = try_headless_gpu()?;
            Some(chain::ChainedPipelineCache::new(&gpu))
        })
        .as_ref()
}

// ── Cross-pipeline helpers ───────────────────────────────────────
//
// Five pipelines (LUT / Bilateral / Tonal / Sharpen × 2 / Auto-WB)
// repeated the same texture-creation + readback boilerplate. Lifted
// here so adding a sixth shader doesn't grow the per-module footprint
// (and so a future shell-side cache can swap these out wholesale for
// the texture-cached path without touching individual pipelines).

/// Build a `Rgba8Unorm` input texture (sampled, not storage) sized to
/// `(w, h)` and upload `rgba` into it via `queue.write_texture`.
pub(super) fn make_input_texture(
    gpu: &GpuContext,
    label: &str,
    rgba: &[u8],
    w: u32,
    h: u32,
) -> wgpu::Texture {
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
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

/// Build a write-only `Rgba8Unorm` storage texture sized to `(w, h)`
/// suitable for compute output + downstream `copy_texture_to_buffer`
/// readback.
pub(super) fn make_storage_texture(gpu: &GpuContext, label: &str, w: u32, h: u32) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// Copy `output_tex` → an intermediate `Buffer` with the wgpu-mandatory
/// `COPY_BYTES_PER_ROW_ALIGNMENT` row padding, submit the encoder, map
/// the buffer, then deinterleave the padded rows into `rgba` (which is
/// unpadded). Replaces `encoder` with a dummy so the caller's later
/// `encoder.finish()` call (if any) is a no-op.
pub(super) fn readback_into(
    encoder: &mut wgpu::CommandEncoder,
    gpu: &GpuContext,
    output_tex: &wgpu::Texture,
    rgba: &mut [u8],
    w: u32,
    h: u32,
) {
    let bytes_per_pixel = 4_u32;
    let unpadded_bpr = w * bytes_per_pixel;
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bpr = unpadded_bpr.div_ceil(alignment) * alignment;
    let readback_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ceq.shared.readback"),
        size: (padded_bpr as u64) * (h as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: output_tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback_buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    // Take ownership of the encoder so we can `finish()` it; the caller
    // gets back a fresh (empty) encoder that drops harmlessly.
    let cmd = std::mem::replace(
        encoder,
        gpu.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ceq.shared.readback_dummy"),
            }),
    );
    gpu.queue.submit(std::iter::once(cmd.finish()));

    let slice = readback_buf.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = sender.send(res);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll readback");
    match receiver.recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => panic!("readback map_async failed: {e}"),
        Err(_) => panic!("readback channel closed"),
    }
    let data = slice.get_mapped_range();
    for y in 0..h as usize {
        let src_off = y * padded_bpr as usize;
        let dst_off = y * unpadded_bpr as usize;
        rgba[dst_off..dst_off + unpadded_bpr as usize]
            .copy_from_slice(&data[src_off..src_off + unpadded_bpr as usize]);
    }
    drop(data);
    readback_buf.unmap();
}
