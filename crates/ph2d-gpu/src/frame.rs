//! FrameTarget — RAII handle for a single frame's render target.
//!
//! Acquired via [`crate::SurfaceContext::acquire_frame`]. Auto-presents
//! on `Drop` (so caller can use `?` without leaking a frame). Use
//! [`FrameTarget::clear`] for the M3 minimal "fill with color" case;
//! M5+ will add render-graph integration.

use crate::context::GpuContext;

pub struct FrameTarget {
    texture: Option<wgpu::SurfaceTexture>,
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    gpu: GpuContext,
}

impl FrameTarget {
    pub(crate) fn new(
        texture: wgpu::SurfaceTexture,
        format: wgpu::TextureFormat,
        gpu: GpuContext,
    ) -> Self {
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("ph2d-gpu frame view"),
            format: Some(format),
            ..Default::default()
        });
        Self {
            texture: Some(texture),
            view,
            format,
            gpu,
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Encode + submit a single render pass that clears the surface
    /// with `color`. Minimal M3 path; M5 replaces this with the
    /// render-graph traversal.
    pub fn clear(&self, color: wgpu::Color) {
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-gpu clear encoder"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-gpu clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        self.gpu.queue.submit(Some(encoder.finish()));
    }

    /// Explicit present. Idempotent — calling more than once is a
    /// no-op. Drop also presents if not already done.
    pub fn present(mut self) {
        if let Some(tex) = self.texture.take() {
            tex.present();
        }
    }
}

impl Drop for FrameTarget {
    fn drop(&mut self) {
        if let Some(tex) = self.texture.take() {
            tex.present();
        }
    }
}
