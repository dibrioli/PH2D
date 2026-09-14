//! Layout constants used by every painter in this crate. Ported
//! verbatim from `ph2d_editor_core::grid_snap::panel`.
//!
//! Wave 10 / Etapa 5.1: ROW_H/pad()/row_gap()/LABEL_FONT_SIZE now flow
//! from `ph2d_tokens` (no literal pixels). ⭐ E a coluna do rótulo saiu daqui
//! em 2026-09-14: ela é a porta `property_label_col_w`, perguntada no sítio da
//! pintura com a largura REAL da linha.

use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};

pub(crate) const ROW_H: f32 = ROW_H_PX;
pub(crate) fn pad() -> f32 {
    Spacing::Lg.px()
}
/// ⛔ **Era a QUINTA cópia da mesma resposta, e a única escondida atrás de uma função** — por
/// isso o censo por `grep` de 2026-09-06 a subestimou. Ela devolvia `Sm` (6 px) enquanto o resto
/// do app avançava 4, e é o que fazia este painel respirar diferente dos vizinhos. Delega.
pub(crate) fn row_gap() -> f32 {
    ph2d_tokens::control_gap_px()
}
pub(crate) const LABEL_FONT_SIZE: f32 = TypeToken::Base.px();
