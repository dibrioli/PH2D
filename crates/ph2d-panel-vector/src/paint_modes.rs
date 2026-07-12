//! A grade de modos do painel Vector (ADR-0112): Select · Node · Pen · formas.
//!
//! Módulo irmão de `paint_sections` (teto de 600 LOC por arquivo de painel). As duas
//! primeiras opções não desenham nada — Select transforma pela gizmo, Node edita
//! âncoras — e é isso que separa a caneta da manipulação.

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::paint_sections::LABEL_COL_W;
use crate::state;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
use ph2d_editor_core::widget::showcase::read_number_input;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, Dropdown, DropdownOption, NumberInput, paint_button,
    paint_dropdown_chip, paint_number_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};
use ph2d_tool_vector::params::{
    DEFAULT_TEXT_LINE_HEIGHT, DEFAULT_TEXT_SIZE, DEFAULT_TEXT_TRACKING, DEFAULT_TEXT_WEIGHT,
    DrawMode, arc_degrees_to_slider, radius_to_slider, sides_to_slider, spiral_turns_to_slider,
    star_inner_to_slider, star_points_to_slider, text_line_height_to_slider, text_size_to_slider,
    text_tracking_to_slider, text_weight_to_slider,
};
use ph2d_tool_vector::{TextAlign, VectorStyleSnapshot};

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
            (ids::VECTOR_MODE_LINE, "Line", DrawMode::Line),
            (ids::VECTOR_MODE_ARC, "Arc", DrawMode::Arc),
            (ids::VECTOR_MODE_TEXT, "Text", DrawMode::Text),
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
            DrawMode::Arc => {
                let track = self
                    .store
                    .slider(ids::VECTOR_ARC_DEGREES)
                    .map(|(_, v)| v)
                    .unwrap_or_else(|| arc_degrees_to_slider(snap.arc_degrees));
                let val = self
                    .store
                    .number_value(ids::VECTOR_ARC_DEGREES_NUM)
                    .unwrap_or(snap.arc_degrees);
                y = self.slider_row(
                    "Degrees",
                    ids::VECTOR_ARC_DEGREES,
                    ids::VECTOR_ARC_DEGREES_NUM,
                    track,
                    val,
                    &format!("{}", val.round() as i64),
                    y,
                );
            }
            _ => {}
        }
        // A seção Text aparece no modo Text **ou** com um objeto de TEXTO selecionado
        // (as configs do texto valem enquanto ele for texto — não-curva — mesmo na
        // ferramenta Select; a shell publica a visibilidade).
        if state::text_visible() {
            y = self.text_size_and_string(y);
        }
        y
    }

    /// Text-mode controls: the glyph Size slider + a read-only preview of the
    /// active session's string (typed on the canvas — the editable field is A3).
    fn text_size_and_string(&mut self, mut y: f32) -> f32 {
        let track = self
            .store
            .slider(ids::VECTOR_TEXT_SIZE)
            .map(|(_, v)| v)
            .unwrap_or_else(|| text_size_to_slider(DEFAULT_TEXT_SIZE));
        let val = self
            .store
            .number_value(ids::VECTOR_TEXT_SIZE_NUM)
            .unwrap_or(DEFAULT_TEXT_SIZE);
        y = self.slider_row(
            "Size",
            ids::VECTOR_TEXT_SIZE,
            ids::VECTOR_TEXT_SIZE_NUM,
            track,
            val,
            &format!("{val:.2}"),
            y,
        );
        // Weight (`wght` axis) slider — the variable-font weight.
        let wtrack = self
            .store
            .slider(ids::VECTOR_TEXT_WEIGHT)
            .map(|(_, v)| v)
            .unwrap_or_else(|| text_weight_to_slider(DEFAULT_TEXT_WEIGHT));
        let wval = self
            .store
            .number_value(ids::VECTOR_TEXT_WEIGHT_NUM)
            .unwrap_or(DEFAULT_TEXT_WEIGHT);
        y = self.slider_row(
            "Weight",
            ids::VECTOR_TEXT_WEIGHT,
            ids::VECTOR_TEXT_WEIGHT_NUM,
            wtrack,
            wval,
            &format!("{}", wval.round() as i64),
            y,
        );
        y = self.font_picker_row(y);
        y = self.paragraph_section(y);
        y = self.axes_section(y);
        // Read-only string preview (the active session's text, or a hint when empty).
        y = self.section_label("Text", y);
        let text = state::current_text().unwrap_or_default();
        let (shown, color) = if text.trim().is_empty() {
            ("Click the canvas and type".to_owned(), ColorToken::Text2)
        } else {
            (text.replace('\n', " / "), ColorToken::Text1)
        };
        paint_text(
            self.text_system,
            self.scene,
            &shown,
            self.inner_x,
            y,
            self.font,
            self.inner_w,
            resolve(color, self.theme),
        );
        y + self.row_h + self.row_gap
    }

    /// Font-family picker row: `<` prev | **dropdown chip** | `>` next. The arrows
    /// cycle; the chip opens the styled popover (each family drawn in its own
    /// outline — [`crate::font_dropdown`]). The shell owns the family list + fonts.
    fn font_picker_row(&mut self, mut y: f32) -> f32 {
        y = self.section_label("Font", y);
        let btn_w = self.row_h;
        let gap = Spacing::Sm.px();
        self.arrow_button(ids::VECTOR_TEXT_FONT_PREV, "<", self.inner_x, btn_w, y);
        self.arrow_button(
            ids::VECTOR_TEXT_FONT_NEXT,
            ">",
            self.inner_x + self.inner_w - btn_w,
            btn_w,
            y,
        );
        // Center: the font dropdown chip (name + chevron), filling the gap between
        // the arrows. Reads its open flag from the store (toggled by the generic
        // dispatch on click); when open, stash the rect for the deferred popover.
        let chip = Rect::new(
            self.inner_x + btn_w + gap,
            y,
            (self.inner_w - 2.0 * (btn_w + gap)).max(1.0),
            self.row_h,
        );
        let open = matches!(
            self.store.get(ids::VECTOR_TEXT_FONT_DD),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        let name = state::current_text_font().unwrap_or_default();
        let dd = Dropdown::new(
            ids::VECTOR_TEXT_FONT_DD,
            "",
            vec![DropdownOption::new(ids::VECTOR_TEXT_FONT_DD, (), name)],
        )
        .selected(())
        .open(open);
        paint_dropdown_chip(&dd, chip, self.scene, self.text_system, self.theme);
        self.hit_index.register(ids::VECTOR_TEXT_FONT_DD, chip);
        if open {
            state::set_pending_font_dd(Some(chip));
        }
        y += self.row_h + self.row_gap;
        // Import a font file (.ttf/.otf) as the current text font.
        self.action_button(ids::VECTOR_TEXT_FONT_IMPORT, "Import Font...", y)
    }

    /// Paragraph controls (Text mode): alignment L / C / R + Line-height + Tracking.
    fn paragraph_section(&mut self, mut y: f32) -> f32 {
        y = self.section_label("Paragraph", y);
        let align = state::current_text_align().unwrap_or(TextAlign::Left);
        y = self.segmented3(
            "Align",
            [
                (
                    ids::VECTOR_TEXT_ALIGN_LEFT,
                    "Left",
                    align == TextAlign::Left,
                ),
                (
                    ids::VECTOR_TEXT_ALIGN_CENTER,
                    "Center",
                    align == TextAlign::Center,
                ),
                (
                    ids::VECTOR_TEXT_ALIGN_RIGHT,
                    "Right",
                    align == TextAlign::Right,
                ),
            ],
            y,
        );
        // Line-height slider (× size).
        let lh_track = self
            .store
            .slider(ids::VECTOR_TEXT_LINE_HEIGHT)
            .map(|(_, v)| v)
            .unwrap_or_else(|| text_line_height_to_slider(DEFAULT_TEXT_LINE_HEIGHT));
        let lh_val = self
            .store
            .number_value(ids::VECTOR_TEXT_LINE_HEIGHT_NUM)
            .unwrap_or(DEFAULT_TEXT_LINE_HEIGHT);
        y = self.slider_row(
            "Line height",
            ids::VECTOR_TEXT_LINE_HEIGHT,
            ids::VECTOR_TEXT_LINE_HEIGHT_NUM,
            lh_track,
            lh_val,
            &format!("{lh_val:.2}"),
            y,
        );
        // Tracking slider (em fraction).
        let tr_track = self
            .store
            .slider(ids::VECTOR_TEXT_TRACKING)
            .map(|(_, v)| v)
            .unwrap_or_else(|| text_tracking_to_slider(DEFAULT_TEXT_TRACKING));
        let tr_val = self
            .store
            .number_value(ids::VECTOR_TEXT_TRACKING_NUM)
            .unwrap_or(DEFAULT_TEXT_TRACKING);
        self.slider_row(
            "Tracking",
            ids::VECTOR_TEXT_TRACKING,
            ids::VECTOR_TEXT_TRACKING_NUM,
            tr_track,
            tr_val,
            &format!("{tr_val:.2}"),
            y,
        )
    }

    /// Variation Axes (Text mode): one number field per non-`wght` axis the current
    /// font exposes (Optical Size / Width / Slant / …). Hidden for static fonts. The
    /// field value + range are seeded by the shell each frame (paint Phase B).
    fn axes_section(&mut self, mut y: f32) -> f32 {
        let names: Vec<String> =
            state::with_text_axes(|axes| axes.iter().map(|a| a.name.clone()).collect());
        if names.is_empty() {
            return y;
        }
        y = self.section_label("Axes", y);
        for (i, name) in names.iter().enumerate() {
            y = self.axis_field(name, ids::vector_text_axis_id(i), y);
        }
        y
    }

    /// One labelled variation-axis number field (`<axis name> [ value ]`).
    fn axis_field(&mut self, name: &str, id: ph2d_a11y::NodeId, y: f32) -> f32 {
        let gap = Spacing::Sm.px();
        paint_text(
            self.text_system,
            self.scene,
            name,
            self.inner_x,
            y + (self.row_h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            LABEL_COL_W,
            resolve(ColorToken::Text2, self.theme),
        );
        let field_x = self.inner_x + LABEL_COL_W + gap;
        let field_w = (self.inner_w - LABEL_COL_W - gap).max(1.0);
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
        y + self.row_h + self.row_gap
    }

    /// A small square action button (used by the font-picker `<` / `>`).
    fn arrow_button(&mut self, id: ph2d_a11y::NodeId, label: &str, x: f32, w: f32, y: f32) {
        let rect = Rect::new(x, y, w, self.row_h);
        let st = self.store.button_state(id).unwrap_or(ButtonState::Normal);
        let btn = Button::new(id, label).kind(ButtonKind::Default).state(st);
        paint_button(&btn, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
    }
}
