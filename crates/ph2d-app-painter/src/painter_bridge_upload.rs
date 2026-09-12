//! **A FAIXA DE UPLODE da pré-visualização de CPU** — o que a shell manda para a ranhura de
//! textura em cada quadro, e como.
//!
//! Irmão do [`crate::painter_bridge`] por **RESPONSABILIDADE**, e o corte é o que o próprio gate
//! do teto sugere: *«pipeline-setup + N effect kernels + buffer marshalling — each its own
//! module»*. Aqui está o *marshalling*: encher o buffer que a shell POSSUI sem segurar o `Arc` da
//! tela do tool, decidir o que a ranhura precisa de receber ([`UploadPlan`]), e devolvê-la quando
//! a pré-visualização morre.
//!
//! # ⚠️ Por que o corte aconteceu AGORA (W2 Fase D)
//!
//! O `painter_bridge.rs` tinha **1093** linhas e um marcador `// ph2d-loc-cap: mid-refactor` no
//! topo. Esse marcador é honrado pelo teto da **shell** (`file_loc_caps.rs`, 600) e ⛔ **é inerte
//! em `crates/`**: ali manda o `architecture_workspace_file_loc_cap` (700), que só aceita uma
//! entrada em `FILE_OVERAGE_OK` com assinatura do Coordenador.
//!
//! ⇒ *mover um ficheiro pode trocar o REGIME de teto que o governa, e uma isenção textual não
//! viaja com ele.* A cura é a que o `CLAUDE.md` §5.0 manda — **corte por responsabilidade**, nunca
//! uma entrada nova na lista de folgas.

use ph2d_editor_core::toast::{Toast, ToastQueue};
use ph2d_preview_slot::PreviewGpu as PainterPreviewGpu;
use ph2d_render::{SpriteRenderer, premultiply_rgba8};
use ph2d_tool_runtime::PreviewCache as PainterPreview;
use std::sync::Arc;

/// Fill the shell's OWN preview buffer for this frame without holding the tool's canvas `Arc`.
///
/// Reuse the shell's prior buffer and patch only the dirty region when the geometry matches, so the
/// tool is left the sole owner of its canvas (its next `stamp_dabs` writes in place instead of
/// copying the whole plane — the per-move cost that scaled with the canvas, not the brush). A seed —
/// no prior buffer, a dims/entity change, or a full recompose (`dirty_bbox == None`) — copies the
/// drained composite once. `prior` is the shell-owned buffer from last frame (sole owner ⇒ the
/// `make_mut` here never copies); `drained` is the tool's composite for THIS frame, borrowed only
/// long enough to copy the region out, then dropped by the caller.
///
/// `pub(super)` so the display-pipeline gate drives THIS function, not a mirror of it — the whole
/// point of the pipeline gates is that what they hold byte-exact is the code the app runs.
pub(crate) fn own_preview_buffer(
    prior: Option<PainterPreview>,
    entity_bits: u64,
    width: u32,
    height: u32,
    drained: &Arc<Vec<u8>>,
    dirty_bbox: Option<(u32, u32, u32, u32)>,
) -> Arc<Vec<u8>> {
    match (prior, dirty_bbox) {
        (Some(p), Some((bx, by, bw, bh)))
            if p.entity_bits == entity_bits
                && p.width == width
                && p.height == height
                && (*p.rgba).len() == (*drained).len()
                && bw > 0
                && bh > 0
                && bx + bw <= width
                && by + bh <= height =>
        {
            let mut mirror = p.rgba;
            let m = Arc::make_mut(&mut mirror); // shell is the sole owner ⇒ in place, no copy
            let row = (bw * 4) as usize;
            for ry in 0..bh {
                let off = (((by + ry) * width + bx) * 4) as usize;
                m[off..off + row].copy_from_slice(&drained[off..off + row]);
            }
            mirror
        }
        // Seed: no reusable prior buffer — take a full copy the shell then owns outright.
        _ => Arc::new((**drained).clone()),
    }
}

/// The CPU lane's slot upkeep for one frame — plan the upload ([`plan_upload`]), execute it
/// against the renderer, keep the bookkeeping, release the slot when the CPU cache is gone. The
/// single door both [`dispatch`] and the display gates drive, so what the tests hold byte-exact is
/// the code the app runs — not a mirror of it.
pub(crate) fn upload_cpu_preview(
    renderer: &mut SpriteRenderer,
    cpu_preview: Option<&PainterPreview>,
    painter_dirty_bbox: Option<(u32, u32, u32, u32)>,
    cache_version: u64,
    gpu_owns_preview: bool,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) {
    match cpu_preview {
        Some(preview) => {
            // Bisection toggle `PH2D_PAINT_FULL_UPLOAD=1`: force a FULL upload (disable the B.1 partial
            // lane) to bisect the "rectangular artifacts". See `HANDOFF_per_layer_color_perf_artifacts`.
            static FORCE_FULL_UPLOAD: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
            let force_full = *FORCE_FULL_UPLOAD
                .get_or_init(|| std::env::var_os("PH2D_PAINT_FULL_UPLOAD").is_some());
            let plan = plan_upload(
                preview,
                *painter_preview_gpu,
                painter_dirty_bbox,
                cache_version,
                force_full,
            );
            let upload_result: Option<Result<u32, _>> = match plan {
                UploadPlan::Skip => None,
                UploadPlan::Partial {
                    texture_id,
                    rect: (bx, by, bw, bh),
                } => {
                    // Gather + premultiply ONLY the bbox sub-rect (tightly
                    // packed bw*bh*4) and upload it over the existing texture.
                    let mut region = extract_region(&preview.rgba, preview.width, bx, by, bw, bh);
                    premultiply_rgba8(&mut region);
                    Some(
                        renderer
                            .replace_individual_pixels_region(texture_id, bx, by, bw, bh, &region)
                            .map(|()| texture_id),
                    )
                }
                UploadPlan::Full { reuse } => {
                    let mut premul_bytes = (*preview.rgba).clone();
                    premultiply_rgba8(&mut premul_bytes);
                    Some(match reuse {
                        Some(texture_id) => renderer
                            .replace_individual_pixels(
                                texture_id,
                                preview.width,
                                preview.height,
                                &premul_bytes,
                            )
                            .map(|()| texture_id),
                        None => renderer.acquire_individual(
                            preview.width,
                            preview.height,
                            &premul_bytes,
                        ),
                    })
                }
            };
            match upload_result {
                None => {}
                Some(Ok(texture_id)) => {
                    *painter_preview_gpu = Some(PainterPreviewGpu {
                        texture_id,
                        width: preview.width,
                        height: preview.height,
                        // The tool's content version, NOT `Arc::as_ptr(rgba)`: the shell no longer
                        // holds a clone of the tool's canvas, so its pointer would be meaningless here
                        // (the mirror is patched in place ⇒ its pointer never changes).
                        arc_token: cache_version as usize,
                        entity_bits: preview.entity_bits,
                    });
                }
                Some(Err(e)) => {
                    toasts.push(Toast::error(format!(
                        "Painter: upload da preview pra GPU falhou ({e}). \
                         Tentando novamente no próximo frame."
                    )));
                    release_preview_texture(renderer, painter_preview_gpu);
                }
            }
        }
        None => {
            // Release only when the CPU path owns the slot; on a GPU-owned frame
            // the GPU producer owns it — leave it intact for next frame.
            if !gpu_owns_preview {
                release_preview_texture(renderer, painter_preview_gpu);
            }
        }
    }
}

/// What the CPU lane must send the preview-slot texture this frame, decided from the drained
/// composite + the slot's bookkeeping — the DECISION half of the upload block, pure so a headless
/// test can drive it over a real stroke (the `hit_plan` pattern: the policy is a function, the wgpu
/// copies stay in [`dispatch`]). The screen samples the slot, so this plan — applied to the slot's
/// bytes — is exactly "what the artist sees"; the display gates in
/// `painter_preview_pipeline_tests.rs` hold it byte-equal to the tool's composite across a stroke's
/// whole life.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum UploadPlan {
    /// The slot already holds this composite (same content version, entity and dims) — no upload.
    Skip,
    /// Premultiply + upload the WHOLE canvas; `reuse` = overwrite that slot texture, `None` =
    /// acquire a fresh one (first frame, or dims/entity changed and the old slot was released).
    Full { reuse: Option<u32> },
    /// Premultiply + upload only `rect` (x, y, w, h) over the already-seeded slot texture — the
    /// B.1 partial lane. Only offered when the seeded texture matches the composite's entity+dims
    /// and the rect is in bounds; anything else falls back to `Full` (never panics the render loop).
    Partial {
        texture_id: u32,
        rect: (u32, u32, u32, u32),
    },
}

/// The B.1 upload decision (see [`UploadPlan`]). Change is detected by `cache_version` (the tool's
/// monotonic canvas version) rather than the buffer pointer — the shell owns its mirror and patches
/// it in place, so its pointer never moves even as pixels do; only the version says the content
/// changed. The fast lane fires only after a full upload seeded the texture (a full recompose hands
/// `bbox == None`), and any structural / metadata / dims / entity change forces a full upload — so
/// the un-touched slot pixels are always current. An idle frame reads the unchanged version → `Skip`.
pub(crate) fn plan_upload(
    preview: &PainterPreview,
    gpu: Option<PainterPreviewGpu>,
    dirty_bbox: Option<(u32, u32, u32, u32)>,
    cache_version: u64,
    force_full: bool,
) -> UploadPlan {
    let cache_token = cache_version as usize;
    let needs_upload = match gpu {
        None => true,
        Some(g) => {
            g.arc_token != cache_token
                || g.entity_bits != preview.entity_bits
                || g.width != preview.width
                || g.height != preview.height
        }
    };
    if !needs_upload {
        return UploadPlan::Skip;
    }
    let partial = (!force_full)
        .then_some(dirty_bbox)
        .flatten()
        .and_then(|(bx, by, bw, bh)| match gpu {
            // `g.arc_token != 0`: a partial patch is only sound over a slot the CPU lane itself
            // seeded. The GPU producer stamps its slots with token 0 (it has no CPU content version)
            // exactly so this transition forces a FULL re-upload — a rect patched over the GPU
            // compositor's output would leave every other pixel to a different producer (unlit,
            // and possibly older than the CPU cache), which is the GPU→CPU handoff artifact.
            Some(g)
                if g.arc_token != 0
                    && g.entity_bits == preview.entity_bits
                    && g.width == preview.width
                    && g.height == preview.height
                    && bw > 0
                    && bh > 0
                    && bx + bw <= preview.width
                    && by + bh <= preview.height =>
            {
                Some(UploadPlan::Partial {
                    texture_id: g.texture_id,
                    rect: (bx, by, bw, bh),
                })
            }
            _ => None,
        });
    partial.unwrap_or(UploadPlan::Full {
        reuse: gpu.map(|g| g.texture_id),
    })
}

/// Gather a tightly-packed `w*h*4` RGBA8 sub-rect at `(x, y)` out of a
/// canvas-sized straight buffer (row stride `stride_px*4`) — the inverse of the
/// compositor's `blit_region`, for the B.1 partial GPU upload. The caller's
/// guard (`x+w <= stride_px`, `y+h <= height`) keeps every row copy in bounds.
/// `pub(super)` so the display-pipeline gates apply the real gather, not a mirror of it.
pub(crate) fn extract_region(
    full: &[u8],
    stride_px: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Vec<u8> {
    let row_bytes = (w * 4) as usize;
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    for ry in 0..h {
        let src_off = (((y + ry) * stride_px + x) * 4) as usize;
        let dst_off = (ry * w * 4) as usize;
        out[dst_off..dst_off + row_bytes].copy_from_slice(&full[src_off..src_off + row_bytes]);
    }
    out
}

/// Release the Painter live-preview's `IndividualTextureStore` slot (if any)
/// and zero the GPU cache. Called when the preview cache turns `None` (tool
/// deactivated, Apply committed, no source) and on upload error — next frame
/// re-acquires from scratch.
pub(crate) fn release_preview_texture(
    renderer: &mut SpriteRenderer,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
) {
    if let Some(gpu) = painter_preview_gpu.take() {
        renderer.individual_mut().release(gpu.texture_id);
    }
}
