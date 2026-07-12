//! O seletor de MODO + o seletor de FORMA do painel Vector — ambos data-driven.
//!
//! Módulo irmão de `paint_sections` (teto de 600 LOC por arquivo de painel).
//!
//! Quatro modos, e só (ADR-0112): **Select** (o gizmo manda) · **Node** (edita âncoras)
//! · **Pen** (cria) · **Text**. As FORMAS não são modos — são um **catálogo**: o painel
//! itera `ph2d_tool_vector::shapes`, pinta um botão por forma (id gerado,
//! `vector_shape_id(i)`) e, abaixo, um campo por parâmetro da forma em foco (rótulo,
//! faixa e passo vindos do descritor). Nada aqui sabe que existe estrela, seta ou balão
//! — e é por isso que uma forma nova custa uma linha de tabela, não uma seção de painel.

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
    DrawMode, text_line_height_to_slider, text_size_to_slider, text_tracking_to_slider,
    text_weight_to_slider,
};
use ph2d_tool_vector::shapes;
use ph2d_tool_vector::{TextAlign, VectorStyleSnapshot};

impl BodyCtx<'_> {
    /// A grade de modos (Select / Node / Pen / Text) + o catálogo de formas + os campos
    /// da forma em foco.
    pub(crate) fn draw_modes(&mut self, snap: &VectorStyleSnapshot, mut y: f32) -> f32 {
        y = self.section_label("Tool", y);
        let modes = [
            (ids::VECTOR_MODE_SELECT, "Select", DrawMode::Select),
            (ids::VECTOR_MODE_NODE, "Node", DrawMode::Node),
            (ids::VECTOR_MODE_PEN, "Pen", DrawMode::Pen),
            (ids::VECTOR_MODE_TEXT, "Text", DrawMode::Text),
        ];
        y = self.button_grid(y, 2, modes.len(), |i| {
            let (id, label, m) = modes[i];
            (id, label, snap.mode == m)
        });

        // ── O catálogo de formas ──────────────────────────────────────────────
        // A família é uma aba; o grid mostra só as formas dela (com 25+ formas um grid
        // plano seria ilegível). O botão da forma ATIVA fica aceso.
        y = self.section_label("Shape", y);
        let groups = shapes::ALL_GROUPS;
        // Sem escolha explícita de aba, ela segue a forma ativa — voltar à tool mostra a
        // família do que se está desenhando.
        let active_group = state::current_shape_group().unwrap_or(shapes::desc(snap.shape).group);
        if groups.len() > 1 {
            y = self.button_grid(y, 3, groups.len(), |i| {
                (
                    ids::vector_shape_group_id(i),
                    groups[i].label(),
                    groups[i] == active_group,
                )
            });
        }
        let in_group: Vec<(usize, &shapes::ShapeDesc)> = shapes::SHAPES
            .iter()
            .enumerate()
            .filter(|(_, d)| d.group == active_group)
            .collect();
        y = self.button_grid(y, 3, in_group.len(), |i| {
            let (cat_index, d) = in_group[i];
            (
                ids::vector_shape_id(cat_index),
                d.label,
                snap.shape == d.kind && snap.mode == DrawMode::Shape,
            )
        });

        // ── Os parâmetros da forma em foco ────────────────────────────────────
        // A forma em foco é a VIVA selecionada, quando há uma (a shell publica) — assim
        // os campos editam a forma que está na tela, mesmo na ferramenta Select. Sem
        // seleção, é a forma ativa do catálogo (o default do próximo traço).
        let focus = state::shape_focus(state::current_shape_focus(), snap.shape);
        for (i, f) in shapes::desc(focus).fields.iter().enumerate() {
            y = self.labeled_number_field(f.label, ids::vector_shape_field_id(i), f.step, y);
        }

        // A seção Text aparece no modo Text **ou** com um objeto de TEXTO selecionado (a
        // shell publica a visibilidade).
        if state::text_visible() {
            y = self.text_size_and_string(y);
        }
        y
    }

    /// Uma grade de botões segmentados de `cols` colunas — o desenho compartilhado do
    /// seletor de modo, das abas de família e do catálogo de formas.
    fn button_grid(
        &mut self,
        y: f32,
        cols: usize,
        n: usize,
        item: impl Fn(usize) -> (ph2d_a11y::NodeId, &'static str, bool),
    ) -> f32 {
        if n == 0 {
            return y;
        }
        let gap = Spacing::Sm.px();
        let w = ((self.inner_w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
        for i in 0..n {
            let (id, label, active) = item(i);
            let rx = self.inner_x + (i % cols) as f32 * (w + gap);
            let ry = y + (i / cols) as f32 * (self.row_h + gap);
            let rect = Rect::new(rx, ry, w, self.row_h);
            let st = self.store.button_state(id).unwrap_or(ButtonState::Normal);
            paint_segmented_button(
                rect,
                label,
                active,
                st,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(id, rect);
        }
        let rows = n.div_ceil(cols) as f32;
        y + rows * self.row_h + (rows - 1.0) * gap + self.row_gap
    }

    /// Um campo numérico rotulado (`<rótulo> [ valor ]`), largura cheia — os parâmetros
    /// de forma e os eixos de variação da fonte compartilham este desenho. **Caixa, não
    /// slider:** as faixas variam demais entre formas (3 lados · 500 px de raio · 360°)
    /// para um knob servir a todas.
    fn labeled_number_field(
        &mut self,
        label: &str,
        id: ph2d_a11y::NodeId,
        step: f64,
        y: f32,
    ) -> f32 {
        let gap = Spacing::Sm.px();
        paint_text(
            self.text_system,
            self.scene,
            label,
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
        let (st, value, buffer, caret, anchor) = read_number_input(self.store, id);
        let input = NumberInput::new(id, "", value).step(step).state(st);
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
    /// font exposes. Hidden for static fonts.
    fn axes_section(&mut self, mut y: f32) -> f32 {
        let names: Vec<String> =
            state::with_text_axes(|axes| axes.iter().map(|a| a.name.clone()).collect());
        if names.is_empty() {
            return y;
        }
        y = self.section_label("Axes", y);
        for (i, name) in names.iter().enumerate() {
            y = self.labeled_number_field(name, ids::vector_text_axis_id(i), 1.0, y);
        }
        y
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
