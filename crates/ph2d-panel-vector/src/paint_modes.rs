//! A grade de modos do painel Vector (ADR-0112): Select · Node · Pen · formas.
//!
//! Módulo irmão de `paint_sections` (teto de 600 LOC por arquivo de painel). As duas
//! primeiras opções não desenham nada — Select transforma pela gizmo, Node edita
//! âncoras — e é isso que separa a caneta da manipulação.

use crate::ids;
use crate::paint_sections::BodyCtx;
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::{
    DrawMode, radius_to_slider, sides_to_slider, spiral_turns_to_slider, star_inner_to_slider,
    star_points_to_slider,
};

impl BodyCtx<'_> {
    /// Draw-mode grid (Pen / shapes) + the active mode's per-shape sliders.
    pub(crate) fn draw_modes(&mut self, snap: &VectorStyleSnapshot, mut y: f32) -> f32 {
        y = self.section_label("Draw", y);
        // Nine modes in a 3-column grid. As duas primeiras não desenham: Select
        // transforma pelo gizmo, Node edita âncoras (ADR-0112).
        let modes = [
            (ids::VECTOR_MODE_SELECT, "Select", DrawMode::Select),
            (ids::VECTOR_MODE_NODE, "Node", DrawMode::Node),
            (ids::VECTOR_MODE_PEN, "Pen", DrawMode::Pen),
            (ids::VECTOR_MODE_RECT, "Rect", DrawMode::Rectangle),
            (ids::VECTOR_MODE_ELLIPSE, "Oval", DrawMode::Ellipse),
            (ids::VECTOR_MODE_POLYGON, "Poly", DrawMode::Polygon),
            (ids::VECTOR_MODE_STAR, "Star", DrawMode::Star),
            (ids::VECTOR_MODE_RRECT, "Round", DrawMode::RoundRect),
            (ids::VECTOR_MODE_SPIRAL, "Spiral", DrawMode::Spiral),
        ];
        let mode_cols = 3usize;
        let seg_gap = Spacing::Sm.px();
        let seg_w =
            ((self.inner_w - seg_gap * (mode_cols as f32 - 1.0)) / mode_cols as f32).max(1.0);
        let mode_top = y;
        for (i, (id, label, m)) in modes.iter().enumerate() {
            let rx = self.inner_x + (i % mode_cols) as f32 * (seg_w + seg_gap);
            let ry = mode_top + (i / mode_cols) as f32 * (self.row_h + seg_gap);
            let rect = Rect::new(rx, ry, seg_w, self.row_h);
            let state = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            paint_segmented_button(
                rect,
                label,
                snap.mode == *m,
                state,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(*id, rect);
        }
        let mode_rows = modes.len().div_ceil(mode_cols) as f32;
        y = mode_top + mode_rows * self.row_h + (mode_rows - 1.0) * seg_gap + self.row_gap;

        // Per-shape sliders (only for the active shape mode).
        match snap.mode {
            DrawMode::Polygon => {
                let track = self
                    .store
                    .slider(ids::VECTOR_SIDES)
                    .map(|(_, v)| v)
                    .unwrap_or_else(|| sides_to_slider(snap.polygon_sides));
                let val = self
                    .store
                    .number_value(ids::VECTOR_SIDES_NUM)
                    .unwrap_or(f64::from(snap.polygon_sides));
                y = self.slider_row(
                    "Sides",
                    ids::VECTOR_SIDES,
                    ids::VECTOR_SIDES_NUM,
                    track,
                    val,
                    &format!("{}", val.round() as i64),
                    y,
                );
            }
            DrawMode::Star => {
                let pt_track = self
                    .store
                    .slider(ids::VECTOR_STAR_POINTS)
                    .map(|(_, v)| v)
                    .unwrap_or_else(|| star_points_to_slider(snap.star_points));
                let pt_val = self
                    .store
                    .number_value(ids::VECTOR_STAR_POINTS_NUM)
                    .unwrap_or(f64::from(snap.star_points));
                y = self.slider_row(
                    "Points",
                    ids::VECTOR_STAR_POINTS,
                    ids::VECTOR_STAR_POINTS_NUM,
                    pt_track,
                    pt_val,
                    &format!("{}", pt_val.round() as i64),
                    y,
                );
                let in_track = self
                    .store
                    .slider(ids::VECTOR_STAR_INNER)
                    .map(|(_, v)| v)
                    .unwrap_or_else(|| star_inner_to_slider(snap.star_inner_ratio));
                let in_val = self
                    .store
                    .number_value(ids::VECTOR_STAR_INNER_NUM)
                    .unwrap_or(snap.star_inner_ratio);
                y = self.slider_row(
                    "Inner",
                    ids::VECTOR_STAR_INNER,
                    ids::VECTOR_STAR_INNER_NUM,
                    in_track,
                    in_val,
                    &format!("{in_val:.2}"),
                    y,
                );
            }
            DrawMode::RoundRect => {
                let track = self
                    .store
                    .slider(ids::VECTOR_RRECT_RADIUS)
                    .map(|(_, v)| v)
                    .unwrap_or_else(|| radius_to_slider(snap.corner_radius_px));
                let val = self
                    .store
                    .number_value(ids::VECTOR_RRECT_RADIUS_NUM)
                    .unwrap_or(snap.corner_radius_px);
                y = self.slider_row(
                    "Radius",
                    ids::VECTOR_RRECT_RADIUS,
                    ids::VECTOR_RRECT_RADIUS_NUM,
                    track,
                    val,
                    &format!("{}", val.round() as i64),
                    y,
                );
            }
            DrawMode::Spiral => {
                let track = self
                    .store
                    .slider(ids::VECTOR_SPIRAL_TURNS)
                    .map(|(_, v)| v)
                    .unwrap_or_else(|| spiral_turns_to_slider(snap.spiral_turns));
                let val = self
                    .store
                    .number_value(ids::VECTOR_SPIRAL_TURNS_NUM)
                    .unwrap_or(f64::from(snap.spiral_turns));
                y = self.slider_row(
                    "Turns",
                    ids::VECTOR_SPIRAL_TURNS,
                    ids::VECTOR_SPIRAL_TURNS_NUM,
                    track,
                    val,
                    &format!("{}", val.round() as i64),
                    y,
                );
            }
            _ => {}
        }
        y
    }
}
