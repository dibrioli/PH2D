//! **As seções de TEXTO do painel** — `TEXT` · `FONT` · `PARAGRAPH` · `AXES`.
//!
//! Irmão de [`super::paint_modes`] pelo teto de 600 LOC do painel. O corte é na fronteira que o
//! cabeçalho dele já declarava (*"o seletor de MODO + os PARÂMETROS da forma em foco + as seções
//! de TEXTO"*): as quatro seções aqui partilham o gate `state::text_visible()` e descrevem como um
//! TEXTO se compõe, não que ferramenta está na mão.

use crate::ids;
use crate::paint_sections::BodyCtx;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::{
    Button, ButtonKind, Dropdown, DropdownOption, paint_button, paint_dropdown_chip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing};
use ph2d_tool_vector::TextAlign;
use ph2d_tool_vector::params::{
    DEFAULT_TEXT_LINE_HEIGHT, DEFAULT_TEXT_SIZE, DEFAULT_TEXT_TRACKING, DEFAULT_TEXT_WEIGHT,
    text_line_height_to_slider, text_size_to_slider, text_tracking_to_slider,
    text_weight_to_slider, text_wrap_to_slider,
};

use crate::state;

impl BodyCtx<'_> {
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
        let dd_visual = self.store.dropdown_visual(ids::VECTOR_TEXT_FONT_DD);
        let name = state::current_text_font().unwrap_or_default();
        let dd = Dropdown::new(
            ids::VECTOR_TEXT_FONT_DD,
            "",
            vec![DropdownOption::new(ids::VECTOR_TEXT_FONT_DD, (), name)],
        )
        .selected(())
        .open(open)
        .visual(dd_visual);
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
        y = self.slider_row(
            "Tracking",
            ids::VECTOR_TEXT_TRACKING,
            ids::VECTOR_TEXT_TRACKING_NUM,
            tr_track,
            tr_val,
            &format!("{tr_val:.2}"),
            y,
        );
        self.wrap_rows(y)
    }

    /// **Width: Auto | Fixed** + a largura, quando ela existe.
    ///
    /// ⚠️ **Só UMA row viva de cada vez**, e é a lei do knob-morto: em `Auto` não há largura
    /// nenhuma a editar, e um slider ali seria um controle que não faz nada. É a forma do
    /// `Mass: Auto | Manual` do editor de áudio — *duas portas para uma grandeza* é o que se
    /// evita, e aqui a grandeza tem presença E valor.
    fn wrap_rows(&mut self, y: f32) -> f32 {
        let wrap = state::current_text_wrap();
        let mut y = self.segmented(
            "Width",
            &[
                (ids::VECTOR_TEXT_WRAP_AUTO, "Auto", wrap.is_none()),
                (ids::VECTOR_TEXT_WRAP_FIXED, "Fixed", wrap.is_some()),
            ],
            y,
        );
        let Some(w) = wrap else { return y };
        let track = self
            .store
            .slider(ids::VECTOR_TEXT_WRAP_W)
            .map(|(_, v)| v)
            .unwrap_or_else(|| text_wrap_to_slider(w));
        let val = self
            .store
            .number_value(ids::VECTOR_TEXT_WRAP_W_NUM)
            .unwrap_or(w);
        y = self.slider_row(
            "Wrap width",
            ids::VECTOR_TEXT_WRAP_W,
            ids::VECTOR_TEXT_WRAP_W_NUM,
            track,
            val,
            &format!("{val:.2}"),
            y,
        );
        y
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
        let st = self.store.button_visual(id);
        let btn = Button::new(id, label).kind(ButtonKind::Default).visual(st);
        paint_button(&btn, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
    }
}
