//! Path-reshape subsection painter for the Vector Style panel: the Smooth /
//! Sharpen / Simplify / Subdivide buttons that act on ALL vertices of the
//! selected path. Split from `paint_sections` to keep that file under the
//! 600-LOC panel cap; it's an `impl BodyCtx` block over there.

use crate::paint_sections::BodyCtx;
use crate::state::FillKind;
use crate::{ids, state};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::{Button, ButtonKind, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;

/// Full turn in degrees (Angle slider track `0..1` maps to `0..FULL_TURN_DEG`).
const FULL_TURN_DEG: f64 = 360.0; // LITERAL-PX-OK: degrees in a full turn (math constant)

impl BodyCtx<'_> {
    /// Fill-type selector (Solid / Linear / Radial) + the Linear angle slider —
    /// acts on the SELECTED path. Hidden when no path is selected / has no fill.
    pub(crate) fn fill_type_controls(&mut self, mut y: f32) -> f32 {
        let Some(kind) = state::current_fill_kind() else {
            return y;
        };
        y = self.section_label("Fill Type", y);
        let kinds = [
            (ids::VECTOR_FILL_KIND_SOLID, "Solid", FillKind::Solid),
            (ids::VECTOR_FILL_KIND_LINEAR, "Linear", FillKind::Linear),
            (ids::VECTOR_FILL_KIND_RADIAL, "Radial", FillKind::Radial),
            (ids::VECTOR_FILL_KIND_MULTI, "Multi", FillKind::MultiPoint),
        ];
        let gap = Spacing::Sm.px();
        let cw = ((self.inner_w - gap * (kinds.len() as f32 - 1.0)) / kinds.len() as f32).max(1.0);
        for (i, (id, label, k)) in kinds.iter().enumerate() {
            let rx = self.inner_x + i as f32 * (cw + gap);
            let rect = Rect::new(rx, y, cw, self.row_h);
            let bstate = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            paint_segmented_button(
                rect,
                label,
                kind == *k,
                bstate,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(*id, rect);
        }
        y += self.row_h + self.row_gap;

        // Multi-point: Add / Remove point (drag points on-canvas to move them) +
        // Influence slider for the selected point.
        if kind == FillKind::MultiPoint {
            let two_col = ((self.inner_w - gap) / 2.0).max(1.0);
            y = self.row2(
                two_col,
                gap,
                [
                    (ids::VECTOR_GRAD_ADD_POINT, "Add Point"),
                    (ids::VECTOR_GRAD_REMOVE_POINT, "Remove Point"),
                ],
                y,
            );
            if let Some(inf) = state::current_grad_influence() {
                const MAX_INFLUENCE: f64 = 4.0; // LITERAL-PX-OK: IDW strength range (domain)
                let track = self
                    .store
                    .slider(ids::VECTOR_GRAD_INFLUENCE)
                    .map(|(_, v)| v)
                    .unwrap_or((inf / MAX_INFLUENCE) as f32);
                let val = f64::from(track) * MAX_INFLUENCE;
                y = self.slider_row(
                    "Influence",
                    ids::VECTOR_GRAD_INFLUENCE,
                    ids::VECTOR_GRAD_INFLUENCE_NUM,
                    track,
                    val,
                    &format!("{val:.1}"),
                    y,
                );
            }
            // Jitter (per-texel grain, 0..1) for the selected point — track == value.
            if let Some(jit) = state::current_grad_jitter() {
                let track = self
                    .store
                    .slider(ids::VECTOR_GRAD_JITTER)
                    .map(|(_, v)| v)
                    .unwrap_or(jit as f32);
                let val = f64::from(track);
                y = self.slider_row(
                    "Jitter",
                    ids::VECTOR_GRAD_JITTER,
                    ids::VECTOR_GRAD_JITTER_NUM,
                    track,
                    val,
                    &format!("{val:.2}"),
                    y,
                );
            }
        }
        // Linear/Radial: Add / Remove ramp stop (drag interior stops on-canvas).
        if kind == FillKind::Linear || kind == FillKind::Radial {
            let two_col = ((self.inner_w - gap) / 2.0).max(1.0);
            y = self.row2(
                two_col,
                gap,
                [
                    (ids::VECTOR_GRAD_ADD_STOP, "Add Stop"),
                    (ids::VECTOR_GRAD_REMOVE_STOP, "Remove Stop"),
                ],
                y,
            );
        }
        // Angle slider (Linear only) — track 0..1 → 0..360°.
        if kind == FillKind::Linear {
            let angle = state::current_grad_angle().unwrap_or(0.0);
            let track = self
                .store
                .slider(ids::VECTOR_GRAD_ANGLE)
                .map(|(_, v)| v)
                .unwrap_or((angle / FULL_TURN_DEG) as f32);
            let deg = f64::from(track) * FULL_TURN_DEG;
            y = self.slider_row(
                "Angle",
                ids::VECTOR_GRAD_ANGLE,
                ids::VECTOR_GRAD_ANGLE_NUM,
                track,
                deg,
                &format!("{}", deg.round() as i64),
                y,
            );
        }
        y
    }
    /// "Align" + "Distribute" section — shown only with a multi-path OBJECT
    /// selection (≥2 for Align, ≥3 for Distribute). Two 3-col rows (X-align then
    /// Y-align) + a 2-col Distribute row. Buttons drive the shell drain.
    pub(crate) fn align_section(&mut self, mut y: f32) -> f32 {
        let count = state::current_selection_count();
        if count < 2 {
            return y;
        }
        y = self.section_label("Align", y);
        let gap = Spacing::Sm.px();
        let cols = 3usize;
        let cw = ((self.inner_w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
        let rows = [
            [
                (ids::VECTOR_ALIGN_LEFT, "Left"),
                (ids::VECTOR_ALIGN_HCENTER, "Center"),
                (ids::VECTOR_ALIGN_RIGHT, "Right"),
            ],
            [
                (ids::VECTOR_ALIGN_TOP, "Top"),
                (ids::VECTOR_ALIGN_VCENTER, "Middle"),
                (ids::VECTOR_ALIGN_BOTTOM, "Bottom"),
            ],
        ];
        for row in rows {
            for (i, (id, label)) in row.iter().enumerate() {
                let rx = self.inner_x + i as f32 * (cw + gap);
                let rect = Rect::new(rx, y, cw, self.row_h);
                let bstate = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
                let btn = Button::new(*id, *label)
                    .kind(ButtonKind::Default)
                    .state(bstate);
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
                    (ids::VECTOR_DISTRIBUTE_H, "Dist H"),
                    (ids::VECTOR_DISTRIBUTE_V, "Dist V"),
                ],
                y,
            );
        }
        y + self.row_gap
    }

    /// "Path" section — reshape the whole selected path. Smooth / Sharpen are a
    /// 2-col row; Simplify (fewer points) / Subdivide (more points) are the
    /// point-density pair on a second 2-col row; then a full-width Close/Open
    /// toggle (label from the published `closed` flag). `w`/`gap` are the shared
    /// Arrange column metrics; returns the advanced `y`.
    pub(crate) fn path_section(&mut self, w: f32, gap: f32, mut y: f32) -> f32 {
        y = self.section_label("Path", y);
        y = self.row2(
            w,
            gap,
            [
                (ids::VECTOR_PATH_SMOOTH, "Smooth"),
                (ids::VECTOR_PATH_SHARPEN, "Sharpen"),
            ],
            y,
        );
        y = self.row2(
            w,
            gap,
            [
                (ids::VECTOR_PATH_SIMPLIFY, "Simplify"),
                (ids::VECTOR_PATH_SUBDIVIDE, "Subdivide"),
            ],
            y,
        );
        // Close/Open toggle — label reflects the current state (default "Close"
        // when no path is selected / not yet closed).
        let label = if state::current_path_closed() == Some(true) {
            "Open Path"
        } else {
            "Close Path"
        };
        self.action_button(ids::VECTOR_PATH_CLOSE, label, y)
    }
}
