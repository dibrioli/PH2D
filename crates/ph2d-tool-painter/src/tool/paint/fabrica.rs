//! **A FÁBRICA do pincel na mão** — a régua de todo Reset de secção (decisão do dono, 2026-10-04:
//! *«o Reset deve manter o meio escolhido e só repor os valores dele»*). Medição e mecanismo: doc 45
//! §2.1; gate `tests/reset_de_fabrica.rs`.

use super::{PaintState, PainterTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

/// O Falloff com que o Impasto nasce — **Sphere**, não o Smooth do pincel (Enio 2026-07-17: o ombro
/// hemisférico lê-se como corpo sob a luz do impasto; o planalto do Smooth lê-se como autocolante).
pub(super) const FALLOFF_DO_IMPASTO: Falloff = Falloff::Sphere;

impl PainterTool {
    /// O pincel com que o MODO na mão nasce, no MEIO escolhido: o pincel do slot (Spacing `0,025` no
    /// Wet Paint, `0,05` no Smear/Blur/Clone, o *rake* do Sculpt…) com o que o meio acrescenta ao ser
    /// escolhido (o Falloff do Impasto; o da Aquarela com a Shape manual).
    ///
    /// ⚠️ **O meio NÃO é um valor de secção**: os dois interruptores dele (`watercolor`, `impasto`)
    /// saem daqui com o valor de AGORA, e um Reset que copie a fábrica nunca troca o meio.
    pub(super) fn spec_de_fabrica(&self) -> BrushSpec {
        let b = &self.paint.brush;
        let mut s = PaintState::pinceis_de_fabrica()[self.paint.paint_mode.slot()];
        s.watercolor = b.watercolor;
        s.impasto = b.impasto;
        if b.impasto {
            s.falloff = FALLOFF_DO_IMPASTO;
        } else if b.watercolor && !b.watercolor_shape_auto {
            s.falloff = Falloff::Watercolor;
        }
        s
    }
}
