//! Layout constants used by every painter in this crate. Ported
//! verbatim from `ph2d_editor_core::grid_snap::panel`.
//!
//! Wave 10 / Etapa 5.1: ROW_H/pad()/row_gap()/LABEL_FONT_SIZE now flow
//! from `ph2d_tokens` (no literal pixels). LABEL_COL_W stays as a
//! panel-specific design measurement (see comment).

use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};

pub(crate) const ROW_H: f32 = ROW_H_PX;
pub(crate) fn pad() -> f32 {
    Spacing::Lg.px()
}
/// ⛔ **Era a QUINTA cópia da mesma resposta, e a única escondida atrás de uma função** — por
/// isso o censo por `grep` de 2026-09-06 a subestimou. Ela devolvia `Sm` (6 px) enquanto o resto
/// do app avançava 4, e é o que fazia este painel respirar diferente dos vizinhos. Delega.
pub(crate) fn row_gap() -> f32 {
    ph2d_tokens::row_gap_px()
}
pub(crate) const LABEL_FONT_SIZE: f32 = TypeToken::Base.px();
/// Column where the widget (right side of a "Label: [widget]" row)
/// starts, measured from the inner-x of the row.
/// Width reserved for the label column in NumberInput rows. Widened
/// from 110 → 150 on 2026-05-15 so the longest labels ("QT bounds max
/// X / Y", "Chunk size (cells)") fit on one line.
pub(crate) const LABEL_COL_W: f32 = 150.0; // LITERAL-PX-OK: panel-specific label-column width (longest grid-snap labels)
