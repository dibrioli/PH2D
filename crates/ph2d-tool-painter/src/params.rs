//! Painter UI vocabulary — `PainterParams` + `PainterUiEdit` + `PainterUiSnapshot`
//! + `PainterMode`. Contrato congelado em [ADR-0043 §2.3](../../../docs/architecture/decisions/0043-painter-contract.md).
//!
//! Caps congelados (auditados por `ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs`):
//! - `PainterUiEdit ≤ 24` variants
//! - `PainterUiSnapshot ≤ 18` fields
//! - `PainterParams ≤ 12` fields
//! - `PainterMode ≤ 6` variants
//!
//! # T1.1 stubs + HR-14 forward-compat policy (audit 2026-05-26 F7)
//!
//! T1.1 ship com **stub types locais** (`BrushHandle(pub u32)`, `ThumbHandle(pub u32)`,
//! `OklchColor { l,c,h,a: f32 }`, `SymmetryAxis`) que serão substituídos por canon
//! quando crates filhos nascerem (T1.3 `ph2d-painter-brush`, etc.).
//!
//! **HR-14 forward-compat:** o stub `BrushHandle(pub u32)` é **estruturalmente
//! idêntico** ao canon ADR-0044 §2.8 (`pub struct BrushHandle(u32)` com bit-31
//! flag). Serde deserialization é newtype-tuple-transparent (`u32` wire) — savefiles
//! v1 produzidos por T1.1 deserializam corretamente após T1.3 substituir o tipo,
//! **desde que**:
//!   1. T1.3 mantenha `pub struct BrushHandle(u32)` (mesma representação Serde).
//!   2. T1.3 NÃO use `#[serde(rename)]` para mudar o nome lógico.
//!   3. `OklchColor` no canon (ADR-0042 ph2d-color) deve ter mesmos 4 campos
//!      l/c/h/a f32 OR usar `#[serde(remote)]` adapter.
//!
//! Validação em T1.3: round-trip test `PainterParams_v1_postcard_deserializes_in_t13`
//! que grava com stub types e lê com canon types. Audit registrado como **gate
//! de transição** (não bypass deferral).
//!
//! `SymmetryAxis` (W9) e `ThumbHandle` (W2 atlas) materializam em waves futuras —
//! mesmo forward-compat policy aplica.

use ph2d_painter_brush::MAX_STAMP_SIZE_PX;
use serde::{Deserialize, Serialize};

// ----------------------------------------------------------------------------
// Size / opacity affine maps — SINGLE SOURCE OF TRUTH (audit Y-6, 2026-05-28)
// ----------------------------------------------------------------------------
//
// The sidebar slider stores a normalized `0..1` value; the chip + tool
// store display units (px / %). The forward/inverse maps below are the
// ONLY place the `size01 ↔ size_px` (and `opacity01 ↔ %`) conversion is
// defined. `tool.rs` (apply_ui_edit / ui_snapshot), `paint.rs` (display),
// and `populate.rs` (seed + `link_slider_number_mapped`) ALL call these —
// no crate re-derives the `MAX_STAMP_SIZE_PX` literal locally.

/// Forward map: normalized `0..1` slider storage → brush size in pixels
/// (`1.0..=MAX_STAMP_SIZE_PX`).
#[inline]
#[must_use]
pub fn size01_to_px(size01: f32) -> f32 {
    1.0 + size01.clamp(0.0, 1.0) * (MAX_STAMP_SIZE_PX as f32 - 1.0)
}

/// Inverse map: brush size in pixels → normalized `0..1` slider storage.
#[inline]
#[must_use]
pub fn px_to_size01(size_px: f32) -> f32 {
    ((size_px - 1.0) / (MAX_STAMP_SIZE_PX as f32 - 1.0)).clamp(0.0, 1.0)
}

/// Affine `(scale, offset)` for `link_slider_number_mapped` so the Size
/// chip displays pixels while the slider stores `0..1` (`px = s*scale + offset`).
/// Derived from [`size01_to_px`] so it can never drift from the forward map.
#[inline]
#[must_use]
pub fn size_chip_mapping() -> (f32, f32) {
    let offset = size01_to_px(0.0); // = 1.0
    let scale = size01_to_px(1.0) - offset; // = MAX_STAMP_SIZE_PX - 1
    (scale, offset)
}

/// Forward map: normalized `0..1` opacity → display percent (`0..=100`).
#[inline]
#[must_use]
pub fn opacity01_to_pct(opacity01: f32) -> f32 {
    opacity01.clamp(0.0, 1.0) * 100.0
}

/// Affine `(scale, offset)` for the Opacity chip (`% = o*100 + 0`).
#[inline]
#[must_use]
pub fn opacity_chip_mapping() -> (f32, f32) {
    (100.0, 0.0)
}

// ----------------------------------------------------------------------------
// Stub types — substituídos quando crates filhos nascem (W1 T1.3+)
// ----------------------------------------------------------------------------

/// Stub `BrushHandle` — substituído por `ph2d_painter_brush::BrushHandle`
/// quando esse crate nascer (T1.3, ADR-0044 §2.8).
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct BrushHandle(pub u32);

/// Stub `ThumbHandle` — atlas-resident thumb id (W2 sidebar).
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct ThumbHandle(pub u32);

/// Stub `OklchColor` — substituído por `ph2d_color::OklchColor`
/// quando integration sólida (T1.X).
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct OklchColor {
    pub l: f32,
    pub c: f32,
    pub h: f32,
    pub a: f32,
}

/// Stub `SymmetryAxis` — W9 Drawing Assist.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SymmetryAxis {
    Vertical,
    Horizontal,
    Radial(u8),
}

// ----------------------------------------------------------------------------
// PainterMode — ADR-0043 §2.3
// ----------------------------------------------------------------------------

/// Painting mode toggled via topbar (Brush / Smudge / Eraser).
///
/// Mapping para `ToolMode` (history schema ADR-0046 §2.4):
/// `Brush ↔ Paint`, `Smudge ↔ Smudge`, `Eraser ↔ Erase`.
/// Vide ADR-0043 §2.6.1 (mapping congelado cross-ADR).
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PainterMode {
    #[default]
    Brush,
    Smudge,
    Eraser,
}

// ----------------------------------------------------------------------------
// PainterUiEdit — ADR-0043 §2.3 (cap ≤ 24)
// ----------------------------------------------------------------------------

/// Sidebar + topbar events traduzidos do `PanelEvent` genérico para
/// semântica Painter dentro de `Tool::handle_panel_event`.
///
/// v1 ship: ~15 variants; +`SetColorSrgb` (W2.T2.3) = 16; headroom para
/// W9 (3), W11 (2), W14 (1) + 2 residual (cap 24).
///
/// **Audit T1.6 R9 V1-H2:** `#[non_exhaustive]` mirrors the
/// `BgRemovalUiEdit` precedent (R7 I1-1). Additive variants no
/// longer semver-break downstream `match`. Wire format (serde) is
/// unaffected — `non_exhaustive` is a Rust-level exhaustiveness
/// hint, not a serialization annotation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum PainterUiEdit {
    // Sidebar sliders (normalized 0..=1)
    Size(f32),
    Opacity(f32),
    // Color
    SetColor(OklchColor),
    /// Set the primary color from sRGB8 `[r, g, b, a]` bytes — the wire
    /// format the visual picker + hex field speak. Converted to the
    /// painter-native OKLCH (hue in radians) inside `apply_ui_edit` via
    /// `color::srgb8_to_painter_oklch`, so the picker never touches the
    /// radians/degrees hue convention (W2.T2.3). Cap headroom: this is
    /// variant 16/24 (was 15 + 9 reserved; consumes 1 reserved slot).
    SetColorSrgb([u8; 4]),
    // Brush selection
    SelectBrush(BrushHandle),
    // Topbar mode toggles
    ToggleBrushMode,
    ToggleSmudgeMode,
    ToggleEraserMode,
    OpenLayersPopover,
    OpenColorPopover,
    // Eyedropper
    ToggleEyedropper,
    // Undo/Redo (sidebar buttons; ALSO disparáveis via gesture)
    Undo,
    Redo,
    // Brush Studio modal entry
    OpenBrushStudio,
    // Reset (sidebar long-press)
    ResetSidebar,
    // Symmetry (W9 Drawing Assist)
    ToggleSymmetry,
    // W5 pigment mixing — flip the active brush between Linear (OKLab lerp) and
    // Subtractive (Kubelka-Munk pigment). User-facing label: "Pigment".
    TogglePigment,
    // W5 stroke accumulation — flip wash (opacity-capped) ↔ build-up. Orthogonal
    // to pigment. User-facing label: "Accumulate".
    ToggleAccumulate,
    // === 6 slots de headroom (W9+W11+W14+residual) ===
    // Reserved para waves futuras:
    //   SetSymmetryAxis(SymmetryAxis), SetRadialN(u8), SetMirrorOffset(f32) — W9
    //   ToggleOnionSkin, SetAnimFps(f32) — W11
    //   OpenInspector — W14
}

// ----------------------------------------------------------------------------
// PainterUiSnapshot — ADR-0043 §2.3 (cap ≤ 18)
// ----------------------------------------------------------------------------

/// Projeção read-only do `PainterTool` que o `ph2d-panel-painter` (sidebar)
/// pinta a cada frame.
///
/// `Default` (audit W-3, 2026-05-28) **espelha [`PainterParams::default`]**
/// — NÃO é all-zero derivado. O sidebar usa este default em dois lugares:
/// (1) `state::current_snapshot()` fallback antes do host pushar o primeiro
/// snapshot, e (2) seed do `populate`. Um default all-zero pintava um brush
/// size-0 / opacity-0 invisível no primeiro frame e dessincronizava o seed
/// do `populate` do fallback do `paint` (split-brain). Espelhar params
/// garante que ambos refletem o brush real (32 px / 100% / laranja default).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PainterUiSnapshot {
    // Sliders (normalized 0..=1)
    pub size01: f32,
    pub opacity01: f32,
    // Color (display)
    pub active_color: OklchColor,
    pub secondary_color: OklchColor,
    // Brush (display)
    pub active_brush_thumb: ThumbHandle,
    pub active_brush_name: String,
    // Topbar mode (display)
    pub mode: PainterMode,
    // Affordance flags (display state)
    pub eyedropper_armed: bool,
    pub symmetry_enabled: bool,
    pub undo_enabled: bool,
    pub redo_enabled: bool,
    pub stroke_in_flight: bool,
    // Selection / takeover state (display)
    pub takeover_active: bool,
    pub active_layer_name: String,
    pub active_layer_locked: bool,
    // W5 pigment toggle display state: true = Subtractive (pigment), false = Linear.
    pub pigment_enabled: bool,
    // W5 accumulate toggle display state: true = build-up, false = wash (capped).
    pub accumulate_enabled: bool,
    // 17 fields v1 — 1 slot de headroom
}

impl PainterUiSnapshot {
    /// Primary color materialized as sRGB8 `[r, g, b, a]` bytes — the
    /// form the top-bar color thumb + picker swatch paint with.
    ///
    /// Computed on demand from [`Self::active_color`] (OKLCH, hue in
    /// radians) so it costs ZERO snapshot fields (cap 18 stays intact —
    /// W2.T2.3). Out-of-gamut chroma is clamped at the display boundary.
    #[must_use]
    pub fn active_color_srgb8(&self) -> [u8; 4] {
        crate::color::painter_oklch_to_srgb8(self.active_color)
    }

    /// Primary color as a hex string (`#RRGGBB` opaque, `#RRGGBBAA`
    /// otherwise) — what the hex input field displays. Derived from
    /// [`Self::active_color`]; no snapshot field cost.
    #[must_use]
    pub fn active_color_hex(&self) -> String {
        crate::color::painter_oklch_to_hex(self.active_color)
    }

    /// Secondary color (long-press slot) as sRGB8 bytes — companion to
    /// [`Self::active_color_srgb8`] for the dual-swatch affordance.
    #[must_use]
    pub fn secondary_color_srgb8(&self) -> [u8; 4] {
        crate::color::painter_oklch_to_srgb8(self.secondary_color)
    }
}

impl Default for PainterUiSnapshot {
    fn default() -> Self {
        // Mirror PainterParams::default() — see struct doc (audit W-3).
        let p = PainterParams::default();
        Self {
            size01: px_to_size01(p.size_px),
            opacity01: p.opacity.clamp(0.0, 1.0),
            active_color: p.active_color,
            secondary_color: p.secondary_color,
            active_brush_thumb: ThumbHandle::default(),
            active_brush_name: String::new(),
            mode: p.mode,
            eyedropper_armed: p.eyedropper_armed,
            symmetry_enabled: p.symmetry.is_some(),
            undo_enabled: false,
            redo_enabled: false,
            stroke_in_flight: false,
            takeover_active: p.takeover_active,
            active_layer_name: String::new(),
            active_layer_locked: false,
            // Mirror the default smoke brush (Subtractive) so the fallback
            // snapshot doesn't flicker the toggle off before the real push.
            pigment_enabled: true,
            accumulate_enabled: false, // wash by default
        }
    }
}

// ----------------------------------------------------------------------------
// PainterParams — ADR-0043 §2.3 (cap ≤ 12)
// ----------------------------------------------------------------------------

/// Estado serializável do Painter sidebar (não inclui Brush struct nem
/// stroke history — esses vivem em crates dedicados via `BrushHandle` +
/// `StrokeHistoryRef`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PainterParams {
    pub size_px: f32,
    pub opacity: f32,
    pub active_color: OklchColor,
    pub secondary_color: OklchColor,
    pub active_brush: BrushHandle,
    pub mode: PainterMode,
    pub symmetry: Option<SymmetryAxis>,
    pub eyedropper_armed: bool,
    pub takeover_active: bool,
    pub version: u32,
    // 10 fields v1 — 2 slots de headroom
}

impl Default for PainterParams {
    fn default() -> Self {
        // **Day-7 marker:** `active_color` defaults to OPAQUE VIBRANT
        // ORANGE — not black. Round 1 (A-C2/B-C2) fixed the alpha=0
        // bug; round 3 (R3-LE-3) caught that black-on-black is still
        // invisible. An L=0.7 / C=0.18 / H≈0.5rad (~28.6°) color is
        // clearly visible against neutral or dark sprites, on most
        // common backgrounds (sand-tan / amber / orange spectrum is a
        // safe high-contrast pick on a typical asset palette). W2's
        // proper color picker UI will override this on first use; the
        // default exists only to make the Day-7 smoke succeed without
        // any user color selection.
        Self {
            size_px: 32.0,
            opacity: 1.0,
            active_color: OklchColor {
                l: 0.70,
                c: 0.18,
                h: 0.5,
                a: 1.0,
            },
            secondary_color: OklchColor::default(),
            active_brush: BrushHandle::default(),
            mode: PainterMode::default(),
            symmetry: None,
            eyedropper_armed: false,
            takeover_active: false,
            version: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn painter_mode_default_is_brush() {
        assert_eq!(PainterMode::default(), PainterMode::Brush);
    }

    #[test]
    fn painter_params_default_is_sensible() {
        let p = PainterParams::default();
        assert_eq!(p.size_px, 32.0);
        assert_eq!(p.opacity, 1.0);
        assert_eq!(p.version, 1);
        assert!(p.symmetry.is_none());
        // Audit T1.5 round 2 F5: Day-7 marker contract — default color
        // MUST be opaque (alpha = 1.0). Without this assert a future
        // regression resetting active_color to all-zero (alpha=0) would
        // silently make every stroke invisible.
        assert_eq!(
            p.active_color.a, 1.0,
            "Day-7 marker: PainterParams::default().active_color.a must be 1.0 \
             (opaque) so input-dispatch strokes produce visible paint"
        );
        // Audit T1.5 round 3 R3-LE-3: default color MUST have non-zero
        // chroma (otherwise opaque black, invisible on dark sprites).
        // The Day-7 smoke must produce a visibly-colored stamp without
        // the user touching a color picker.
        assert!(
            p.active_color.c > 0.05,
            "Day-7 marker: default active_color must have visible chroma; got c={}",
            p.active_color.c
        );
    }

    #[test]
    fn painter_ui_snapshot_default_mirrors_params() {
        // Audit W-3 (2026-05-28): snapshot Default MUST reflect
        // PainterParams::default(), not all-zero. A size-0 / opacity-0
        // fallback painted an invisible brush on the first frame and
        // desynced the populate seed from the paint fallback.
        let s = PainterUiSnapshot::default();
        let p = PainterParams::default();
        assert_eq!(s.size01, px_to_size01(p.size_px));
        assert!(
            s.size01 > 0.0,
            "default size01 must be non-zero (32 px brush)"
        );
        assert_eq!(s.opacity01, p.opacity);
        assert_eq!(s.active_color, p.active_color);
        assert_eq!(s.mode, p.mode);
        assert!(!s.takeover_active);
        assert!(!s.undo_enabled);
        assert!(!s.redo_enabled);
    }

    #[test]
    fn size_affine_map_round_trips() {
        // SSOT forward/inverse maps (audit Y-6). Exact at the endpoints.
        assert_eq!(size01_to_px(0.0), 1.0);
        assert_eq!(size01_to_px(1.0), MAX_STAMP_SIZE_PX as f32);
        // 32 px default → size01 → px round-trips within fp tolerance.
        let px = 32.0;
        assert!((size01_to_px(px_to_size01(px)) - px).abs() < 1e-3);
        // Chip mapping derived from the forward map can't drift.
        let (scale, offset) = size_chip_mapping();
        assert_eq!(offset, 1.0);
        assert_eq!(scale, MAX_STAMP_SIZE_PX as f32 - 1.0);
    }

    #[test]
    fn opacity_affine_map_matches_chip_mapping() {
        assert_eq!(opacity01_to_pct(0.0), 0.0);
        assert_eq!(opacity01_to_pct(1.0), 100.0);
        assert_eq!(opacity_chip_mapping(), (100.0, 0.0));
    }
}
