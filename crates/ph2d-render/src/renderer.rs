//! SpriteRenderer — owns the pipeline + per-frame buffers and renders
//! one batch (one atlas) of sprites into a [`ph2d_gpu::FrameTarget`].
//!
//! M5 scope is one batch (one atlas). Multi-atlas grouping + sort by
//! z lands in M6 with the asset pipeline. The render method:
//!   1. Pulls every `RenderInstance` from the [`PresentWorld`] into a
//!      scratch `Vec` (no allocation if capacity already sufficient).
//!   2. Uploads via `Queue::write_buffer` into the dynamic instance
//!      buffer.
//!   3. Writes the camera uniform.
//!   4. Issues a single instanced triangle-strip draw (4 vertices,
//!      N instances).

use crate::atlas::TextureAtlas;
use crate::camera::{Camera2d, CameraUniform};
use crate::instance_buffer::InstanceBuffer;
use crate::pipeline::SpritePipeline;
use crate::sprite::{QuadVertex, RenderInstance};
use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;

pub struct SpriteRenderer {
    gpu: GpuContext,
    pipeline: SpritePipeline,
    atlas: TextureAtlas,
    instance_buffer: InstanceBuffer,
    quad_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    frame_bind_group: wgpu::BindGroup,
    material_bind_group: wgpu::BindGroup,
    /// Reused scratch to avoid per-frame allocation when sprite count
    /// is stable (typical case).
    scratch: Vec<RenderInstance>,
}

impl SpriteRenderer {
    pub fn new(
        gpu: GpuContext,
        color_format: wgpu::TextureFormat,
        atlas: TextureAtlas,
        initial_instance_capacity: u32,
    ) -> Self {
        let pipeline = SpritePipeline::new(&gpu, color_format);

        let quad_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render quad vbo"),
            size: std::mem::size_of_val(&QuadVertex::QUAD_STRIP) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(
            &quad_buffer,
            0,
            bytemuck::cast_slice(&QuadVertex::QUAD_STRIP),
        );

        let camera_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render camera ubo"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let frame_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render frame bg"),
            layout: &pipeline.frame_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let material_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render material bg"),
            layout: &pipeline.material_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas.sampler),
                },
            ],
        });

        let instance_buffer = InstanceBuffer::new(&gpu, initial_instance_capacity);

        Self {
            gpu,
            pipeline,
            atlas,
            instance_buffer,
            quad_buffer,
            camera_buffer,
            frame_bind_group,
            material_bind_group,
            scratch: Vec::with_capacity(initial_instance_capacity as usize),
        }
    }

    pub fn atlas(&self) -> &TextureAtlas {
        &self.atlas
    }

    /// Insert a freshly-decoded source image into the renderer's
    /// atlas at native resolution, returning the packed region on
    /// success. Wraps [`TextureAtlas::insert`] so callers (the
    /// image-import path in `shells/desktop`) don't need to thread
    /// the [`GpuContext`] themselves.
    ///
    /// `rgba` must be tightly-packed `width * height * 4` bytes.
    /// Errors when the source is bigger than the atlas, or when
    /// the packer's skyline is exhausted. See
    /// [`AtlasInsertError`](crate::atlas::AtlasInsertError).
    pub fn insert_atlas_sprite(
        &mut self,
        key: u32,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<crate::atlas::AtlasRegion, crate::atlas::AtlasInsertError> {
        self.atlas.insert(&self.gpu, key, width, height, rgba)
    }

    /// Atlas V2 (M14.4f): insert with automatic regrow on
    /// `AtlasFull`. The atlas doubles its texture (capped at
    /// `atlas.max_size_px()`), re-packs every existing region using
    /// `fetch_pixels(key)` to recover the source bytes, rebuilds the
    /// material bind group to point at the new texture, then retries
    /// the original insert.
    ///
    /// `fetch_pixels` is the caller's hook into wherever it stores
    /// the asset source bytes (typically `AssetDb::get(asset_id) →
    /// Asset::ImageRgba8 { pixels, ... }`). Returning `None` for a
    /// key drops that region from the post-regrow atlas; surviving
    /// keys keep their old indices so render instances need no
    /// patching.
    ///
    /// Errors when the atlas is already at its cap, or when the
    /// caller's source dimensions exceed `max_size_px`.
    pub fn insert_atlas_sprite_with_regrow<F>(
        &mut self,
        key: u32,
        width: u32,
        height: u32,
        rgba: &[u8],
        fetch_pixels: F,
    ) -> Result<crate::atlas::AtlasRegion, crate::atlas::AtlasInsertError>
    where
        F: Fn(u32) -> Option<Vec<u8>>,
    {
        match self.atlas.insert(&self.gpu, key, width, height, rgba) {
            Ok(region) => Ok(region),
            Err(crate::atlas::AtlasInsertError::AtlasFull { .. }) => {
                let cap = self.atlas.max_size_px();
                let target = (self.atlas.size_px.saturating_mul(2)).min(cap);
                if target <= self.atlas.size_px {
                    // Already at cap — surface the original error so
                    // the shell can toast something actionable.
                    return Err(crate::atlas::AtlasInsertError::AtlasFull { width, height });
                }
                self.atlas.regrow_inplace(&self.gpu, target, fetch_pixels)?;
                self.rebuild_material_bind_group();
                // Retry — pure path now (no further regrow needed at
                // this granularity; the source that triggered full
                // necessarily fits in a 2× atlas if it fit pre-regrow
                // at all).
                self.atlas.insert(&self.gpu, key, width, height, rgba)
            }
            Err(other) => Err(other),
        }
    }

    /// Release `key`'s atlas slot back into the free-list. The next
    /// `insert_atlas_sprite` of the same dimensions will reuse the
    /// slot without re-packing. Returns the freed region for the
    /// caller's diagnostics, or `None` if `key` wasn't inserted.
    pub fn remove_atlas_sprite(&mut self, key: u32) -> Option<crate::atlas::AtlasRegion> {
        self.atlas.remove(key)
    }

    /// Rebuild the material bind group against the current
    /// `atlas.view`. Called internally after
    /// [`Self::insert_atlas_sprite_with_regrow`] re-creates the
    /// atlas texture; if a future path mutates the atlas externally
    /// (e.g. hot-reload swapping the whole texture) it can call this
    /// to keep the renderer pointing at fresh pixels.
    pub fn rebuild_material_bind_group(&mut self) {
        self.material_bind_group = self
            .gpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render material bg (regrown)"),
                layout: &self.pipeline.material_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.atlas.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.atlas.sampler),
                    },
                ],
            });
    }

    /// Render every `RenderInstance` in `present` into `target`.
    /// Loads with `clear_color` (single pass; M6+ may compose multiple).
    ///
    /// M14.5: `target` is now a generic `&wgpu::TextureView` so the
    /// caller can route the output into either the swap chain (legacy
    /// fixture demo) or an offscreen [`GameRt`](crate::GameRt) (live
    /// editor mode with compositor pass). The pipeline's color format
    /// (`color_format` passed to `SpritePipeline::new`) MUST match
    /// whatever this view points at — mismatch is a wgpu validation
    /// error caught at first draw.
    pub fn render(
        &mut self,
        target: &wgpu::TextureView,
        present: &mut PresentWorld,
        camera: &Camera2d,
        window: WindowSize,
        clear_color: wgpu::Color,
    ) {
        self.scratch.clear();
        let mut q = present.world_mut().query::<&RenderInstance>();
        for inst in q.iter(present.world()) {
            self.scratch.push(*inst);
        }
        let count = self
            .instance_buffer
            .upload(&self.gpu, self.scratch.as_slice());

        let camera_uniform = camera.uniform(window);
        self.gpu
            .queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&camera_uniform));

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render sprite encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ph2d-render sprite pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if count > 0 {
                pass.set_pipeline(&self.pipeline.pipeline);
                pass.set_bind_group(0, &self.frame_bind_group, &[]);
                pass.set_bind_group(1, &self.material_bind_group, &[]);
                pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                pass.set_vertex_buffer(1, self.instance_buffer.buffer().slice(..));
                pass.draw(0..4, 0..count);
            }
        }
        self.gpu.queue.submit(Some(encoder.finish()));
    }
}
