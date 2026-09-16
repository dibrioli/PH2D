//! **A seção ALIGN / DISTRIBUTE** do painel de Vector — irmã de `paint_arrange`.
//!
//! ⚠️ Saiu de `paint_arrange.rs` em 2026-09-16, quando a migração dos rótulos para a tabela de
//! strings (as chamadas `tr(…)` partem linhas no `rustfmt`) o levou a `655` contra o tecto de `600`.
//! O corte é por ASSUNTO: alinhar objectos não é arrumá-los em Z.

use crate::paint_sections::BodyCtx;
use crate::state;
use ph2d_editor_core::widget::{Button, ButtonKind, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;

impl BodyCtx<'_> {
    /// "Align" + "Distribute" section — shown only with a multi-path OBJECT
    /// selection (≥2 for Align, ≥3 for Distribute). Two 3-col rows (X-align then
    /// Y-align) + a 2-col Distribute row. Buttons drive the shell drain.
    pub(crate) fn align_section(&mut self, y: f32) -> f32 {
        let count = state::current_selection_count();
        if count < 2 {
            return y;
        }
        let (mut y, collapsed) = self.section_header(
            ph2d_tool_vector::ids::VECTOR_SECTION_ALIGN,
            tr("panel.vector.section.align"),
            y,
        );
        if collapsed {
            return y;
        }
        let gap = Spacing::Xs.px();
        let cols = 3usize;
        let cw = ((self.inner_w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
        let rows = [
            [
                (
                    ph2d_tool_vector::ids::VECTOR_ALIGN_LEFT,
                    tr("panel.vector.arrange.left"),
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_ALIGN_HCENTER,
                    tr("panel.vector.arrange.center"),
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_ALIGN_RIGHT,
                    tr("panel.vector.arrange.right"),
                ),
            ],
            [
                (
                    ph2d_tool_vector::ids::VECTOR_ALIGN_TOP,
                    tr("panel.vector.arrange.top"),
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_ALIGN_VCENTER,
                    tr("panel.vector.arrange.middle"),
                ),
                (
                    ph2d_tool_vector::ids::VECTOR_ALIGN_BOTTOM,
                    tr("panel.vector.arrange.bottom"),
                ),
            ],
        ];
        for row in rows {
            for (i, (id, label)) in row.iter().enumerate() {
                let rx = self.inner_x + i as f32 * (cw + gap);
                let rect = Rect::new(rx, y, cw, self.row_h);
                let bstate = self.store.button_visual(*id);
                let btn = Button::new(*id, *label)
                    .kind(ButtonKind::Default)
                    .visual(bstate);
                paint_button(&btn, rect, self.scene, self.text_system, self.theme);
                self.hit_index.register(*id, rect);
            }
            y += self.row_h + self.row_gap;
        }
        // Distribute needs ≥3 paths (two are always the fixed extremes).
        if count >= 3 {
            let two_col = ((self.inner_w - gap) / 2.0).max(1.0);
            y = self.row2(
                two_col,
                gap,
                [
                    (
                        ph2d_tool_vector::ids::VECTOR_DISTRIBUTE_H,
                        tr("panel.vector.arrange.dist_h"),
                    ),
                    (
                        ph2d_tool_vector::ids::VECTOR_DISTRIBUTE_V,
                        tr("panel.vector.arrange.dist_v"),
                    ),
                ],
                y,
            );
        }
        y + self.row_gap
    }
}
