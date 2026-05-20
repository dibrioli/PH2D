//! Layout constants used by every painter in this crate. Ported
//! verbatim from `ph2d_editor_core::grid_snap::panel`.

pub(crate) const ROW_H: f32 = 28.0;
pub(crate) const PAD: f32 = 12.0;
pub(crate) const ROW_GAP: f32 = 6.0;
pub(crate) const LABEL_FONT_SIZE: f32 = 13.0;
/// Column where the widget (right side of a "Label: [widget]" row)
/// starts, measured from the inner-x of the row.
/// Width reserved for the label column in NumberInput rows. Widened
/// from 110 → 150 on 2026-05-15 so the longest labels ("QT bounds max
/// X / Y", "Chunk size (cells)") fit on one line.
pub(crate) const LABEL_COL_W: f32 = 150.0;
