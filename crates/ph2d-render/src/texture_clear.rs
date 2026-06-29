//! Clear-on-alloc helpers for GPU textures that may be SAMPLED before their first full write.
//!
//! wgpu textures are undefined memory until written. A texture that some code path can read before its
//! first upload/copy lands (e.g. the Painter GPU-preview slot, acquired empty by
//! [`crate::IndividualTextureStore`] and filled by a later region copy) must be cleared on creation, or
//! an early frame samples garbage. Kept in its own module so the (frozen, LOC-capped) `individual.rs`
//! only calls in.

use ph2d_gpu::GpuContext;

/// Clear EVERY mip level of `texture` to transparent via no-draw render passes — the texture must carry
/// `RENDER_ATTACHMENT` (it does, for the mip generator). The whole chain is cleared because the trilinear
/// sampler can pick any level and `regen_mips` only runs after the first upload, so a level-0-only clear
/// would still leave the minified levels undefined. One-time cost at texture creation.
pub(crate) fn clear_all_mips_transparent(
    gpu: &GpuContext,
    texture: &wgpu::Texture,
    mip_count: u32,
) {
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-render texture clear-on-alloc encoder"),
        });
    for level in 0..mip_count {
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("ph2d-render texture clear-on-alloc view"),
            base_mip_level: level,
            mip_level_count: Some(1),
            ..Default::default()
        });
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-render texture clear-on-alloc pass"),
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
    }
    gpu.queue.submit([encoder.finish()]);
}
