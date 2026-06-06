//! Free-function helpers + `ToolPixelSource` for the Painter tool,
//! split out of the former `tool.rs` god-object (pure mechanical move).
//! `pub(crate)` so the impl submodules can call them; re-exported from
//! `tool/mod.rs` via `pub(crate) use internal::*`.

use super::*;

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
/// - `PAINTER_SMOKE_PIGMENT` — `"mixbox"` / `"linear"` (default); sets
///   `brush.rendering.pigment_mode` (W5 Mixbox smoke: blue+yellow→green).
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
pub(crate) fn build_smoke_brush_from_env() -> Brush {
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
    // W5 Mixbox smoke: `PAINTER_SMOKE_PIGMENT=mixbox` turns on subtractive pigment
    // mixing for the active brush (paint yellow over blue at <100% opacity → green,
    // not grey). Temporary surface — mirror of the other smoke vars — until the
    // sidebar / Brush Studio per-brush pigment toggle lands.
    if let Ok(s) = std::env::var("PAINTER_SMOKE_PIGMENT") {
        match s.trim_matches(|c| c == '\'' || c == '"').trim() {
            "mixbox" | "Mixbox" => {
                brush.rendering.pigment_mode = ph2d_painter_brush::PigmentMode::Mixbox;
            }
            "linear" | "Linear" | "" => {}
            other => eprintln!(
                "[painter] PAINTER_SMOKE_PIGMENT={other:?} unknown; keeping Linear. \
                 Valid: mixbox / linear."
            ),
        }
    }

    brush
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
pub(crate) fn oklch_to_oklab(c: OklchColor) -> [f32; 4] {
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

/// Bounding box (canvas px, clamped to `[0,w)×[0,h)`) of a batch of stamps —
/// the dirty region a stroke event touched. `None` if the batch lands fully
/// outside the canvas (degenerate). Each stamp footprint is `position ± size/2`
/// plus a 1 px margin for soft (anti-aliased) edges.
pub(crate) fn stamps_bbox(stamps: &[Stamp], w: u32, h: u32) -> Option<Region> {
    let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for s in stamps {
        let r = s.size_px * 0.5 + 1.0; // +1px: soft-edge / AA margin (canvas px, not a UI value)
        minx = minx.min(s.position_world[0] - r);
        miny = miny.min(s.position_world[1] - r);
        maxx = maxx.max(s.position_world[0] + r);
        maxy = maxy.max(s.position_world[1] + r);
    }
    let x0 = minx.floor().clamp(0.0, w as f32) as u32;
    let y0 = miny.floor().clamp(0.0, h as f32) as u32;
    let x1 = maxx.ceil().clamp(0.0, w as f32) as u32;
    let y1 = maxy.ceil().clamp(0.0, h as f32) as u32;
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(Region {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    })
}

/// Smallest [`Region`] covering both `a` and `b`.
pub(crate) fn union_region(a: Region, b: Region) -> Region {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = (a.x + a.w).max(b.x + b.w);
    let y1 = (a.y + a.h).max(b.y + b.h);
    Region {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

/// Blit a freshly-composited `region` (its own `bbox.w × bbox.h` RGBA8 buffer)
/// into the full-canvas composite `cache` at `bbox`, row by row.
pub(crate) fn blit_region(cache: &mut [u8], canvas_w: u32, region: &[u8], bbox: Region) {
    let row_bytes = (bbox.w * 4) as usize;
    for ry in 0..bbox.h {
        let src_off = (ry * bbox.w * 4) as usize;
        let dst_off = (((bbox.y + ry) * canvas_w + bbox.x) * 4) as usize;
        cache[dst_off..dst_off + row_bytes].copy_from_slice(&region[src_off..src_off + row_bytes]);
    }
}

/// [`LayerPixelSource`] over the tool's live buffers: the ACTIVE layer reads
/// `canvas_rgba` (the Arc working buffer — zero-copy, always current), every
/// other layer reads its `images` entry. Built transiently inside
/// `current_preview` for the non-trivial composite path.
pub(crate) struct ToolPixelSource<'a> {
    pub(crate) active_id: RtLayerId,
    pub(crate) active_rgba: &'a [u8],
    pub(crate) images: &'a BTreeMap<RtLayerId, LayerImage>,
}

impl LayerPixelSource for ToolPixelSource<'_> {
    fn layer_rgba(&self, id: RtLayerId) -> Option<&[u8]> {
        if id == self.active_id {
            Some(self.active_rgba)
        } else {
            self.images.get(&id).map(|img| img.rgba8.as_slice())
        }
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
pub(crate) fn pointer_to_raw_sample(sample: PointerSample) -> Option<RawPointerSample> {
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
    // **B.4 audit-2:** CHECKED Q16.16 conversion (the frozen crate's documented
    // path for un-validated input). A position outside the useful window
    // (|v| >= 32768, ADR-0046 §2.2) is out-of-spec — return `None` so the caller
    // drops the whole sample, rather than record a CLAMPED position (silent W12
    // replay divergence) or trip `_saturating`'s debug_assert. The earlier
    // finite-guard in `queue_pointer` already rejected NaN/Inf.
    Some(RawPointerSample {
        x_q1616: f32_to_q1616_checked(sample.position[0])?,
        y_q1616: f32_to_q1616_checked(sample.position[1])?,
        pressure_q88: f32_to_q88(sample.pressure),
        tilt_q88: f32_to_q88(sample.tilt),
        flags,
        ..Default::default()
    })
}

/// Converte `PainterTool` `OklchColor` (ph2d-tool-painter stub) → ph2d-color
/// `OklchColor` (canonical type). Painter mantém type local em params.rs
/// pra evitar dep transitive antes do T-color full (ADR-0051).
///
/// **HUE UNIT BOUNDARY (correctness — radians → degrees):** the painter
/// stub's `h` is in **RADIANS** (its native unit — `color.rs` derives it
/// via `atan2` and `tool::oklch_to_oklab` consumes it as radians, asserting
/// `|h| ≤ 4π`). The canonical [`ph2d_color::OklchColor`] documents `h` as
/// **degrees `[0, 360)`** and every consumer that turns it into pixels calls
/// `.to_linear()` → `.to_radians()`. `StrokeRecord.primary_color` is this
/// canonical type and `ph2d-painter-stroke` stores/serializes/replays it as
/// opaque data (zero hue math anywhere in that crate — verified), so the
/// degrees contract is only honored when the *first* canonical consumer runs
/// (render / reproject / Inspector W12+). A naïve field-by-field copy wrote
/// e.g. `π rad ≈ 3.14` as "3.14 degrees" into the WAL — a wildly wrong hue
/// baked permanently into the persisted stroke. Convert at this boundary so
/// the painter keeps radians internally and the canon stays in degrees.
pub(crate) fn painter_color_to_stroke_oklch(c: OklchColor) -> StrokeOklchColor {
    StrokeOklchColor {
        l: c.l,
        c: c.c,
        h: c.h.to_degrees(),
        a: c.a,
    }
}

/// Materializa `PartialStroke` (in-progress) + `Vec<RawPointerSample>`
/// (collected samples) num `StrokeRecord` (canon — push em StrokeHistory).
pub(crate) fn partial_to_record(
    partial: PartialStroke,
    samples: Vec<RawPointerSample>,
) -> StrokeRecord {
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
pub(crate) fn painter_mode_to_tool_mode(mode: PainterMode) -> ToolMode {
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
pub(crate) fn sanitize_oklch_or_default(c: OklchColor) -> StrokeOklchColor {
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
pub(crate) fn brush_handle_stub_to_canon(stub: BrushHandle) -> ph2d_painter_brush::BrushHandle {
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
