// ⚠️ O `// ph2d-loc-cap:` que estava aqui SAIU: ele é **inerte** em `crates/` — porquê, e o corte
// que o substituiu, no header do irmão `painter_bridge_upload`.
//! Painter (layers + effects) panel ⟷ tool bridge + on-canvas live preview.
//!
//! Modeled after `bgremoval_preview.rs`. What it does:
//!
//! 1. (Generic) Pushes the active sprite's RGBA into `PainterTool`
//!    via [`ph2d_tool_runtime::drive_source_push`] over the
//!    [`RasterEditTool`] upcast (so the layer stack reflects the live
//!    sprite pixels when the selection changes).
//! 2. (Painter-specific) Zero-copy preview drain via downcast +
//!    [`ph2d_tool_painter::PainterTool::take_preview_arc`] — bypasses
//!    [`ph2d_tool_runtime::drive_preview_cache`] (which would `to_vec` the
//!    buffer). Touches the allocator only on `Arc::make_mut` cycles.
//! 3. (Generic) Captures multi-sprite Apply selection via
//!    [`ph2d_tool_runtime::drive_pending_commit`] — `request_commit`
//!    sets the flag; bridge converts to `EditorAction::OneShotImageOp`.
//! 4. (Painter-specific) Inactive-path cache clear (mirror of
//!    BgRemoval's safety pattern).
//! 5. (Painter-specific) GPU preview lifecycle — uploads the composite
//!    into an `IndividualTextureStore` slot for the next frame's
//!    `PreviewOverride` (sprite suppression — see below).
//!
//! ## Cleanup semantics
//!
//! Apply (`pending_commit` true) returns the multi-sprite selection;
//! the caller drops the preview cache and pushes
//! `EditorAction::OneShotImageOp { tool_id: "painter", entity_bits }`
//! per entity. `Painter`'s `run_full` returns the composited layer stack which
//! the shell's image_edit dispatch writes back into the sprite texture
//! (same path as bgremoval / CEQ / upscale).
//!
//! ## Sprite suppression (replaces the Vello overlay)
//!
//! The live preview is the LAYER COMPOSITE (base layer = the sprite image
//! itself). It no longer paints as a Vello overlay ON TOP of the
//! still-rendered sprite (that duplicated the image: lowering the base
//! layer's opacity faded only the overlay, revealing the full-opacity sprite
//! underneath — Enio smoke 2026-06-01). Instead, mirroring BgRemoval, the
//! bridge uploads the premultiplied composite into an `IndividualTextureStore`
//! slot ([`PainterPreviewGpu`]); the next frame's `sim_extract` emits a
//! `PreviewOverride` that SUPPRESSES the source sprite and samples this
//! texture in its place. So the composite (incl. base-layer opacity) IS the
//! sprite, in-place, through the same sprite shader as Apply.

use crate::painter_bridge_assets::{load_brush_shape_image, load_brush_texture_image};
use crate::painter_gpu_preview::{self, PainterGpuPreview};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_editor::toast::ToastQueue;
use ph2d_host::WindowSize;
use ph2d_preview_slot::PreviewGpu as PainterPreviewGpu;
use ph2d_render::{Camera2d, SpriteRenderer};
use ph2d_tool_runtime::PreviewCache as PainterPreview;
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

/// Frame cap for the `PH2D_PREVIEW_DUMP` diagnostic trap (BUGS_painter.md #11). A stroke is tens of
/// frames, so this holds several gestures while keeping a long session from filling the disk.
const PREVIEW_DUMP_MAX_FRAMES: u32 = 240;

// `painter_has_unflushed_strokes` + `apply_layer_reparent` (tool-concrete downcast
// queries) moved to `painter_bridge_queries.rs` (HR-18 file-LOC cap).

/// Returns `true` iff an Apply committed this frame (caller tears the
/// tool down — deactivate + restore Inspector — so the on-canvas overlay
/// stops re-rendering on top of the freshly baked sprite).
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    sim: &SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    // The composite preview goes through the sprite pipeline (PreviewOverride),
    // not a Vello overlay — but these drive the on-canvas **brush cursor ring**
    // (a UI hint drawn into the overlay scene). See module header + the ring below.
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    // Text system for on-canvas labels (the Line dimension overlay's px / angle numbers).
    text_system: &mut ph2d_text::TextSystem,
    // Last cursor position in screen pixels (for the brush cursor ring).
    cursor: (f32, f32),
    last_painter_pushed_entity: &mut Option<u64>,
    painter_preview: &mut Option<PainterPreview>,
    painter_preview_gpu: &mut Option<PainterPreviewGpu>,
    // GPU live-preview session (compositor + premul blit). `None` until the
    // first GPU-representable frame lazily builds it (ADR-0045 Phase 3 step 2).
    painter_gpu_preview: &mut Option<PainterGpuPreview>,
    commit_requested: &mut bool,
    undo_requested: &mut bool,
    redo_requested: &mut bool,
    // **A DOAÇÃO de forma** — o canal nos dois sentidos: aqui se PUBLICA o tamanho do canvas (o
    // produtor precisa dele e não pode perguntar ao tool) e se CONSOME o plano que ele rasterizou.
    // ⚠️ Nenhum tipo do módulo 3D atravessa: o que chega é `Vec<f32>`. Ver `donated_form`.
    donated_form: &mut ph2d_form_donation::donated_form::DonatedForm,
    toasts: &mut ToastQueue,
    // `true` enquanto um botão de ponteiro está preso — o sinal de *"há um gesto em voo"* que o
    // `post_frame_undo` já usa. Aqui ele fecha o ciclo do rascunho de figura: ver `set_shape_draft_hold`.
    pointer_held: bool,
    // ⚠️ **O arrasto de cor do balde, RESOLVIDO pela shell** (W2 Fase D). O estado vive num
    // `thread_local` do `input_dispatch::fill_drag`, que é a camada de entrada — e era a última
    // aresta deste grupo de ficheiros para a `shells/desktop`. ⛔ Não é uma porta nova no
    // `AppHost`: escrito em TIPOS, o que a função precisa é de um `bool`.
    fill_drag_armed: bool,
    // ⚠️ **O LEITOR de pixels de um sprite, vindo da shell.** Devolve já em alfa DIRECTO
    // (`into_straight`), que é o que o `bind_document` quer. ⛔ Nenhum tipo do funil de
    // textura da shell (`SourceRead`) atravessa esta fronteira: o que chega é `Vec<u8>` + dims.
    read_source: impl FnOnce(
        ph2d_ecs::Entity,
        &SimWorld,
        &mut SpriteRenderer,
        &AssetDb,
        &BTreeMap<u32, AssetId>,
    ) -> Option<(Vec<u8>, u32, u32)>,
    // ⚠️ **O contador do perfilador de quadro, entregue pela shell.** Ele vive no LAÇO
    // (`render_loop::note_preview_px`, atrás do `PH2D_FLUID_PROFILE`) e era a única aresta
    // `super::` deste ficheiro para fora da família. ⛔⛔ E era **invisível à régua do fecho**,
    // que não resolve `super::` — foi preciso varrer à mão para a achar.
    note_preview_px: &dyn Fn(u64),
) -> bool {
    // Diagnostic TRAP for the mask-path FPS report (2026-07-24): `PH2D_PAINT_PERF=1` logs, per frame
    // the painter is active, WHICH producer owned the preview + WHICH drain path ran + the phase
    // timings + the canvas dims. This is what tells the difference between "the GPU producer's partial
    // upload fired" and "the mask stroke fell to a full CPU composite every frame" — the two the
    // headless proxy cannot tell apart. Zero cost when the var is unset.
    let perf_t0 = crate::paint_perf::on().then(std::time::Instant::now);
    let mut dbg_trivial = false;
    let mut dbg_gray = false;
    let mut dbg_active_is_mask = false;
    let mut dbg_dims = (0u32, 0u32);
    // Which arm the last drain took + WHY a trivial stack fell to the full arm (impasto / mask scratch).
    let mut dbg_branch = ph2d_tool_painter::DrainBranch::Idle;
    let mut dbg_impasto = false;
    let mut dbg_mask_scratch = false;

    let painter_is_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("painter"))
        .unwrap_or(false);

    // W2.T2.5: consume the Cmd/Ctrl+Enter commit flag (set in
    // `handle_editor_key`). Taken unconditionally so it can't leak to a
    // later painter activation; only acted on in the downcast block below.
    let commit_requested = std::mem::take(commit_requested);
    // W2.T2.2: same unconditional-take discipline for the stroke
    // undo/redo flags (Cmd+Z / Cmd+Shift+Z while Painter is active).
    let undo_requested = std::mem::take(undo_requested);
    let redo_requested = std::mem::take(redo_requested);

    // ── Dock visibility ───────────────────────────────────────────────────
    crate::painter_bridge_phases::dock_visibility(hero, painter_is_active);

    // ── C&F colour picker → brush colour (single source of truth, all modes) ──
    crate::painter_bridge_phases::forward_picker_colour(hero, tools);

    // ── Source push when the painter has no document for the selection → bind it ──
    crate::painter_bridge_phases::bind_document(
        crate::painter_bridge_phases::BindCtx {
            hero,
            tools,
            renderer,
            last_painter_pushed_entity,
            painter_preview_gpu,
            painter_gpu_preview,
            toasts,
        },
        painter_is_active,
        // ⚠️ O trio de leitura (`sim`, `asset_db`, `atlas_asset_map`) é CAPTURADO aqui, e não
        // passado adiante: o bind não os usa para mais nada, e capturá-los tira três argumentos
        // de uma assinatura que a própria ferramenta já dizia ser larga demais.
        |entity, renderer| read_source(entity, sim, renderer, asset_db, atlas_asset_map),
    );

    // ── A DOAÇÃO de forma: publica o TAMANHO, instala a NOTÍCIA ───────────
    crate::painter_bridge_phases::donate_form(tools, painter_is_active, donated_form);

    // Audit T1.5 round 1 B-H2: NO ghost `panel_visibility` insert. Painter
    // has no docked panel in T1.5 (sidebar lands W2 via
    // `ph2d-panel-painter`); inserting into the BTreeMap every frame just
    // to flip an unread bit risks colliding with the W2 sidebar's own
    // panel_visibility key. The Painter pill's pressed state is computed
    // directly off `tools.active().id()` in the topbar paint pass.

    let mut apply_selection: Vec<u64> = Vec::new();

    // ── Drain current_preview (FAST PATH) + capture Apply ─────────────────
    //
    // **R4-LG-1 fix:** bypass `drive_preview_cache` (which does a 16 MB
    // `pixels.to_vec()` per dirty drain — at 60 fps painting that's ~960
    // MB/s of allocator churn). Painter-specific downcast lets us pull
    // the canvas as a 1-atomic-inc `Arc<Vec<u8>>` clone via
    // `take_preview_arc()`. Net: per-stroke painting touches the
    // allocator ONCE per `Arc::make_mut` cycle (i.e., once per preview-
    // drain frame), not every pointer event.
    //
    // `drive_pending_commit` stays on the generic `&mut dyn RasterEditTool`
    // path — it's only called once per Apply, not per frame.
    // B.1: carries the partial dirty bbox from the preview drain to the GPU
    // upload below (frame-local — same function scope, so no `PreviewCache`
    // field change needed). `Some` = upload only this sub-rect; `None` = full.
    let mut painter_dirty_bbox: Option<(u32, u32, u32, u32)> = None;
    // The tool's monotonic canvas CONTENT version this frame — the shell keys its GPU-slot upload on
    // this instead of the drained `Arc`'s pointer. Keying on the pointer meant the shell had to HOLD a
    // clone of the live canvas so `Arc::make_mut` would hand back a fresh pointer on a change; holding it
    // is exactly what made every `stamp_dabs` copy the whole canvas per move (measured: 0.34 ms/move @
    // 2048², 10 ms/move @ 4096², flat across brush size — the CPU-bound FPS drop). Captured whether the
    // drain produced a frame or not (an idle frame reads the SAME version → the plan Skips).
    let mut cache_version: u64 = 0;
    // True when the GPU producer owns the preview slot this frame (representable
    // stack) — gates the CPU lifecycle block off so the two never fight the slot.
    let mut gpu_owns_preview = false;
    // PH2D_PAINT_PERF sub-phase accumulators (ms) — which part of dispatch a slow frame spends its
    // time in. `elapsed_ms` reads the mark only when the env is on (the `Instant` is `None` otherwise).
    let (mut ph_preview, mut ph_panel, mut ph_overlay) = (0f32, 0f32, 0f32);
    // …and the OVERLAY phase split by CALL, because 3,9 ms shared by three of them names none of
    // them (Enio, 2026-07-25, 4096²: o preview zerou e o custo mudou de lugar).
    let (mut ph_ov_tol, mut ph_ov_selection, mut ph_ov_chrome) = (0f32, 0f32, 0f32);
    // …and the same treatment for PANEL (by step) and CHROME (by overlay call).
    let mut ph_panel_sub = [0f32; crate::paint_perf::PANEL_SUB];
    let mut ph_chrome_sub = [0f32; crate::paint_perf::CHROME_SUB];
    let elapsed_ms =
        |m: Option<std::time::Instant>| m.map_or(0.0, |t| t.elapsed().as_secs_f64() as f32 * 1e3);
    let m_preview = perf_t0.map(|_| std::time::Instant::now());
    if let Some(tool) = tools.active_mut()
        && let Some(painter) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_painter::PainterTool>()
    {
        // **O SOLTAR do arrasto de knob** (`shape_draft`, 2º fio): o `set_shape_draft_hold(true)` é
        // publicado no instante do edit de painel (`render_loop::mod`, o drain de `ToolPanelEvent`),
        // mas quando o artista SOLTA não chega evento nenhum — então a queda mora aqui, no passe que
        // roda todo quadro, e é ela que ASSENTA a figura de volta na tela. Chamar com o valor vivo nos
        // dois sítios não é duas respostas: é a mesma pergunta feita no momento em que ela decide algo
        // (antes do re-carimbo) e uma vez por quadro (para o soltar ser visto).
        painter.set_shape_draft_hold(pointer_held);
        // Cmd/Ctrl+Enter Apply — bake the layer composite into the sprite this
        // same frame (the `drive_pending_commit` drain below picks it up).
        if commit_requested {
            painter.request_commit();
        }
        // Structural undo/redo (Cmd+Z / Cmd+Shift+Z). Both mark the preview
        // dirty so the `take_preview_arc` drain below re-composites this frame.
        if undo_requested {
            painter.undo_last();
        } else if redo_requested {
            painter.redo_last();
        }
        // The user picked the Image texture kind → open a native file picker, decode + install the
        // luminance as the brush texture (the pure engine has no file I/O). Cancel/failure reverts.
        if painter.take_brush_texture_image_request() {
            load_brush_texture_image(painter, asset_db, toasts);
        }
        // Same path for the brush **Shape** (silhouette) slot when the user picks Image in the Shape
        // dropdown. Cancel/failure simply leaves the silhouette as the falloff (nothing to revert).
        if painter.take_brush_shape_image_request() {
            load_brush_shape_image(painter, asset_db, toasts);
        }
        // Selection-drift invalidation (mirror of drive_preview_cache).
        if let (Some(existing), Some(sel)) = (painter_preview.as_ref(), hero.gizmo.selection)
            && existing.entity_bits != sel
        {
            *painter_preview = None;
        }
        // GPU-vs-CPU preview decision (ADR-0045 Phase 3 step 2): representable
        // stack → GPU composite (fast slider drags), else CPU `take_preview_arc`
        // below. Both end in `painter_preview_gpu`. See `try_drive`.
        if !gpu_owns_preview {
            gpu_owns_preview = painter_gpu_preview::try_drive(
                painter_gpu_preview,
                renderer,
                painter,
                hero.gizmo.selection,
                painter_preview_gpu,
                toasts,
            );
        }
        if gpu_owns_preview {
            // CPU cache unused while the GPU owns the slot — clear it so the
            // inactive/apply release + the gated CPU block below see `None`.
            *painter_preview = None;
        } else if let (Some(sel), Some((drained, w, h))) =
            (hero.gizmo.selection, painter.take_preview_arc())
        {
            // B.1: the bbox the drain recomposed (Some = partial fast lane).
            painter_dirty_bbox = painter.take_preview_upload_bbox();
            // O DIVISOR do `painter-dispatch` — a area publicada e' o que o
            // gather + premultiply + upload adiante tem de mover, e sem ela o
            // numero do log nao distingue um retangulo grande de um pequeno.
            // ⚠️ A bbox e' `(x, y, w, h)` -- NAO `(x0, y0, x1, y1)`, e o
            // doc-comment do `take_preview_upload_bbox` o diz. A 1a versao
            // subtraiu como se fossem cantos, o que da `(w - x) * (h - y)` e
            // SATURA EM ZERO para todo retangulo longe da origem: o log do
            // smoke veio `0.00 M px publicados em 80 quadros` -- o sitio de
            // contagem disparando e o valor sendo lixo.
            note_preview_px(painter_dirty_bbox.map_or_else(
                || u64::from(w) * u64::from(h),
                |(_, _, bw, bh)| u64::from(bw) * u64::from(bh),
            ));
            // Diagnostic TRAP for the per-layer-colour "rectangle residue" (BUGS_painter.md #11 — OPEN,
            // intermittent). `PH2D_PREVIEW_DUMP=<dir>` writes each frame's CPU composite (the exact bytes
            // about to be uploaded, BEFORE any overlay) to `<dir>/preview_NNNN.png`, capped so a long
            // session can't fill the disk. If the rectangle shows in these PNGs it is the composite; if
            // they are clean while the artifact is on screen, it is an on-top overlay or the GPU producer
            // (neither of which this lane touches). Zero cost when the var is unset.
            if let Some(dir) = std::env::var_os("PH2D_PREVIEW_DUMP") {
                use std::sync::atomic::{AtomicU32, Ordering};
                static N: AtomicU32 = AtomicU32::new(0);
                let n = N.fetch_add(1, Ordering::Relaxed);
                if n < PREVIEW_DUMP_MAX_FRAMES {
                    let path = std::path::Path::new(&dir).join(format!("preview_{n:04}.png"));
                    let _ = image::save_buffer(&path, &drained[..], w, h, image::ColorType::Rgba8);
                }
            }
            // Own the preview buffer: patch the dirty region out of the drained composite into the
            // shell's OWN buffer (in-place — the shell is its sole owner), then let `drained` drop. The
            // old code stashed `drained` (a clone of the tool's live `canvas_rgba` on the trivial path,
            // of its `composited` cache otherwise) and held it across the frame, so the tool's next
            // `Arc::make_mut` saw a second owner and copied the WHOLE plane per move. Patching a
            // shell-owned mirror is O(dirty bbox); the seed (no prior buffer / dims-or-entity change /
            // full recompose) copies the composite once. Either way the tool is left the SOLE owner of
            // its canvas, so its next stamp writes in place.
            let mirror = crate::painter_bridge_upload::own_preview_buffer(
                painter_preview.take(),
                sel,
                w,
                h,
                &drained,
                painter_dirty_bbox,
            );
            *painter_preview = Some(ph2d_tool_runtime::PreviewCache {
                entity_bits: sel,
                rgba: mirror,
                width: w,
                height: h,
            });
        }
        // Capture the content version for the upload plan below — bumped by the drain above on a
        // change, unchanged on an idle frame (so the plan Skips). Read here, inside the downcast, in
        // both the drained and idle cases.
        cache_version = painter.canvas_version();

        // PH2D_PAINT_PERF: close the PREVIEW phase (try_drive + drain), open the PANEL phase.
        ph_preview = elapsed_ms(m_preview);
        let m_panel = perf_t0.map(|_| std::time::Instant::now());

        // PH2D_PAINT_PERF diagnostic context (cheap reads; only used when the var is set).
        if perf_t0.is_some() {
            dbg_trivial = painter.preview_is_trivial_stack();
            dbg_gray = painter.mask_view_grayscale().is_some();
            dbg_active_is_mask = painter.active_is_mask();
            dbg_dims = painter.source_size();
            let (branch, impasto, mask_scratch) = painter.preview_drain_diag();
            dbg_branch = branch;
            dbg_impasto = impasto;
            dbg_mask_scratch = mask_scratch;
        }
        // Diagnostic TRAP, half 1 (BUGS_painter.md #11 — OPEN): `PH2D_PREVIEW_DIAG=1` logs which producer
        // owns the preview slot each frame + the CPU partial-upload bbox. This is what proved the per-layer
        // shape edits run on the CPU lane (`gpu_owns=false`) while a slider drag hands the slot to the GPU
        // producer — the CPU↔GPU handoff the headless harness cannot reach. Zero cost when unset.
        // ⚠️ Logs on CHANGE, not per frame. Per-frame it emitted 60 lines a second and the interesting
        // ones — a producer flip, a bbox going `None` — scrolled off before anyone could read them
        // (Enio, 2026-07-25: *"Muitos logs. tente diminuir"*). What this trap exists to show is a
        // TRANSITION, and a line per frame is the one format that hides transitions.
        if std::env::var_os("PH2D_PREVIEW_DIAG").is_some() {
            let now = (gpu_owns_preview, painter_dirty_bbox);
            if crate::paint_perf::preview_diag_changed(now) {
                eprintln!(
                    "[preview-diag] gpu_owns={gpu_owns_preview} cpu_dirty_bbox={painter_dirty_bbox:?}"
                );
            }
        }
        // Apply / commit capture — same trait path as bgremoval.
        // ⚠️ **DUAS declarações, e não um `allow(unused_mut)`.** Só o bloco do painel de camadas
        // reatribui este relógio, logo sem a feature ele nunca muta. Silenciar o diagnóstico seria
        // armengo mesmo com a ferramenta defeituosa — e aqui a ferramenta tem razão: nesta build a
        // variável **não** precisa de ser mutável.
        #[cfg(feature = "panel-painter-layers")]
        let mut m_p = perf_t0.map(|_| std::time::Instant::now());
        #[cfg(not(feature = "panel-painter-layers"))]
        let m_p = perf_t0.map(|_| std::time::Instant::now());
        apply_selection = ph2d_tool_runtime::drive_pending_commit(
            painter as &mut dyn ph2d_editor::tool::RasterEditTool,
            hero.gizmo.iter_selected(),
        );
        ph_panel_sub[0] = elapsed_ms(m_p);

        // (B.5 perf) Layers snapshot publish pro docked layers panel.
        // The panel paints a row per layer off this clone. **Gated on
        // `layers_revision()`:** the `LayerStack` is metadata-only, but the clone
        // (N rows × name `String`) ran EVERY frame Painter was active — including
        // every mouse-move during a layer drag, which made the reparent feel
        // sluggish (Enio 2026-06-02 "muito lenta"). `layers_revision` bumps only
        // on structural/metadata edits (`invalidate_composite` + `set_source`),
        // NOT strokes and NOT cursor moves. So during an in-flight drag the
        // structure is stable → we skip the clone entirely; the panel keeps its
        // last published snapshot and reads the live `painter_layer_drag()` cursor
        // for the overlay (panel re-paints every frame regardless). First
        // activation always publishes (sentinel `u64::MAX` ≠ any real revision);
        // the single persistent `PainterTool` instance keeps `layers_revision`
        // monotonic for the app lifetime, so an unchanged revision genuinely means
        // an unchanged stack (never a stale skip).
        // ⚠️ **A reatribuição do relógio vive DENTRO do `cfg`, e é obrigatório que viva** (W2 Fase
        // D): só este bloco o volta a ler, e na shell a feature era `default` — logo o código morto
        // sem ela nunca aparecia. Uma crate própria compila-se também SEM a feature, e aí um
        // `m_p = …` fora daqui é uma atribuição que ninguém lê.
        #[cfg(feature = "panel-painter-layers")]
        {
            m_p = perf_t0.map(|_| std::time::Instant::now());
            use std::sync::atomic::{AtomicU64, Ordering};
            static LAST_LAYERS_REV: AtomicU64 = AtomicU64::new(u64::MAX);
            let rev = painter.layers_revision();
            if LAST_LAYERS_REV.swap(rev, Ordering::Relaxed) != rev {
                ph2d_panel_painter_layers::set_current_layers(Some(painter.layers().clone()));
            }
            ph_panel_sub[1] = elapsed_ms(m_p);
            m_p = perf_t0.map(|_| std::time::Instant::now());
            // (W3 multi-select) Publish the selection set every frame — a tiny
            // BTreeSet (≤ HARD_CAP_LAYERS u64s). NOT gated on `layers_revision`:
            // a plain re-click that collapses a multi-selection onto the
            // already-active layer changes the selection WITHOUT a structural
            // edit, so a revision gate would miss it. The panel reads this for
            // the multi-row highlight (active = strong outline, others = wash).
            ph2d_panel_painter_layers::set_current_selection(painter.selection());
            // (Mask view) Publish which mask's grayscale-view eye is open so its row draws it open.
            ph2d_panel_painter_layers::set_current_mask_grayscale_view(
                painter.mask_view_grayscale(),
            );
            // (Brush UI) Publish the active brush snapshot every frame — a tiny
            // Copy struct (size/colour/blend). The panel's Brush section reads it
            // to position the Size/RGB sliders + the blend chip. Not revision-
            // gated: brush edits don't bump `layers_revision`, and the cost is a
            // few floats.
            let brush_snapshot = painter.brush_settings();
            ph_panel_sub[2] = elapsed_ms(m_p);
            m_p = perf_t0.map(|_| std::time::Instant::now());
            let stroke_method_u8 = brush_snapshot.stroke_method;
            ph2d_panel_painter_layers::set_current_brush(Some(brush_snapshot));
            // Wet Tuning side panel (doc 22): same snapshot, same cadence — and
            // its visibility MIRRORS the tool's authored Tuning checkbox
            // (painter active + wet armed + tuning_open). Edge-triggered z
            // bump: `WET_TUNING_PANEL` rides the z fallback list, but bumping
            // on open keeps it above its dock neighbours.
            ph2d_panel_wet_tuning::set_current_brush(Some(brush_snapshot));
            let tuning_open =
                painter_is_active && brush_snapshot.wetpaint && brush_snapshot.wet_tuning_open;
            hero.panel_visibility.insert("wet_tuning", tuning_open);
            {
                use std::sync::atomic::{AtomicBool, Ordering};
                static TUNING_WAS_OPEN: AtomicBool = AtomicBool::new(false);
                if !TUNING_WAS_OPEN.swap(tuning_open, Ordering::Relaxed) && tuning_open {
                    hero.store.bump_panel_z(ph2d_editor::ids::WET_TUNING_PANEL);
                }
            }
            // (Tool rail) The rail radio FOLLOWS the painter's mode rather than remembering which button
            // was clicked. The Impasto section's unified TOOL list can change the mode too (picking
            // "Chisel" there enters Sculpt), and a rail that only learned about its own clicks would go
            // on highlighting "Brush" while the artist sculpts — two answers to "which tool am I
            // holding?", with the wrong one on screen. Self-gating: writes only when it actually moved.
            ph2d_editor::screens::hero::chrome::sync_painter_rail_to_mode(
                &mut hero.store,
                painter.active_paint_mode_id(),
            );
            // (Eyedropper) When the on-canvas colour pick completes (armed → not armed), snap the tool
            // rail radio back to Brush — the pick is a MOMENTARY tool, so its button stops looking
            // selected once a colour is sampled. Edge-detected via a static so arming the pick (which
            // sets armed=true) never triggers an immediate reset (only the true→false pick does).
            {
                use std::sync::atomic::{AtomicBool, Ordering};
                static PREV_EYEDROPPER_ARMED: AtomicBool = AtomicBool::new(false);
                let armed = painter.eyedropper_armed();
                if PREV_EYEDROPPER_ARMED.swap(armed, Ordering::Relaxed) && !armed {
                    ph2d_editor::screens::hero::chrome::reset_painter_rail_to_brush(
                        &mut hero.store,
                    );
                }
            }
            // (Shapes rail) Keep the tool-rail's active button in sync with the stroke method: choosing a
            // shape in the Brush panel's Method dropdown moves the rail to the matching Shapes button
            // (a non-shape method leaves the radio alone — returning to Brush is the Brush button's job).
            // Edge-detected so it fires only on a real method change, not every frame.
            {
                use std::sync::atomic::{AtomicU8, Ordering};
                static PREV_STROKE_METHOD: AtomicU8 = AtomicU8::new(u8::MAX);
                if PREV_STROKE_METHOD.swap(stroke_method_u8, Ordering::Relaxed) != stroke_method_u8
                {
                    ph2d_editor::screens::hero::chrome::sync_painter_rail_to_stroke_method(
                        &mut hero.store,
                        stroke_method_u8,
                    );
                }
            }
            // (Brush UI) Publish the dock view-mode so the panel renders either
            // the Layers/Effects body or the Brush-properties body (header toggle).
            ph2d_panel_painter_layers::set_current_dock_shows_layers(painter.dock_shows_layers());
            // (Texture preview) Publish the brush Image texture (lum + dims) for the panel's Texture
            // preview — gated on the tool's image version so the heavy `Vec` is cloned only on change.
            {
                use std::sync::atomic::{AtomicU64, Ordering};
                static LAST_TEX_IMG_VER: AtomicU64 = AtomicU64::new(u64::MAX);
                let ver = painter.brush_texture_image_version();
                if LAST_TEX_IMG_VER.swap(ver, Ordering::Relaxed) != ver {
                    let img = painter
                        .brush_texture_image()
                        .map(|(lum, w, h)| (std::sync::Arc::new(lum.to_vec()), w, h));
                    ph2d_panel_painter_layers::set_current_brush_texture_image(img);
                }
            }
            // (Shape preview) Publish the brush Shape image (the silhouette tip) the same way — gated
            // on the tool's shape-image version so the heavy `Vec` is cloned only on change.
            {
                use std::sync::atomic::{AtomicU64, Ordering};
                static LAST_SHAPE_IMG_VER: AtomicU64 = AtomicU64::new(u64::MAX);
                let ver = painter.brush_shape_image_version();
                if LAST_SHAPE_IMG_VER.swap(ver, Ordering::Relaxed) != ver {
                    let img = painter
                        .brush_shape_image()
                        .map(|(lum, w, h)| (std::sync::Arc::new(lum.to_vec()), w, h));
                    ph2d_panel_painter_layers::set_current_brush_shape_image(img);
                }
            }
            // (Paper preview) Publish the watercolor Paper slot image the same way — gated on its version.
            {
                use std::sync::atomic::{AtomicU64, Ordering};
                static LAST_PAPER_IMG_VER: AtomicU64 = AtomicU64::new(u64::MAX);
                let ver = painter.brush_paper_image_version();
                if LAST_PAPER_IMG_VER.swap(ver, Ordering::Relaxed) != ver {
                    let img = painter
                        .brush_paper_image()
                        .map(|(lum, w, h)| (std::sync::Arc::new(lum.to_vec()), w, h));
                    ph2d_panel_painter_layers::set_current_brush_paper_image(img);
                }
            }
            // (Shape source linkage) If the multi-layer Shape was captured from the ACTIVE sprite, re-capture
            // it when that sprite changed (paint / opacity / visibility / undo) — keeping the per-layer
            // colours. Cheap revision compare per frame; re-captures only on a change. Before the preview
            // refresh so the preview reflects the re-captured Shape the same frame.
            ph_panel_sub[3] = elapsed_ms(m_p);
            m_p = perf_t0.map(|_| std::time::Instant::now());
            painter.refresh_shape_source_if_changed();
            // (Shape preview, Per-Layer Color) Publish the multi-layer COLOURED composite so the Shape
            // preview shows the per-layer colours — the colours need the per-layer pixels, which only the
            // tool has. The tool re-bakes the composite ONLY when the Shape appearance changes (a cheap
            // key-compare per frame), so we publish (and pay the bake) on an edit, never per frame.
            if painter.refresh_shape_color_preview() {
                ph2d_panel_painter_layers::set_current_brush_shape_color_preview(
                    painter.shape_color_preview(),
                );
            }
            ph_panel_sub[4] = elapsed_ms(m_p);
        }

        // PH2D_PAINT_PERF: close the PANEL phase (snapshot publish + shape re-bake), open OVERLAY.
        ph_panel = elapsed_ms(m_panel);
        let m_overlay = perf_t0.map(|_| std::time::Instant::now());

        // Keep the shape-editor grab tolerance in sync with the live camera every frame (not just on a
        // painter Down/Move/Up), so the on-canvas handles are drawn where they'll be grabbed — no snap
        // when the first grab after a zoom refreshes the tol.
        crate::painter_bridge_overlays::refresh_shape_grab_tol(
            painter,
            hero,
            sim,
            camera,
            window_size,
        );
        ph_ov_tol = elapsed_ms(m_overlay);
        let m_ov_sel = perf_t0.map(|_| std::time::Instant::now());
        // Repeat Image FIRST: the 3×3 tile preview is canvas CONTENT (the composite at the 8 neighbour
        // positions), so it must sit UNDER all editing chrome. Drawn after the overlays it covered any
        // chrome extending past the sprite border — a shape crossing the seam lost its editor overlay
        // beyond the edge, and the brush ring / marching ants vanished over the neighbour tiles
        // (the "overlay stops at the seam" bug, Enio 2026-07-11).
        crate::painter_bridge_overlays::draw_repeat_image(
            painter,
            hero,
            sim,
            camera,
            window_size,
            vector_scene,
            painter_preview.as_ref(),
        );
        // Selection overlay next (under the editor handles + brush ring): marching ants + hatching + the
        // crosshair cursor.
        crate::painter_bridge_selection_overlay::draw_selection_overlay(
            painter,
            hero,
            sim,
            camera,
            window_size,
            vector_scene,
            cursor,
        );
        ph_ov_selection = elapsed_ms(m_ov_sel);
        let m_ov_chrome = perf_t0.map(|_| std::time::Instant::now());
        crate::painter_bridge_overlays::draw_overlays(
            painter,
            hero,
            sim,
            camera,
            window_size,
            vector_scene,
            text_system,
            cursor,
            &mut ph_chrome_sub,
            perf_t0.is_some(),
            fill_drag_armed,
        );
        // PH2D_PAINT_PERF: close the OVERLAY phase (and its last sub-call).
        ph_ov_chrome = elapsed_ms(m_ov_chrome);
        ph_overlay = elapsed_ms(m_overlay);
    }

    // ── O caminho INACTIVO e o APPLY ──────────────────────────────────────
    if crate::painter_bridge_phases::settle_inactive_and_apply(
        hero,
        renderer,
        painter_is_active,
        &apply_selection,
        painter_preview,
        last_painter_pushed_entity,
        painter_preview_gpu,
    ) {
        gpu_owns_preview = false;
    }
    // ── GPU lifecycle for the live-preview texture (W3 sprite-suppression) ──
    // Mirror of `bgremoval_preview`: upload the premultiplied composite into a
    // transient `IndividualTextureStore` slot; NEXT frame's `sim_extract` reads
    // `painter_preview_gpu` to emit a `PreviewOverride` that SUPPRESSES the
    // source sprite and samples THIS texture in its place. The composite is
    // STRAIGHT sRGB8 (the canvas / `take_preview_arc`); byte-space
    // `premultiply_rgba8` matches EXACTLY what Apply's
    // `SpriteImage::into_premultiplied` produces, so the live preview is
    // byte-for-byte identical to the committed result on the same
    // `Rgba8UnormSrgb` + premul-blend sprite shader (no Vello gamma/blend
    // divergence, no image duplication). 1-frame lag is imperceptible.
    //
    // On a GPU-owned frame the GPU producer fills the slot; hide the CPU cache
    // from this block so it neither re-uploads nor releases that slot.
    let m_upload = perf_t0.map(|_| std::time::Instant::now());
    crate::painter_bridge_upload::upload_cpu_preview(
        renderer,
        painter_preview.as_ref().filter(|_| !gpu_owns_preview),
        painter_dirty_bbox,
        cache_version,
        gpu_owns_preview,
        painter_preview_gpu,
        toasts,
    );

    if let Some(t0) = perf_t0
        && painter_is_active
    {
        // Record this frame's dispatch info + sub-phase split; the frame timer (`run_render_frame`)
        // pairs it with the whole-frame time and the aggregator prints ONE summary per window.
        crate::paint_perf::record_dispatch(crate::paint_perf::FrameInfo {
            gpu: gpu_owns_preview,
            dispatch_ms: t0.elapsed().as_secs_f64() as f32 * 1e3,
            preview_ms: ph_preview,
            panel_ms: ph_panel,
            overlay_ms: ph_overlay,
            ov_tol_ms: ph_ov_tol,
            ov_selection_ms: ph_ov_selection,
            ov_chrome_ms: ph_ov_chrome,
            panel_sub: ph_panel_sub,
            chrome_sub: ph_chrome_sub,
            upload_ms: elapsed_ms(m_upload),
            // ⚠️ Preenchidos pelo `record_dispatch` a partir do acumulador: o fold é anotado de DENTRO
            // do `try_drive` (que roda na fase `preview` acima), onde a janela dele é resolvida — aqui
            // não há como saber se ela foi um retângulo ou a tela.
            fold_ms: 0.0,
            fold_full: false,
            w: dbg_dims.0,
            h: dbg_dims.1,
            gray: dbg_gray,
            active_is_mask: dbg_active_is_mask,
            lane_partial: painter_dirty_bbox.is_some(),
            trivial: dbg_trivial,
            branch: dbg_branch,
            impasto: dbg_impasto,
            mask_scratch: dbg_mask_scratch,
        });
    }
    !apply_selection.is_empty()
}
