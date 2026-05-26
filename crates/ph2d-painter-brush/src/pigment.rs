//! `PigmentMode` enum — axis ortogonal ao `RenderingMode`. Controla a
//! matemática de mistura de cor entre stamp e layer. ADR-0044 §2.5.
//!
//! Linear = lerp em OKLab (Photoshop default). Mixbox = pigment mixing
//! subtrativo (Sochorová+Jamriška SIGGRAPH 2021 — azul + amarelo = verde
//! vibrante em vez de cinza lamacento).

use serde::{Deserialize, Serialize};

/// Modo de mistura de cor pigment-aware. Aplicado **antes** do Rendering mode
/// escolher a fórmula de composição alpha.
///
/// **Excluído do det-mode** (ADR-0044 §2.5.1): em `--features det-painter`,
/// Mixbox força fallback Linear porque LUT 3D float não é IEEE-bit-identical
/// cross-OS. Port Q-fixed-point é follow-up (~2 sessões).
#[repr(u32)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PigmentMode {
    /// Lerp em OKLab linear (Photoshop default). Funciona em qualquer brush.
    /// **Único modo det-mode disponível.**
    #[default]
    Linear = 0,
    /// Mixbox — Kubelka-Munk simplificado, pigment mixing real-world.
    /// Per-brush default em paints/watercolors (Library §1.6.9).
    /// NÃO funciona em --features det-painter (fallback Linear).
    Mixbox = 1,
    // === 2 slots de headroom (cap ≤ 4 ADR-0044 §2.5) ===
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_linear() {
        assert_eq!(PigmentMode::default(), PigmentMode::Linear);
    }

    #[test]
    fn discriminants_match_stamp_pigment_mode_u32() {
        // Stamp.pigment_mode = 0 (Linear) / 1 (Mixbox) — ABI freeze ADR-0044 §2.3.
        assert_eq!(PigmentMode::Linear as u32, 0);
        assert_eq!(PigmentMode::Mixbox as u32, 1);
    }
}
