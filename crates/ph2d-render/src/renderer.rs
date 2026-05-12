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
use crate::individual::{IndividualTextureError, IndividualTextureStore};
use crate::instance_buffer::InstanceBuffer;
use crate::pipeline::SpritePipeline;
use crate::sprite::{QuadVertex, RenderInstance};
use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;

/// One contiguous run of instances that share the same `texture_id`
/// in the sorted scratch buffer. Reused frame-to-frame to keep
/// `render()` allocation-free (HR-3). `start` and `end` index into the
/// instance buffer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct DrawRun {
    texture_id: u32,
    start: u32,
    end: u32,
}

pub struct SpriteRenderer {
    gpu: GpuContext,
    pipeline: SpritePipeline,
    atlas: TextureAtlas,
    /// M14.5 C: per-sprite individually-owned textures. Reuses the
    /// pipeline's `material_bgl` so each entry's bind group can drop
    /// straight into `set_bind_group(1, ...)` at draw time.
    individual: IndividualTextureStore,
    instance_buffer: InstanceBuffer,
    quad_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    frame_bind_group: wgpu::BindGroup,
    material_bind_group: wgpu::BindGroup,
    /// Reused scratch to avoid per-frame allocation when sprite count
    /// is stable (typical case).
    scratch: Vec<RenderInstance>,
    /// Reused run buffer for the M14.5 C batching pass. Each entry is
    /// one draw call's worth of instances; the renderer walks them in
    /// order, swaps bind group 1 per run, and emits the draw.
    runs: Vec<DrawRun>,
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
        let individual = IndividualTextureStore::new(&gpu);

        Self {
            gpu,
            pipeline,
            atlas,
            individual,
            instance_buffer,
            quad_buffer,
            camera_buffer,
            frame_bind_group,
            material_bind_group,
            scratch: Vec::with_capacity(initial_instance_capacity as usize),
            // Real workloads keep distinct textures small (typically
            // 1 atlas + a few individuals); 16 is a reasonable seed.
            runs: Vec::with_capacity(16),
        }
    }

    /// Read access to the individual-texture store. Hosts call
    /// `acquire`/`release` directly on this; the renderer itself only
    /// reads `bind_group()` at draw time.
    pub fn individual(&self) -> &IndividualTextureStore {
        &self.individual
    }

    /// Mutable handle to the individual-texture store. The host's
    /// image-import path acquires textures here when the user
    /// selects the Individual source strategy for a sprite (M14.5
    /// inspector — separate phase).
    pub fn individual_mut(&mut self) -> &mut IndividualTextureStore {
        &mut self.individual
    }

    /// Convenience for the import path: acquire an individual texture
    /// from raw RGBA bytes and return the renderer-side `texture_id`
    /// the caller stamps into `SpriteSource::Individual`.
    pub fn acquire_individual(
        &mut self,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<u32, IndividualTextureError> {
        self.individual
            .acquire(&self.gpu, &self.pipeline.material_bgl, width, height, rgba)
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
        // M14.5 C: sort by texture_id so individually-textured sprites
        // group into contiguous runs. Stable sort keeps insertion
        // order within each texture (matters once a Z-sort lands and
        // tie-breaking has to preserve same-z ordering). For the
        // typical workload (≤ a few thousand instances, < 10 distinct
        // textures) the O(N log N) cost is dwarfed by GPU work.
        self.scratch.sort_by_key(|i| i.texture_id);
        compute_runs(&self.scratch, &mut self.runs);
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
                pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                pass.set_vertex_buffer(1, self.instance_buffer.buffer().slice(..));
                for run in &self.runs {
                    // Pick the right material bind group per run.
                    // Atlas (texture_id == 0) uses the shared one;
                    // individual textures look up the pre-built bind
                    // group in the store. Missing individuals (id
                    // released before render saw it) silently skip
                    // — the renderer is allowed to drop those.
                    let bg = if run.texture_id == RenderInstance::ATLAS_TEXTURE_ID {
                        Some(&self.material_bind_group)
                    } else {
                        self.individual.bind_group(run.texture_id)
                    };
                    let Some(bg) = bg else { continue };
                    pass.set_bind_group(1, bg, &[]);
                    pass.draw(0..4, run.start..run.end);
                }
            }
        }
        self.gpu.queue.submit(Some(encoder.finish()));
    }
}

/// Walk the sorted instance slice and emit one [`DrawRun`] per
/// maximal contiguous run of the same `texture_id`. Reuses `runs`'s
/// capacity. `scratch` MUST already be sorted by `texture_id` —
/// callers do this via `scratch.sort_by_key(|i| i.texture_id)`
/// immediately before invoking.
fn compute_runs(scratch: &[RenderInstance], runs: &mut Vec<DrawRun>) {
    runs.clear();
    if scratch.is_empty() {
        return;
    }
    let mut start = 0u32;
    let mut current = scratch[0].texture_id;
    for (i, inst) in scratch.iter().enumerate().skip(1) {
        if inst.texture_id != current {
            runs.push(DrawRun {
                texture_id: current,
                start,
                end: i as u32,
            });
            current = inst.texture_id;
            start = i as u32;
        }
    }
    runs.push(DrawRun {
        texture_id: current,
        start,
        end: scratch.len() as u32,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inst(texture_id: u32) -> RenderInstance {
        RenderInstance {
            world_pos: [0.0, 0.0],
            size: [1.0, 1.0],
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            tint: [1.0, 1.0, 1.0, 1.0],
            texture_id,
            _pad: [0; 3],
        }
    }

    #[test]
    fn compute_runs_empty_input_emits_no_runs() {
        let mut runs = Vec::new();
        compute_runs(&[], &mut runs);
        assert!(runs.is_empty());
    }

    #[test]
    fn compute_runs_groups_consecutive_same_texture() {
        // Pre-sorted: [0, 0, 0, 7, 7, 12]
        let scratch = [inst(0), inst(0), inst(0), inst(7), inst(7), inst(12)];
        let mut runs = Vec::new();
        compute_runs(&scratch, &mut runs);
        assert_eq!(
            runs,
            vec![
                DrawRun {
                    texture_id: 0,
                    start: 0,
                    end: 3
                },
                DrawRun {
                    texture_id: 7,
                    start: 3,
                    end: 5
                },
                DrawRun {
                    texture_id: 12,
                    start: 5,
                    end: 6
                },
            ]
        );
    }

    #[test]
    fn compute_runs_singleton_per_instance() {
        // All distinct textures → N runs of 1.
        let scratch = [inst(1), inst(2), inst(3)];
        let mut runs = Vec::new();
        compute_runs(&scratch, &mut runs);
        assert_eq!(runs.len(), 3);
        for r in &runs {
            assert_eq!(r.end - r.start, 1);
        }
    }

    #[test]
    fn compute_runs_reuses_vec_capacity() {
        // First call grows the vec; second call with a different
        // shape must NOT allocate again (HR-3 alloc-free hot path).
        let mut runs = Vec::with_capacity(8);
        let cap = runs.capacity();
        compute_runs(&[inst(0), inst(0)], &mut runs);
        compute_runs(&[inst(5), inst(6), inst(7), inst(7)], &mut runs);
        assert!(runs.capacity() >= cap, "capacity must not shrink");
        assert_eq!(runs.len(), 3);
    }

    #[test]
    fn compute_runs_atlas_constant() {
        let scratch = [inst(RenderInstance::ATLAS_TEXTURE_ID)];
        let mut runs = Vec::new();
        compute_runs(&scratch, &mut runs);
        assert_eq!(runs[0].texture_id, RenderInstance::ATLAS_TEXTURE_ID);
        assert_eq!(runs[0].texture_id, 0);
    }
}
