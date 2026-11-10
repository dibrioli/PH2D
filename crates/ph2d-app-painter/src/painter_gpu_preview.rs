//! Painter GPU live-preview producer (ADR-0045 Phase 3, step 2).
//!
//! The CPU producer in [`super::painter_bridge`] composites the layer stack on
//! the CPU (`take_preview_arc`) and uploads premultiplied bytes into the preview
//! slot. This is the GPU sibling: when the stack is GPU-representable
//! ([`ph2d_tool_painter::flatten_for_gpu`] returns `Some`), it
//! composites on the GPU [`LayerCompositor`], premultiplies the straight output
//! via [`PreviewPremul`], and copies the result straight into the SAME
//! `IndividualTextureStore` preview slot — **no CPU readback**. Both producers
//! end in `painter_preview_gpu`; the next frame's `sim_extract` emits the
//! `PreviewOverride` either way.
//!
//! ## Why it wins
//!
//! A slider drag changes only `gpu_params`, so the compositor keeps every layer slice cached and
//! re-runs just the compute; the straight→premul handoff is [`ph2d_render::premul`].
//! (história: docs/Painter/historia_comentarios_app_painter_2026-11-10.md)

use ph2d_editor_core::toast::{Toast, ToastQueue};
use ph2d_gpu::GpuContext;
use ph2d_preview_slot::PreviewGpu as PainterPreviewGpu;
use ph2d_render::PreviewPremul;
use ph2d_render::SpriteRenderer;
use ph2d_render::layer_compositor::{
    LayerCompositor, LayerOp, LayerPixelProvider, LayerPixels, Region,
};
use ph2d_render::{ImpastoLamp, ImpastoLightInput, ImpastoLightPass};
use ph2d_tool_painter::PainterTool;
use std::cell::RefCell;

/// Per-session GPU state for the Painter live preview: compositor, straight→premul blit and a
pub struct PainterGpuPreview {
    gpu: GpuContext,
    compositor: LayerCompositor,
    /// Impasto's light — the relief made visible. Runs BETWEEN the composite and the premultiply, which
    light: ImpastoLightPass,
    premul: PreviewPremul,
    /// A geração de memória da placa em que a sessão nasceu (F36): no fim de um episódio sem memória
    geracao: u64,
    /// Os planos do último envio, cujos buffers a próxima dobra reescreve (B15-cont: sem os ~107 MB alocados e
    planos: Option<ph2d_tool_painter::ImpastoPlanes>,
    /// A placa produziu o último quadro do documento: ao largá-lo, a CPU compõe a tela já nesse quadro.
    dona: bool,
    /// B20.A2: `(camada, versão)` entregues ao compositor na última drenagem da placa. O sujo do instrumento
    entregue: Vec<(u64, u64)>,
    /// O vector da drenagem em curso (reusado: zero alocação por quadro).
    a_entregar: Vec<(u64, u64)>,
    /// B20.A1: a tela `(w, h)` que a placa recusou nesta geração — a CPU produz, sem nova tentativa nem aviso.
    recusou: Option<(u32, u32)>,
}

/// Builds the session BEFORE the first stroke needs it (doc 28 §4.8): its passes compile
pub(crate) fn prewarm(
    session_slot: &mut Option<PainterGpuPreview>,
    renderer: &mut SpriteRenderer,
    painter: &PainterTool,
    selection: u64,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) {
    if session_slot.is_none() {
        *session_slot = Some(PainterGpuPreview::new(renderer.gpu()));
        // A água do Wet Paint corre no MESMO device (LIGACAO_PLACA.md).
        ph2d_tool_painter::PainterTool::instala_placa_do_wet(renderer.gpu());
    }
    // Also allocates the canvas-sized compositor/premul resources (first execution only).
    let Some((ops, adj_luts)) = ph2d_tool_painter::flatten_for_gpu(painter.layers()) else {
        return;
    };
    let (w, h) = painter.source_size();
    if w == 0 || h == 0 {
        return;
    }
    let pronta = drive(
        session_slot,
        renderer,
        painter,
        selection,
        ops,
        adj_luts,
        w,
        h,
        (0, 0, w, h),
        true,
        painter_preview_gpu,
        toasts,
    );
    // O prewarm não drena o sujo (a CPU, se a placa recusou, compõe-no): não é uma drenagem anterior.
    if let Some(s) = session_slot.as_mut() {
        s.entregue.clear();
        s.recusou = (!pronta).then_some((w, h));
    }
}

/// B20.A1: a placa recusou a tela. A CPU compõe-na inteira JÁ neste quadro e fica dona até a tela ou a
fn recusa(session_slot: &mut Option<PainterGpuPreview>, painter: &mut PainterTool, w: u32, h: u32) {
    if let Some(s) = session_slot.as_mut() {
        s.recusou = Some((w, h));
        s.dona = false;
        s.entregue.clear();
    }
    painter.pede_a_composicao_inteira();
}

impl PainterGpuPreview {
    fn new(gpu: &GpuContext) -> Self {
        Self {
            gpu: gpu.clone(),
            compositor: LayerCompositor::new(gpu),
            light: ImpastoLightPass::new(gpu),
            premul: PreviewPremul::new(gpu),
            geracao: ph2d_gpu::memoria::geracao(),
            planos: None,
            dona: false,
            entregue: Vec::new(),
            a_entregar: Vec::new(),
            recusou: None,
        }
    }
}

/// Adapts [`PainterTool::preview_layer_pixels`] to the render crate's
struct PainterLayerProvider<'a> {
    tool: &'a PainterTool,
    /// O que a drenagem anterior entregou (B20.A2).
    anteriores: &'a [(u64, u64)],
    /// O que esta drenagem entrega.
    novas: RefCell<Vec<(u64, u64)>>,
}

impl LayerPixelProvider for PainterLayerProvider<'_> {
    fn layer_pixels(&self, key: u64) -> Option<LayerPixels<'_>> {
        let (version, rgba8, dirty) = self.tool.preview_layer_pixels(key)?;
        self.novas.borrow_mut().push((key, version));
        // A versão é um relógio do documento (sobe a cada toque, de qualquer camada): o sujo só remenda a
        let contigua = self.anteriores.iter().any(|&(k, _)| k == key);
        Some(LayerPixels {
            version,
            rgba8,
            dirty: dirty
                .filter(|_| contigua)
                .map(|(x, y, w, h)| Region { x, y, w, h }),
        })
    }
}

/// Decide GPU-vs-CPU for this frame and, when GPU, recomposite into the preview slot. Returns
pub(super) fn try_drive(
    session_slot: &mut Option<PainterGpuPreview>,
    renderer: &mut SpriteRenderer,
    painter: &mut PainterTool,
    selection: Option<u64>,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) -> bool {
    let Some(sel) = selection else {
        return false;
    };
    let Some((ops, adj_luts)) = gpu_eligible(painter) else {
        larga_a_placa(session_slot.as_mut().map(|s| &mut s.dona), painter);
        return false;
    };
    let (w, h) = painter.source_size();
    let geracao = ph2d_gpu::memoria::geracao();
    if session_slot
        .as_ref()
        .is_some_and(|s| s.geracao == geracao && s.recusou == Some((w, h)))
    {
        return false;
    }
    let renasce = session_slot.as_ref().is_some_and(|s| s.geracao != geracao);
    if painter.take_preview_dirty() || renasce {
        // CPU/hover path: a full recomposite, seeding the whole canvas.
        if !drive(
            session_slot,
            renderer,
            painter,
            sel,
            ops,
            adj_luts,
            w,
            h,
            (0, 0, w, h),
            true,
            painter_preview_gpu,
            toasts,
        ) {
            recusa(session_slot, painter, w, h);
            return false;
        }
    }
    let _ = painter.take_preview_upload_bbox();
    if let Some(s) = session_slot.as_mut() {
        s.dona = true;
    }
    true
}

/// The GPU let go of the document: the CPU lane composes the whole canvas THIS frame (B16.12).
fn larga_a_placa(dona: Option<&mut bool>, painter: &mut PainterTool) {
    if let Some(d) = dona.filter(|d| **d) {
        *d = false;
        painter.pede_a_composicao_inteira();
    }
}

/// The GPU-vs-CPU gate, pure half: `Some(ops, adj_luts)` iff this frame composites on the GPU.
fn gpu_eligible(painter: &PainterTool) -> Option<(Vec<LayerOp>, Vec<f32>)> {
    // Trivial bow-out unless relief: relief forces a CPU full composite + light, where the GPU is
    if painter.preview_layer_stack_is_trivial() && !painter.impasto_visible() {
        return None;
    }
    // Repeat Image draws its 3×3 tiles from the CPU composite, which a GPU-owned slot clears:
    if painter.repeat_image() {
        return None;
    }
    // The mask PROTECTION tint (`apply_mask_overlay`) lives in the CPU composite and the GPU
    if painter.mask_scratch_active() {
        return None;
    }
    // Wash GLASS under coloured paper (doc 48): paper crosses each channel by its alpha and the GPU
    if painter.papel_pede_a_cpu() {
        return None;
    }
    ph2d_tool_painter::flatten_for_gpu(painter.layers())
}

/// Composite `ops` on the GPU, premultiply, copy into the preview slot ((re)acquired when
#[allow(clippy::too_many_arguments)]
pub(super) fn drive(
    session_slot: &mut Option<PainterGpuPreview>,
    renderer: &mut SpriteRenderer,
    tool: &PainterTool,
    entity_bits: u64,
    ops: Vec<LayerOp>,
    adj_luts: Vec<f32>,
    width: u32,
    height: u32,
    rect: (u32, u32, u32, u32),
    seed_full: bool,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    toasts: &mut ToastQueue,
) -> bool {
    if width == 0 || height == 0 {
        return false;
    }
    let mut seed_full = seed_full;
    if session_slot
        .as_ref()
        .is_some_and(|s| s.geracao != ph2d_gpu::memoria::geracao())
    {
        // Fim de um episódio sem memória: o que a sessão e o slot criaram sob recusa é inválido.
        *session_slot = None;
        release_slot(renderer, painter_preview_gpu);
        seed_full = true;
    }
    ph2d_tool_painter::PainterTool::instala_placa_do_wet(renderer.gpu());
    let session = session_slot.get_or_insert_with(|| PainterGpuPreview::new(renderer.gpu()));

    // The composite region: the WHOLE canvas on a seed frame (so the persistent
    let region = if seed_full {
        Region::full(width, height)
    } else {
        Region {
            x: rect.0,
            y: rect.1,
            w: rect.2,
            h: rect.3,
        }
    };

    // 1-3) Composite, light, premultiply — everything that turns the layer stack into the bytes the
    if let Err(e) = compose_light_premul(session, tool, &ops, &adj_luts, width, height, region) {
        toasts.push(Toast::error(ph2d_i18n::tr_with(
            "app.painter.painter_gpu_preview.gpu_preview_failed",
            &[("e", &e)],
        )));
        release_slot(renderer, painter_preview_gpu);
        return false;
    }

    // 4) Ensure a right-sized slot, then COPY the premultiplied result in: the whole canvas on a
    let slot = ensure_slot(renderer, painter_preview_gpu, entity_bits, width, height);
    let Some(premul_tex) = session.premul.output_texture() else {
        return false;
    };
    let copy = if seed_full {
        renderer.copy_texture_into_individual(slot, premul_tex, width, height)
    } else {
        renderer.copy_texture_region_into_individual(
            slot, premul_tex, rect.0, rect.1, rect.0, rect.1, rect.2, rect.3,
        )
    };
    if let Err(e) = copy {
        toasts.push(Toast::error(ph2d_i18n::tr_with(
            "app.painter.painter_gpu_preview.gpu_preview_copy_failed",
            &[("e", &e)],
        )));
        release_slot(renderer, painter_preview_gpu);
        return false;
    }
    true
}

/// Composite, light, premultiply — everything that turns the layer stack into the bytes the preview
fn compose_light_premul(
    session: &mut PainterGpuPreview,
    tool: &PainterTool,
    ops: &[LayerOp],
    adj_luts: &[f32],
    width: u32,
    height: u32,
    region: Region,
) -> Result<(), String> {
    // 1) GPU composite into the PERSISTENT canvas-sized `out`, writing at canvas
    let mut novas = std::mem::take(&mut session.a_entregar);
    novas.clear();
    let provider = PainterLayerProvider {
        tool,
        anteriores: &session.entregue,
        novas: RefCell::new(novas),
    };
    // O PAPEL do documento: o compositor compõe cada saída SOBRE ele com a lei inteira da CPU
    session.compositor.set_paper(tool.papel());
    let composto = session.compositor.composite_region_into_canvas(
        &session.gpu,
        ops,
        adj_luts,
        &provider,
        width,
        height,
        region,
    );
    let mut feitas = provider.novas.into_inner();
    if composto.is_err() {
        feitas.clear();
    }
    session.a_entregar = std::mem::replace(&mut session.entregue, feitas);
    composto.map_err(|e| {
        ph2d_i18n::tr_with("app.painter.painter_gpu_preview.composite", &[("e", &e)])
    })?;

    // 2) Impasto light on the freshly composited region (lighting is NOT idempotent). `None` planes
    let comp_out = session.compositor.output_texture().ok_or(ph2d_i18n::tr(
        "app.painter.painter_gpu_preview.composite_produced_no_texture",
    ))?;
    // Fold window: the tool's `preview_gpu_region` (Some only for a rect-confined change) AND the
    let plane_win = tool
        .preview_gpu_region()
        .filter(|_| session.planos.is_some() && session.light.planes_seeded(width, height))
        .unwrap_or((0, 0, width, height));
    // The fold clock travels with its window (rect vs canvas).
    let m_fold = crate::paint_perf::on().then(std::time::Instant::now);
    let planes = tool.impasto_gpu_planes_reusing(plane_win, session.planos.take());
    if let Some(t0) = m_fold {
        crate::paint_perf::note_gpu_fold(
            t0.elapsed().as_secs_f64() as f32 * 1e3,
            plane_win == (0, 0, width, height),
        );
    }
    let finished: &wgpu::Texture = match planes {
        None => comp_out,
        Some(planes) => {
            let lamps: Vec<ImpastoLamp> = planes
                .lamps
                .iter()
                .map(|l| ImpastoLamp {
                    dir: l.dir,
                    half: l.half,
                    tint: l.tint,
                })
                .collect();
            let input = ImpastoLightInput {
                width,
                height,
                // The WHOLE canvas: the light owns a second persistent canvas, so its freshness
                region: Region::full(width, height),
                // …but the PLANES are only the fold's window (the textures persist).
                plane_region: Region {
                    x: planes.region.0,
                    y: planes.region.1,
                    w: planes.region.2,
                    h: planes.region.3,
                },
                relief: &planes.relief,
                cover: &planes.cover,
                mat0: &planes.mat0,
                mat1: &planes.mat1,
                lamps: &lamps,
                spec_lut: planes.spec_lut,
                lut_width: planes.lut_width,
                rough_levels: planes.rough_levels,
                paper_body: planes.paper_body,
            };
            // Falling through UNLIT would silently drop the shading.
            let lit = session.light.run(&session.gpu, comp_out, &input);
            session.planos = Some(planes);
            lit.map_err(|e| {
                ph2d_i18n::tr_with(
                    "app.painter.painter_gpu_preview.impasto_light",
                    &[("e_", &format!("{:?}", e))],
                )
            })?
        }
    };

    // 3) Premultiply into a COPY_SRC texture (the slot samples PREMULTIPLIED);
    session
        .premul
        .run(&session.gpu, finished, width, height)
        .ok_or(ph2d_i18n::tr(
            "app.painter.painter_gpu_preview.premultiply_produced_no_texture",
        ))?;
    Ok(())
}

/// Ensure a `width × height` slot (reuse on matching dims, else release + acquire EMPTY);
fn ensure_slot(
    renderer: &mut SpriteRenderer,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    entity_bits: u64,
    width: u32,
    height: u32,
) -> u32 {
    if let Some(existing) = *painter_preview_gpu
        && existing.width == width
        && existing.height == height
    {
        *painter_preview_gpu = Some(PainterPreviewGpu {
            texture_id: existing.texture_id,
            width,
            height,
            arc_token: 0,
            entity_bits,
        });
        return existing.texture_id;
    }
    if let Some(old) = painter_preview_gpu.take() {
        renderer.individual_mut().release(old.texture_id);
    }
    let id = renderer.acquire_individual_empty(width, height);
    *painter_preview_gpu = Some(PainterPreviewGpu {
        texture_id: id,
        width,
        height,
        arc_token: 0,
        entity_bits,
    });
    id
}

/// Release the preview slot (if any) — local mirror of
fn release_slot(
    renderer: &mut SpriteRenderer,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
) {
    if let Some(gpu) = painter_preview_gpu.take() {
        renderer.individual_mut().release(gpu.texture_id);
    }
}

#[cfg(test)]
#[path = "painter_gpu_preview_tests.rs"]
mod tests;
