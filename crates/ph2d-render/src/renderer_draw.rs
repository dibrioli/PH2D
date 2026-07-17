//! The **draw pass**: `SpriteRenderer`'s three `render*` entry points, split out
//! of `renderer.rs` at its LOC cap.
//!
//! Everything else in that file BUILDS the renderer (pipelines, atlases,
//! texture stores, the clip stencil); these three spend it — they take a frame's
//! `PresentWorld`, sort it into runs, and encode the passes. The split is along
//! that seam, not at a convenient line count.
//!
//! `render_with_streams` is the GPU-cook path (ADR-0126): its `gpu_extra` buffer
//! is the cook's lowering output, bound straight as an instance vertex buffer —
//! the cook's last write is this pass's input, with no readback in between.

use super::*;

impl SpriteRenderer {
    pub fn render(
        &mut self,
        target: &wgpu::TextureView,
        present: &mut PresentWorld,
        camera: &Camera2d,
        window: WindowSize,
        clear_color: wgpu::Color,
    ) {
        self.render_with_extra(target, present, camera, window, clear_color, &[], None);
    }

    /// [`render`](Self::render) plus two Motion Nodes hooks. `extra` (M0.T11) is
    /// an external instance slice appended to the scene, sorted + batched in the
    /// same pass — a cooked node-graph stream draws without being spawned into
    /// `PresentWorld` (stream ≠ ECS, ADR-0035); `&[]` = scene-only. `scene_viewport`
    /// (M0.T13) optionally frames the scene into a target sub-rect `[x, y, w, h]`
    /// px via `set_viewport`/`set_scissor_rect` + [`Camera2d::uniform_for_subrect`]
    /// (the split viewport-vs-graph); it applies only on the plain single-pass
    /// path — a clip/mask frame ignores it and renders full-window (still covered
    /// by the graph panel on top).
    #[allow(clippy::too_many_arguments)]
    pub fn render_with_extra(
        &mut self,
        target: &wgpu::TextureView,
        present: &mut PresentWorld,
        camera: &Camera2d,
        window: WindowSize,
        clear_color: wgpu::Color,
        extra: &[RenderInstance],
        scene_viewport: Option<[f32; 4]>,
    ) {
        self.render_with_streams(
            target,
            present,
            camera,
            window,
            clear_color,
            extra,
            None,
            scene_viewport,
        );
    }

    /// [`render_with_extra`](Self::render_with_extra) plus a **GPU-resident**
    /// extra (GPU/M5 Fase 1, ADR-0126): `gpu_extra` is a buffer ALREADY laid
    /// out as `[RenderInstance; n]` — the `ph2d-gpu-cook` lowering's output —
    /// bound directly as the instance vertex buffer for one appended draw.
    /// **No readback, no CPU marshalling**: the cook's last write is this
    /// pass's input. Drawn after the scene's normal runs with the shared-atlas
    /// material, default sampler and default blend (exactly the run a lowered
    /// Motion stream produces on the CPU path — its instances are all
    /// `texture_id 0 / z_order 0 / sampling 0 / no clip`), on the plain
    /// single-pass path only (a clip/mask frame ignores it, like `subrect`).
    #[allow(clippy::too_many_arguments)]
    pub fn render_with_streams(
        &mut self,
        target: &wgpu::TextureView,
        present: &mut PresentWorld,
        camera: &Camera2d,
        window: WindowSize,
        clear_color: wgpu::Color,
        extra: &[RenderInstance],
        gpu_extra: Option<(&wgpu::Buffer, u32)>,
        scene_viewport: Option<[f32; 4]>,
    ) {
        // Collect scene instances + the `extra` slice into `scratch` and sort
        // (extracted to keep this file under its LOC cap; M0.T11).
        crate::sprite_collect::collect_sorted_instances(&mut self.scratch, present, extra);
        compute_runs(&self.scratch, &mut self.runs);
        // Ensure an atlas bind group exists for every distinct sampling
        // used by an atlas run (built lazily; one per filter/repeat pair).
        for run in 0..self.runs.len() {
            let r = self.runs[run];
            if r.texture_id == RenderInstance::ATLAS_TEXTURE_ID {
                self.ensure_atlas_sampler_bg(r.sampling);
            }
        }
        let count = self
            .instance_buffer
            .upload(&self.gpu, self.scratch.as_slice());

        // W3 §8: does this frame contain any ClipChildren group or Mask2D /
        // MaskInteraction role? The common case (neither) takes the exact
        // pre-stencil single-pass path below — zero regression, and the
        // stencil attachment is never even allocated.
        let has_clip = count > 0 && self.runs.iter().any(|r| r.clip_group != 0);
        let has_mask = count > 0 && self.runs.iter().any(|r| r.mask_role != 0);

        // Motion Nodes M0.T13 — the split sub-rect is honored only on the plain
        // single-pass path; a clip/mask frame renders full-window (mixing the
        // subrect projection with the full-target clip pass would mis-project).
        let subrect = scene_viewport.filter(|_| !has_clip && !has_mask);
        let camera_uniform = match subrect {
            Some([_, _, w, h]) => camera.uniform_for_subrect(w, h),
            None => camera.uniform(window),
        };
        self.gpu
            .queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&camera_uniform));

        if has_clip || has_mask {
            // Stencil must match the color target; the live editor's GameRt
            // tracks the window size, so size the stencil to the window.
            self.ensure_clip_stencil((window.width, window.height));
        }

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render sprite encoder"),
            });
        // Resolve a run's material bind group. Atlas (texture_id == 0) uses
        // the per-sampling cached bind group (W3.T3.11; falls back to the
        // project-default material bg if not yet built); a cooked id (W2.T4,
        // `COOKED_TEXTURE_ID_BIT` set) binds the cooked-texture store; every
        // other id is an individual texture. A missing entry in either store
        // (id released / not-yet-uploaded before render saw it) yields
        // `None` → the run is skipped (sprite renders nothing this frame).
        let material_bg = |run: &DrawRun| -> Option<&wgpu::BindGroup> {
            if run.texture_id == RenderInstance::ATLAS_TEXTURE_ID {
                self.atlas_sampler_bgs
                    .get(&run.sampling)
                    .or(Some(&self.material_bind_group))
            } else if RenderInstance::is_cooked_texture_id(run.texture_id) {
                self.cooked.bind_group(run.texture_id)
            } else {
                self.individual.bind_group(run.texture_id)
            }
        };
        {
            // Normal pass: every plain run (`clip_group == 0 && mask_role
            // == 0`). When the frame has no clip/mask this is ALL runs —
            // byte-identical to the legacy single pass.
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
                timestamp_writes: ph2d_gpu::pass_profiler::render_writes("render.sprite"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            // M0.T13 — confine the scene to the split sub-rect (uniform above uses
            // its aspect; the clear still fills the whole attachment).
            if let Some([x, y, w, h]) = subrect {
                pass.set_viewport(x, y, w, h, 0.0, 1.0);
                pass.set_scissor_rect(x as u32, y as u32, (w.max(1.0)) as u32, (h.max(1.0)) as u32);
            }
            if count > 0 {
                pass.set_bind_group(0, &self.frame_bind_group, &[]);
                pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                pass.set_vertex_buffer(1, self.instance_buffer.buffer().slice(..));
                // §10: bind the per-run blend pipeline. Runs are keyed by
                // blend tag in `compute_runs`, so a run is uniform; rebind
                // only when the tag changes (tracked to avoid redundant
                // set_pipeline calls).
                let mut bound_blend: Option<u8> = None;
                for run in self
                    .runs
                    .iter()
                    .filter(|r| r.clip_group == 0 && r.mask_role == 0)
                {
                    let Some(bg) = material_bg(run) else { continue };
                    if bound_blend != Some(run.blend) {
                        pass.set_pipeline(self.pipeline.blend_pipeline(run.blend));
                        bound_blend = Some(run.blend);
                    }
                    pass.set_bind_group(1, bg, &[]);
                    pass.draw(0..4, run.start..run.end);
                }
            }
            // GPU-resident extra (ADR-0126): one draw whose instance vertex
            // buffer IS the compute lowering's output — the readback-free
            // seam. Shared-atlas material + default blend, the exact run a
            // CPU-lowered Motion stream forms. Plain path only (mirrors the
            // `subrect` rule: a clip/mask frame renders scene-only).
            if let Some((buffer, n)) = gpu_extra
                && n > 0
                && !has_clip
                && !has_mask
            {
                pass.set_bind_group(0, &self.frame_bind_group, &[]);
                pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                pass.set_vertex_buffer(1, buffer.slice(..));
                pass.set_pipeline(self.pipeline.blend_pipeline(0));
                pass.set_bind_group(1, &self.material_bind_group, &[]);
                pass.draw(0..4, 0..n);
            }
        }
        // Clip pass (only if a clip group exists): stencil mark → test the
        // descendants → optional ClipAndDraw mask color. Composites on top
        // of the normal pass (color Load).
        if has_clip {
            let stencil = &self.clip_stencil.as_ref().expect("ensured above").view;
            crate::clip_pass::encode_clip_groups(
                &mut encoder,
                target,
                stencil,
                &self.pipeline,
                &self.frame_bind_group,
                &self.quad_buffer,
                self.instance_buffer.buffer(),
                &self.runs,
                material_bg,
            );
        }
        // Mask pass (only if a Mask2D source / responder exists): mark every
        // Mask2D silhouette into a fresh stencil, then draw VisibleInside
        // responders where stencil == ref and VisibleOutside where != ref.
        // Global scope (one shared ref), composited on top (color Load).
        if has_mask {
            let stencil = &self.clip_stencil.as_ref().expect("ensured above").view;
            crate::clip_pass::encode_mask_pass(
                &mut encoder,
                target,
                stencil,
                &self.pipeline,
                &self.frame_bind_group,
                &self.quad_buffer,
                self.instance_buffer.buffer(),
                &self.runs,
                material_bg,
            );
        }
        self.gpu.queue.submit(Some(encoder.finish()));
    }
}
