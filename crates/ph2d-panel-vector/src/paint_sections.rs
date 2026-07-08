//! Body-section painters for the Vector Style panel, extracted from
//! [`crate::paint`] so the orchestrator fn + the file stay under the panel
//! LOC caps (600/file, 200/fn — `architecture_panel_loc_cap`).
//!
//! [`BodyCtx`] bundles the per-frame mutables (Vello scene, text shaper, widget
//! store, hit index) + the shared layout metrics; each `paint_*` method takes
//! the running `y` and returns the advanced `y`. Pure relocation of the drawing
//! calls — no behavioral change.

use crate::ids;
use crate::state;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, ColorSwatch, SwatchSize, paint_button, paint_color_swatch,
    paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_tool_vector::params::{
    DrawMode, dash_to_slider, gap_to_slider, opacity_to_slider, radius_to_slider, sides_to_slider,
    spiral_turns_to_slider, star_inner_to_slider, star_points_to_slider,
};
use ph2d_tool_vector::{StrokeCap, StrokeJoin, VectorStyleSnapshot, VertexType, px_to_slider};
use ph2d_vector::VectorScene;

/// Label column width for the Width slider row + the Stroke / Fill labels.
pub(crate) const LABEL_COL_W: f32 = 64.0; // LITERAL-PX-OK: panel grid metric (per-panel label gutter width)

/// Per-frame paint context for the Vector Style panel body — the mutable render
/// targets + the shared layout metrics. Constructed once per frame in
/// [`crate::paint`]; each section method borrows disjoint fields.
pub(crate) struct BodyCtx<'a> {
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
    pub store: &'a WidgetStore,
    pub hit_index: &'a mut HitIndex,
    pub theme: Theme,
    pub inner_x: f32,
    pub inner_w: f32,
    pub row_h: f32,
    pub row_gap: f32,
    pub chip_w: f32,
    pub font: f32,
}

impl BodyCtx<'_> {
    /// A full-width slider + linked value chip row; returns the advanced `y`.
    #[allow(clippy::too_many_arguments)]
    fn slider_row(
        &mut self,
        label: &str,
        slider_id: ph2d_a11y::NodeId,
        chip_id: ph2d_a11y::NodeId,
        track: f32,
        val: f64,
        display: &str,
        y: f32,
    ) -> f32 {
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(self.inner_x, y, self.inner_w, self.row_h),
            label,
            track,
            val,
            Some(display),
            slider_id,
            chip_id,
            LABEL_COL_W,
            self.chip_w,
            self.store,
            self.hit_index,
            self.scene,
            self.text_system,
            self.theme,
        );
        y + used + self.row_gap
    }

    /// A labelled 3-across segmented button row (Cap / Join).
    fn segmented3(
        &mut self,
        label: &str,
        opts: [(ph2d_a11y::NodeId, &str, bool); 3],
        mut y: f32,
    ) -> f32 {
        let sd_font = TypeToken::Sm.px();
        let sd_gap = Spacing::Sm.px();
        let sd_w = ((self.inner_w - sd_gap * 2.0) / 3.0).max(1.0); // LITERAL-PX-OK: 3-column segmented grid (button count, not a metric)
        paint_text(
            self.text_system,
            self.scene,
            label,
            self.inner_x,
            y,
            sd_font,
            self.inner_w,
            resolve(ColorToken::Text2, self.theme),
        );
        y += sd_font + Spacing::Xs.px();
        for (i, (id, lbl, active)) in opts.iter().enumerate() {
            let rx = self.inner_x + i as f32 * (sd_w + sd_gap);
            let rect = Rect::new(rx, y, sd_w, self.row_h);
            let st = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            paint_segmented_button(
                rect,
                lbl,
                *active,
                st,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(*id, rect);
        }
        y + self.row_h + self.row_gap
    }

    /// A `Section` label line (Sm, Text2) + its advance.
    pub(crate) fn section_label(&mut self, label: &str, mut y: f32) -> f32 {
        let label_font = TypeToken::Sm.px();
        paint_text(
            self.text_system,
            self.scene,
            label,
            self.inner_x,
            y,
            label_font,
            self.inner_w,
            resolve(ColorToken::Text2, self.theme),
        );
        y += label_font + Spacing::Xs.px();
        y
    }

    /// A full-width action button (Boolean / Vertex-delete / Duplicate).
    fn action_button(&mut self, id: ph2d_a11y::NodeId, label: &str, y: f32) -> f32 {
        let rect = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        let st = self.store.button_state(id).unwrap_or(ButtonState::Normal);
        let btn = Button::new(id, label).kind(ButtonKind::Default).state(st);
        paint_button(&btn, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        y + self.row_h + Spacing::Xs.px()
    }

    /// Width + Stroke swatch + Stroke opacity + Cap / Join + Dash / Gap.
    pub(crate) fn stroke_style(&mut self, snap: &VectorStyleSnapshot, mut y: f32) -> f32 {
        // Width slider + px chip.
        let track = self
            .store
            .slider(ids::VECTOR_WIDTH)
            .map(|(_, v)| v)
            .unwrap_or_else(|| px_to_slider(snap.stroke_width_px));
        let px = self
            .store
            .number_value(ids::VECTOR_WIDTH_NUM)
            .unwrap_or(snap.stroke_width_px);
        y = self.slider_row(
            "Width",
            ids::VECTOR_WIDTH,
            ids::VECTOR_WIDTH_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );

        let swatch_w = SwatchSize::Md.px();
        // Stroke colour swatch.
        paint_text(
            self.text_system,
            self.scene,
            "Stroke",
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let stroke_swatch_rect = Rect::new(
            self.inner_x + self.inner_w - swatch_w,
            y,
            swatch_w,
            self.row_h,
        );
        let stroke_swatch =
            ColorSwatch::new(ids::VECTOR_STROKE_SWATCH, "Stroke color", snap.stroke)
                .size(SwatchSize::Md);
        paint_color_swatch(&stroke_swatch, stroke_swatch_rect, self.scene, self.theme);
        self.hit_index
            .register(ids::VECTOR_STROKE_SWATCH, stroke_swatch_rect);
        y += self.row_h + self.row_gap;

        // Stroke Opacity slider (single source of the stroke alpha).
        let track = self
            .store
            .slider(ids::VECTOR_STROKE_OPACITY)
            .map(|(_, v)| v)
            .unwrap_or_else(|| opacity_to_slider(snap.stroke[3]));
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent for the opacity chip
        y = self.slider_row(
            "Opacity",
            ids::VECTOR_STROKE_OPACITY,
            ids::VECTOR_STROKE_OPACITY_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        );

        // Cap / Join segmented rows.
        y = self.segmented3(
            "Cap",
            [
                (ids::VECTOR_CAP_BUTT, "Butt", snap.cap == StrokeCap::Butt),
                (ids::VECTOR_CAP_ROUND, "Round", snap.cap == StrokeCap::Round),
                (
                    ids::VECTOR_CAP_SQUARE,
                    "Square",
                    snap.cap == StrokeCap::Square,
                ),
            ],
            y,
        );
        y = self.segmented3(
            "Join",
            [
                (
                    ids::VECTOR_JOIN_MITER,
                    "Miter",
                    snap.join == StrokeJoin::Miter,
                ),
                (
                    ids::VECTOR_JOIN_ROUND,
                    "Round",
                    snap.join == StrokeJoin::Round,
                ),
                (
                    ids::VECTOR_JOIN_BEVEL,
                    "Bevel",
                    snap.join == StrokeJoin::Bevel,
                ),
            ],
            y,
        );

        // Dash length (multiple of width; 0 = solid).
        let track = self
            .store
            .slider(ids::VECTOR_DASH)
            .map(|(_, v)| v)
            .unwrap_or_else(|| dash_to_slider(snap.dash));
        let px = self
            .store
            .number_value(ids::VECTOR_DASH_NUM)
            .unwrap_or(snap.dash);
        y = self.slider_row(
            "Dash",
            ids::VECTOR_DASH,
            ids::VECTOR_DASH_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        );

        // Gap length between dashes.
        let track = self
            .store
            .slider(ids::VECTOR_GAP)
            .map(|(_, v)| v)
            .unwrap_or_else(|| gap_to_slider(snap.gap));
        let px = self
            .store
            .number_value(ids::VECTOR_GAP_NUM)
            .unwrap_or(snap.gap);
        self.slider_row(
            "Gap",
            ids::VECTOR_GAP,
            ids::VECTOR_GAP_NUM,
            track,
            px,
            &format!("{}", px.round() as i64),
            y,
        )
    }

    /// Fill swatch + Fill opacity (0 % = none).
    pub(crate) fn fill_style(&mut self, snap: &VectorStyleSnapshot, mut y: f32) -> f32 {
        let swatch_w = SwatchSize::Md.px();
        paint_text(
            self.text_system,
            self.scene,
            "Fill",
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, self.theme),
        );
        let fill_swatch_rect = Rect::new(
            self.inner_x + self.inner_w - swatch_w,
            y,
            swatch_w,
            self.row_h,
        );
        let fill_swatch =
            ColorSwatch::new(ids::VECTOR_FILL_SWATCH, "Fill color", snap.fill).size(SwatchSize::Md);
        paint_color_swatch(&fill_swatch, fill_swatch_rect, self.scene, self.theme);
        self.hit_index
            .register(ids::VECTOR_FILL_SWATCH, fill_swatch_rect);
        y += self.row_h + self.row_gap;

        let track = self
            .store
            .slider(ids::VECTOR_FILL_OPACITY)
            .map(|(_, v)| v)
            .unwrap_or_else(|| opacity_to_slider(snap.fill[3]));
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent for the opacity chip
        self.slider_row(
            "Opacity",
            ids::VECTOR_FILL_OPACITY,
            ids::VECTOR_FILL_OPACITY_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        )
    }

    /// Draw-mode grid (Pen / shapes) + the active mode's per-shape sliders.
    pub(crate) fn draw_modes(&mut self, snap: &VectorStyleSnapshot, mut y: f32) -> f32 {
        y = self.section_label("Draw", y);
        // Seven modes in a 3-column grid: Pen/Rect/Oval · Poly/Star/Round · Spiral.
        let modes = [
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

    /// Vertex type (conditional) + Boolean ops + Arrange (Duplicate + z-order).
    pub(crate) fn vertex_boolean_arrange(&mut self, mut y: f32) -> f32 {
        // Vertex type (rich handle editing) — only with a vertex selected.
        if let Some(vtype) = state::current_vertex_type() {
            y = self.section_label("Vertex", y);
            let verts = [
                (ids::VECTOR_VERT_CORNER, "Corner", VertexType::Corner),
                (ids::VECTOR_VERT_SMOOTH, "Smooth", VertexType::Smooth),
                (ids::VECTOR_VERT_SYMMETRIC, "Symm", VertexType::Symmetric),
            ];
            let vseg_gap = Spacing::Sm.px();
            let vseg_w = ((self.inner_w - vseg_gap * (verts.len() as f32 - 1.0))
                / verts.len() as f32)
                .max(1.0);
            for (i, (id, label, t)) in verts.iter().enumerate() {
                let rx = self.inner_x + i as f32 * (vseg_w + vseg_gap);
                let rect = Rect::new(rx, y, vseg_w, self.row_h);
                let bstate = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
                paint_segmented_button(
                    rect,
                    label,
                    vtype == *t,
                    bstate,
                    self.scene,
                    self.text_system,
                    self.theme,
                );
                self.hit_index.register(*id, rect);
            }
            y += self.row_h + Spacing::Xs.px();

            // Delete-node button (full width). Insert is a canvas gesture (click a
            // segment) — no button.
            y = self.action_button(ids::VECTOR_VERT_DELETE, "Delete Node", y);
            y += self.row_gap - Spacing::Xs.px();
        }

        // Boolean ops — act on the two last closed regions.
        y = self.section_label("Boolean", y);
        for (id, label) in [
            (ids::VECTOR_BOOL_UNION, "Union"),
            (ids::VECTOR_BOOL_SUBTRACT, "Subtract"),
            (ids::VECTOR_BOOL_INTERSECT, "Intersect"),
            (ids::VECTOR_BOOL_EXCLUDE, "Exclude"),
        ] {
            y = self.action_button(id, label, y);
        }
        y += self.row_gap;

        // Arrange — Duplicate + z-order (act on the selected path).
        y = self.section_label("Arrange", y);
        y = self.action_button(ids::VECTOR_ARRANGE_DUPLICATE, "Duplicate", y);
        // Z-order: 2×2 grid — To Back | To Front · Backward | Forward.
        let zorder = [
            (ids::VECTOR_ARRANGE_TO_BACK, "To Back"),
            (ids::VECTOR_ARRANGE_TO_FRONT, "To Front"),
            (ids::VECTOR_ARRANGE_BACKWARD, "Backward"),
            (ids::VECTOR_ARRANGE_FORWARD, "Forward"),
        ];
        let z_cols = 2usize;
        let z_gap = Spacing::Sm.px();
        let z_w = ((self.inner_w - z_gap * (z_cols as f32 - 1.0)) / z_cols as f32).max(1.0);
        let z_top = y;
        for (i, (id, label)) in zorder.iter().enumerate() {
            let rx = self.inner_x + (i % z_cols) as f32 * (z_w + z_gap);
            let ry = z_top + (i / z_cols) as f32 * (self.row_h + z_gap);
            let rect = Rect::new(rx, ry, z_w, self.row_h);
            let bstate = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            let btn = Button::new(*id, *label)
                .kind(ButtonKind::Default)
                .state(bstate);
            paint_button(&btn, rect, self.scene, self.text_system, self.theme);
            self.hit_index.register(*id, rect);
        }
        let z_rows = zorder.len().div_ceil(z_cols) as f32;
        y = z_top + z_rows * self.row_h + (z_rows - 1.0) * z_gap + self.row_gap;

        // Flip (mirror) + Rotate (90°) — each a 2-col row of action buttons.
        y = self.row2(
            z_w,
            z_gap,
            [
                (ids::VECTOR_ARRANGE_FLIP_H, "Flip H"),
                (ids::VECTOR_ARRANGE_FLIP_V, "Flip V"),
            ],
            y,
        );
        self.row2(
            z_w,
            z_gap,
            [
                (ids::VECTOR_ARRANGE_ROTATE_CW, "Rotate CW"),
                (ids::VECTOR_ARRANGE_ROTATE_CCW, "Rotate CCW"),
            ],
            y,
        )
    }

    /// A 2-column row of two half-width action buttons; returns the advanced `y`.
    fn row2(&mut self, w: f32, gap: f32, items: [(ph2d_a11y::NodeId, &str); 2], y: f32) -> f32 {
        for (i, (id, label)) in items.iter().enumerate() {
            let rx = self.inner_x + i as f32 * (w + gap);
            let rect = Rect::new(rx, y, w, self.row_h);
            let bstate = self.store.button_state(*id).unwrap_or(ButtonState::Normal);
            let btn = Button::new(*id, *label)
                .kind(ButtonKind::Default)
                .state(bstate);
            paint_button(&btn, rect, self.scene, self.text_system, self.theme);
            self.hit_index.register(*id, rect);
        }
        y + self.row_h + self.row_gap
    }
}
