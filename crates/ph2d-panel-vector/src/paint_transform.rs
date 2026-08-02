//! Numeric Transform section painter for the Vector Style panel: the X/Y
//! (position) and W/H (size) fields of the selected path's bbox. Split from
//! `paint_sections` to keep that file under the 600-LOC panel cap; it's an
//! `impl BodyCtx` block over there.

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::state;
use ph2d_i18n::tr;

impl BodyCtx<'_> {
    /// Numeric Transform — X/Y (position) + W/H (size) of the selected path's
    /// anchor bbox, two 2-column rows. Hidden when no path is selected.
    pub(crate) fn transform_section(&mut self, y: f32) -> f32 {
        if state::current_transform().is_none() {
            return y;
        }
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_TRANSFORM,
            tr("panel.vector.section.transform"),
            y,
        );
        if collapsed {
            return y;
        }
        y = self.number_row(
            "X",
            ids::VECTOR_TRANSFORM_X,
            "Y",
            ids::VECTOR_TRANSFORM_Y,
            y,
        );
        y = self.number_row(
            "W",
            ids::VECTOR_TRANSFORM_W,
            "H",
            ids::VECTOR_TRANSFORM_H,
            y,
        );
        // Rotation — a full-width relative scrub (° per gesture about the bbox
        // center). Standalone row (no paired field).
        self.number_cell("R", ids::VECTOR_TRANSFORM_R, self.inner_x, self.inner_w, y);
        y += self.row_h + self.row_gap;
        // "Set Center" — arma a edição de pivô; a próxima pressão no canvas põe a
        // ORIGEM da entidade ali (a geometria desloca junto, a forma não se move).
        // ADR-0112: o pivô nasce no centro da forma; este botão o move.
        let label = if state::pivot_edit_armed() {
            "Click canvas to set center"
        } else {
            "Set Center"
        };
        self.action_button(ids::VECTOR_PIVOT_EDIT, label, y)
    }
}
