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
    Brush, BrushParamsHash, MAX_STAMP_SIZE_PX, PointerSample, Stamp, StampScheduler,
    apply_stamps_with_options, library,
};
use ph2d_painter_stroke::{
    CanvasId, FlushPolicy, JournalError, LayerId, PartialStroke, RawPointerSample,
    SAMPLE_FLAG_AZIMUTH_UNAVAILABLE, SAMPLE_FLAG_BARREL_ROLL_UNAVAILABLE,
    SAMPLE_FLAG_TILT_UNAVAILABLE, SAMPLE_FLAG_TIMESTAMP_UNAVAILABLE, StrokeHistory, StrokeJournal,
    StrokeRecord, ToolMode, f32_to_q88, f32_to_q1616_checked,
};

use crate::compositor::{
    CompositorCache, LayerImage, LayerPixelSource, Region, composite, composite_region,
    composite_with_cache,
};
use crate::layers::{LayerId as RtLayerId, LayerKind, LayerStack};
use crate::params::{BrushHandle, OklchColor, PainterMode, PainterParams};
use ph2d_painter_brush::BlendMode;
use std::collections::BTreeMap;

// Marker for the empty-apply guard (R3-LF-5): a stamp was actually
// deposited since the last `set_source`. `drain_painter` early-returns
// when this is false to avoid wasting a new Individual texture + undo
// snapshot on identity-baking the source pixels.

thread_local! {
    // Pending Cmd/Shift modifiers for layer-row selects, KEYED BY the row's
    // widget NodeId. Written by the layers panel apply_event (the only place
    // that can read host.store().cmd_held / shift_held) right before it forwards
    // a row Click, and read + removed by handle_panel_event keyed by the SAME
    // click NodeId. The frozen PanelEvent (4 variants) cannot carry the bits and
    // handle_panel_event gets no store, so this side channel bridges the gap.
    //
    // Keyed by NodeId (not a single slot) on purpose: the panel writes
    // SYNCHRONOUSLY during apply_event, but the matching ToolPanelEvent::Click
    // is queued on the action bus and drained later, once per frame. The bus is
    // a FIFO that can hold MULTIPLE row clicks in one drain batch; a single
    // shared slot would let a later click's modifiers clobber an earlier,
    // still-undrained click. Keying by the click's own NodeId keeps each click's
    // modifiers with it. Bounded by the rows clicked between drains; entries are
    // removed on consume (a rare failed-decode leaves one stale entry, harmless
    // and overwritten on the next click of that exact row).
    static PENDING_SELECT_MODS:
        std::cell::RefCell<std::collections::BTreeMap<ph2d_a11y::NodeId, (bool, bool)>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

/// Stash the Cmd/Ctrl + Shift state for the row-select click on `row_id`.
/// Called by the layers panel `apply_event` immediately before it forwards that
/// row's `Click` (it is the only side with `host.store()` access to the live
/// modifier state). Consumed by [`PainterTool::handle_panel_event`] on the
/// matching click. See `PENDING_SELECT_MODS` for why this is keyed by id.
pub fn set_pending_select_mods(row_id: ph2d_a11y::NodeId, cmd: bool, shift: bool) {
    PENDING_SELECT_MODS.with(|m| {
        m.borrow_mut().insert(row_id, (cmd, shift));
    });
}

/// Take + remove the pending modifiers for `row_id` (default = no modifiers).
fn take_pending_select_mods(row_id: ph2d_a11y::NodeId) -> (bool, bool) {
    PENDING_SELECT_MODS.with(|m| m.borrow_mut().remove(&row_id).unwrap_or((false, false)))
}

/// Per-frame inputs the shell GPU fluid drive (W15.3) pulls from the tool to step
/// the resident solver + composite, without reading the GPU pigment back. The
/// `deposit` is THIS frame's dabs (the grid pigment, then cleared); `water` is the
/// CPU water mirror (uploaded for the gate/flow, evaporated CPU-side afterwards);
/// `region` is the wet grid-cell bbox (union with the previous frame) scoping the
/// composite; `dims` is the grid size.
pub struct FluidFrameInputs {
    /// This frame's dab pigment (low-res `gw*gh`, xyz mass, w=0) — added to `pig_a`.
    pub deposit: Vec<[f32; 4]>,
    /// The CPU water mirror (low-res `gw*gh`) — uploaded for the GPU gate/flow.
    pub water: Vec<f32>,
    /// Wet grid-cell bbox (inclusive) ∪ last frame's — the composite region.
    pub region: (u32, u32, u32, u32),
    /// Grid (low-res) dimensions `(gw, gh)`.
    pub dims: (u32, u32),
}

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
    /// **W3.T3.1/T3.2 — layer model (runtime canon, ADR-0046-amд-1 Option A).**
    /// The `LayerStack` is the source of truth for layer structure + per-layer
    /// blend/opacity/visibility/flags. The ACTIVE layer's working pixels live
    /// in `canvas_rgba` (the Arc zero-copy preview buffer strokes mutate);
    /// non-active layers' pixels live in `images`. For the common single-layer
    /// case the stack is "trivial" and `current_preview` returns `canvas_rgba`
    /// byte-for-byte (no composite) — preserving the T1.5 fast path exactly.
    layers: LayerStack,
    /// Per-(non-active-)layer pixel buffers (canvas-sized straight sRGB8).
    /// The active layer is NOT stored here — it's `canvas_rgba`. `BTreeMap`
    /// per HR-5 (deterministic iteration even in `PresentWorld`).
    images: BTreeMap<RtLayerId, LayerImage>,
    /// Cached composite output (non-trivial stacks only), so `current_preview`
    /// can hand back a stable `&[u8]` and `take_preview_arc` can hand back an
    /// `Arc::clone` (no full-canvas copy). Stored behind an `Arc` so the bridge
    /// drain is zero-copy; the dirty-rect blit mutates in place via
    /// `Arc::make_mut` (clones once only if the bridge still holds the prior
    /// Arc). Invalidated (`None`) on any layer edit.
    composited: Option<Arc<Vec<u8>>>,
    /// The dirty bbox `(x, y, w, h)` the LAST `take_preview_arc` recomposed —
    /// `Some` iff that drain took the partial fast lane (a stroke into a valid
    /// cache), `None` for a full composite (trivial stack / full recompose /
    /// post-structural-edit). The bridge reads this via `take_preview_upload_bbox`
    /// to upload only the sub-rect (B.1) instead of the full GPU texture. Reset
    /// on every drain, so a `None`-returning (not-dirty) drain leaves it stale —
    /// harmless, the bridge only reads it right after a `Some` drain.
    preview_upload_bbox: Option<Region>,
    /// Monotonic counter bumped on every change to the PUBLISHED layer structure
    /// (add / select / reorder / visibility / opacity / blend / source reset) —
    /// i.e. exactly the `invalidate_composite` chokepoint plus `set_source`. The
    /// bridge reads `layers_revision()` to publish the `LayerStack` snapshot ONLY
    /// when it changed (B.5), instead of deep-cloning it every frame. Strokes do
    /// NOT bump it (pixels aren't reflected in the panel structure), which is the
    /// whole point — no republish mid-paint.
    layers_revision: u64,
    /// Per-layer pixel CONTENT version for the GPU preview compositor
    /// (`ph2d_render::LayerPixelProvider`, consumed via the shell bridge's
    /// `preview_layer_pixels`). A layer's entry bumps to the next `pixel_clock`
    /// value whenever its PIXELS change — a stroke stamp, undo/redo, mask
    /// flatten, or a fresh `set_source` — and ONLY then. Metadata edits
    /// (opacity / blend / visibility / adjustment params) funnel through
    /// `invalidate_composite`, which does NOT touch this, so the GPU compositor
    /// keeps a layer's texture-array slice cached across an adjustment-slider
    /// drag (zero pixels changed → zero re-upload → pure GPU recompute, the
    /// whole point of the GPU path). `BTreeMap` per HR-5 (deterministic order).
    layer_pixel_versions: BTreeMap<RtLayerId, u64>,
    /// Monotonic source for [`Self::layer_pixel_versions`] bumps. Never reset
    /// (not even on `set_source`), so a layer key REUSED across a source swap
    /// (the `LayerStack` restarts ids at 1) always gets a strictly-greater
    /// version than the compositor cached for the previous image → no stale
    /// slice. A never-seen layer's default `0` differs from any real bump.
    pixel_clock: u64,
    /// The brush color saved when entering mask-edit, restored on leaving. A
    /// mask starts WHITE (all visible) and the expected action is to HIDE, so
    /// editing a mask defaults the brush to BLACK; this remembers the user's
    /// real color so it comes back when they return to a normal layer.
    color_before_mask: Option<OklchColor>,
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
    // === W2.T2.2 fields — snapshot-based undo/redo (ADR-0046 §2.6) ===
    /// Snapshot-based undo/redo over the layer texture. Owns the *pixels* the
    /// user steps back/forward through; `stroke_history` owns the parallel
    /// *semantic* records. Driven at stroke boundaries (vide [`crate::undo`]).
    undo: crate::undo::UndoController,
    /// Layer texture captured at `begin_stroke` (the pre-stroke pre-image).
    /// Handed to [`crate::undo::UndoController::record_pre_stroke`] at commit
    /// (non-empty `end_stroke`); discarded on the empty-stroke recycle path so
    /// a no-paint gesture never pollutes the undo stack.
    pending_pre_stroke: Option<Vec<u8>>,
    /// Redo branch of *semantic* records. `stroke_history.undo()` returns the
    /// popped [`StrokeRecord`]; `stroke_history.redo(record)` needs it back, so
    /// undo parks it here and redo replays it. Kept parallel (same LIFO order)
    /// to the texture redo stack inside [`crate::undo::UndoController`], and
    /// cleared at the same boundary: a NEW committed stroke clears BOTH (the
    /// controller's redo stack in `record_pre_stroke`, this Vec in
    /// `end_stroke`) so a later redo can never resurrect a record from a
    /// discarded branch. (See `new_stroke_after_undo_invalidates_redo`.)
    undo_redo_records: Vec<StrokeRecord>,
    /// **W3.T3.4 dock toggle (mode C):** which painter panel occupies the
    /// shared right-dock slot — `false` = brush sidebar, `true` = layers panel.
    /// Toggled via `handle_panel_event` (either panel's header toggle button);
    /// the shell `painter_bridge` reads this (downcast) to drive
    /// `panel_visibility` for `painter_sidebar` / `painter_layers`. Lives on the
    /// tool (not a panel) so both panels + the shell agree without a
    /// panel→panel dependency (`architecture_cycle_prevention` forbids it).
    dock_shows_layers: bool,
    /// **W5 Brush Studio (mode C, third dock state):** when `true` the shared
    /// right-dock slot shows the Brush Studio (`ph2d-panel-brush-studio`) instead
    /// of the brush sidebar / layers panel. Flipped via `PainterUiEdit::
    /// OpenBrushStudio` (sidebar header button) / `close_brush_studio` (panel X).
    /// Read by the shell `painter_bridge` (downcast) to drive `panel_visibility`;
    /// lives on the tool (not a panel) so all three panels + the shell agree
    /// without a panel→panel dependency (mirror of [`Self::dock_shows_layers`]).
    show_brush_studio: bool,
    /// **W3 perf — dirty-rect preview.** Accumulated bbox (canvas px) of stamps
    /// deposited since the last preview drain. When set AND a full composite is
    /// cached (`composited`), `take_preview_arc` recomposites ONLY this region
    /// (`composite_region`) and blits it into the cache — O(N×bbox) instead of
    /// O(N×W×H) per stroke frame. Cleared by `invalidate_composite` (a
    /// structural edit forces a full recompose) and consumed each drain.
    dirty_rect: Option<Region>,
    /// **W3 multi-selection.** The set of layer rows highlighted in the panel.
    /// A plain row click collapses it to one layer; Cmd/Ctrl-click toggles a
    /// member; Shift-click selects a contiguous run along the visible row
    /// order; `group_selected` wraps the whole set in a new group. The active
    /// (paint-target) layer is conceptually always a member — `selection()`
    /// folds it in for the panel highlight publish. The authoritative copy
    /// lives here (the tool owns layer structure); the panel renders a per-
    /// frame snapshot (bridge `set_current_selection`). Masks are never
    /// members (owner-attached, not in the z-order run).
    selection: std::collections::BTreeSet<RtLayerId>,
    /// **W5 cut-point cache (ADR-0045 §2.7).** Caches the composite-below each
    /// root adjustment so a slider-drag on an adjustment param restarts from the
    /// cut instead of recomposing the whole stack every frame (the dominant
    /// preview cost — base recompose ~15 ms @1024²). Populated/consumed ONLY via
    /// `composite_with_cache` on the `adjustment_cache_pending` path; ANY other
    /// composite (stroke fast-lane / cold full / structural `invalidate_composite`)
    /// drops the cuts, since they mutate below-layers without updating the cut.
    compositor_cache: CompositorCache,
    /// Set by `set_adjustment_param` so the next `take_preview_arc` full recompose
    /// routes through the cut-point cache (`composite_with_cache`) instead of a
    /// cold `composite`. Honoured only when no stroke also dirtied the frame;
    /// taken on consume.
    adjustment_cache_pending: bool,
    /// **W5 Mixbox wash — per-pixel stroke coverage.** Allocated (zeroed) at
    /// `begin_stroke` ONLY for pigment (Mixbox) brushes; `None` for the normal
    /// per-dab path. Each entry ∈[0,1] is the fraction of THIS stroke's pigment
    /// deposited at that pixel, built monotonically from `flow`×shape. The wash
    /// render ([`ph2d_painter_brush::apply_stamps_wash`]) composites the pixel
    /// from the pre-stroke backdrop (`pending_pre_stroke`) at `opacity·coverage`,
    /// so overlapping dabs inside one stroke stay a *stable mix* (yellow over
    /// blue = green) instead of building up to pure brush colour. Cleared at
    /// `end_stroke` / reset.
    wash_coverage: Option<Vec<f32>>,
    /// **W5 wash colour accumulation (Color Dynamics smoothness).** Per-pixel
    /// coverage-weighted average of the dab COLOURS deposited at that pixel this
    /// stroke (straight linear RGB). Parallel to `wash_coverage`. With Color
    /// Dynamics jitter, consecutive dabs carry different colours; mixing only the
    /// LAST dab's colour against the backdrop made overlapping dabs read as
    /// discrete coloured discs ("resolution drop" report). Averaging the colours
    /// here lets the jittered dabs blend into a smooth gradient. For a single-
    /// colour brush every entry equals that colour, so the render is byte-
    /// identical (the wash uses the exact stamp colour when the two agree).
    wash_color: Option<Vec<[f32; 3]>>,
    /// Opacity cap for the active wash stroke, snapshotted from `params.opacity`
    /// at `begin_stroke` so a mid-stroke UI change can't destabilise coverage.
    /// Meaningful only while `wash_coverage` is `Some`.
    wash_opacity_cap: f32,
    /// **W15 — live watercolor wet-on-wet field (ADR-0049 / ADR-0077 D11).** A
    /// low-resolution [`ph2d_painter_brush::diffusion::DiffusionGrid`], allocated at
    /// `begin_stroke` only when `brush.rendering.fluid_enabled`. Stamps splat into
    /// it (re-wet + pigment); `on_tick` steps the diffusion every frame and
    /// composites it over `pending_pre_stroke` into the canvas, so the wash keeps
    /// blooming + drying AFTER pen-up. Dropped when the field dries out (water → 0).
    /// `None` for every non-fluid brush ⇒ zero behaviour change + zero cost.
    wet_field: Option<ph2d_painter_brush::diffusion::DiffusionGrid>,
    /// **W15 — backdrop the live wet field composites OVER.** Snapshot of the
    /// canvas as it was when the fluid stroke began (the pre-stroke pixels). Kept
    /// SEPARATE from `pending_pre_stroke`: that one is consumed by the undo stack
    /// at `end_stroke` (`take()`), but the wash keeps blooming for many frames
    /// AFTER pen-up, so `composite_wet_field` needs a backdrop that outlives the
    /// stroke. Allocated with `wet_field` at `begin_stroke`; dropped in lock-step
    /// whenever the field is (dry-out / undo / redo / source swap). `None` ⇒ no
    /// live wash, so the composite no-ops.
    wet_backdrop: Option<Vec<u8>>,
    /// **W15 — last frame's wet bbox (grid cells, inclusive).** The composite runs
    /// only over the union of the current + previous wet region, so it touches the
    /// wash neighbourhood instead of the whole canvas (the 16-tap bicubic + K–M is
    /// the dominant cost). The *previous* frame is unioned in so cells that just
    /// dried get their canvas pixel reset to the backdrop. Reset with the field.
    wet_composite_bbox: Option<(u32, u32, u32, u32)>,
    /// **W15.3 GPU drive (ADR-0049).** When the shell is stepping the wet field on
    /// the GPU (`ph2d-painter-fluid` eligible), it sets this so the tool SKIPS its
    /// CPU diffusion in `on_tick`/`queue_pointer` — the dabs still splat into the
    /// grid, but the step + composite are driven shell-side via `fluid_grid_mut` +
    /// `composite_and_settle_fluid`. Default false ⇒ the CPU path is unchanged.
    gpu_fluid_driven: bool,
    /// **W15.3 GPU resident path.** Bumped each time a fresh fluid `wet_field` is
    /// allocated (`begin_stroke`). The shell watches it: on change it resets the
    /// GPU-resident pigment (`pig_a`) + re-uploads the static paper, so a reused
    /// solver (same grid size, new stroke) starts from a bare field instead of the
    /// previous stroke's bloom. Monotonic for the tool's lifetime.
    fluid_stroke_epoch: u64,
    /// **W15.3 full-res GPU watercolor.** When the shell confirms a capable GPU it
    /// sets this, so a NEW fluid field runs at FULL canvas resolution (`scale=1`)
    /// instead of the CPU-budget half-res (`WET_FIELD_SCALE=2`) — finer bleeds + sharp
    /// edges (Enio: "bordas finas"). The CPU fallback / default build keeps half-res.
    fluid_hires: bool,
    /// The canvas/grid ratio of the LIVE field — `1` (hires GPU) or `WET_FIELD_SCALE`
    /// (half-res). Captured at `begin_stroke` from [`Self::fluid_hires`] + used by the
    /// splat, composite + the shell drive, so a mid-stroke flag flip can't desync the
    /// field's actual resolution.
    wet_field_scale: u32,
    /// **W15.3 GPU composite envelope.** The MONOTONIC (never-receding) union of the
    /// wet bboxes over the field's life — a hard upper bound on where the conserved
    /// pigment can ever be. The GPU path composites over THIS, not the current water
    /// bbox: water evaporates (its bbox marches inward) but pigment is conserved +
    /// even leaks one cell past the gate, so a receding water rect hard-cut the round
    /// dab into an axis-aligned rectangle (Enio's "quinas retangulares"). Reset to
    /// `None` with the field. CPU path keeps using the true pigment bbox.
    wet_pigment_envelope: Option<(u32, u32, u32, u32)>,
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
            layers: LayerStack::new(),
            images: BTreeMap::new(),
            composited: None,
            preview_upload_bbox: None,
            layers_revision: 0,
            layer_pixel_versions: BTreeMap::new(),
            pixel_clock: 0,
            color_before_mask: None,
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
            undo: crate::undo::UndoController::default(),
            pending_pre_stroke: None,
            undo_redo_records: Vec::new(),
            dock_shows_layers: false,
            show_brush_studio: false,
            dirty_rect: None,
            selection: std::collections::BTreeSet::new(),
            compositor_cache: CompositorCache::new(),
            adjustment_cache_pending: false,
            wash_coverage: None,
            wash_color: None,
            wash_opacity_cap: 1.0,
            wet_field: None,
            wet_backdrop: None,
            wet_composite_bbox: None,
            gpu_fluid_driven: false,
            fluid_stroke_epoch: 0,
            fluid_hires: false,
            wet_field_scale: lifecycle::WET_FIELD_SCALE,
            wet_pigment_envelope: None,
        }
    }
}

// ── Submodules (god-object split, 2026-06-04; pure mechanical move) ──
mod internal;
pub(crate) use internal::*;
mod layers;
mod lifecycle;
mod runtime;
#[cfg(test)]
mod tests;
mod trait_impls;
