//! ClipChildren stencil clip pass (W3 §8; ADR-0070-amendment-7 /
//! ADR-0074-amendment-1).
//!
//! Runs AFTER the normal pass (which drew every `clip_group == 0`
//! sprite). For each clip group it does the canonical stencil triple
//! (spec §6.2):
//!   1. **mark** — the clip-parent silhouette into the stencil buffer
//!      (`Replace ref`, color off), thresholded by the per-instance
//!      alpha cutoff in the fragment.
//!   2. **draw mask color** (ClipAndDraw only) — the parent's own pixels,
//!      via the test pipeline (`Equal ref` coincides with the silhouette).
//!   3. **test** — the clipped descendants, drawn only where
//!      `stencil == ref` so they stay inside the silhouette.
//!
//! All groups share ONE render pass; each gets a distinct stencil `ref`
//! (incremental, no inter-group clears) so an earlier group's marks don't
//! leak into a later group's test (the marks accumulate but the refs
//! differ). Extracted from `renderer.rs` to keep `render()` readable
//! (handoff §1.6).

use crate::pipeline::SpritePipeline;
use crate::renderer::DrawRun;
use crate::sprite::RenderInstance;

/// Encode the clip pass for every non-zero `clip_group` in `runs`.
///
/// `runs` MUST be the same z-sorted run list the normal pass walked, so a
/// clip group's runs are contiguous (subtree contiguity in z; handoff
/// §1.3). `resolve_material` maps a run to its material bind group (atlas
/// per-sampling or individual texture), returning `None` for a texture id
/// that vanished before render — that run is skipped, matching the normal
/// pass.
#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_clip_groups<'a>(
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    stencil: &wgpu::TextureView,
    pipe: &SpritePipeline,
    frame_bg: &wgpu::BindGroup,
    quad_buf: &wgpu::Buffer,
    instance_buf: &wgpu::Buffer,
    runs: &[DrawRun],
    resolve_material: impl Fn(&DrawRun) -> Option<&'a wgpu::BindGroup>,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("ph2d-render clip pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            // Preserve the normal pass's color; the clip groups composite
            // on top (premultiplied blend in the test pipeline).
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: stencil,
            // Stencil8 has no depth aspect.
            depth_ops: None,
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                // The stencil is scratch for this pass only.
                store: wgpu::StoreOp::Discard,
            }),
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_bind_group(0, frame_bg, &[]);
    pass.set_vertex_buffer(0, quad_buf.slice(..));
    pass.set_vertex_buffer(1, instance_buf.slice(..));

    let mut i = 0usize;
    let mut span_index = 0u32;
    while i < runs.len() {
        // Skip the normal-pass runs (already drawn) and find the next span.
        if runs[i].clip_group == 0 {
            i += 1;
            continue;
        }
        let group = runs[i].clip_group;
        let start = i;
        while i < runs.len() && runs[i].clip_group == group {
            i += 1;
        }
        let span = &runs[start..i];

        // Distinct stencil ref per span — no inter-group clears. Wraps at
        // 255 (ref 0 is "unmarked"); >255 clip groups in one frame would
        // alias, which is a clear-on-overflow future-wave hardening
        // (W3 single-level scenes have a handful of groups).
        let stencil_ref = (span_index % 255) + 1;
        span_index += 1;

        let is_mask = |r: &&DrawRun| r.clip_role != RenderInstance::CLIP_ROLE_MEMBER;
        let is_member = |r: &&DrawRun| r.clip_role == RenderInstance::CLIP_ROLE_MEMBER;
        let clip_and_draw = span
            .iter()
            .any(|r| r.clip_role == RenderInstance::CLIP_ROLE_MASK_CLIP_AND_DRAW);

        // 1. MARK — write the silhouette into the stencil (no color).
        pass.set_pipeline(&pipe.mark_pipeline);
        pass.set_stencil_reference(stencil_ref);
        for run in span.iter().filter(is_mask) {
            if let Some(bg) = resolve_material(run) {
                pass.set_bind_group(1, bg, &[]);
                pass.draw(0..4, run.start..run.end);
            }
        }

        pass.set_pipeline(&pipe.test_pipeline);
        pass.set_stencil_reference(stencil_ref);
        // 2. ClipAndDraw: paint the parent's own color (Equal ref lands
        //    exactly on its silhouette pixels). ClipOnly skips this — the
        //    parent is an invisible mould.
        if clip_and_draw {
            for run in span.iter().filter(is_mask) {
                if let Some(bg) = resolve_material(run) {
                    pass.set_bind_group(1, bg, &[]);
                    pass.draw(0..4, run.start..run.end);
                }
            }
        }
        // 3. Members — clipped to the silhouette.
        for run in span.iter().filter(is_member) {
            if let Some(bg) = resolve_material(run) {
                pass.set_bind_group(1, bg, &[]);
                pass.draw(0..4, run.start..run.end);
            }
        }
    }
}
