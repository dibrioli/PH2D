//! Numeric Transform section painter for the Vector Style panel: the X/Y
//! (position) and W/H (size) fields of the selected path's bbox. Split from
//! `paint_sections` to keep that file under the 600-LOC panel cap; it's an
//! `impl BodyCtx` block over there.

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::state;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::showcase::read_number_input;
use ph2d_editor_core::widget::{NumberInput, paint_number_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};

impl BodyCtx<'_> {
    /// Numeric Transform — X/Y (position) + W/H (size) of the selected path's
    /// anchor bbox, two 2-column rows. Hidden when no path is selected.
    pub(crate) fn transform_section(&mut self, mut y: f32) -> f32 {
        if state::current_transform().is_none() {
            return y;
        }
        y = self.section_label("Transform", y);
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

    /// A 2-column row of two labelled number fields (X | Y, W | H).
    fn number_row(
        &mut self,
        la: &str,
        ida: ph2d_a11y::NodeId,
        lb: &str,
        idb: ph2d_a11y::NodeId,
        y: f32,
    ) -> f32 {
        let gap = Spacing::Sm.px();
        let cw = ((self.inner_w - gap) / 2.0).max(1.0);
        self.number_cell(la, ida, self.inner_x, cw, y);
        self.number_cell(lb, idb, self.inner_x + cw + gap, cw, y);
        y + self.row_h + self.row_gap
    }

    /// One labelled number field (`<label> [ value ]`) in a cell; registers hit.
    fn number_cell(&mut self, label: &str, id: ph2d_a11y::NodeId, cx: f32, cw: f32, y: f32) {
        let lab_w = Spacing::Md.px();
        paint_text(
            self.text_system,
            self.scene,
            label,
            cx,
            y + (self.row_h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            lab_w,
            resolve(ColorToken::Text2, self.theme),
        );
        let field_x = cx + lab_w + Spacing::Xs.px();
        let field_w = (cw - lab_w - Spacing::Xs.px()).max(1.0);
        let rect = Rect::new(field_x, y, field_w, self.row_h);
        self.hit_index.register(id, rect);
        let (state, value, buffer, caret, anchor) = read_number_input(self.store, id);
        let input = NumberInput::new(id, "", value).step(1.0).state(state);
        paint_number_input_with_buffer(
            &input,
            Some(buffer),
            caret,
            anchor,
            rect,
            self.scene,
            self.text_system,
            self.theme,
        );
    }
}
