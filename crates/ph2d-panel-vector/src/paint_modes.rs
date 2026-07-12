//! O seletor de MODO + os PARÂMETROS da forma em foco + as seções de TEXTO.
//!
//! Módulo irmão de `paint_sections` (teto de 600 LOC por arquivo de painel). O CATÁLOGO
//! de formas (categoria + grade de thumbnails) mora em `paint_catalog`.
//!
//! **Seis modos** (ADR-0112 + o 5º pill da reforma + o **Connect**): **Select** (o gizmo
//! manda) · **Node** (edita âncoras) · **Pen** (cria) · **Shape** (desenha a forma ATIVA
//! do catálogo) · **Text** · **Connect** (liga duas formas com uma linha que as segue).
//! As FORMAS não são modos — são um **catálogo**; escolher uma põe a tool em `Shape`, e
//! por isso o modo Shape precisa de um pill: sem ele a fileira de modos ficava TODA
//! apagada justamente enquanto se desenhava uma forma. O **conector**, ao contrário, É um
//! modo: a geometria dele não é autorada, é derivada da relação entre duas formas.
//!
//! Os campos de parâmetro vivem numa seção IRMÃ cujo **título é o NOME da forma em foco**
//! (`STAR`, `GEAR`, `SPEECH`…) — é assim que o usuário descobre a quem os campos
//! pertencem; soltos, como estavam, não diziam nada.

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
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};
use ph2d_tool_vector::params::{
    DEFAULT_TEXT_LINE_HEIGHT, DEFAULT_TEXT_SIZE, DEFAULT_TEXT_TRACKING, DEFAULT_TEXT_WEIGHT,
    DrawMode, text_line_height_to_slider, text_size_to_slider, text_tracking_to_slider,
    text_weight_to_slider,
};
use ph2d_tool_vector::shapes;
use ph2d_tool_vector::{TextAlign, VectorStyleSnapshot};

impl BodyCtx<'_> {
    /// Seção **TOOL** — os seis modos, numa grade de 3 colunas
    /// (`Select | Node | Pen` · `Shape | Text | Connect`).
    pub(crate) fn tool_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_TOOL, tr("panel.vector.section.tool"), y);
        if collapsed {
            return y;
        }
        let modes = [
            (
                ids::VECTOR_MODE_SELECT,
                tr("panel.vector.mode.select"),
                DrawMode::Select,
            ),
            (
                ids::VECTOR_MODE_NODE,
                tr("panel.vector.mode.node"),
                DrawMode::Node,
            ),
            (
                ids::VECTOR_MODE_PEN,
                tr("panel.vector.mode.pen"),
                DrawMode::Pen,
            ),
            (
                ids::VECTOR_MODE_SHAPE,
                tr("panel.vector.mode.shape"),
                DrawMode::Shape,
            ),
            (
                ids::VECTOR_MODE_TEXT,
                tr("panel.vector.mode.text"),
                DrawMode::Text,
            ),
            (
                ids::VECTOR_MODE_CONNECT,
                tr("panel.vector.mode.connect"),
                DrawMode::Connect,
            ),
        ];
        let cols = 3usize;
        self.button_grid(y, cols, modes.len(), |i| {
            let (id, label, m) = modes[i];
            (id, label, snap.mode == m)
        })
    }

    /// Seção dos **PARÂMETROS** da forma em foco — o cabeçalho É o nome dela
    /// (`paint_section_header` já pinta em MAIÚSCULAS), o que responde "de quem são estes
    /// campos?" sem gastar uma linha a mais.
    ///
    /// **De quem são os campos** é decidido em [`crate::shape_focus`] (as três respostas:
    /// forma viva · catálogo · ninguém). `None` ⇒ a seção INTEIRA some — foi selecionado
    /// algo que não é forma viva (um conector, uma curva comum), e um campo editável que
    /// não edita nada é pior que campo nenhum.
    ///
    /// Formas sem campo (Rect / Decision / Terminal / Delay / Junction…) mostram uma
    /// linha "No parameters": a seção nunca parece quebrada.
    pub(crate) fn shape_params_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let Some(focus) = crate::shape_focus::resolved(snap) else {
            return y;
        };
        let desc = shapes::desc(focus);
        let (mut y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_SHAPE_PARAMS, desc.label, y);
        if collapsed {
            return y;
        }
        if desc.fields.is_empty() {
            paint_text(
                self.text_system,
                self.scene,
                tr("panel.vector.shape.no_params"),
                self.inner_x,
                y + (self.row_h - TypeToken::Sm.px()) * 0.5,
                TypeToken::Sm.px(),
                self.inner_w,
                resolve(ColorToken::Text3, self.theme),
            );
            return y + self.row_h + self.row_gap;
        }
        for (i, f) in desc.fields.iter().enumerate() {
            let id = ph2d_editor_core::ids::vector_shape_field_id(i);
            y = match f.unit {
                // Uma ESCOLHA não é um número. Um campo mostrando "Viewed: 1" não diz nada;
                // um botão dizendo "From below" diz tudo — e clicar nele cicla.
                shapes::FieldUnit::Choice(_) => {
                    let cur = self.store.number_value(id).unwrap_or(0.0);
                    let text = shapes::choice_label(focus, i, cur).unwrap_or("");
                    // O HIT vai no gêmeo botão; o valor continua morando no slot numérico.
                    let btn = ph2d_editor_core::ids::vector_shape_choice_id(i);
                    self.labeled_choice_button(f.label, btn, text, y)
                }
                _ => self.labeled_number_field(f.label, id, f.step, y),
            };
        }
        y
    }

    /// Um campo de ESCOLHA: rótulo + um botão que mostra a opção corrente e CICLA ao clique.
    /// Ocupa a mesma coluna da caixa numérica, então a seção continua alinhada.
    ///
    /// `pub(crate)`: a seção do CONECTOR (`paint_connector`) pinta o campo **Route** com este
    /// mesmo desenho — ela é irmã da seção de parâmetros de forma, não um enxerto.
    pub(crate) fn labeled_choice_button(
        &mut self,
        label: &str,
        id: ph2d_a11y::NodeId,
        current: &str,
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
        paint_segmented_button(
            rect,
            current,
            false,
            self.store.button_state(id).unwrap_or_default(),
            self.scene,
            self.text_system,
            self.theme,
        );
        y + self.row_h + self.row_gap
    }

    /// Uma grade de botões segmentados de `cols` colunas — o desenho compartilhado do
    /// seletor de modo e da grade de tipos do catálogo (que empilha o thumbnail por cima
    /// da mesma chrome).
    pub(crate) fn button_grid(
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
    /// de forma, os eixos de variação da fonte e os campos do CONECTOR (Jetty / Spread)
    /// compartilham este desenho. **Caixa, não slider:** as faixas variam demais entre
    /// formas (3 lados · 500 px de raio · 360°) para um knob servir a todas.
    pub(crate) fn labeled_number_field(
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

    /// Seção **TEXT** — o Size + o Weight (variável) da sessão viva, e um preview
    /// read-only da string (a digitação acontece no canvas). Aparece no modo Text **ou**
    /// com um objeto de TEXTO selecionado (a shell publica a visibilidade).
    pub(crate) fn text_section(&mut self, y: f32) -> f32 {
        if !state::text_visible() {
            return y;
        }
        let (mut y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_TEXT, tr("panel.vector.section.text"), y);
        if collapsed {
            return y;
        }
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
        // Read-only string preview (the active session's text, or a hint when empty).
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

    /// Seção **FONT** — `<` prev | **dropdown chip** | `>` next + Import. As setas
    /// ciclam; o chip abre o popover estilizado (cada família no PRÓPRIO contorno —
    /// [`crate::font_dropdown`]). A shell é dona da lista de famílias + das fontes.
    pub(crate) fn font_section(&mut self, y: f32) -> f32 {
        if !state::text_visible() {
            return y;
        }
        let (mut y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_FONT, tr("panel.vector.section.font"), y);
        if collapsed {
            return y;
        }
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

    /// Seção **PARAGRAPH** — alinhamento L / C / R + Line-height + Tracking.
    pub(crate) fn paragraph_section(&mut self, y: f32) -> f32 {
        if !state::text_visible() {
            return y;
        }
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_PARAGRAPH,
            tr("panel.vector.section.paragraph"),
            y,
        );
        if collapsed {
            return y;
        }
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

    /// Seção **AXES** — um campo por eixo de variação (fora o `wght`, que tem slider
    /// próprio) que a fonte corrente expõe. Some para fontes estáticas.
    pub(crate) fn axes_section(&mut self, y: f32) -> f32 {
        if !state::text_visible() {
            return y;
        }
        let names: Vec<String> =
            state::with_text_axes(|axes| axes.iter().map(|a| a.name.clone()).collect());
        if names.is_empty() {
            return y;
        }
        let (mut y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_AXES, tr("panel.vector.section.axes"), y);
        if collapsed {
            return y;
        }
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
