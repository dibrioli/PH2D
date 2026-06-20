//! GPU-side texture operations for the atlas: region upload, texture
//! creation (with full mip chain), the transparent level-0 clear, and
//! the diagnostics readback. These wrap raw `wgpu` calls so the
//! `TextureAtlas` orchestrator stays focused on packing/free-list state.

use super::region::AtlasRegion;
use ph2d_gpu::GpuContext;

/// Upload `rgba` into `texture` at `region`'s pixel offset.
/// `bytes_per_row` matches the region's stride exactly — wgpu's
/// 256-byte alignment requirement only applies to
/// `copy_buffer_to_texture`, NOT `write_texture` (which takes the
/// stride verbatim and handles internal staging).
pub(super) fn upload_region(
    gpu: &GpuContext,
    texture: &wgpu::Texture,
    region: AtlasRegion,
    rgba: &[u8],
) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: region.x,
                y: region.y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(region.w * 4),
            rows_per_image: Some(region.h),
        },
        wgpu::Extent3d {
            width: region.w,
            height: region.h,
            depth_or_array_layers: 1,
        },
    );
}

pub(super) fn create_texture(
    device: &wgpu::Device,
    size_px: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render atlas"),
        size: wgpu::Extent3d {
            width: size_px,
            height: size_px,
            depth_or_array_layers: 1,
        },
        // Full mip chain (2026-06-18 Phase 2). +33 % memory over a
        // single-level texture (8192² RGBA8: 256 → ~341 MiB) buys
        // trilinear/anisotropic minification for every atlas sprite.
        mip_level_count: crate::mipgen::mip_levels(size_px, size_px),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        // RENDER_ATTACHMENT lets `MipGenerator` blit each mip level;
        // COPY_SRC feeds `readback_mip` (tests/diagnostics).
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    // The sampled view spans ALL mip levels (default) so the trilinear
    // sampler can pick the right level; the generator makes its own
    // single-level views.
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// Clear mip level 0 of `texture` to transparent via a no-draw render
/// pass. Run once at atlas creation / regrow so untouched packing space
/// downsamples toward transparent (a correct premultiplied edge) rather
/// than bleeding undefined garbage into region-edge mips.
pub(super) fn clear_level0_transparent(gpu: &GpuContext, texture: &wgpu::Texture) {
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("ph2d-render atlas clear view"),
        base_mip_level: 0,
        mip_level_count: Some(1),
        ..Default::default()
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-render atlas clear encoder"),
        });
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("ph2d-render atlas clear pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    gpu.queue.submit([encoder.finish()]);
}

/// Copy `mip_level` (a `width × height` region from the texture origin)
/// out of GPU memory into a tightly-packed `Vec<u8>`. Mirrors the
/// individual store's `readback_texture`: pads rows to
/// [`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`] for the staging copy, then
/// strips the padding. One-shot blocking map — diagnostics/tests only.
pub(super) fn readback_atlas_texture(
    gpu: &GpuContext,
    texture: &wgpu::Texture,
    mip_level: u32,
    width: u32,
    height: u32,
) -> Vec<u8> {
    if width == 0 || height == 0 {
        return Vec::new();
    }
    let unpadded_bpr = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let buffer_size = (padded_bpr as u64) * (height as u64);

    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-render atlas readback staging"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-render atlas readback encoder"),
        });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);

    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("atlas readback poll");
    rx.recv()
        .expect("atlas readback channel")
        .expect("atlas readback map");

    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded_bpr as usize) * (height as usize));
    for row in 0..height as usize {
        let start = row * padded_bpr as usize;
        let end = start + unpadded_bpr as usize;
        out.extend_from_slice(&mapped[start..end]);
    }
    drop(mapped);
    staging.unmap();
    out
}
