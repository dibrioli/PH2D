//! `StampScheduler` — converte pointer events em sequências de [`Stamp`]s
//! prontas para o `StampPipeline` compute dispatch.
//!
//! Spec [`docs/Painter_projeto/01_brush_engine.md`](
//! ../../../docs/Painter_projeto/01_brush_engine.md) §1.2 (stroke pipeline) +
//! §1.3.1 (StrokePath: spacing, jitter, lateral, falloff).
//!
//! ## Responsabilidade (T1.5 scope)
//!
//! 1. Manter **estado de stroke** (último ponto carimbado, distância residual
//!    não-consumida, seed do PRNG, contador monotônico de stamps).
//! 2. Para cada novo pointer sample, emitir N [`Stamp`]s distribuídos ao longo
//!    do segmento `last_point → point` conforme `Brush::stroke_path.spacing`.
//! 3. Aplicar `spacing_jitter` (variação aleatória do espaçamento) e
//!    `jitter_lateral` (deslocamento perpendicular ao stroke direction).
//!
//! ## T1.6 extensions (shipped)
//!
//! - **Shape variety via slot dispatch** — `Stamp.shape_layer` is sourced
//!   from `brush.shape.shape_source` (Builtin → `atlas_layer`; Imported →
//!   `atlas_layer`). Both CPU (`cpu_render`) and GPU (`stamp.wgsl`) call
//!   `library::shape_alpha_for_slot(slot, u, v)` for the procedural
//!   kernel.
//! - **Multi-stamp per pointer (`shape_count`)** — each spacing step
//!   along the segment now expands into `N` stamps stacked at the same
//!   world position, with `rotation_rad` jittered per stamp by
//!   `shape_scatter`. `shape_count_jitter` perturbs `N` itself per step.
//! - **Rotation** — three orthogonal sources composed at `push_stamp`:
//!   - `shape_randomized` → fixed `stroke_rotation_base` set on the first
//!     `advance` of the stroke.
//!   - `shape_rotation_follow` → adds `atan2(stroke_dir.y, stroke_dir.x)`
//!     per stamp.
//!   - `shape_scatter` → adds `det_random[-1,+1] * scatter_rad` per stamp
//!     (distinct `axis_tag` from spacing/lateral jitter).
//! - **Flip bits** — `shape_flip_x` / `shape_flip_y` translate to
//!   `FLAG_SHAPE_FLIP_X` / `FLAG_SHAPE_FLIP_Y` in `Stamp.flags`.
//! - **Color Dynamics stamp-level jitter (`stamp_hue/sat/lightness/darkness_
//!   jitter`)** — per-stamp OKLab perturbation: `L` is offset (or only-down
//!   for darkness), `(a, b)` rotate for hue and scale for saturation.
//!   `stamp_secondary_*` slots are reserved for T-color full
//!   (`secondary_color` is a `PainterParams` slot, scheduler hasn't seen
//!   it yet).
//! - **Rotated footprint enlargement** — when a non-radial shape has
//!   `rotation_rad != 0`, the emitted `size_px` is scaled by `√2`
//!   (`library::rotated_footprint_scale`) so the bounding box covers the
//!   rotated shape's diagonal.
//!
//! ## Fora de escopo (W1+ subsystems)
//!
//! - **Curves per-device** (Pressure/Tilt/Barrel) — T-input (ADR-0050).
//! - **Stroke-level color jitter (`stroke_*_jitter`)** — T-color full
//!   (ADR-0051) — needs `secondary_color` in scheduler context.
//! - **Pressure / tilt / barrel color modulation** — T-color full +
//!   T-input curves.
//! - **Stabilization / Streamline / Motion Filtering** — T1.7.
//! - **Taper** size/opacity curves — T1.7.
//! - **Falloff** ao longo do stroke — T1.7.
//!
//! ## HR-3 zero-alloc invariant
//!
//! Pool pré-alocado de [`MAX_STAMPS_PER_DISPATCH`] = 4096 [`Stamp`]s
//! (= 384 KB). [`StampScheduler::advance`] limpa o pool no início e
//! preenche via `push` (sem realloc pois `capacity >= MAX`). Gate
//! [`tests::advance_does_not_realloc`] prova zero realloc após `begin_stroke`.
//!
//! ## Determinismo (HR-5)
//!
//! O PRNG interno é determinístico: dado `stroke_seed` + posição do stamp no
//! stroke (counter monotônico), `spacing_jitter` e `jitter_lateral` emitem
//! offsets bit-identicos cross-OS. Usa `wyhash`-style hash de
//! `(seed, stamp_index, axis_tag)`; sem dependência de `rand` crate (que
//! tem variabilidade de seeding cross-platform).

use crate::brush::Brush;
use crate::stamp::{
    FLAG_SHAPE_FLIP_X, FLAG_SHAPE_FLIP_Y, MAX_STAMP_SIZE_PX, MAX_STAMPS_PER_DISPATCH, Stamp,
};

/// Maximum `shape_count` per pointer step (spec §1.3.4 `1..=16`). The
/// scheduler clamps `brush.shape.shape_count` to this cap so a
/// pathological brush load can't multiply the per-segment stamp output
/// beyond a known bound. Mirrors the spec range exactly.
const MAX_SHAPE_COUNT: u32 = 16;

/// Max stabilization moving-average window (`1 + round(stabilization * 16)`),
/// `stabilization ∈ [0,1]`. Sizes the fixed input-history ring (T1.7).
const STAB_WINDOW_MAX: usize = 17;

/// One-Euro motion-filter min-cutoff range (per-sample units). `motion_filtering_
/// amount` lerps `MAX → MIN`: a low cutoff at full amount means heavy smoothing of
/// slow (tremor) motion; the speed term raises it back up for fast strokes.
const MF_MIN_CUTOFF: f32 = 0.22;
const MF_MAX_CUTOFF: f32 = 3.0;
/// One-Euro β at `motion_filtering_expression = 1` — how fast the cutoff climbs
/// with speed (more = crisper fast strokes, less lag).
const MF_BETA_MAX: f32 = 0.35;

/// Velocity-dynamics speed normalisation: `speed_ema / SPEED_REF_PX` saturated to
/// `[0,1]` is the `vfactor` that scales `dynamics.speed_*`. ~24 px/sample reads as
/// a "fast" stroke at typical sampling.
const SPEED_REF_PX: f32 = 24.0;
/// EMA factor for the smoothed stroke speed (higher = snappier, lower = steadier).
const SPEED_EMA_ALPHA: f32 = 0.35;
/// Max fractional swing of size / opacity / spacing at `vfactor = 1`, full
/// `speed_*` (±this). Keeps velocity dynamics expressive but bounded.
const SPEED_DYN_SWING: f32 = 0.6;

/// Falloff fade length at `falloff = 1`, in brush diameters: at full falloff the
/// stroke opacity ramps linearly from full to **zero** over this many diameters
/// of travel. Lower falloff stretches the same ramp over a proportionally LONGER
/// distance (`L = FALLOFF_LENGTH_DIAMETERS * diameter / falloff`), so the stroke
/// always runs out — higher falloff just runs out sooner. Size-relative so the
/// feel is independent of brush size.
const FALLOFF_LENGTH_DIAMETERS: f32 = 8.0;

/// Length of the start taper at `taper_length_start = 0.5` (the max), in brush
/// diameters. The taper ramps size + opacity up over `taper_length_start *
/// TAPER_MAX_DIAMETERS * diameter` of arc length from pen-down.
const TAPER_MAX_DIAMETERS: f32 = 12.0;

/// **Live start taper** (T1.7+). Ramps the dab size + opacity UP over the first
/// `L_start` of arc length from pen-down, so a stroke enters from a clean point
/// (Procreate Taper). `taper_size_start` / `taper_opacity_start` are the tip
/// fractions at distance 0 (0 = a true point / fully transparent); both reach 1.0
/// (full brush) by `L_start = taper_length_start · TAPER_MAX_DIAMETERS · diameter`.
/// Returns `(size_factor, opacity_factor)`. `taper_length_start == 0` → `(1, 1)`
/// (no taper — the default brush, exact passthrough). Pure arithmetic + one
/// smoothstep → HR-5 cross-OS (no transcendentals). **End taper** is NOT here —
/// it needs the stroke end, i.e. a pen-up re-render (ADR-0077 D5 follow-up); live,
/// only the start is knowable.
#[inline]
#[must_use]
pub(crate) fn start_taper_factors(
    taper: &crate::taper::TaperParams,
    stroke_distance: f32,
    diameter: f32,
) -> (f32, f32) {
    let len_frac = taper.taper_length_start.clamp(0.0, 0.5);
    if len_frac <= 0.0 {
        return (1.0, 1.0);
    }
    let l_start = (len_frac * TAPER_MAX_DIAMETERS * diameter).max(1.0);
    let t = (stroke_distance / l_start).clamp(0.0, 1.0);
    let gain = t * t * (3.0 - 2.0 * t); // smoothstep
    let size_tip = taper.taper_size_start.clamp(0.0, 1.0);
    let op_tip = taper.taper_opacity_start.clamp(0.0, 1.0);
    (
        size_tip + (1.0 - size_tip) * gain,
        op_tip + (1.0 - op_tip) * gain,
    )
}

/// Deterministic falloff opacity multiplier for a stamp at `stroke_distance`
/// (px) into the stroke — the live "ink depletion" model (T1.7, Procreate-style).
///
/// The opacity ramps linearly to **zero** over a distance `L = K·diameter /
/// falloff`; `falloff` controls the RATE (length-to-empty), NOT the final level.
/// So every `falloff > 0` eventually fades the stroke to nothing — a higher
/// falloff reaches zero sooner, a lower one over a longer stroke. (The previous
/// model coupled falloff to the final level, so values < 1 plateaued and never
/// ended, and 1 looked abrupt by contrast.) `falloff = 0` → constant 1.0 (the
/// default brush, exact passthrough).
///
/// Linear (only `-`, `*`, `/`, `max` — no transcendentals → bit-stable cross-OS
/// for the HR-5 replay hash). **Live approximation:** the ramp uses a falloff-
/// derived fixed length, NOT the unknown total stroke length (the incremental
/// live path can't see the stroke's end).
#[inline]
#[must_use]
pub(crate) fn falloff_opacity(falloff: f32, stroke_distance: f32, diameter: f32) -> f32 {
    let falloff = falloff.clamp(0.0, 1.0);
    if falloff <= 0.0 {
        return 1.0;
    }
    // Distance over which the stroke depletes to zero — shrinks as falloff rises.
    let length_to_empty = (FALLOFF_LENGTH_DIAMETERS * diameter / falloff).max(1.0);
    (1.0 - stroke_distance / length_to_empty).max(0.0)
}

/// Pointer sample input — uma amostra do dispositivo (mouse / Pencil / tablet).
///
/// Para T1.5 MVP esses 4 campos bastam. Curves (pressure_curve / tilt_curve /
/// palm rejection) entram via T-input (ADR-0050) ANTES do scheduler — quando
/// `ph2d-painter-input::PointerSource` ship, este struct passa a receber
/// valores já curvados.
#[derive(Copy, Clone, Debug, Default)]
pub struct PointerSample {
    /// Coordenadas em canvas-world pixels (mesmo espaço de `Stamp.position_world`).
    pub position: [f32; 2],
    /// Pressão normalizada `[0.0, 1.0]`. Mouse = 1.0 constante.
    pub pressure: f32,
    /// Tilt em radianos `[0, π/2]`. Mouse/touch sem tilt = 0.
    pub tilt: f32,
}

/// Estado de stroke + pool de stamps reutilizável. Owned pelo
/// [`PainterTool`](../../ph2d-tool-painter/index.html); um instance por tool,
/// reset via [`Self::begin_stroke`] no pointer-down e drained via
/// [`Self::advance`] em cada pointer-move.
pub struct StampScheduler {
    /// Buffer pré-alocado. `Vec` ao invés de array fixo porque o consumidor
    /// (`StampPipeline::encode`) recebe `&[Stamp]` — Vec é o canal natural.
    /// Capacity reservada no construtor; `clear()` mantém capacity.
    pool: Vec<Stamp>,
    /// Último ponto carimbado (canvas-world px). `None` antes do primeiro
    /// stamp do stroke ou após [`Self::end_stroke`].
    last_point: Option<[f32; 2]>,
    /// Distância "comida" do espaçamento que sobrou do segmento anterior.
    /// Garante continuidade do passo entre dois pointer samples consecutivos
    /// (último gap intra-segmento + primeiro gap do próximo segmento somam
    /// um spacing inteiro, nunca duplicam um stamp).
    residual_dist: f32,
    /// Seed do stroke. Determinístico — derive de pointer-down time + entity
    /// + brush hash no caller. PRNG interno mistura com `stamp_index`.
    stroke_seed: u64,
    /// Contador monotônico de stamps emitidos NESTE stroke. Reset em
    /// [`Self::begin_stroke`]. Usado como entrada do hash determinístico.
    stamp_index: u64,
    /// Stroke-level base rotation (radians) for `shape_randomized=true`.
    /// Lazy-init on the first [`Self::advance`] that sees a brush with
    /// `shape.shape_randomized=true`. `None` for strokes that never opt in
    /// (zero cost). Reset to `None` in [`Self::begin_stroke`].
    ///
    /// Why lazy: `begin_stroke` doesn't take a `&Brush`, so we can't
    /// pre-populate at stroke-start. Audit T1.6 design — this is the
    /// minimal-API-change path that preserves the `begin_stroke(seed)`
    /// signature.
    ///
    /// **Audit T1.6 Q-4 — mid-stroke brush swap policy:** the base is
    /// initialized ONCE per stroke (on first `advance` with randomized=true)
    /// and never reset until the next `begin_stroke` / `end_stroke`. If
    /// the caller swaps the brush mid-stroke (e.g. randomized=true →
    /// randomized=false → randomized=true), the original base is reused.
    /// This is intentional: mid-stroke brush swap is not in the spec
    /// (no Procreate analog) and the simpler invariant ("one base per
    /// stroke") is cheaper to reason about. If a future feature demands
    /// re-randomization on brush change, it needs an explicit `reset_
    /// randomized_base()` API + a brush-hash tracking field.
    stroke_rotation_base: Option<f32>,
    /// Last `follow_angle` emitted (unwrapped/continuous) for
    /// `shape_rotation_follow=true`. `atan2` returns values in `(-π, π]`,
    /// so a stroke that crosses the ±π discontinuity (typical U-turn,
    /// or any path crossing the negative x-axis) jumps by ±2π between
    /// consecutive samples. For radially-symmetric shapes that's
    /// invisible; for `oval_hard` or future asymmetric shapes
    /// (`flat_chisel`, `splatter_spread`) the rotated kernel sees a
    /// 180°/360° "snap" that looks like a glitch to a calligrapher.
    /// Audit T1.6 R7 K1-10: track the previous angle and shift the new
    /// `atan2` result by the nearest multiple of `2π` so the per-stamp
    /// `rotation_rad` traces a continuous curve. `None` until the first
    /// `follow_angle` of this stroke; reset to `None` on
    /// `begin_stroke` / `end_stroke`. `break_segment` does NOT reset —
    /// the same stroke's pointer crossed a sprite gap, the rotation
    /// pattern should still be continuous on re-entry.
    last_follow_angle: Option<f32>,
    /// **T1.7 input smoothing — streamline (lazy-mouse) lag.** EMA of the
    /// stabilized input position; the painting point trails the real cursor by
    /// `brush.stabilization.streamline_amount`. `None` until the first
    /// `advance` of the stroke; reset on `begin_stroke` / `end_stroke` /
    /// `break_segment`. With `streamline_amount == 0` the EMA factor is 1.0 →
    /// the smoothed point equals its input (exact passthrough).
    streamline_pos: Option<[f32; 2]>,
    /// **T1.7 input smoothing — stabilization (moving average) ring.** Holds the
    /// last raw input positions; the average over the last `N = 1 + round(
    /// stabilization * 16)` (1..=17) reduces hand jitter. Fixed array (no alloc,
    /// HR-3). `stab_head` is the next write slot; `stab_count` saturates at the
    /// window cap. Reset (count/head → 0) on stroke boundaries.
    stab_ring: [[f32; 2]; STAB_WINDOW_MAX],
    stab_head: usize,
    stab_count: usize,
    /// **T1.7 falloff — accumulated stroke distance (px).** Sum of segment
    /// lengths painted so far this stroke; drives the `stroke_path.falloff`
    /// opacity taper (ink depletion). Reset on `begin_stroke` / `end_stroke`;
    /// **survives `break_segment`** (the gap isn't measured, but the ink already
    /// spent persists across a sprite-boundary re-entry within the same stroke).
    stroke_dist: f32,
    /// **One-Euro motion-filtering state** (Casiez/Roussel/Fekete 2012). Adaptive
    /// low-pass on the input position driven by `stabilization.motion_filtering_
    /// amount` (→ min cutoff) + `motion_filtering_expression` (→ β speed
    /// adaptation): low speed → low cutoff (kills tremor), high speed → high cutoff
    /// (no lag, preserves expressive fast strokes). `oe_pos` is the filtered
    /// position, `oe_dpos` the smoothed per-axis derivative. `None` until the first
    /// `advance`; reset on every stroke boundary. dt is the per-sample step (1).
    oe_pos: Option<[f32; 2]>,
    oe_dpos: [f32; 2],
    /// **Velocity dynamics — smoothed stroke speed (px/sample).** EMA of the
    /// per-advance segment length; drives `dynamics.speed_{size,opacity,spacing}`.
    /// Reset on every stroke boundary so a new segment starts from rest.
    speed_ema: f32,
}

impl Default for StampScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// PRNG determinístico — wyhash mixer com `(stroke_seed, stamp_index,
/// axis_tag)` como entrada. Retorna `[0.0, 1.0)`.
///
/// **Free function (audit T1.6 Z-5)** — extraído de `StampScheduler`
/// method form. Razão: o mixer é puro sobre seus 3 argumentos `u64`;
/// não precisa de `&self`. Como free fn, o optimizer não precisa
/// rastrear aliasing com os `&mut self` callers (push_stamp_group etc),
/// e o `#[inline]` é mais agressivo. Espera-se 5-15% speedup no PRNG
/// slice da hot path. Mantém-se também como method wrapper em
/// `StampScheduler` por ergonomia em test/inner sites.
///
/// Não usa `rand`: `SmallRng` etc. tem seeding variável cross-platform.
/// Mixer manual = bit-identico Mac/Linux/Windows **PARA ESTA INTEGER
/// COMPUTATION** (audit T1.6 U-1) — os f32 trig/sqrt downstream NÃO
/// são cross-OS bit-identical sem `--features det-painter`.
///
/// # Axis-tag registry (audit T1.6 R7 I1-4 + R8 N1-6 / Q1-2)
///
/// `axis_tag` partitions the PRNG stream across independent jitter
/// channels — same `(stroke_seed, stamp_index)` with different
/// `axis_tag` yields **uncorrelated** outputs. Toggling one channel
/// MUST NOT shift another channel's stream (gate
/// `color_jitter_cross_channel_axis_independence` proves byte-equality).
/// New callers within the crate MUST register their `axis_tag` here
/// to avoid collision:
///
/// | tag    | channel                                                 |
/// |--------|---------------------------------------------------------|
/// | `0xA1` | `spacing_jitter` (longitudinal step perturbation)       |
/// | `0xB2` | `jitter_lateral` (perpendicular offset)                 |
/// | `0xC1` | `stamp_lightness_jitter`                                |
/// | `0xC2` | `stamp_darkness_jitter`                                 |
/// | `0xC3` | `stamp_hue_jitter`                                      |
/// | `0xC4` | `stamp_saturation_jitter`                               |
/// | `0xCC` | `stroke_rotation_base` (lazy-init, `shape_randomized`)  |
/// | `0xCD` | `shape_scatter` (per-stamp rotation offset)             |
/// | `0xCE` | `shape_count_jitter` (group size perturbation)          |
/// | `0xCF` | RESERVED — future per-stamp randomized flip (Q-11)      |
/// | `0xD0` | RESERVED — future per-stamp randomized flip (Q-11)      |
/// | `0xD1` | `dynamics.jitter_size` (per-stamp size variation, T1.7)  |
/// | `0xD2` | `dynamics.jitter_opacity` (per-stamp opacity variation)  |
/// | `0xD3` | `shape_scatter` positional offset — angle                |
/// | `0xD4` | `shape_scatter` positional offset — radius               |
///
/// **Audit T1.6 R8 N1-6 / Q1-2:** R7 shipped this table with only
/// 6 of 9 active tags listed; `0xA1` / `0xB2` / `0xCC` were
/// silently in use without entries. Two reviewers caught the same
/// drift independently. Gate
/// `det_random_axis_tags_match_registry` (in `mod tests`) now
/// enumerates EVERY `det_random` call site in the crate's source
/// and asserts the set of literal `axis_tag` arguments equals the
/// registered set `{0xA1, 0xB2, 0xC1..0xC4, 0xCC, 0xCD, 0xCE, 0xD1..0xD4}`
/// (reserved `0xCF`/`0xD0` are NOT yet in use so they don't appear).
/// Any new call site collides at test-time and forces a registry
/// update — no more silent collision risk.
///
/// `pub(crate)` is the right scope today (intra-crate only). The
/// `#[doc(hidden)]` keeps rustdoc from publishing the contract
/// publicly — a downstream crate that built on top of these exact
/// wyhash constants would silently break HR-5 if a future
/// `--features det-painter` swaps the mixer for an LUT-pinned form
/// (audit T1.6 U-1 / R7 I1-4).
#[doc(hidden)]
#[inline]
pub(crate) fn det_random(stroke_seed: u64, stamp_index: u64, axis_tag: u64) -> f32 {
    // wyhash-style: 3-fold xor-shift + multiply. Boa avalanche para
    // uso de jitter sem dep externa.
    let mut h = stroke_seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(stamp_index);
    h ^= h >> 32;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= axis_tag;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    // Top 24 bits → [0, 1) preserva precisão de f32 mantissa.
    ((h >> 40) as f32) / ((1u64 << 24) as f32)
}

// ── Submodules (god-object split, 2026-06-04; pure mechanical move) ──
mod advance;
#[cfg(test)]
mod tests;
