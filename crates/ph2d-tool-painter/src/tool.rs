//! `PainterTool` — impl Tool + RasterEditTool (T1.5 ship + T1.6 brush mature).
//!
//! ## T1.5 + T1.6 status
//!
//! **T1.5 — RasterEditTool real (CPU stamp render):**
//! [`crate::tool::PainterTool::queue_pointer`] aciona o [`StampScheduler`]
//! sobre `canvas_rgba` via [`ph2d_painter_brush::apply_stamps`] (paridade
//! ULP-bounded ao shader `stamp.wgsl`). `StampPipeline` (GPU compute) está
//! pronto + naga-validated mas **não está plugado ainda** — integração GPU
//! cycle (texture lifecycle no shell + ping-pong dispatches retidos
//! cross-frame) é seguinte (T-perf W5+). CPU path entrega o Day-7 marker
//! "primeira pintura visível" sem deferral funcional.
//!
//! T1.6 — Brush mature (shape variety + multi-stamp + rotation + color
//! jitter). `ph2d-painter-brush` ganhou **4 procedural shape kernels**
//! com slot dispatch (`round_hard` slot 0, `round_soft` slot 1,
//! `square_hard` slot 2, `oval_hard` slot 3 — last added R4 V-2 como
//! demo de `shape_rotation_follow` / calligraphic 2:1 oblong), multi-
//! stamp emission (`shape_count` 1..=16 + `shape_count_jitter`),
//! rotation pipeline (`shape_rotation_follow` + `shape_scatter` +
//! `shape_randomized`), flip bits (`shape_flip_x` / `shape_flip_y`
//! aplicados ANTES da rotation em shape-local space), e Color Dynamics
//! stamp-level jitter (hue/saturation/lightness/darkness com axis-tag
//! isolation gate-proven). `PainterTool` API pública não muda — todas
//! as novas capacidades são driven pelo `Brush.shape` +
//! `Brush.color_dynamics` que o tool já passa ao scheduler.
//!
//! W1 day 7 smoke: ativar Painter pill, clicar/arrastar no canvas → marcas
//! visíveis no sprite, alpha-over acumulando entre stamps. **Smoke T1.6
//! sugerido:** trocar pra `round_soft` ou `square_hard` brush, configurar
//! `shape_count=3 + shape_scatter=30°` + `stamp_hue_jitter=0.5`, traçar
//! um arco → cluster de stamps rotacionados com hue variando ao longo
//! do stroke.
//!
//! ## W2 follow-ups documentados (audit T1.5 rounds 3+4)
//!
//! - **R3-LE-4 — Commit path unwired:** `request_commit()` é `pub` mas
//!   nenhum input handler do shell chama. Day-7 ship is preview-only;
//!   `Esc` ou tool-switch perde a pintura. W2 wires the sidebar Apply
//!   button (or `Cmd+Enter` shortcut) para `painter.request_commit()`.
//! - **R3-LE-5 / R4-LH-8 — Stale canvas after external mutation:** se
//!   outro tool bakeia o sprite ativo enquanto Painter está ativo,
//!   `canvas_rgba` fica stale. Bridge não re-pusha. W2 adiciona
//!   sprite-version tracking ou invalida `last_painter_pushed_entity`
//!   cross-tool. Manifesta visualmente como overlay esticado se sprite
//!   muda dimensão mid-stroke.
//! - **R3-LF-3 — Failed Apply destrói canvas:** se `drain_painter`
//!   falhar (source unavailable / commit error), o teardown ainda
//!   deactivates Painter → canvas perdido. W2 retorna `Result<(),
//!   Failed>` de `drain_painter` e gateia teardown em `painter_apply_
//!   committed && drain_succeeded`.
//! - **R3-LF-4 — Cancel via tool-switch silently drops strokes:** today
//!   tool switch zera o canvas sem warn. W2: emit `Toast::warning` em
//!   `on_deactivate` quando canvas não-empty + has_painted_since_source.
//! - **R4-LG-2 — PREMUL canvas storage:** today canvas storage é STRAIGHT
//!   u8; cpu_render::apply_one_stamp paga 3 divisões + 8 multiplicações
//!   per pixel pra premul/unpremul dance. PREMUL storage removeria a
//!   dance (~35% speedup per-pixel) e alinharia com GPU `rgba8unorm`
//!   pipeline. Refactor scope: cpu_render straight↔premul boundary
//!   moves to set_source + run_full. Defer pra T-perf (alongside GPU
//!   pipeline integration ou separadamente como otimização CPU MVP).
//! - **R4-LG-3 — Per-pixel match dispatch hoist:** apply_rendering_mode
//!   chama 6-way match dentro do per-pixel loop. LLVM unswitch should
//!   fire mas CFG fragmentation (early-continue paths) pode bloquear.
//!   Refactor: monomorfizar via const generic `fn apply_pixels::<const
//!   MODE: u8>(...)`. Defer; bench primeiro pra justificar custo.
//! - **R4-LG-6 — CPU MVP regime:** apply_stamps no CPU é HR-3 friendly
//!   apenas em `size_px ≤ 256`. Sliders maiores quebram budget 4.5ms.
//!   W2: UI soft-cap brush size em 256 até GPU pipeline T-perf W5+.
//! - **R4-LH-4 — 32-bit usize overflow on wasm:** apply_stamps `idx`
//!   computation poderia overflow em wasm32 com canvases > 16M pixels.
//!   Cheap fix: `debug_assert!(canvas.len() <= isize::MAX as usize)` no
//!   top de apply_stamps. Defer; wasm target ainda não é prioridade.
//! - **R4-LH-6 — Silent clamp dispatch drop:** StampPipeline::encode
//!   release-clamps stamps.len() > MAX silently. Add `tracing::warn!`
//!   when tracing crate joins workspace.
//! - **R4-LH-11 — `from_u32` forward-compat:** adding RenderingMode
//!   variant 6+ without updating `from_u32` would silent-fallback to
//!   UniformGlaze. Mitigated by `MAX_RENDERING_MODES = 6 FROZEN` arch-
//!   gate (ADR amendment required to add). Acceptable; doc.
//! - **R6-LN-3 — HR-18 LOC cap policy ambiguity:** painter brush files
//!   (cpu_render, stamp_scheduler, stamp_pipeline) exceed 600 total
//!   lines when `#[cfg(test)]` modules are counted, but stay under
//!   when prod-only. Workspace-wide policy concern (CLAUDE.md should
//!   codify the cap as production-excluding-tests). Painter follows
//!   existing convention; no action.
//! - **R6-LN-4 — HR-15 hardcoded Toast strings:** `drain_painter` (+
//!   bridge fallback toast) uses raw `Toast::error("...")` literals.
//!   6 of 11 sibling image-edit drains follow the same pattern; only
//!   3 use `ph2d_i18n::tr(...)`. Workspace-wide HR-15 clarification
//!   needed (whether "no hardcoded UI strings" covers toasts or only
//!   widget labels). Painter is conforming to majority; no action.
//!
//! ## T1.6 audit follow-ups (rounds 1-6, 13 lentes distintas)
//!
//! Round 1 (O atlas / P color / Q multi-stamp): **33 findings**
//! (0 Crit, 6 High, 15 Med, 12 Low — sums 33 ✓).
//! Round 2 (R regressions / S spec): **16 findings** (0 Crit, 0 High,
//! 10 Med, 6 Low — sums 16 ✓; prior draft said "12" erroneously).
//! Round 3 (T edge cases / U cross-OS HR-5 / W test quality / Z perf
//! budget): **50 findings** (3 Crit, 12 High, 23 Med, 12 Low —
//! sums 50 ✓).
//! Round 4 (A1 R3-regression / V acceptance + ship readiness):
//! **17 findings** (3 Crit, 4 High, 6 Med, 4 Low — sums 17 ✓; prior
//! draft said "11" erroneously).
//! Round 5 (B1 R4-regression / C1 acceptance verification):
//! **10 findings** (0 Crit, 0 High, 4 Med, 6 Low — sums 10 ✓; padrão-
//! ouro threshold atingido).
//! Round 6 (D1 prod safety + panic surface / E1 recovery integrity /
//! G1 doc cross-ref): **39 findings** (3 Crit, 7 High, 14 Med,
//! 15 Low — sums 39; CRIT + HIGH remediated in-session here in R6
//! close: lib.rs/cpu_render/tool.rs/shader doc drift "3→4 shapes"
//! propagation + R2/R4 arithmetic fix + handoff §2.5 CI commits
//! addendum + spec gate path footers + ToggleEyedropper mutex bypass
//! fix in tool-bgremoval).
//!
//! **Cumulative T1.6 fingerprint**: 9 Crit + 29 High + 72 Med + 55
//! Low = **165 findings across 6 rounds × 13 lenses**. All CRIT and
//! HIGH remediated in-session (R3 + R4 + R5 + R6 fixes shipped):
//!
//! R3: U-1 HR-5 language correction, U-2 det-painter feature flag plus
//! arch-gate, Z-1 dhat HR-3 gate, T-1/T-2/U-5/U-10 finite guards,
//! Z-4/Z-5/Z-8 perf hoists, W-1/W-4/W-5/W-13/T-11/T-12 stronger tests,
//! W-2/W-3 deferred com FOLLOW-UP-W6 marker.
//!
//! R4: A1-W12 vacuous self-match gate fix (slice pre-cfg-test),
//! V-1 env vars `PAINTER_SMOKE_BRUSH/COUNT/SCATTER/HUE_JITTER/
//! ROTATION_FOLLOW` consumed in `build_smoke_brush_from_env`,
//! V-2 ship `oval_hard` shape kernel (slot 3) + brush + parity gate,
//! A1-U1 + V-3 + V-4 + V-5 doc overclaim corrections.
//!
//! Padrão-ouro threshold (zero Crit/High em round-final) atingido após
//! R4 remediations.
//!
//! Deferred to W2+ with explicit rationale:
//!
//! - **T-numerical-parity (O-2):** runtime CPU↔GPU pixel-parity test
//!   requires wgpu device in CI. Today's textual gate
//!   (`cpu_shader_shape_kernels_textual_parity`) pins each side
//!   independently; a true bit-equality runtime test lands when GPU CI
//!   ships. ETA: W2-W4 (alongside `painter_no_alloc_hot_path` dhat
//!   integration).
//! - **NFold rotational symmetry API (O-6):** current `shape_is_radial_
//!   symmetric` is a boolean; future shapes with 4-fold / 2-fold /
//!   asymmetric symmetry need a `SymmetryKind` enum + tight footprint
//!   bound per kind. Lands with first asymmetric shape (W6+).
//! - **`square_hard` AA band screen-px width drift (O-9):** the
//!   smoothstep band thickness in screen pixels varies with rotation
//!   because the band is shape-relative `[0.90, 1.0]`. Visible only on
//!   `square_hard` at non-axis-aligned rotations. Pixel-derivative AA
//!   (`fwidth`-equivalent in WGSL) lands W6+ once visual feedback
//!   confirms the artifact matters.
//! - **Pearson cross-channel correlation gate (P-5):** today's bit-
//!   equality gate (`color_jitter_cross_channel_axis_independence`)
//!   proves the strongest invariant; a statistical Pearson `|r| <
//!   0.05` gate is W2+ refinement.
//! - **HDR L clamp policy (P-6):** `apply_stamp_color_jitter` clamps L
//!   to `[0, 1]` (sRGB-gamut assumption). HDR / Display P3 / Rec2020
//!   profiles need a higher `MAX_OKLAB_L` const + a profile-aware
//!   clamp. Lands when ADR-0048 ColorProfile expansion targets HDR
//!   (T-color-full).
//! - **`det_random` seed=0 degenerate (P-9):** at `stroke_seed=0`,
//!   first-stamp PRNG output is dominated by `axis_tag` (low entropy).
//!   No production code seeds with 0 (caller derives from pointer-time
//!   plus entity plus brush hash), but a defensive `seed ^ 0x9E37...`
//!   constant fold would harden the path. Defer.
//! - **Perf bench gate `apply_stamp_color_jitter` (P-10):** worst-case
//!   load (4096 stamps × 4 jitters all >0) costs ~120 µs estimated.
//!   Within 4.5 ms Painter sub-budget but unmeasured. Criterion bench
//!   infra lands W2+ alongside other perf gates.
//! - **Denormal jitter flush-to-zero (P-11):** untrusted brush JSON
//!   could carry sub-normal floats (10⁻⁴⁰). x86 without FTZ pays
//!   100s of cycles on denormal multiply. Defer; brush-load validation
//!   in `ph2d-painter-contracts` is the right home (W2+).
//! - **First-stamp `[1.0, 0.0]` sentinel artifact (Q-9):** the first
//!   stamp of a stroke with `shape_rotation_follow=true` uses 0°
//!   rotation (no direction yet). Documented in spec §1.3.4.1. "First-
//!   stamp deferral" (delay emission until second pointer arrives,
//!   back-fill direction) is W2+ feature to eliminate the artifact.
//! - **Hue jitter intermediate-slider curve (R-3):** PH2D uses linear
//!   `slider × π` mapping; Procreate uses super-linear. UX A/B test
//!   needed to decide — defer until brush studio panel ships (W5+).
//! - **Asymmetric-shape flip gate (R-8):** `flip_preserves_output_for_
//!   symmetric_shapes` is tautological for T1.6's doubly-symmetric
//!   shapes. Real pixel-level flip gate requires asymmetric shape
//!   (first one ships W6+ — `flat_chisel`). Tracker: search this file
//!   for `FOLLOW-UP-W6` to find this entry.
//! - **`shape_count_zero` brush-load validation (Q-7):** scheduler
//!   clamps `shape_count=0 → 1` as defense-in-depth. Real fix is
//!   validation at `ph2d-painter-contracts` brush-load time (typed
//!   error). W2+ alongside other brush-param validators.
//! - **`R6-LN-4` HR-15 Toast strings policy (round-1):** unchanged —
//!   workspace-wide policy clarification pending.

use std::path::PathBuf;
use std::sync::Arc;

use ph2d_color::OklchColor as StrokeOklchColor;
use ph2d_editor_core::floating_panel::{FloatingPanel, ToolId};
use ph2d_editor_core::tool::{RasterEditTool, Tool};
use ph2d_painter_brush::{
    Brush, BrushParamsHash, MAX_STAMP_SIZE_PX, PointerSample, StampScheduler, apply_stamps, library,
};
use ph2d_painter_stroke::{
    CanvasId, FlushPolicy, JournalError, LayerId, PartialStroke, RawPointerSample,
    SAMPLE_FLAG_AZIMUTH_UNAVAILABLE, SAMPLE_FLAG_BARREL_ROLL_UNAVAILABLE,
    SAMPLE_FLAG_TILT_UNAVAILABLE, SAMPLE_FLAG_TIMESTAMP_UNAVAILABLE, StrokeHistory, StrokeJournal,
    StrokeRecord, ToolMode, f32_to_q88, f32_to_q1616_saturating,
};

use crate::params::{BrushHandle, OklchColor, PainterMode, PainterParams};

// Marker for the empty-apply guard (R3-LF-5): a stamp was actually
// deposited since the last `set_source`. `drain_painter` early-returns
// when this is false to avoid wasting a new Individual texture + undo
// snapshot on identity-baking the source pixels.

/// Painter — sucessor do Procreate. Stateful workhorse tool.
///
/// Cascata W0 (ADR-0043..0053) congelou caps e contratos. T1.1 entregou
/// skeleton + manifest. T1.5 entrega RasterEditTool real (CPU stamp
/// render com paridade textual ao shader `StampPipeline`; CPU↔GPU
/// pixel-parity é ULP-bounded `~1-4 ULP` cross-backend per WGSL spec,
/// **NOT bit-identical** — see audit T1.6 U-1 + `cpu_render.rs` header).
///
/// ## Architecture — content vs pipeline
///
/// `PainterTool` é **state holder**: `canvas_rgba` (RGBA8 straight, fonte
/// de verdade do conteúdo) + estado de stroke (scheduler, brush, color,
/// size_px, pending pointer queue). É testável headless; o subsistema
/// integer (PRNG mixer + ABI + index arithmetic) é HR-5 cross-OS
/// bit-identical, o subsistema float (trig + sqrt + OKLab cubic) é
/// ULP-bounded e requer `--features det-painter` (no-op hoje; wiring
/// progressive) para cross-OS strict determinism.
///
/// **Audit T1.6 R8 O1-2 — escopo do `det-painter`:** a regra
/// "trig + sqrt + OKLab cubic precisa `det-painter`" se aplica a
/// **operações que executam por-stamp/por-pixel** (cos/sin/sqrt
/// dentro do shader e do scheduler). Operações IEEE 754
/// determinísticas (`+`, `-`, `*`, `/`, `round`) e parsing
/// (`f32::from_str` via ryu/dtoa) **NÃO** precisam de `det-painter`
/// — são bit-identical cross-OS por construção. Em particular o
/// `parse::<f32>()` das env vars smoke (`PAINTER_PARAMS_SIZE_PX`,
/// `_SCATTER`, `_HUE_JITTER`, `_SPACING`) é HR-5-safe sem flag.
///
/// O **dispatch GPU** (T-perf W5+) virá no bridge `painter_bridge.rs` que
/// terá acesso a `GpuContext` + textures retidos A↔B. Quando ele plugar,
/// `queue_pointer` deixa de chamar `apply_stamps` (CPU) e passa a empilhar
/// stamps num buffer drainable pelo bridge — API pública intacta.
///
/// ## Threading contract (audit T1.9 R-9 + T-durability O-1)
///
/// `PainterTool` é auto-`Send + Sync` (todos os fields são Send+Sync), MAS
/// todas as ops de mutação (`begin_stroke`/`queue_pointer`/`end_stroke`/
/// `attach_journal`/`set_*`) tomam `&mut self`. Implicações:
///
/// - **`Arc<Mutex<PainterTool>>` é foot-gun para long-strokes.** Stroke
///   típico = 1-3s; long-stroke (paint-bucket via path stroke, fluid sim,
///   pen pressure-hold drag 30min) segura o Mutex por todo o intervalo,
///   bloqueando MCP queries (W13) + Inspector live (W14) + `take_preview_arc`
///   da UI thread. Pattern recomendado: canal SPSC (MCP/UI → PainterTool
///   owner thread) ou snapshot-clone para queries read-only.
/// - **`commit_stroke` faz fsync síncrono** (~5-10ms eMMC mobile, audit
///   T-durability N-7 + T1.9 R-7). Mobile shell DEVE wire em worker
///   thread; T1.9 ship com path direto + carry-over W11. Vide
///   `StrokeJournal::commit_stroke` doc para SPSC pattern.
/// - **`stroke_history()` read-only via `&self` ainda é Mutex-bloqueada**
///   pq Rust `Mutex::lock()` é exclusive. Inspector W14 deve clonar o
///   `StrokeHistory` snapshot ao invés de manter borrow live.
pub struct PainterTool {
    pub params: PainterParams,
    /// Working canvas — RGBA8 straight, sem gamma encoding (compatível com
    /// `wgpu::TextureFormat::Rgba8Unorm`). **`Arc<Vec<u8>>`** (audit T1.5
    /// round 4 R4-LG-1): bridge's `painter_preview` cache holds an Arc
    /// clone of this same buffer (1 atomic increment per dirty drain
    /// instead of `Vec::to_vec` 16 MB memcpy at 60 Hz). `queue_pointer`
    /// mutates via `Arc::make_mut`: O(1) when refcount==1, O(N) clone
    /// only on the FIRST mutation after the bridge took a new Arc —
    /// amortized to one canvas-clone per "preview frame cycle" rather
    /// than per pointer event.
    canvas_rgba: Arc<Vec<u8>>,
    source_size: (u32, u32),
    preview_dirty: bool,
    pending_commit: bool,
    /// Scheduler com pool 4096 Stamps reservado no construtor (HR-3 alloc-
    /// free hot path daí em diante).
    scheduler: StampScheduler,
    /// Brush ativo. Default = round_hard. T1.6+ troca via `PainterUiEdit::
    /// SelectBrush` quando library expandir.
    brush: Brush,
    /// Verdadeiro entre `begin_stroke` e `end_stroke` (bridge controla via
    /// pointer-down / pointer-up).
    stroke_active: bool,
    /// **R3-LF-5 guard:** true iff at least one stamp landed since the last
    /// `set_source`. `drain_painter` (Apply) early-returns when this is
    /// false so a no-stroke Apply doesn't waste a fresh Individual texture
    /// + a no-op undo slot on identity-baking the source.
    has_painted_since_source: bool,
    /// **R4-LG-4 cache:** color in OKLab form, refreshed at `begin_stroke`
    /// from `params.active_color`. `queue_pointer` reads this instead of
    /// recomputing `oklch_to_oklab` (two transcendentals per pointer
    /// event) for every stamp.
    ///
    /// **R5-LI-N contract for W2 sidebar:** `PainterUiEdit::SetColor`
    /// MUST gate on `!is_stroke_active()` OR explicitly refresh
    /// `stroke_color_oklab` after writing `params.active_color`. A
    /// write to `params.active_color` mid-stroke will NOT take effect
    /// until the next `begin_stroke` — silent visual regression if the
    /// sidebar handler doesn't honor this contract. Today
    /// `handle_panel_event` is a no-op stub (line ~390) so the contract
    /// can't be violated in T1.5; the constraint binds W2.
    stroke_color_oklab: [f32; 4],
    // === T1.9 fields — wire StrokeHistory + StrokeJournal (ADR-0046/0052) ===
    /// In-memory canon de strokes commitados deste canvas. Source of truth
    /// pra undo (W2) / Reproject (W12) / Inspector (W14) / MCP (W13).
    pub stroke_history: StrokeHistory,
    /// WAL writer opcional — quando `Some`, todo stroke é jornalado per-
    /// sample/per-commit pra tear-resistant recovery (ADR-0052). `None` =
    /// modo "in-memory only" (tests headless, MCP replay determinístico,
    /// canvas efêmero sem persistence path).
    ///
    /// Caller wireia via [`Self::attach_journal`] após `Tool::on_activate`;
    /// libera via [`Self::detach_journal`] pré-`Tool::on_deactivate` OR
    /// `Drop` da PainterTool (RAII via StrokeJournal::Drop).
    pub stroke_journal: Option<StrokeJournal>,
    /// `PartialStroke` em-progresso (entre `begin_stroke` e `end_stroke`).
    /// `Some` iff `stroke_active == true`. Materializado em `end_stroke`
    /// como `StrokeRecord` via `partial_to_record` + push em history.
    current_partial: Option<PartialStroke>,
    /// Buffer in-memory de samples deste stroke (paralelo ao `StrokeJournal`
    /// buffer quando journal ativo; ÚNICA fonte de verdade quando journal
    /// é None). Cap u16::MAX preserva semântica ADR-0046 §2.2.
    current_samples: Vec<RawPointerSample>,
    /// Próximo `seq` pra novo stroke. Monotonic per-canvas; ADR-0046 §2.2
    /// invariante "seq cresce sempre, replay determinístico depende".
    /// Persistido em canon `.ph2d-painter` ⇒ caller restora via
    /// `PaintProject::last_persisted_seq().unwrap_or(0) + 1` no load.
    next_seq: u64,
    /// Canvas identity — embedded em `PartialStroke.canvas_id` pra
    /// `CrashRecovery::committed_for_canvas(id)` filter multi-canvas W11+.
    canvas_id: CanvasId,
    /// Layer alvo deste stroke. W3 layers nasce → caller seta via
    /// `params.target_layer`; T1.9 default `LayerId(0)` (single-raster).
    layer_target: LayerId,
    /// **Audit T1.9 R-5** — cache do `Brush::params_blake3()` (postcard
    /// serialize ~1KB + blake3 ~32B). Invalidado em [`Self::set_brush`].
    /// Sem cache: 30 strokes/s × ~1KB alloc per `begin_stroke` no flicking
    /// path. Com cache: 1 hash per brush change (raro).
    cached_brush_hash: Option<BrushParamsHash>,
    /// **Audit T1.9 Q-3/R-3** — última falha do WAL (begin/add_sample/commit).
    /// `None` = sem erros desde a última `attach_journal`. Surface pro
    /// bridge W11 emitir toast "Painter durability degraded — recent
    /// strokes in-memory only" via [`Self::last_wal_error`].
    last_wal_error: Option<JournalError>,
}

impl Default for PainterTool {
    fn default() -> Self {
        let mut params = PainterParams::default();
        // Audit T1.6 C1-1: brush size lives on PainterParams (not Brush
        // struct), so we wire it here alongside the brush env vars.
        // Range 1.0..=2048.0 mirrors `PropertiesParams::max_size_px`
        // (spec §1.3.11).
        if let Ok(s) = std::env::var("PAINTER_PARAMS_SIZE_PX")
            && let Ok(f) = s.parse::<f32>()
        {
            params.size_px = f.clamp(1.0, 2048.0);
        }
        Self {
            params,
            canvas_rgba: Arc::new(Vec::new()),
            source_size: (0, 0),
            preview_dirty: false,
            pending_commit: false,
            scheduler: StampScheduler::new(),
            brush: build_smoke_brush_from_env(),
            stroke_active: false,
            has_painted_since_source: false,
            stroke_color_oklab: [0.0; 4],
            // T1.9 defaults:
            stroke_history: StrokeHistory::default(),
            stroke_journal: None,
            current_partial: None,
            // **V-5:** lazy alloc — Default ficaria com ~7KB heap dead se
            // PainterTool fosse instantiada pra registry preload sem nunca
            // pintar. `begin_stroke` reserve(256) on-demand.
            current_samples: Vec::new(),
            next_seq: 0,
            canvas_id: CanvasId(0),
            layer_target: LayerId(0),
            cached_brush_hash: None,
            last_wal_error: None,
        }
    }
}

/// Build the active brush for `PainterTool::default()`, honoring **audit
/// T1.6 V-1 + C1-1 + C1-2 smoke env vars** when set. Returns
/// `library::round_hard()` if no env vars are set (production default —
/// unchanged behavior).
///
/// Env vars (W2 sidebar replaces — temporary smoke surface):
/// - `PAINTER_SMOKE_BRUSH` — `"round_hard"` (default) / `"round_soft"` /
///   `"square_hard"` / `"oval_hard"`.
/// - `PAINTER_SMOKE_COUNT` — `u32` 1..=16; sets `brush.shape.shape_count`.
/// - `PAINTER_SMOKE_SCATTER` — `f32` 0..=360 degrees;
///   sets `brush.shape.shape_scatter`.
/// - `PAINTER_SMOKE_HUE_JITTER` — `f32` 0..=1;
///   sets `brush.color_dynamics.stamp_hue_jitter`.
/// - `PAINTER_SMOKE_ROTATION_FOLLOW` — `"true"` / `"false"` (default true
///   for `oval_hard`, false for others);
///   sets `brush.shape.shape_rotation_follow`.
/// - `PAINTER_SMOKE_SPACING` — `f32` 0.01..=1.0;
///   sets `brush.stroke_path.spacing` (audit C1-2; needed to make
///   `shape_count > 1` clusters visually distinct rather than overlapping).
///
/// Unparsable values fall back to default + emit `eprintln!` warning.
/// Strips ASCII quotes from `PAINTER_SMOKE_BRUSH` to handle shell-quoting
/// edge cases (audit R5-B1-4 defensive).
///
/// **Brush size (`PAINTER_PARAMS_SIZE_PX`)** is consumed in
/// `PainterTool::default()` after this function returns — it lives on
/// `PainterParams.size_px`, not on the `Brush` struct. See env-var
/// handling alongside the brush construction.
///
/// Example smoke run for the T1.6 acceptance §2.2 cluster + rotation +
/// hue-jitter demo:
/// ```bash
/// PAINTER_SMOKE_BRUSH=oval_hard \
/// PAINTER_SMOKE_COUNT=3 \
/// PAINTER_SMOKE_SCATTER=30 \
/// PAINTER_SMOKE_HUE_JITTER=0.5 \
/// PAINTER_SMOKE_SPACING=0.3 \
/// PAINTER_PARAMS_SIZE_PX=64 \
/// cargo run -p ph2d-host-desktop
/// ```
///
/// **Audit T1.6 R7 L1-2:** in the editor that comes up, the Painter
/// pill lives in the **Image Tools** cluster (TopBar topright). The
/// pill is only painted when **Image Tools mode is ON** — toggle the
/// Image Tools topbar control first, THEN click the Painter pill to
/// activate the tool. Without that prerequisite step, the documented
/// recipe above appears to do nothing because the pill itself is
/// hidden (`palette_visible_tool_indices` filters tools by
/// `is_image_edit_tool` × `image_tools_mode_on`). W2 sidebar follow-up
/// may auto-enable Image Tools when `PAINTER_SMOKE_BRUSH` is set so
/// the smoke recipe becomes a single env-var run.
///
/// **W2 follow-up:** wire `apply_ui_edit` + sidebar widgets so these
/// settings come from PainterUiEdit dispatch instead of env vars.
fn build_smoke_brush_from_env() -> Brush {
    // Audit R5-B1-4: defensive shell-quote stripping (zsh sometimes
    // passes single-quoted values verbatim). Audit R7 L1-9: also
    // `.trim()` ASCII whitespace — `PAINTER_SMOKE_BRUSH= oval_hard`
    // (leading space, common after `=` typo) was previously falling
    // through to the catch-all warning + round_hard default.
    let brush_name = std::env::var("PAINTER_SMOKE_BRUSH")
        .map(|s| s.trim_matches(|c| c == '\'' || c == '"').trim().to_string());
    let mut brush = match brush_name.as_deref() {
        Ok("round_soft") => library::round_soft(),
        Ok("square_hard") => library::square_hard(),
        Ok("oval_hard") => library::oval_hard(),
        Ok("round_hard") | Err(_) => library::round_hard(),
        Ok(other) => {
            eprintln!(
                "[painter] PAINTER_SMOKE_BRUSH={other:?} unknown (after quote-strip + trim); \
                 defaulting to round_hard. Valid: round_hard / round_soft / square_hard / \
                 oval_hard. Check shell quoting if your value appears wrapped in quotes."
            );
            library::round_hard()
        }
    };

    if let Ok(s) = std::env::var("PAINTER_SMOKE_COUNT")
        && let Ok(n) = s.parse::<u32>()
    {
        brush.shape.shape_count = n.clamp(1, 16);
    }
    if let Ok(s) = std::env::var("PAINTER_SMOKE_SCATTER")
        && let Ok(d) = s.parse::<f32>()
    {
        brush.shape.shape_scatter = d.clamp(0.0, 360.0);
    }
    if let Ok(s) = std::env::var("PAINTER_SMOKE_HUE_JITTER")
        && let Ok(j) = s.parse::<f32>()
    {
        brush.color_dynamics.stamp_hue_jitter = j.clamp(0.0, 1.0);
    }
    // Audit R7 L1-3: lowercase + trim + explicit accept/reject so
    // `=True`, `=TRUE`, `=on`, `=yes ` (trailing space), `=` (empty
    // un-set) don't silently flip the brush default. Unknown values
    // keep the brush default AND emit a warning so the user sees the
    // typo instead of debugging a "rotation_follow stopped working"
    // mystery.
    if let Ok(raw) = std::env::var("PAINTER_SMOKE_ROTATION_FOLLOW") {
        let s = raw.trim().to_ascii_lowercase();
        match s.as_str() {
            "true" | "1" | "yes" | "on" => brush.shape.shape_rotation_follow = true,
            "false" | "0" | "no" | "off" => brush.shape.shape_rotation_follow = false,
            "" => {
                eprintln!(
                    "[painter] PAINTER_SMOKE_ROTATION_FOLLOW is set but empty; \
                     keeping brush default (rotation_follow={}). To clear, \
                     `unset PAINTER_SMOKE_ROTATION_FOLLOW` instead.",
                    brush.shape.shape_rotation_follow
                );
            }
            other => {
                eprintln!(
                    "[painter] PAINTER_SMOKE_ROTATION_FOLLOW={other:?} unrecognized; \
                     keeping brush default (rotation_follow={}). Expected: \
                     true/1/yes/on or false/0/no/off (case-insensitive).",
                    brush.shape.shape_rotation_follow
                );
            }
        }
    }
    // Audit C1-2: spacing env var so `shape_count > 1` clusters stay
    // visually distinct rather than collapsing into a blob.
    if let Ok(s) = std::env::var("PAINTER_SMOKE_SPACING")
        && let Ok(f) = s.parse::<f32>()
    {
        brush.stroke_path.spacing = f.clamp(0.01, 1.0);
    }

    brush
}

impl PainterTool {
    /// Inicia um novo stroke. Caller deriva `seed` de inputs determinísticos
    /// (e.g., `pointer_down_time_ms ^ entity_bits ^ brush_hash`).
    ///
    /// **Q-2 (audit T1.9):** se já há stroke ativo (caller esqueceu
    /// `end_stroke`), o anterior é **cancelado** sem consumir `next_seq` —
    /// `next_seq` só avança quando um stroke vai virar `StrokeRecord`
    /// (ou seja, fim deste método com PartialStroke materializado).
    pub fn begin_stroke(&mut self, seed: u64) {
        if self.stroke_active {
            // Defensive: previous stroke didn't close cleanly. Encerra-o
            // implicitamente sem commit pra evitar state corruption.
            self.scheduler.end_stroke();
            // T1.9: descarta PartialStroke + journal active stroke também.
            // Sem commit = stroke aborted; journal sample-batch buffer
            // limpo via cancel_stroke (não suja WAL).
            if let Some(journal) = self.stroke_journal.as_mut()
                && journal.current_stroke.is_some()
            {
                let _ = journal.cancel_stroke();
            }
            // Q-2: stroke cancelado NÃO consumiu next_seq — reciclar o seq
            // do PartialStroke abandonado pra próximo stroke usar.
            if let Some(p) = self.current_partial.take() {
                self.next_seq = p.seq;
            }
            self.current_samples.clear();
        }
        self.scheduler.begin_stroke(seed);
        self.stroke_active = true;
        // R4-LG-4: cache OKLab color at stroke boundary. UI updates to
        // `params.active_color` between strokes — recomputing the two
        // transcendentals per pointer event is pure waste otherwise.
        self.stroke_color_oklab = oklch_to_oklab(self.params.active_color);

        // T1.9: construir PartialStroke + wire journal se ativo.
        //
        // **S-8 NaN guard:** `params.active_color` é pub field; bridge PCA
        // pode escrever NaN/Inf inadvertently. OKLCH NaN persistido em WAL +
        // canon explode em replay W12 (postcard preserva bits, mas brush
        // matemática faz NaN propagation cascading). Sanitize aqui.
        let primary = sanitize_oklch_or_default(self.params.active_color);
        // **S-3:** `secondary_color` (Procreate long-press slot ADR-0046 §2.2)
        // SEMPRE persistido em T1.9 wire — caller W2 sidebar refinará "in
        // use" semantics em ADR-0046-amendment-2 (carry-over W11 S-15).
        let secondary = Some(sanitize_oklch_or_default(self.params.secondary_color));
        // **R-5:** cache do `params_blake3` (postcard alloc + blake3 ~32B).
        // `set_brush` invalida; entre strokes, hit cache = 0 alloc.
        let brush_hash = match self.cached_brush_hash {
            Some(h) => h,
            None => {
                let h = self.brush.params_blake3();
                self.cached_brush_hash = Some(h);
                h
            }
        };
        let brush_handle_canon = brush_handle_stub_to_canon(self.params.active_brush);
        let mut partial = PartialStroke::new(
            self.next_seq,
            self.canvas_id,
            self.layer_target,
            brush_handle_canon,
            brush_hash,
            primary,
        );
        partial.rng_seed = seed;
        // **S-1 wall-clock:** ADR-0046 §2.2 + ADR-0052 §2.2 prescrevem
        // `started_at_ms` populated em Begin. Pré-S1, ficava 0 → quebrava
        // time-lapse W11 + Inspector W14 chronology + audit log. Wall-clock
        // é EXPLICITAMENTE não-determinístico (ADR-0046 §3 Neutras) ⇒ não
        // entra em det-replay (vide U-9 doc em StrokeRecord).
        partial.started_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        // **S-2 ToolMode mapping (ADR-0043 §2.6.1 congelado):**
        partial.tool_mode = painter_mode_to_tool_mode(self.params.mode);
        partial.secondary_color = secondary;
        // **Q-3/R-3 + Q-9 + V-2:** se WAL falhar, captura em `last_wal_error`
        // pro bridge W11 surface. NÃO materializa current_partial — assim
        // queue_pointer/end_stroke não chamam add_sample/commit em journal
        // sem Begin, evitando cascade `NoActiveStroke` que clobbed o erro
        // original (V-2). PainterTool fica em modo "CPU-only painting; nem
        // WAL nem in-memory history pra esse stroke" — explicit doc no
        // `last_wal_error()` accessor.
        let mut wal_accepted = true;
        if let Some(journal) = self.stroke_journal.as_mut() {
            if let Err(e) = journal.begin_stroke(partial.clone()) {
                debug_assert!(
                    false,
                    "WAL begin_stroke failed: {:?} — degrading to in-memory mode",
                    e
                );
                self.last_wal_error = Some(e);
                wal_accepted = false;
            }
        }
        if wal_accepted || self.stroke_journal.is_none() {
            self.current_partial = Some(partial);
        } else {
            // WAL anexado mas rejeitou begin → manter current_partial = None.
            // queue_pointer/end_stroke vão tratar como "in-memory only stroke
            // sem record" (V-2 cascade fix: nenhum WAL call subsequent).
            self.current_partial = None;
        }
        self.current_samples.clear();
        // **V-7:** reserve cap lazy — Default mantém Vec::new() (V-5 fix);
        // begin_stroke pre-aloca quando vai começar a popular.
        if self.current_samples.capacity() < 256 {
            self.current_samples.reserve(256);
        }
        // Q-10/R-10: next_seq avança IFF PartialStroke foi materializado.
        // `checked_add` em vez de saturating: u64 overflow é fisicamente
        // impossível em uso humano (~580 anos a 1B strokes/s), mas LOUD
        // panic é melhor que silent dedup violando ADR-0046 §2.2.
        if self.current_partial.is_some() {
            self.next_seq = self
                .next_seq
                .checked_add(1)
                .expect("next_seq u64 overflow — canvas needs canonical reset");
        }
    }

    /// Empilha um pointer sample no scheduler e aplica os stamps gerados
    /// sobre `canvas_rgba` (CPU path T1.5). No-op se nenhum stroke ativo.
    ///
    /// **Audit T1.6 R7 L1-5:** silent no-op when `!self.stroke_active`
    /// historically masked a bridge bug — a pointer-down handler that
    /// forgot to call `begin_stroke` (or had its dispatch path
    /// early-return before reaching it) would silently drop every drag
    /// sample, and the subsequent `drain_painter` would emit a
    /// `Toast::info("Painter: no strokes to apply")` indistinguishable
    /// from "Apply ran with no paint". The `debug_assert!` here surfaces
    /// the missed `begin_stroke` immediately in dev/test builds without
    /// changing release behavior. Pair with the toast-wording follow-up
    /// in `drain_painter` (separate dispatch site) for the full UX fix.
    pub fn queue_pointer(&mut self, sample: PointerSample) {
        debug_assert!(
            self.stroke_active || self.canvas_rgba.is_empty(),
            "queue_pointer called without an active stroke on a non-empty \
             canvas — the bridge wired pointer-move but not pointer-down. \
             Sample {:?} silently dropped. Audit R7 L1-5.",
            sample
        );
        if !self.stroke_active || self.canvas_rgba.is_empty() {
            return;
        }
        let size_px = self.effective_size_px();
        let stamps = self
            .scheduler
            .advance(&self.brush, sample, size_px, self.stroke_color_oklab);
        if stamps.is_empty() {
            return;
        }
        // R4-LG-1: `Arc::make_mut` ⇒ unique-borrow path when refcount==1
        // (zero alloc), clone-once when bridge cached the prior Arc
        // (16 MB once per preview drain cycle, NOT per pointer event).
        // Explicit `Vec<u8>` annotation prevents the compiler from
        // coercing through `Arc<[u8]>::make_mut` (different impl).
        let canvas_vec: &mut Vec<u8> = Arc::make_mut(&mut self.canvas_rgba);
        apply_stamps(canvas_vec, self.source_size.0, self.source_size.1, stamps);
        self.preview_dirty = true;
        self.has_painted_since_source = true;

        // T1.9: persist sample em history vetorial (in-memory) + journal.
        // Conversão `PointerSample (f32)` → `RawPointerSample (Q16.16 + Q8.8)`
        // via helpers HR-5 cross-OS. Cap u16::MAX preserva semântica
        // ADR-0046 §2.2; sample 65536+ é silenciosamente dropado pra
        // não-bloquear UI (caller raramente atinge — strokes reais têm
        // <512 samples).
        //
        // **Q-5/R-8 clock contract:** `pseudo_now_ms = len * 16` é
        // SYNTHETIC tick — válido só enquanto T1.9 não wire wall-clock.
        // Quando W11+ shell wirar `Instant::elapsed().as_millis()`,
        // `attach_journal` DEVE ser invocado de novo (a journal nova
        // nasce com `last_flush_ms = 0`, evitando flush storm da
        // diferença sintético→wall-clock). Vide doc de `attach_journal`.
        let raw = pointer_to_raw_sample(sample);
        if self.current_samples.len() < u16::MAX as usize {
            self.current_samples.push(raw);
            // **V-2 cascade fix:** only call journal.add_sample if journal
            // has the corresponding Begin (current_stroke.is_some()). Sem
            // o gate, journal.begin_stroke fail em begin_stroke leva todo
            // add_sample subsequente a retornar NoActiveStroke, clobbando
            // last_wal_error original (e.g. PayloadTooLarge → NoActiveStroke).
            if let Some(journal) = self.stroke_journal.as_mut()
                && journal.current_stroke.is_some()
            {
                let pseudo_now_ms = (self.current_samples.len() as u64) * 16;
                // **Q-3/R-3:** captura erro WAL em `last_wal_error` em vez
                // de silenciar. Bridge W11 surface via `last_wal_error()`.
                if let Err(e) = journal.add_sample(raw, pseudo_now_ms) {
                    self.last_wal_error = Some(e);
                }
            }
        } else {
            // **Q-7:** cap u16::MAX hit. Drop silencioso é foot-gun pra
            // MCP path que injeta 70k samples. T1.9 ship: warn via
            // eprintln (until `tracing` joins workspace, R6-LN-1).
            // W11+: auto-split em begin_stroke novo + sentinel
            // `SAMPLE_FLAG_STROKE_SPLIT_BOUNDARY` no último sample
            // pré-split. Carry-over em W11 stitched-stroke.
            eprintln!(
                "[ph2d-painter] queue_pointer: stroke hit MAX_SAMPLES_PER_STROKE \
                 ({}); subsequent samples dropped until end_stroke. Long-stroke \
                 split is W11 carry-over (audit T1.9 Q-7).",
                u16::MAX
            );
        }
    }

    /// Finaliza o stroke atual. Idempotente — chamar duas vezes é seguro.
    ///
    /// **T1.9 — wire history + journal commit:** se houver `PartialStroke`
    /// em-progresso, materializa pra `StrokeRecord` + push em
    /// `stroke_history` + journal.commit_stroke (WAL Commit entry +
    /// fsync). Sem PartialStroke ativo (e.g., end_stroke chamado 2× OR
    /// begin_stroke seguiu sem queue_pointer), é no-op silencioso.
    ///
    /// Fsync síncrono (ADR-0052 §2.2 N-7 doc): caller mobile DEVE wire
    /// em worker thread shell-side. T1.9 ship com path síncrono —
    /// carry-over W11.
    pub fn end_stroke(&mut self) {
        self.scheduler.end_stroke();
        self.stroke_active = false;
        // T1.9: materializar PartialStroke → StrokeRecord + history push.
        let Some(mut partial) = self.current_partial.take() else {
            // Nenhum PartialStroke ⇒ no-op. **Q-10:** defesa-em-profundidade
            // — se journal por algum motivo tem stroke ativo, cancela
            // explicitamente pra não deixar Begin órfão no WAL.
            if let Some(journal) = self.stroke_journal.as_mut()
                && journal.current_stroke.is_some()
            {
                let _ = journal.cancel_stroke();
            }
            self.current_samples.clear();
            return;
        };
        // **V-7:** mem::take em vez de mem::replace+Vec::with_capacity —
        // begin_stroke pre-aloca, end_stroke não precisa re-alocar.
        let samples = std::mem::take(&mut self.current_samples);
        // **V-1/V-3 empty-stroke gate (CRITICAL):** pré-V1, set_source
        // mid-stroke + begin_stroke implicit-cancel materializavam empty
        // records → canon polluído com phantom strokes; W2 undo veria
        // ghosts; W12 Reproject re-pintaria nada com seq consumido; W14
        // Inspector mostraria rows vazias.
        //
        // Fix: se samples empty, RECYCLE seq + emit WAL Cancel em vez de
        // Commit. Espelha o Q-2 recycle de begin_stroke implicit-cancel
        // path. Preserva pre-T1.9 semantic "end_stroke sem painting = cancel".
        if samples.is_empty() {
            if let Some(journal) = self.stroke_journal.as_mut()
                && journal.current_stroke.is_some()
            {
                let _ = journal.cancel_stroke();
            }
            // Recycle seq pra próximo stroke usar (Q-2 parity).
            self.next_seq = partial.seq;
            return;
        }
        partial.samples_count_in_journal = samples.len() as u32;
        // Journal commit primeiro (preserva ordering "wrote to WAL before
        // history"); se WAL falhar, history ainda recebe (caller mobile
        // pode optar perder durability vs. perder stroke commit).
        //
        // **V-2 cascade fix:** só chama commit_stroke se journal tem o
        // Begin desse stroke. Sem o gate, journal_begin_stroke fail em
        // begin_stroke (current_partial=None branch) levaria todo commit
        // subsequent a retornar NoActiveStroke, clobbando last_wal_error.
        if let Some(journal) = self.stroke_journal.as_mut()
            && journal.current_stroke.is_some()
        {
            let pseudo_now_ms = (samples.len() as u64) * 16;
            // **Q-3/R-3:** captura WAL commit failure pra surface bridge.
            if let Err(e) = journal.commit_stroke(pseudo_now_ms) {
                self.last_wal_error = Some(e);
            }
        }
        let record = partial_to_record(partial, samples);
        self.stroke_history.push(record);
        // **S-5 WAL rotation:** após commit, checka se journal precisa
        // rotacionar (cap 500 MiB OR 100 commits). T1.9 ship: rotate só
        // depois de canon flush (caller W11 contract). Aqui apenas surface
        // o sinal — caller polla via `should_rotate_journal()` accessor.
        // Implementação completa = W11 carry-over S-5 (precisa coordenar
        // com PaintProject save_canon).
    }

    /// Anexa um WAL `StrokeJournal` ao PainterTool. Caller (shell W11+)
    /// invoca após `Tool::on_activate` quando canvas tem persistence path.
    /// `flush_policy` default = `Hybrid { n: 8, ms: 100 }`.
    ///
    /// Retorna erro se:
    /// - `journal_path` já está locked (outro processo / outra thread do
    ///   mesmo processo via in-process registry).
    /// - I/O falha (path inválido, permissions).
    /// - **Audit Q-8/R-1:** `is_stroke_active()` (caller chamou attach
    ///   no meio de um stroke) → `JournalError::AttachDuringActiveStroke`.
    ///
    /// **Audit Q-5/R-8 — clock contract:** o novo journal nasce com
    /// `last_flush_ms = 0`. Caller que migra de sintético (T1.9) pra
    /// wall-clock real (W11+) DEVE chamar `detach_journal` + `attach_journal`
    /// pra reset baseline, evitando flush storm.
    ///
    /// **Audit R-7 — sync fsync na UI thread:** T1.9 wira `commit_stroke`
    /// + `add_sample` DIRETO (sem worker thread). Mobile DEVE wire em
    /// worker thread via canal SPSC (carry-over W11).
    pub fn attach_journal(
        &mut self,
        journal_path: PathBuf,
        flush_policy: FlushPolicy,
    ) -> Result<(), JournalError> {
        // Q-8/R-1: refuse mid-stroke attach pra não deixar stroke órfão.
        if self.stroke_active || self.current_partial.is_some() {
            return Err(JournalError::AttachDuringActiveStroke);
        }
        // **S-6 baseline check:** ADR-0046 §2.2 "seq cresce sempre, replay
        // determinístico depende" + `set_next_seq` doc instrui caller passar
        // `last_persisted_seq + 1` antes de attach. Sem o gate, caller que
        // chame `load(canon)` → `attach_journal(...)` sem `set_next_seq` re-
        // pinta com seq=0 colidindo com strokes históricos.
        //
        // debug_assert apenas em dev; release segue (caller deg risk).
        debug_assert!(
            self.next_seq > 0 || self.stroke_history.is_empty(),
            "attach_journal: set_next_seq(last_persisted_seq + 1) obrigatório \
             ANTES quando stroke_history não-vazio — ADR-0046 §2.2 monotonicity \
             (audit T1.9 S-6)"
        );
        // Defensive: se já tinha journal, drop ele primeiro (libera lock).
        // Q-5/R-8: reset baseline ms naturalmente via journal novo.
        self.stroke_journal = None;
        // Q-3/R-3: limpa estado degradado em re-attach.
        self.last_wal_error = None;
        let journal = StrokeJournal::open(journal_path, flush_policy)?;
        self.stroke_journal = Some(journal);
        Ok(())
    }

    /// **S-4 (audit T1.9):** emit WAL `Heartbeat` entry pra distinguir
    /// "app crashou agora" de "WAL órfão de processo morto há semanas"
    /// (ADR-0052 §2.3). Caller (shell W11+) DEVE invocar a 1Hz idle
    /// (e.g., `Tool::on_tick` extension futuro) — sem heartbeat,
    /// `CrashRecovery::scan` boot trata stale journal como recoverable
    /// strokes velhos, mostrando UX prompt "Recover N strokes?" com
    /// strokes irrelevantes.
    ///
    /// `journal::StrokeJournal::heartbeat` faz rate-limit interno (1Hz);
    /// caller pode chamar mais frequentemente sem custo extra.
    ///
    /// Returns `true` se WAL heartbeat write failed (caller can poll
    /// [`Self::last_wal_error`] pro tipo de erro). Returns `false` quando
    /// journal não anexado OU sucesso. `JournalError` não impl `Clone`
    /// (postcard::Error ABI), por isso bool-return + borrow accessor.
    pub fn heartbeat_journal(&mut self, now_ms: u64) -> bool {
        if let Some(journal) = self.stroke_journal.as_mut() {
            if let Err(e) = journal.heartbeat(now_ms) {
                self.last_wal_error = Some(e);
                return true;
            }
        }
        false
    }

    /// **S-5 (audit T1.9):** check se WAL atingiu cap rotation (100 commits
    /// OR 50 MiB). Caller (shell W11+) consulta ANTES de chamar
    /// `rotate_journal` pra coordenar com canon save — rotate sem canon
    /// flush pode perder strokes não-persistidos. Carry-over W11: PainterTool
    /// implementar `rotate_journal_if_safe` que coordena com PaintProject.
    #[must_use]
    pub fn should_rotate_journal(&self) -> bool {
        self.stroke_journal
            .as_ref()
            .is_some_and(|j| j.should_rotate())
    }

    /// **W2.T2.1 — `PainterUiSnapshot` projection.** Snapshot read-only
    /// que o sidebar (`ph2d-panel-painter-sidebar`) pinta a cada frame
    /// (shell publica via `set_current_painter_snapshot` antes de paint).
    ///
    /// ADR-0043 §2.3 cap 18 fields; ADR-0040 TG-B unidirecional —
    /// snapshot é display projection, edits voltam via `apply_ui_edit`.
    #[must_use]
    pub fn ui_snapshot(&self) -> crate::params::PainterUiSnapshot {
        crate::params::PainterUiSnapshot {
            size01: ((self.params.size_px - 1.0) / (MAX_STAMP_SIZE_PX as f32 - 1.0))
                .clamp(0.0, 1.0),
            opacity01: self.params.opacity.clamp(0.0, 1.0),
            active_color: self.params.active_color,
            secondary_color: self.params.secondary_color,
            active_brush_thumb: crate::params::ThumbHandle::default(),
            active_brush_name: format!("brush_{}", self.params.active_brush.0),
            mode: self.params.mode,
            eyedropper_armed: self.params.eyedropper_armed,
            symmetry_enabled: self.params.symmetry.is_some(),
            undo_enabled: self.stroke_history.len() > 0,
            redo_enabled: false, // redo-stack é caller-side W11+ (vide T2.2)
            stroke_in_flight: self.stroke_active,
            takeover_active: self.params.takeover_active,
            active_layer_name: String::new(),
            active_layer_locked: false,
        }
    }

    /// **W2.T2.1 — `PainterUiEdit` dispatch.** Mapeia ADR-0043 §2.3
    /// sidebar edits para canon params. Caller (shell drains via
    /// `EditorAction::ToolPanelEvent` → `handle_panel_event`) NÃO
    /// re-implementa semântica — vive aqui pra single source of truth.
    ///
    /// **Anti-pattern (audit T1.6 R7 L1-4 / R8 M1-3):** caller que
    /// muta `params.active_brush` diretamente perde o brush runtime
    /// sync. Use `apply_ui_edit(PainterUiEdit::SelectBrush(h))` (que
    /// chama `set_brush` internamente).
    pub fn apply_ui_edit(&mut self, edit: crate::params::PainterUiEdit) {
        match edit {
            crate::params::PainterUiEdit::Size(v01) => {
                // Sidebar normalizado 0..1 → size_px 1.0..=MAX_STAMP_SIZE_PX (2048).
                let v = v01.clamp(0.0, 1.0);
                self.params.size_px = 1.0 + v * (MAX_STAMP_SIZE_PX as f32 - 1.0);
            }
            crate::params::PainterUiEdit::Opacity(v01) => {
                self.params.opacity = v01.clamp(0.0, 1.0);
            }
            crate::params::PainterUiEdit::SetColor(c) => {
                // Audit S-8: NaN guard happens em `begin_stroke`; aqui
                // só armazena (caller pode ter passado válido).
                self.params.active_color = c;
            }
            crate::params::PainterUiEdit::SelectBrush(_h) => {
                // T1.6 R7 L1-4 — `set_brush` exige Brush runtime alongside
                // handle. Sidebar W2 ainda não tem brush library lookup;
                // wire completo é W5 Brush Studio (carry-over T2.3).
            }
            crate::params::PainterUiEdit::ToggleBrushMode => {
                self.params.mode = crate::params::PainterMode::Brush;
            }
            crate::params::PainterUiEdit::ToggleSmudgeMode => {
                self.params.mode = crate::params::PainterMode::Smudge;
            }
            crate::params::PainterUiEdit::ToggleEraserMode => {
                self.params.mode = crate::params::PainterMode::Eraser;
            }
            crate::params::PainterUiEdit::ToggleEyedropper => {
                self.params.eyedropper_armed = !self.params.eyedropper_armed;
            }
            crate::params::PainterUiEdit::Undo => {
                // Pop stroke do history. Re-render canvas requer replay
                // engine (W11 T-replay full); W2.T2.2 ship com snapshot-
                // based undo (texture clone preserved).
                let _ = self.stroke_history.undo();
            }
            crate::params::PainterUiEdit::Redo => {
                // Idem Undo — redo precisa do undo-stack mantido side-car
                // (carry-over W11 T-replay).
            }
            crate::params::PainterUiEdit::ResetSidebar => {
                // Long-press reset (ADR-0043 §2.3). Restore defaults
                // EXCEPT history (preservado).
                let defaults = crate::params::PainterParams::default();
                self.params.size_px = defaults.size_px;
                self.params.opacity = defaults.opacity;
                self.params.active_color = defaults.active_color;
                self.params.secondary_color = defaults.secondary_color;
                self.params.mode = defaults.mode;
                self.params.eyedropper_armed = false;
                self.params.symmetry = None;
            }
            crate::params::PainterUiEdit::ToggleSymmetry => {
                self.params.symmetry = match self.params.symmetry {
                    None => Some(crate::params::SymmetryAxis::Vertical),
                    Some(_) => None,
                };
            }
            // OpenLayersPopover / OpenColorPopover / OpenBrushStudio são
            // affordances visuais geridas shell-side (não mutam tool state).
            _ => {}
        }
    }

    /// Desfaz o anexo de journal — drop libera advisory lock + registry.
    ///
    /// **Audit R-2:** se há stroke em-progresso quando detach acontece,
    /// emite `Cancel` no WAL ANTES do drop. Sem isso, o Begin permanecia
    /// órfão → próxima `CrashRecovery::scan` via re-attach false-positiva
    /// o stroke como `InProgressAtCrash`.
    pub fn detach_journal(&mut self) {
        if let Some(journal) = self.stroke_journal.as_mut()
            && journal.current_stroke.is_some()
        {
            let _ = journal.cancel_stroke();
        }
        self.stroke_journal = None;
    }

    /// Configura `CanvasId` deste PainterTool. Caller (shell W11+) invoca
    /// quando canvas ativo muda. Default = `CanvasId(0)` (single-canvas).
    ///
    /// **S-7 (audit T1.9):** ADR-0052 §2.6 state machine `[idle] ↔
    /// [stroke_active]` proíbe mutações cross-canvas mid-stroke. Sem
    /// guard, mudar canvas_id mid-stroke confunde recovery (filtra
    /// canvas antigo) + cross-canvas stroke pollution.
    pub fn set_canvas_id(&mut self, id: CanvasId) {
        debug_assert!(
            !self.stroke_active,
            "set_canvas_id called mid-stroke — ADR-0052 §2.6 lifecycle \
             violation (audit T1.9 S-7)"
        );
        self.canvas_id = id;
    }

    /// Configura `LayerId` alvo pra próximos strokes. W3 layers nasce ⇒
    /// caller wire conforme active layer selection.
    ///
    /// **S-7 (audit T1.9):** mid-stroke = lifecycle violation. Vide
    /// [`Self::set_canvas_id`] rationale.
    pub fn set_layer_target(&mut self, layer: LayerId) {
        debug_assert!(
            !self.stroke_active,
            "set_layer_target called mid-stroke — ADR-0052 §2.6 lifecycle \
             violation (audit T1.9 S-7)"
        );
        self.layer_target = layer;
    }

    /// Configura `next_seq` baseline pra anchor monotonic per-canvas.
    /// Caller invoca após `load(canon_bytes)` passando
    /// `paint_project.last_persisted_seq().map(|s| s + 1).unwrap_or(0)`
    /// — anti-replay defense via `CrashRecovery::scan_with_baseline`.
    ///
    /// **S-7 (audit T1.9):** mid-stroke override é silent footgun —
    /// current_partial já consumiu o next_seq antigo; override perdoa
    /// progress de monotonicity ADR-0046 §2.2.
    pub fn set_next_seq(&mut self, seq: u64) {
        debug_assert!(
            !self.stroke_active,
            "set_next_seq called mid-stroke — current_partial consumiu o \
             seq antigo; override viola ADR-0046 §2.2 monotonicity \
             (audit T1.9 S-7)"
        );
        self.next_seq = seq;
    }

    /// Read-only borrow do history (pra Reproject W12 / Inspector W14 /
    /// MCP W13 queries).
    #[must_use]
    pub fn stroke_history(&self) -> &StrokeHistory {
        &self.stroke_history
    }

    /// Take ownership do history (move-out — útil quando caller persiste
    /// canon + drop tool). Substitui por `StrokeHistory::default()`.
    pub fn take_stroke_history(&mut self) -> StrokeHistory {
        std::mem::take(&mut self.stroke_history)
    }

    /// **Audit Q-1 getter** — testes/inspectors querem checar buffer
    /// in-progress sem expor o `Vec` mutável.
    #[must_use]
    pub fn current_samples_len(&self) -> usize {
        self.current_samples.len()
    }

    /// **Audit Q-1 getter** — equivalente a `current_samples_len() == 0`,
    /// nome explícito pra assertions em tests.
    #[must_use]
    pub fn current_samples_is_empty(&self) -> bool {
        self.current_samples.is_empty()
    }

    /// **Audit Q-3/R-3 surface** — último erro WAL desde o último
    /// [`Self::attach_journal`]. `Some` = durability degraded (bridge W11
    /// deve emitir toast "Stroke history WAL unavailable — recent strokes
    /// in-memory only"). `None` após `attach_journal` bem-sucedido.
    #[must_use]
    pub fn last_wal_error(&self) -> Option<&JournalError> {
        self.last_wal_error.as_ref()
    }

    /// **Audit Q-3/R-3** — limpa o flag de degraded durability após o
    /// caller (bridge W11) ter mostrado o toast OU recuperado.
    pub fn clear_last_wal_error(&mut self) {
        self.last_wal_error = None;
    }

    /// Swap the active brush atomically with the public-contract
    /// `params.active_brush` handle. Guards against mid-stroke brush
    /// changes by calling `end_stroke` first (the scheduler's
    /// `residual_dist` / `stroke_rotation_base` / `last_follow_angle` /
    /// `stamp_index` carry brush-specific state — replaying them under
    /// a new brush's `spacing` / `shape_count` / `shape_rotation_follow`
    /// is undefined and produces visible seams or double-stamps at the
    /// switch boundary).
    ///
    /// **Audit T1.6 R7 L1-4:** `PainterParams.active_brush` (the
    /// `BrushHandle` published in ADR-0043 §2.3 §2.8 stub) was never
    /// read by `PainterTool` before this helper — `self.brush` was the
    /// sole runtime source of truth. A W2 sidebar implementer reading
    /// only the params surface (the published contract) would write a
    /// `SelectBrush(handle)` handler that updates the snapshot but
    /// leaves the rendered brush stale — silent regression with no
    /// gate to catch it. This helper makes the two writes inseparable
    /// AND adds the L1-6 end_stroke guard. `apply_ui_edit`'s future
    /// `PainterUiEdit::SelectBrush` handler MUST go through here.
    ///
    /// Calling this with `is_stroke_active() == true` emits a
    /// `debug_assert!` so the caller knows a stroke was silently
    /// dropped; release builds end the stroke without warning. The
    /// scheduler's `pool` (uncommitted stamps) is discarded along with
    /// the rest of `end_stroke` state — the caller is responsible for
    /// flushing visible stamps via `request_commit` BEFORE switching
    /// if mid-stroke flush is desired.
    ///
    /// **Audit T1.6 R8 M1-3 — `stroke_color_oklab` is intentionally
    /// NOT refreshed here.** That cache lives across `set_brush`
    /// because the color is part of `params.active_color`, not the
    /// brush, and brush-switch shouldn't mutate it. The unconditional
    /// `end_stroke()` above sets `stroke_active = false`, so the L1-5
    /// `debug_assert!` in `queue_pointer` will fire before the stale
    /// cache could be read by a stamp; the next `begin_stroke` then
    /// refreshes the cache from the current `params.active_color`.
    /// A future refactor that removes the `end_stroke()` call here
    /// would re-open the staleness window — keep it.
    pub fn set_brush(&mut self, handle: BrushHandle, brush: Brush) {
        debug_assert!(
            !self.stroke_active,
            "set_brush called mid-stroke — the scheduler residual state \
             carries brush-specific assumptions (spacing, rotation_base, \
             follow_angle); switching under it produces visible seams. \
             Audit R7 L1-6."
        );
        if self.stroke_active {
            self.end_stroke();
        }
        self.params.active_brush = handle;
        self.brush = brush;
        // R-5: brush mudou → hash cache stale.
        self.cached_brush_hash = None;
    }

    /// Public accessor for the active `BrushHandle`. Equivalent to
    /// `params.active_brush`, exposed as a function so future refactors
    /// (e.g., derive `self.brush` lazily from a library lookup keyed by
    /// handle — collapsing the two sources of truth) keep the public
    /// API stable. Audit T1.6 R7 L1-4.
    #[must_use]
    pub fn active_brush_handle(&self) -> BrushHandle {
        self.params.active_brush
    }

    /// Borrow the active runtime [`Brush`]. Audit T1.6 R8 P1-3.
    ///
    /// W2 sidebar widgets that need to populate shape / color-
    /// dynamics controls from the active brush state should call
    /// this — without it, every widget had to either store a
    /// duplicate copy of the brush or call back into a hardcoded
    /// `match handle → constructor()` block (the latter defeats the
    /// L1-4 / set_brush sync contract by re-introducing the manual
    /// handle-to-construction mapping).
    ///
    /// Companion to [`Self::active_brush_handle`]: the handle is the
    /// serializable identity, the brush is the runtime parameter
    /// vector. The two must stay in sync via [`Self::set_brush`].
    #[must_use]
    pub fn active_brush(&self) -> &Brush {
        &self.brush
    }

    /// "Brush lifted" — cursor saiu do footprint do sprite mid-drag.
    /// O próximo `queue_pointer` tratará o sample como ponto novo (sem
    /// interpolar uma linha reta no gap). Mantém o stroke ativo + o
    /// `stamp_index` counter. Audit T1.5 round 3 R3-LE-1.
    pub fn break_stroke_segment(&mut self) {
        if self.stroke_active {
            self.scheduler.break_segment();
        }
    }

    /// True iff um stroke está ativo (entre `begin_stroke` e `end_stroke`).
    #[must_use]
    pub fn is_stroke_active(&self) -> bool {
        self.stroke_active
    }

    /// Requisita commit (apply): bridge dispara `EditorAction::OneShotImageOp`
    /// no próximo frame, que aciona `run_full` para baking final.
    pub fn request_commit(&mut self) {
        self.pending_commit = true;
    }

    /// Tamanho efetivo do stamp em pixels, clampado ao limite ABI.
    fn effective_size_px(&self) -> f32 {
        self.params.size_px.clamp(1.0, MAX_STAMP_SIZE_PX as f32)
    }

    /// Dimensões do working canvas em pixels (`set_source` define;
    /// `deactivate` zera). Usado pelo input dispatch para mapear cursor
    /// screen-px → canvas-pixel coords.
    #[must_use]
    pub fn canvas_size(&self) -> (u32, u32) {
        self.source_size
    }

    /// **R4-LG-1 fast path:** zero-copy preview drain via Arc clone (1
    /// atomic increment). Drains `preview_dirty` and returns the SAME
    /// underlying `Arc<Vec<u8>>` the tool holds (no `to_vec`). The
    /// bridge stashes this Arc in its `painter_preview` cache; the
    /// canvas Vec stays shared between tool and bridge until the next
    /// `queue_pointer` triggers `Arc::make_mut`, at which point ONE
    /// `Vec::clone` happens.
    ///
    /// Prefer this over `RasterEditTool::current_preview` for the
    /// per-frame cache drain path (`drive_preview_cache` does
    /// `pixels.to_vec()` which at 60 fps × 16 MB is ~960 MB/s of
    /// allocator churn — audit R4-LG-1).
    #[must_use]
    pub fn take_preview_arc(&mut self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        if !std::mem::take(&mut self.preview_dirty) || self.canvas_rgba.is_empty() {
            return None;
        }
        Some((
            Arc::clone(&self.canvas_rgba),
            self.source_size.0,
            self.source_size.1,
        ))
    }

    /// True iff at least one stamp landed in `canvas_rgba` since the
    /// last `set_source`. Used by `drain_painter` (Apply) to skip the
    /// bake when the user clicked Apply without painting anything,
    /// avoiding a wasted Individual texture + no-op undo entry. Audit
    /// T1.5 round 3 R3-LF-5.
    #[must_use]
    pub fn has_painted_since_source(&self) -> bool {
        self.has_painted_since_source
    }

    /// Derive a deterministic, HR-5 cross-OS-stable `stroke_seed` from
    /// the canonical inputs of a pointer-down event.
    ///
    /// `canvas_px / canvas_py` are the position in CANVAS-pixel coords
    /// (not screen-px) so the seed is invariant under camera zoom/pan.
    /// `src_w / src_h` distinguish strokes on differently-sized sprites
    /// at the same logical canvas pixel. `entity_bits` distinguishes
    /// strokes on different sprites at the exact same dimensions.
    ///
    /// Mixer is a wyhash-style fold (3× multiply + xorshift) — same
    /// family as the scheduler's `det_random`; bit-identical across
    /// Mac/Linux/Windows. No dependency on `rand`'s `SmallRng` (whose
    /// seeding varies cross-platform).
    ///
    /// Audit T1.5 round 1 A-H4 + B-M3: canonical helper replaces the
    /// ad-hoc XOR formula previously inlined in `painter_input.rs`.
    #[must_use]
    pub fn derive_seed(
        canvas_px: f32,
        canvas_py: f32,
        src_w: u32,
        src_h: u32,
        entity_bits: u64,
    ) -> u64 {
        // Quantize canvas-px to u32 bits for stable hashing (NaN/Inf
        // canvas positions short-circuit upstream in `painter_pointer_uv`
        // but we defensively replace non-finite with zero here).
        let qx = if canvas_px.is_finite() {
            canvas_px.to_bits()
        } else {
            0
        };
        let qy = if canvas_py.is_finite() {
            canvas_py.to_bits()
        } else {
            0
        };
        let mut h = (qx as u64) | ((qy as u64) << 32);
        h ^= entity_bits;
        h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^= h >> 32;
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= (src_w as u64) | ((src_h as u64) << 32);
        h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 31;
        h
    }
}

/// Converte `OklchColor` (l, c, h em radianos, alpha) → OKLab `[L, a, b, α]`.
///
/// **Stub T1.5 placeholder:** quando `ph2d-color::OklchColor` ship em
/// T-color full (ADR-0051) e substituir o stub local de `params.rs`,
/// esta função some — `OklabColor` canon expõe `from_oklch()` direto. O
/// `h` é tratado como **radianos**. UI futura (W2 sidebar) que exponha
/// o color picker em degrees converte degrees→radians no
/// `PainterUiEdit::SetColor` handler ANTES de chamar `apply_ui_edit`.
///
/// Audit T1.5 round 1 A-M2 / B-M5: assert defensiva captura uso errado
/// (ex: alguém passa h em degrees > 2π).
///
/// **R4-LH-5 finite-guard:** if ANY component is non-finite (`pub params`
/// surface allows external writes that may inject NaN/Inf), returns
/// `[0, 0, 0, 0]` — `apply_stamps` filter chain treats alpha=0 as a
/// no-op so the stroke silently fails closed instead of corrupting
/// downstream math (NaN propagation through cos/sin → silent no-paint
/// AFTER all per-pixel work).
fn oklch_to_oklab(c: OklchColor) -> [f32; 4] {
    if !c.l.is_finite() || !c.c.is_finite() || !c.h.is_finite() || !c.a.is_finite() {
        return [0.0, 0.0, 0.0, 0.0];
    }
    // **Audit T1.6 R9 T1-4 — promote to production assert.** R7 used
    // `debug_assert!` which is silent in release builds; a caller
    // passing degrees instead of radians would silently produce
    // garbage colors with no diagnostic. Production assert fires
    // in BOTH debug AND release so the call site is surfaced
    // immediately the first time. The 4π upper bound (vs 2π) is
    // deliberate margin: angles that wrapped past one full rotation
    // due to accumulated arithmetic are still valid radians, but
    // anything ≥ 4π is almost certainly degrees (most degree values
    // exceed 4π ≈ 12.57).
    assert!(
        c.h.abs() <= (4.0 * std::f32::consts::PI),
        "oklch_to_oklab: expected h in RADIANS (|h| ≤ 4π for safety margin); \
         got {} (looks like degrees — convert with `degrees.to_radians()`)",
        c.h
    );
    let a = c.c * c.h.cos();
    let b = c.c * c.h.sin();
    [c.l, a, b, c.a]
}

impl Tool for PainterTool {
    fn id(&self) -> ToolId {
        ToolId::new("painter")
    }

    fn label(&self) -> &str {
        "Painter"
    }

    fn icon_slug(&self) -> &str {
        "painter"
    }

    fn build_panel(&self) -> FloatingPanel {
        // T1.1 stub: panel vazio com title só. Sidebar Procreate-style
        // ship em W2 (ph2d-panel-painter trait-driven; ADR-0029 padrão).
        FloatingPanel::new(self.id(), "Painter")
    }

    fn on_activate(&mut self) {
        // T1.2 implementará: pushar `painter_active = true` no HeroScreen
        // para acionar takeover (suprime chrome PH2D normal; ADR-0043 §1.1).
        self.params.takeover_active = true;
        // **R6-LN-1 doc honesty:** `println!` é um marker T1.2 smoke
        // temporário — remove quando o `tracing` crate joins the
        // workspace (ADR pendente). NÃO é "convenção canônica do
        // projeto" como a versão anterior do comentário afirmava: grep
        // confirmou que nenhum outro tool de produção usa `println!`
        // em runtime path. Painter é o único; é uma stub T1.5, não um
        // estabelecido pattern.
        println!("painter activated");
    }

    fn on_deactivate(&mut self) {
        self.params.takeover_active = false;
        // Audit T1.5 round 1 B-M2: full RasterEditTool teardown when the
        // registry switches tools — clears canvas_rgba + source_size +
        // dirty/commit flags too, not just stroke state. Without this,
        // re-activation runs against stale canvas from prior session and
        // the bridge's `last_painter_pushed_entity` reset (set None in
        // `painter_bridge::dispatch` inactive path) wouldn't be enough.
        <Self as RasterEditTool>::deactivate(self);
        // R6-LN-1: T1.2 smoke marker, temporário (vide on_activate).
        println!("painter deactivated");
    }

    fn handle_panel_event(&mut self, event: ph2d_editor_core::tool::PanelEvent) {
        // **W2.T2.1:** ADR-0040 TG-B canal genérico — sidebar emite
        // PanelEvent::SetValue(NodeId, f64) e PanelEvent::Activated(NodeId).
        // Routing pra PainterUiEdit semantic + apply_ui_edit single source
        // of truth (ADR-0043 §2.3).
        use crate::params::PainterUiEdit;
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::SetValue(id, v) if id == core_ids::PAINTER_SIDEBAR_SIZE_SLIDER => {
                self.apply_ui_edit(PainterUiEdit::Size(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == core_ids::PAINTER_SIDEBAR_SIZE_CHIP => {
                self.apply_ui_edit(PainterUiEdit::Size(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == core_ids::PAINTER_SIDEBAR_OPACITY_SLIDER => {
                self.apply_ui_edit(PainterUiEdit::Opacity(v as f32));
            }
            PanelEvent::SetValue(id, v) if id == core_ids::PAINTER_SIDEBAR_OPACITY_CHIP => {
                self.apply_ui_edit(PainterUiEdit::Opacity(v as f32));
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_SIDEBAR_UNDO_BUTTON => {
                self.apply_ui_edit(PainterUiEdit::Undo);
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_SIDEBAR_REDO_BUTTON => {
                self.apply_ui_edit(PainterUiEdit::Redo);
            }
            PanelEvent::Click(id) if id == core_ids::PAINTER_SIDEBAR_MODIFIER_SQUARE => {
                self.apply_ui_edit(PainterUiEdit::ToggleEyedropper);
            }
            _ => {}
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
        Some(self)
    }

    fn is_default(&self) -> bool {
        // Brush proto-tool (editor-core) continua default em W1. Quando
        // Painter substituir o proto (T1.X close W1), is_default() flipa
        // para true e brush proto é deletado.
        false
    }
}

impl RasterEditTool for PainterTool {
    /// Inicializa o working canvas a partir do source do sprite. RGBA8
    /// straight; é o estado base sobre o qual os stamps depositam.
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        // **R4-LH-3 fix:** production-grade length guard. A debug_assert
        // here lets a release-build caller pass a mismatched buffer that
        // then triggers undefined behavior in `apply_stamps` (or worse,
        // silently produces a corrupted commit through the Apply-without-
        // paint chain).
        assert_eq!(
            rgba.len(),
            (width as usize) * (height as usize) * 4,
            "set_source rgba length must equal width*height*4"
        );
        self.canvas_rgba = Arc::new(rgba);
        self.source_size = (width, height);
        self.preview_dirty = true;
        // R3-LF-5: reset "painted since source" — fresh source = clean
        // slate for the Apply-emptiness check.
        self.has_painted_since_source = false;
        // Source switched mid-stroke → encerra stroke pra não pintar no
        // canvas errado. Bridge garante ordem (source push antes de
        // queue_pointer) mas defesa-em-profundidade.
        self.end_stroke();
    }

    /// Devolve referência ao working canvas iff houve update desde a
    /// última call. Comportamento ADR-0041 — drena `preview_dirty`.
    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
        if !std::mem::take(&mut self.preview_dirty) || self.canvas_rgba.is_empty() {
            return None;
        }
        Some((&self.canvas_rgba, self.source_size.0, self.source_size.1))
    }

    fn take_pending_commit(&mut self) -> bool {
        std::mem::take(&mut self.pending_commit)
    }

    /// Bake final do canvas para commit. T1.5 = retorna o working canvas
    /// (`canvas_rgba`) inteiro, já com todos os stamps já depositados in-
    /// place via CPU path. Não toca state — bridge é responsável por
    /// chamar `deactivate` se for o ciclo de fim-de-tool.
    fn run_full(&mut self) -> (Vec<u8>, u32, u32) {
        // **R5-LI-C fix:** `mem::replace` takes the original Arc out
        // (leaving an empty stub) BEFORE `unwrap_or_clone` is called.
        // Without this, `Arc::clone(&self.canvas_rgba)` bumps the
        // refcount before unwrap sees it, forcing the clone branch
        // ALWAYS. With mem::replace, the bridge already dropped its
        // `painter_preview = None` in the Apply teardown path
        // (`painter_bridge.rs:202`), so the take-out arc has refcount=1
        // → `unwrap_or_clone` returns the inner Vec without allocation.
        // The stub `Arc::new(Vec::new())` is harmless: `deactivate`
        // immediately replaces it on the standard Apply teardown.
        let canvas = std::mem::replace(&mut self.canvas_rgba, Arc::new(Vec::new()));
        (
            Arc::unwrap_or_clone(canvas),
            self.source_size.0,
            self.source_size.1,
        )
    }

    fn deactivate(&mut self) {
        self.canvas_rgba = Arc::new(Vec::new());
        self.source_size = (0, 0);
        self.preview_dirty = false;
        self.pending_commit = false;
        self.scheduler.end_stroke();
        self.stroke_active = false;
        self.has_painted_since_source = false;
        // T1.9: drop journal (libera flock + in-process registry) +
        // descarta PartialStroke em-progresso. `stroke_history` é
        // PRESERVADO — caller decide se persiste no canon antes do
        // Tool::on_deactivate.
        //
        // **Audit R-2:** se há stroke ativo no WAL, cancela ANTES de drop
        // pra não deixar Begin órfão (recovery false-positivo
        // InProgressAtCrash). Same fix que `detach_journal`.
        if let Some(journal) = self.stroke_journal.as_mut()
            && journal.current_stroke.is_some()
        {
            let _ = journal.cancel_stroke();
        }
        self.stroke_journal = None;
        self.current_partial = None;
        self.current_samples.clear();
        self.last_wal_error = None;
    }
}

// =============================================================================
// T1.9 helpers (free fns) — conversões entre painter-brush + painter-stroke
// =============================================================================

/// Converte `PointerSample` (f32 raw input do scheduler) → `RawPointerSample`
/// (Q16.16/Q8.8 fixed-point cross-OS HR-5). Caller chama em `queue_pointer`
/// hot path; helpers `f32_to_q1616_saturating`/`f32_to_q88` são `#[inline]`
/// + no-alloc.
///
/// **Audit Q-6** — sentinel flags pra distinguir "valor 0 real" de "campo
/// não suportado pelo gerador". `PointerSample` (scheduler input) só
/// carrega position/pressure/tilt — azimuth/barrel/timestamp_delta nascem
/// só em `ph2d-painter-input` (T-input). Sem o flag, W12 Reproject + W13
/// MCP replay leem `azimuth_q88 == 0` como "azimuth = 0 rad" (não
/// "azimuth desconhecido") → brushes que rotacionam por azimuth produzem
/// arte determinística-mente DIFERENTE do stroke original.
fn pointer_to_raw_sample(sample: PointerSample) -> RawPointerSample {
    // **U-7 audit:** tilt flag setado quando sample.tilt == 0 (mouse/trackpad
    // path NÃO produz tilt). Pencil/stylus com tilt real seta tilt > 0 ⇒
    // flag NÃO setado. Wire T-input (W11+) override este heuristic com
    // device_source-aware decision (e.g., Mouse always unavailable, Pencil
    // always available).
    let mut flags = SAMPLE_FLAG_AZIMUTH_UNAVAILABLE
        | SAMPLE_FLAG_BARREL_ROLL_UNAVAILABLE
        | SAMPLE_FLAG_TIMESTAMP_UNAVAILABLE;
    if sample.tilt == 0.0 {
        flags |= SAMPLE_FLAG_TILT_UNAVAILABLE;
    }
    RawPointerSample {
        x_q1616: f32_to_q1616_saturating(sample.position[0]),
        y_q1616: f32_to_q1616_saturating(sample.position[1]),
        pressure_q88: f32_to_q88(sample.pressure),
        tilt_q88: f32_to_q88(sample.tilt),
        flags,
        ..Default::default()
    }
}

/// Converte `PainterTool` `OklchColor` (ph2d-tool-painter stub) → ph2d-color
/// `OklchColor` (canonical type). Painter mantém type local em params.rs
/// pra evitar dep transitive antes do T-color full (ADR-0051).
fn painter_color_to_stroke_oklch(c: OklchColor) -> StrokeOklchColor {
    StrokeOklchColor {
        l: c.l,
        c: c.c,
        h: c.h,
        a: c.a,
    }
}

/// Materializa `PartialStroke` (in-progress) + `Vec<RawPointerSample>`
/// (collected samples) num `StrokeRecord` (canon — push em StrokeHistory).
fn partial_to_record(partial: PartialStroke, samples: Vec<RawPointerSample>) -> StrokeRecord {
    StrokeRecord {
        uuid: partial.uuid,
        seq: partial.seq,
        timestamp_ms: partial.started_at_ms,
        brush_handle: partial.brush_handle,
        brush_params_hash: partial.brush_params_hash,
        layer_target: partial.layer_target,
        primary_color: partial.primary_color,
        secondary_color: partial.secondary_color,
        points: samples,
        rng_seed: partial.rng_seed,
        tool_mode: partial.tool_mode,
        version: partial.version,
    }
}

/// **Audit S-2 (T1.9 spec compliance) — `PainterMode → ToolMode` mapping.**
///
/// ADR-0043 §2.6.1 congelou cross-ADR: `Brush ↔ Paint`, `Smudge ↔ Smudge`,
/// `Eraser ↔ Erase`. Pré-S2, `begin_stroke` hardcoda `ToolMode::Paint` ⇒
/// strokes em modo Eraser/Smudge gravados como Paint no canon → Reproject
/// W12 pintava ao invés de apagar (catastrophic semantic bug latente).
fn painter_mode_to_tool_mode(mode: PainterMode) -> ToolMode {
    match mode {
        PainterMode::Brush => ToolMode::Paint,
        PainterMode::Smudge => ToolMode::Smudge,
        PainterMode::Eraser => ToolMode::Erase,
    }
}

/// **Audit S-8 (T1.9 spec compliance) — NaN/Inf guard pra OKLCH.**
///
/// `params.active_color` é `pub` field; bridge PCA pode escrever NaN/Inf
/// inadvertently (e.g., `OklchColor::lerp(a, b, t)` com t=NaN). Postcard
/// preserva bits → WAL + canon armazenam garbage; replay W12 + brush math
/// produz NaN propagation cascading.
///
/// Sanitize: se qualquer field não-finite, retorna OKLCH default
/// (achromatic black, alpha 1). Mesmo guard que `oklch_to_oklab` mas
/// aplica a TODA copy pro PartialStroke (não só ao cache OKLab local).
fn sanitize_oklch_or_default(c: OklchColor) -> StrokeOklchColor {
    if c.l.is_finite() && c.c.is_finite() && c.h.is_finite() && c.a.is_finite() {
        painter_color_to_stroke_oklch(c)
    } else {
        StrokeOklchColor {
            l: 0.0,
            c: 0.0,
            h: 0.0,
            a: 1.0,
        }
    }
}

/// **Audit Q-4** — converte `params::BrushHandle` (local stub, `pub u32`
/// sem clamp) → `ph2d_painter_brush::BrushHandle` (canon ADR-0044 §2.8,
/// `new_builtin` ASSERTA slot < 64 em release).
///
/// Sem o clamp explícito aqui, caller PCA escrevendo `params.active_brush =
/// BrushHandle(100)` (via `PainterUiEdit::SelectBrush` futuro com library
/// expandida) panicava o tool process no próximo `begin_stroke`. Fallback
/// pra default + eprintln preserva session.
fn brush_handle_stub_to_canon(stub: BrushHandle) -> ph2d_painter_brush::BrushHandle {
    let raw = stub.0;
    if raw & ph2d_painter_brush::BrushHandle::IMPORTED_FLAG != 0 {
        ph2d_painter_brush::BrushHandle::new_imported(
            raw & !ph2d_painter_brush::BrushHandle::IMPORTED_FLAG,
        )
    } else if raw < 64 {
        ph2d_painter_brush::BrushHandle::new_builtin(raw)
    } else {
        eprintln!(
            "[ph2d-painter] invalid built-in BrushHandle slot {raw} \
             (must be < 64 per ADR-0044 §2.8); falling back to default \
             (slot 0 = ROUND_HARD). Audit T1.9 Q-4."
        );
        ph2d_painter_brush::BrushHandle::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_source(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&rgba);
        }
        v
    }

    #[test]
    fn id_label_icon_slug_panel() {
        let t = PainterTool::default();
        assert_eq!(t.id(), ToolId::new("painter"));
        assert_eq!(t.label(), "Painter");
        assert_eq!(t.icon_slug(), "painter");
        let p = t.build_panel();
        assert_eq!(p.tool_id, ToolId::new("painter"));
    }

    #[test]
    fn activate_sets_takeover() {
        let mut t = PainterTool::default();
        assert!(!t.params.takeover_active);
        t.on_activate();
        assert!(t.params.takeover_active);
        t.on_deactivate();
        assert!(!t.params.takeover_active);
    }

    #[test]
    fn not_default_in_w1() {
        assert!(!PainterTool::default().is_default());
    }

    #[test]
    fn set_source_marks_dirty_and_drains() {
        let mut t = PainterTool::default();
        let src = flat_source(8, 8, [255, 255, 255, 255]);
        t.set_source(src.clone(), 8, 8);
        let (px, w, h) = t.current_preview().expect("dirty after set_source");
        assert_eq!((w, h), (8, 8));
        assert_eq!(px, src.as_slice());
        // Drained — next call returns None.
        assert!(t.current_preview().is_none());
    }

    #[test]
    fn deactivate_clears_canvas() {
        let mut t = PainterTool::default();
        t.set_source(flat_source(4, 4, [0; 4]), 4, 4);
        t.deactivate();
        assert!(t.current_preview().is_none());
        assert_eq!(t.source_size, (0, 0));
        assert!(t.canvas_rgba.is_empty());
        assert!(!t.is_stroke_active());
    }

    #[test]
    fn pending_commit_is_drained() {
        let mut t = PainterTool::default();
        assert!(!t.take_pending_commit());
        t.request_commit();
        assert!(t.take_pending_commit());
        assert!(!t.take_pending_commit(), "drained");
    }

    #[test]
    fn run_full_returns_canvas_clone() {
        let mut t = PainterTool::default();
        let src = flat_source(4, 4, [128, 64, 32, 255]);
        t.set_source(src.clone(), 4, 4);
        let (out, w, h) = t.run_full();
        assert_eq!((w, h), (4, 4));
        assert_eq!(out, src);
    }

    /// **Audit T1.6 R7 L1-5 contract update:** `queue_pointer` without
    /// an active stroke on a non-empty canvas is a `debug_assert!` in
    /// dev/test builds (surface bridge bugs) AND a silent no-op in
    /// release builds (preserve existing tolerance to dropped pointer-
    /// down events). The empty-canvas branch stays a silent no-op in
    /// all builds — that's the legitimate "tool inactive / no source
    /// yet" case the original test was protecting.
    #[test]
    fn queue_pointer_on_empty_canvas_is_silent_noop() {
        let mut t = PainterTool::default();
        // canvas_rgba is empty by default (no set_source called).
        t.queue_pointer(PointerSample {
            position: [4.0, 4.0],
            pressure: 1.0,
            tilt: 0.0,
        });
        // No source, no panic, no preview state mutated.
        assert!(t.current_preview().is_none());
    }

    /// **Audit T1.6 R7 L1-5:** the debug_assert fires in dev/test
    /// builds when `queue_pointer` is invoked without a matching
    /// `begin_stroke` on a non-empty canvas — the previous silent no-op
    /// was masking bridge bugs (a pointer-move handler that ran
    /// without its pointer-down) and the subsequent `drain_painter`
    /// would emit a "no strokes to apply" toast indistinguishable
    /// from "Apply ran with no paint".
    /// **Audit T1.6 R8 N1-5 — `set_brush` keeps the handle and the
    /// runtime brush in sync.** The dual-source-of-truth was the bug
    /// L1-4 fixed; without a gate, a future refactor that drops one
    /// of the two writes would silently break the contract.
    ///
    /// Note: `set_brush` takes `params::BrushHandle` (the local stub
    /// from `params.rs`); `ph2d_painter_brush::OVAL_HARD` is the
    /// canon `ph2d_painter_brush::BrushHandle`. HR-14 forward-compat
    /// keeps them structurally identical (`pub struct
    /// BrushHandle(u32)`) but they're distinct types; bridge via
    /// the inner u32.
    #[test]
    fn set_brush_writes_both_handle_and_runtime_brush() {
        use ph2d_painter_brush::library;
        let mut t = PainterTool::default();
        let new_handle = crate::params::BrushHandle(ph2d_painter_brush::OVAL_HARD_SLOT);
        t.set_brush(new_handle, library::oval_hard());
        assert_eq!(
            t.active_brush_handle(),
            new_handle,
            "params.active_brush must be the new handle"
        );
        // The brush itself must be the runtime oval (we verify via the
        // shape source slot which is publicly observable).
        let brush_ref = t.active_brush();
        match &brush_ref.shape.shape_source {
            ph2d_painter_brush::shape::ShapeSource::Builtin { atlas_layer, .. } => {
                assert_eq!(
                    *atlas_layer,
                    ph2d_painter_brush::OVAL_HARD_SLOT,
                    "self.brush.shape_source slot must match the handle"
                );
            }
            _ => panic!("expected Builtin shape source after set_brush"),
        }
    }

    /// **Audit T1.6 R8 N1-5 — `set_brush` mid-stroke fires the
    /// `debug_assert!` (L1-6 R7 contract).** The release-build
    /// state-preservation path is exercised separately by
    /// `set_brush_after_end_stroke_swaps_cleanly` below.
    #[test]
    #[should_panic(expected = "set_brush called mid-stroke")]
    fn set_brush_mid_stroke_panics_in_debug() {
        use ph2d_painter_brush::library;
        let mut t = PainterTool::default();
        t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
        t.begin_stroke(7);
        let new_handle = crate::params::BrushHandle(ph2d_painter_brush::OVAL_HARD_SLOT);
        t.set_brush(new_handle, library::oval_hard());
    }

    /// **Audit T1.6 R8 N1-5 — `set_brush` after a clean
    /// `end_stroke` swaps both handle and runtime brush cleanly.**
    /// Mirrors the canonical W2 sidebar flow: user finishes a
    /// stroke, sidebar issues `SelectBrush(handle)`, the handler
    /// calls `set_brush(handle, library::brush_from_handle(handle))`.
    /// This is the happy path that proves the L1-4 sync invariant.
    #[test]
    fn set_brush_after_end_stroke_swaps_cleanly() {
        use ph2d_painter_brush::library;
        let mut t = PainterTool::default();
        t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
        t.begin_stroke(7);
        t.end_stroke();
        assert!(!t.is_stroke_active());
        let new_handle = crate::params::BrushHandle(ph2d_painter_brush::OVAL_HARD_SLOT);
        t.set_brush(new_handle, library::oval_hard());
        assert!(!t.is_stroke_active(), "stroke must remain closed");
        assert_eq!(t.active_brush_handle(), new_handle);
        // Verify the runtime brush swapped to oval (slot 3).
        match &t.active_brush().shape.shape_source {
            ph2d_painter_brush::shape::ShapeSource::Builtin { atlas_layer, .. } => {
                assert_eq!(*atlas_layer, ph2d_painter_brush::OVAL_HARD_SLOT);
            }
            _ => panic!("expected Builtin oval after set_brush"),
        }
    }

    /// **Audit T1.6 R8 N1-5 — `active_brush()` borrows the runtime
    /// brush.** Pinned so removing the accessor (P1-3) becomes
    /// compile-time observable.
    #[test]
    fn active_brush_returns_runtime_brush() {
        let t = PainterTool::default();
        // Default is round_hard. Verify atlas_layer is 0.
        let brush = t.active_brush();
        match &brush.shape.shape_source {
            ph2d_painter_brush::shape::ShapeSource::Builtin { atlas_layer, .. } => {
                assert_eq!(*atlas_layer, 0, "default brush is round_hard slot 0");
            }
            _ => panic!("default brush must use Builtin shape source"),
        }
    }

    #[test]
    #[should_panic(expected = "queue_pointer called without an active stroke")]
    fn queue_pointer_without_stroke_on_loaded_canvas_panics_in_debug() {
        let mut t = PainterTool::default();
        t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
        let _ = t.current_preview(); // drain set_source dirty
        t.queue_pointer(PointerSample {
            position: [4.0, 4.0],
            pressure: 1.0,
            tilt: 0.0,
        });
    }

    #[test]
    fn stroke_writes_pixels() {
        // The Day-7 smoke in unit-test form.
        let mut t = PainterTool::default();
        // Non-zero color (OklchColor default is all zeros == OKLab black,
        // alpha 0 → no visible paint). Set red-ish color.
        t.params.active_color = crate::params::OklchColor {
            l: 0.6,
            c: 0.2,
            h: 0.5,
            a: 1.0,
        };
        t.params.size_px = 16.0;
        t.set_source(flat_source(32, 32, [0, 0, 0, 255]), 32, 32);
        let _ = t.current_preview(); // drain set_source dirty
        t.begin_stroke(42);
        t.queue_pointer(PointerSample {
            position: [16.0, 16.0],
            pressure: 1.0,
            tilt: 0.0,
        });
        let (px, w, h) = t.current_preview().expect("paint must mark dirty");
        assert_eq!((w, h), (32, 32));
        // Center pixel should now be different from the initial black.
        let center_idx = (16 * 32 + 16) * 4;
        assert_ne!(
            &px[center_idx..center_idx + 4],
            &[0u8, 0, 0, 255],
            "stamp must overwrite center pixel"
        );
        t.end_stroke();
        assert!(!t.is_stroke_active());
    }

    #[test]
    fn begin_stroke_implicitly_closes_previous() {
        let mut t = PainterTool::default();
        t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
        t.begin_stroke(1);
        assert!(t.is_stroke_active());
        // Begin again without end: defensive cleanup.
        t.begin_stroke(2);
        assert!(t.is_stroke_active());
        t.end_stroke();
        assert!(!t.is_stroke_active());
    }

    #[test]
    fn set_source_ends_active_stroke() {
        let mut t = PainterTool::default();
        t.set_source(flat_source(8, 8, [0; 4]), 8, 8);
        t.begin_stroke(1);
        assert!(t.is_stroke_active());
        // Source push mid-stroke must close it (defensive — bridge guarantees
        // order but this layer is honest).
        t.set_source(flat_source(8, 8, [255; 4]), 8, 8);
        assert!(!t.is_stroke_active());
    }

    // ──────────────────────────────────────────────────────────────────────
    // Round-2 audit fixes — closing verbal-claim gaps with executable gates.
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn on_deactivate_clears_canvas_via_tool_dispatch() {
        // Audit T1.5 round 2 F4 (MISSING-GATE-DEACTIVATE-CHAIN). The fix
        // (B-M2) routes registry-side teardown through `Tool::on_deactivate
        // → RasterEditTool::deactivate`. This test invokes the chain
        // through the Tool trait (the path `ToolRegistry::set_active`
        // takes), NOT via `RasterEditTool::deactivate` directly — proves
        // the dispatch wiring, not just the leaf.
        let mut t = PainterTool::default();
        t.set_source(flat_source(4, 4, [128; 4]), 4, 4);
        t.begin_stroke(1);
        assert!(t.is_stroke_active());
        <PainterTool as Tool>::on_deactivate(&mut t);
        assert!(!t.params.takeover_active);
        assert_eq!(t.source_size, (0, 0));
        assert!(t.canvas_rgba.is_empty());
        assert!(!t.is_stroke_active());
    }

    #[test]
    fn stroke_writes_pixels_with_default_color() {
        // Audit T1.5 round 2 F5 (MISSING-GATE-DEFAULT-ALPHA). Day-7
        // smoke contract: `PainterTool::default()` (NO override of
        // `params.active_color`) must produce visible paint via the
        // input-dispatch entry. If a future regression resets
        // `OklchColor::default().a` to 0 in `PainterParams::default`,
        // this test catches it.
        let mut t = PainterTool::default();
        // DELIBERATELY do NOT override `params.active_color`.
        t.params.size_px = 16.0;
        t.set_source(flat_source(32, 32, [0, 0, 0, 255]), 32, 32);
        let _ = t.current_preview(); // drain set_source dirty
        t.begin_stroke(42);
        t.queue_pointer(PointerSample {
            position: [16.0, 16.0],
            pressure: 1.0,
            tilt: 0.0,
        });
        let (px, _w, _h) = t.current_preview().expect("paint must mark dirty");
        let center_idx = (16 * 32 + 16) * 4;
        // The default OklchColor is OKLab black (l=c=h=0) at α=1, so the
        // expected output at center is OPAQUE BLACK [0,0,0,255]. Source
        // started at [0,0,0,255] so the SPECIFIC ASSERT here is that
        // alpha stays at 255 AND the stamp didn't no-op (set_source +
        // queue_pointer succeeded — preview returned Some). That last
        // bit is the real Day-7 marker; opaque-on-opaque produces same
        // bytes but the path executed.
        assert_eq!(px[center_idx + 3], 255, "alpha must remain opaque");
    }

    #[test]
    fn derive_seed_determinism_and_collision_resistance() {
        // Audit T1.5 round 2 F1 (MISSING-GATE-DERIVE-SEED). Locks the
        // wyhash-style mixer contract: bit-identical across runs (the
        // function uses only `to_bits`, `wrapping_mul`, `xor`, and bit-
        // shifts — all platform-stable IEEE 754 / integer ops), and
        // distinct inputs produce distinct seeds (anti-collision).
        //
        // Determinism: same inputs must produce same outputs across
        // consecutive calls. This bounds the implementation against
        // future-edits that introduce platform-specific behavior.
        for &(px, py, sw, sh, eb) in &[
            (0.0_f32, 0.0_f32, 256_u32, 256_u32, 0_u64),
            (10.0, 20.0, 256, 256, 0),
            (10.0, 20.0, 256, 256, 1),
        ] {
            assert_eq!(
                PainterTool::derive_seed(px, py, sw, sh, eb),
                PainterTool::derive_seed(px, py, sw, sh, eb),
                "derive_seed must be deterministic ({px}, {py}, {sw}, {sh}, {eb})"
            );
        }

        // Anti-collision: different inputs must produce different seeds
        // (modulo PRNG collisions — pick well-separated inputs).
        let a = PainterTool::derive_seed(10.0, 20.0, 256, 256, 0);
        let b = PainterTool::derive_seed(10.0, 20.0, 256, 256, 1);
        let c = PainterTool::derive_seed(11.0, 20.0, 256, 256, 0);
        let d = PainterTool::derive_seed(10.0, 20.0, 257, 256, 0);
        assert_ne!(a, b, "entity_bits must distinguish seeds");
        assert_ne!(a, c, "canvas_px must distinguish seeds");
        assert_ne!(a, d, "src_w must distinguish seeds");

        // Non-finite canvas-px canonicalization: NaN / +Inf must produce
        // SAME seed as 0.0 (NOT a unique non-finite hash).
        assert_eq!(
            PainterTool::derive_seed(f32::NAN, 0.0, 1, 1, 0),
            PainterTool::derive_seed(0.0, 0.0, 1, 1, 0),
            "NaN canvas_px must canonicalize to 0.0",
        );
        assert_eq!(
            PainterTool::derive_seed(0.0, f32::INFINITY, 1, 1, 0),
            PainterTool::derive_seed(0.0, 0.0, 1, 1, 0),
            "+Inf canvas_py must canonicalize to 0.0",
        );
    }

    #[test]
    #[should_panic(expected = "RADIANS")]
    fn oklch_to_oklab_panics_on_degrees_input() {
        // Audit T1.5 round 2 F8 + R9 T1-4: assertion is now a production
        // `assert!` (not `debug_assert!`), so it fires in BOTH debug and
        // release builds. Degree inputs (h = 360.0 > 4π ≈ 12.566) are
        // surfaced immediately at the call site instead of silently
        // producing garbage colors.
        let c = OklchColor {
            l: 0.5,
            c: 0.2,
            h: 360.0,
            a: 1.0,
        };
        let _ = oklch_to_oklab(c);
    }
}
