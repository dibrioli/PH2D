//! Painter UI vocabulary — `PainterParams` + `PainterUiEdit` + `PainterUiSnapshot`
//! + `PainterMode`. Contrato congelado em [ADR-0043 §2.3](../../../docs/architecture/decisions/0043-painter-contract.md).
//!
//! Caps congelados (auditados por `ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs`):
//! - `PainterUiEdit ≤ 24` variants
//! - `PainterUiSnapshot ≤ 18` fields
//! - `PainterParams ≤ 12` fields
//! - `PainterMode ≤ 6` variants
//!
//! T1.1 stub: structs com defaults sensatos; campos placeholder até T1.2+
//! conectar sidebar real. Tipo opacos referenciados (`BrushHandle`,
//! `ThumbHandle`, `SymmetryAxis`, `OklchColor`) ficam stubbed locally — quando
//! crates filhos nascerem (T1.3 ph2d-painter-brush etc.), substituem.

use serde::{Deserialize, Serialize};

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
/// v1 ship: ~15 variants; headroom para W9 (3), W11 (2), W14 (1) + 3 residual.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PainterUiEdit {
    // Sidebar sliders (normalized 0..=1)
    Size(f32),
    Opacity(f32),
    // Color
    SetColor(OklchColor),
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
    // === 9 slots de headroom (W9+W11+W14+residual) ===
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
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
    // 15 fields v1 — 3 slots de headroom
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
        Self {
            size_px: 32.0,
            opacity: 1.0,
            active_color: OklchColor::default(),
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
    }

    #[test]
    fn painter_ui_snapshot_default_is_sensible() {
        let s = PainterUiSnapshot::default();
        assert_eq!(s.size01, 0.0);
        assert!(!s.takeover_active);
    }
}
