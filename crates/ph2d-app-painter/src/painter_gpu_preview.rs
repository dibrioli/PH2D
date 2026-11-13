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
    /// As texturas de plano da luz têm o relevo de AGORA fora de qualquer janela: `true` depois de cada envio,
    /// `false` quando a dobra não deu planos (luz apagada: os traços seguintes não sobem, e a volta da luz dobra a
    /// tela inteira).
    planos_em_dia: bool,
    /// A placa produziu o último quadro do documento: ao largá-lo, a CPU compõe a tela já nesse quadro.
    dona: bool,
    /// B20.A2: `(camada, versão)` entregues ao compositor na última drenagem da placa. O sujo do instrumento
    entregue: Vec<(u64, u64)>,
    /// O vector da drenagem em curso (reusado: zero alocação por quadro).
    a_entregar: Vec<(u64, u64)>,
    /// B20.A1: a tela `(w, h)` que a placa recusou nesta geração — a CPU produz, sem nova tentativa nem aviso.
    recusou: Option<(u32, u32)>,
    /// B20.1: as entradas globais (ops, LUTs, papel) do último quadro; mudou uma ⇒ semente inteira.
    entradas: (Vec<LayerOp>, Vec<f32>, Option<[u8; 3]>),
    /// B20.1: lâmpadas e papel da última luz (`None` = sem luz); mudou ⇒ a luz reacende a tela inteira.
    luz: Option<(Vec<ImpastoLamp>, f32)>,
}

#[cfg(test)]
thread_local! {
    /// O caminho antigo (semente inteira em todo quadro), o oráculo de `a_regiao_suja_e_a_tela_inteira_ao_bit`.
    pub(crate) static SEMPRE_INTEIRA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// B18.4: o caminho de antes da cura (a camada nova sobe inteira no 1.º toque), o controlo do gate.
    pub(crate) static NOVA_SEM_LIMPAR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// N7 onda 5: a lista nova de um gesto de estrutura confinado não confina a placa (a mutação/o controlo).
    pub(crate) static ESTRUTURA_RECOMPOE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Quantas drenagens confinaram com a lista nova de um gesto de estrutura (o controlo do gate).
    pub(crate) static CONFINADAS_POR_ESTRUTURA: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// B20.1: o retângulo sujo + 1 px, ou `None` = semente inteira (1.º quadro, renasce, a CPU foi dona,
/// slot de outro tamanho, ou mudou uma entrada global). N7 onda 5: a lista que mudou por um gesto de estrutura
/// confinado (`estrutura`, a `(antes, depois)` da ferramenta) conta como a mesma quando a guardada é `antes`.
fn regiao_suja(
    s: Option<&mut PainterGpuPreview>,
    painter: &PainterTool,
    (ops, luts): (&[LayerOp], &[f32]),
    (w, h): (u32, u32),
    slot: Option<PainterPreviewGpu>,
    estrutura: Option<(Vec<LayerOp>, Vec<LayerOp>)>,
) -> Option<(u32, u32, u32, u32)> {
    let s = s?;
    let mesma_lista = s.entradas.0 == ops;
    let lista = mesma_lista
        || (!estrutura_recompoe_como_antes()
            && estrutura.is_some_and(|(antes, depois)| s.entradas.0 == antes && depois == ops));
    let mesmas = lista && s.entradas.1 == luts && s.entradas.2 == painter.papel();
    s.entradas.0.clear();
    s.entradas.0.extend_from_slice(ops);
    s.entradas.1.clear();
    s.entradas.1.extend_from_slice(luts);
    s.entradas.2 = painter.papel();
    #[cfg(test)]
    if SEMPRE_INTEIRA.get() {
        return None;
    }
    if ph2d_render::mipgen::previa_inteira() {
        return None;
    }
    let r = painter.preview_gpu_region()?;
    let mesmo_slot = slot.is_some_and(|g| (g.width, g.height) == (w, h));
    #[cfg(test)]
    if s.dona && mesmas && mesmo_slot && !mesma_lista {
        CONFINADAS_POR_ESTRUTURA.set(CONFINADAS_POR_ESTRUTURA.get() + 1);
    }
    (s.dona && mesmas && mesmo_slot).then(|| halo(r, w, h))
}

/// N7 onda 5, a chave A×B em execução: `PH2D_ESTRUTURA_RECOMPOE=1` = a placa compõe a tela inteira depois de
/// apagar, duplicar ou agrupar (o caminho de antes). Lida uma vez.
fn estrutura_recompoe_como_antes() -> bool {
    #[cfg(test)]
    if ESTRUTURA_RECOMPOE.get() {
        return true;
    }
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("PH2D_ESTRUTURA_RECOMPOE").is_some_and(|v| v == "1"))
}

/// B18.4: a camada nascida transparente entra na pilha guardada sem recompor (α = 0 deixa a figura) e a sua
/// fatia zera-se na placa — o 1.º toque compõe e envia só o sujo. Só quando a pilha antiga é a nova sem ela.
fn nasce_limpa(
    s: Option<&mut PainterGpuPreview>,
    ops: &[LayerOp],
    (key, versao): (u64, u64),
    wh: (u32, u32),
) {
    #[cfg(test)]
    if SEMPRE_INTEIRA.get() || NOVA_SEM_LIMPAR.get() {
        return;
    }
    let Some(s) = s else { return };
    let e_ela = |o: &LayerOp| matches!(*o, LayerOp::Layer { key: k, mask: None, clipping: false, .. } if k == key);
    let sem = ops.iter().filter(|o| !e_ela(o));
    if ops.iter().filter(|o| e_ela(o)).count() != 1 || !sem.eq(s.entradas.0.iter()) {
        return;
    }
    if s.compositor
        .nasce_limpa(&s.gpu, ops, key, wh, versao)
        .is_ok()
    {
        s.entradas.0 = ops.to_vec();
        s.entregue.push((key, versao));
    }
}

/// O retângulo alargado 1 texel para cada lado, preso à tela.
fn halo((x, y, rw, rh): (u32, u32, u32, u32), w: u32, h: u32) -> (u32, u32, u32, u32) {
    let (x0, y0) = (x.saturating_sub(1), y.saturating_sub(1));
    let (x1, y1) = ((x + rw + 1).min(w), (y + rh + 1).min(h));
    (x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0))
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
            planos_em_dia: false,
            dona: false,
            entregue: Vec::new(),
            a_entregar: Vec::new(),
            recusou: None,
            entradas: (Vec::new(), Vec::new(), None),
            luz: None,
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
    /// B17.5: as caixas do sujo da camada activa (`preview_gpu_caixas`); vazia = o retângulo só.
    partes: Vec<Region>,
}

impl<'a> PainterLayerProvider<'a> {
    fn new(tool: &'a PainterTool, anteriores: &'a [(u64, u64)], novas: Vec<(u64, u64)>) -> Self {
        let partes = tool
            .preview_gpu_caixas()
            .map(|(x, y, w, h)| Region { x, y, w, h })
            .collect();
        Self {
            tool,
            anteriores,
            novas: RefCell::new(novas),
            partes,
        }
    }
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
            dirty_parts: &self.partes,
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
    if let Some(nova) = painter.take_camada_nova_limpa() {
        nasce_limpa(
            session_slot.as_mut().filter(|s| s.dona && !renasce),
            &ops,
            nova,
            (w, h),
        );
    }
    if painter.take_preview_dirty() || renasce {
        let estrutura = painter.take_estrutura_confinada();
        let sujo = regiao_suja(
            session_slot.as_mut().filter(|_| !renasce),
            painter,
            (&ops, &adj_luts),
            (w, h),
            *painter_preview_gpu,
            estrutura,
        );
        if !drive(
            session_slot,
            renderer,
            painter,
            sel,
            ops,
            adj_luts,
            w,
            h,
            sujo.unwrap_or((0, 0, w, h)),
            sujo.is_none(),
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
    // B17.5: o sujo em várias caixas (o `rect` é a união delas + 1 px): cada uma com o seu halo.
    let caixas = janelas_das_caixas(tool, seed_full, width, height);
    let regioes: Vec<Region> = if caixas.is_empty() {
        vec![region]
    } else {
        caixas
            .iter()
            .map(|&c| {
                let (x, y, w, h) = halo(c, width, height);
                Region { x, y, w, h }
            })
            .collect()
    };

    // 1-3) Composite, light, premultiply — everything that turns the layer stack into the bytes the
    let tocadas = match compose_light_premul(
        session, tool, &ops, &adj_luts, width, height, &regioes, &caixas,
    ) {
        Ok(r) => r,
        Err(e) => {
            toasts.push(Toast::error(ph2d_i18n::tr_with(
                "app.painter.painter_gpu_preview.gpu_preview_failed",
                &[("e", &e)],
            )));
            release_slot(renderer, painter_preview_gpu);
            return false;
        }
    };

    // 4) Ensure a right-sized slot, then COPY the premultiplied result in: the whole canvas on a
    let slot = ensure_slot(renderer, painter_preview_gpu, entity_bits, width, height);
    let Some(premul_tex) = session.premul.output_texture() else {
        return false;
    };
    let copy = tocadas.iter().try_for_each(|&tocada| {
        if tocada == Region::full(width, height) {
            renderer.copy_texture_into_individual(slot, premul_tex, width, height)
        } else {
            let Region { x, y, w, h } = tocada;
            renderer.copy_texture_region_into_individual(slot, premul_tex, x, y, x, y, w, h)
        }
    });
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

/// B17.5: as caixas do sujo da ferramenta (vazia = a união sozinha, ou semente inteira).
fn janelas_das_caixas(
    tool: &PainterTool,
    seed_full: bool,
    width: u32,
    height: u32,
) -> Vec<(u32, u32, u32, u32)> {
    if seed_full || tool.preview_gpu_caixas().len() < 2 {
        return Vec::new();
    }
    tool.preview_gpu_caixas()
        .filter(|&(x, y, w, h)| w > 0 && h > 0 && x + w <= width && y + h <= height)
        .collect()
}

/// Composite, light, premultiply — everything that turns the layer stack into the bytes the preview
/// slot shows. `regioes` = as janelas da composição (uma, ou o halo de cada caixa de `caixas`).
#[allow(clippy::too_many_arguments)]
fn compose_light_premul(
    session: &mut PainterGpuPreview,
    tool: &PainterTool,
    ops: &[LayerOp],
    adj_luts: &[f32],
    width: u32,
    height: u32,
    regioes: &[Region],
    caixas: &[(u32, u32, u32, u32)],
) -> Result<Vec<Region>, String> {
    // 1) GPU composite into the PERSISTENT canvas-sized `out`, writing at canvas
    let mut novas = std::mem::take(&mut session.a_entregar);
    novas.clear();
    let provider = PainterLayerProvider::new(tool, &session.entregue, novas);
    // O PAPEL do documento: o compositor compõe cada saída SOBRE ele com a lei inteira da CPU
    session.compositor.set_paper(tool.papel());
    let composto = regioes.iter().try_for_each(|&region| {
        session.compositor.composite_region_into_canvas(
            &session.gpu,
            ops,
            adj_luts,
            &provider,
            width,
            height,
            region,
        )
    });
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
    // light's plane textures already seeded at this size; else the whole canvas. B17.5: with boxes,
    // one fold per box (the planes outside them did not change).
    let inteira = Region::full(width, height);
    let tela = (0, 0, width, height);
    let confinado = session.planos_em_dia && session.light.planes_seeded(width, height);
    let janelas: Vec<(u32, u32, u32, u32)> = match tool.preview_gpu_region().filter(|_| confinado) {
        None => vec![tela],
        Some(_) if !caixas.is_empty() && regioes.len() == caixas.len() => caixas.to_vec(),
        Some(r) => vec![r],
    };
    let mut tocadas: Vec<Region> = Vec::with_capacity(janelas.len());
    let (mut acesa, mut tela_toda) = (false, false);
    for (i, &plane_win) in janelas.iter().enumerate() {
        // The fold clock travels with its window (rect vs canvas).
        let m_fold = crate::paint_perf::on().then(std::time::Instant::now);
        // A dobra escreve direto nos buffers de envio da luz (sem `Vec` nem o `memcpy` do `write_texture`).
        session.planos_em_dia = false;
        let planes = tool.impasto_dobra(plane_win).map(|dobra| {
            let r = dobra.region;
            let janela = Region {
                x: r.0,
                y: r.1,
                w: r.2,
                h: r.3,
            };
            let envio = session
                .light
                .planos_para_envio(&session.gpu, janela, dobra.com_papel());
            dobra.dobrar_faixas(
                |linhas| envio.faixa(linhas),
                |f, j, l| f.escreve(j, l.relief, l.cover, l.mat0, l.mat1, l.papel),
            );
            (dobra, janela, envio)
        });
        if let Some(t0) = m_fold {
            crate::paint_perf::note_gpu_fold(
                t0.elapsed().as_secs_f64() as f32 * 1e3,
                plane_win == tela,
            );
        }
        let Some((planes, plane_region, envio)) = planes else {
            // Sem relevo: a composição sai sem luz (todas as janelas respondem igual).
            tocadas = regioes.to_vec();
            if session.luz.take().is_some() {
                tocadas = vec![inteira]; // a luz apagou-se: o slot tinha a tela acesa
            }
            break;
        };
        let lamps: Vec<ImpastoLamp> = planes
            .lamps
            .iter()
            .map(|l| ImpastoLamp {
                dir: l.dir,
                half: l.half,
                tint: l.tint,
            })
            .collect();
        let mesma_luz = session
            .luz
            .as_ref()
            .is_some_and(|(l, p)| *l == lamps && *p == planes.paper_body);
        // O relevo de fora da janela do fold não mudou: a luz só muda até 1 texel dela (a diferença central).
        let region = regioes.get(i).copied().unwrap_or(inteira);
        let tocada = if region != inteira && plane_win != tela && mesma_luz {
            let (x, y, w, h) = halo((region.x, region.y, region.w, region.h), width, height);
            Region { x, y, w, h }
        } else {
            inteira
        };
        let input = ImpastoLightInput {
            width,
            height,
            region: tocada,
            // …but the PLANES are only the fold's window (the textures persist); they come in `envio`.
            plane_region,
            relief: &[],
            cover: &[],
            mat0: &[],
            mat1: &[],
            lamps: &lamps,
            spec_lut: planes.spec_lut,
            lut_width: planes.lut_width,
            rough_levels: planes.rough_levels,
            paper_body: planes.paper_body,
            papel: &[],
        };
        // Falling through UNLIT would silently drop the shading.
        let lit = session
            .light
            .run_enviado(&session.gpu, comp_out, &input, envio);
        session.luz = Some((lamps, planes.paper_body));
        session.planos_em_dia = lit.is_ok();
        lit.map(|_| ()).map_err(|e| {
            ph2d_i18n::tr_with(
                "app.painter.painter_gpu_preview.impasto_light",
                &[("e_", &format!("{:?}", e))],
            )
        })?;
        acesa = true;
        tela_toda |= tocada == inteira;
        tocadas.push(tocada);
    }
    if tela_toda {
        tocadas = vec![inteira];
    }
    let finished: &wgpu::Texture = if acesa {
        session.light.output_texture().ok_or(ph2d_i18n::tr(
            "app.painter.painter_gpu_preview.composite_produced_no_texture",
        ))?
    } else {
        session.compositor.output_texture().ok_or(ph2d_i18n::tr(
            "app.painter.painter_gpu_preview.composite_produced_no_texture",
        ))?
    };

    // 3) Premultiply into a COPY_SRC texture (the slot samples PREMULTIPLIED);
    for &Region { x, y, w, h } in &tocadas {
        session
            .premul
            .run_region(&session.gpu, finished, width, height, (x, y, w, h))
            .ok_or(ph2d_i18n::tr(
                "app.painter.painter_gpu_preview.premultiply_produced_no_texture",
            ))?;
    }
    Ok(tocadas)
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

#[cfg(test)]
#[path = "painter_gpu_preview_nova_tests.rs"]
mod nova_tests;
