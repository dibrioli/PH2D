//! **Os PRIMITIVOS de LINHA do painel Vector** — irmão do [`super::paint_sections`] pelo teto de
//! 600 LOC dos painéis (o do `architecture_panel_loc_cap`, que o
//! `architecture_workspace_file_loc_cap` **não** cobre — o gotcha que a `line/physics` pagou).
//!
//! O corte é por RESPONSABILIDADE, e a fronteira é limpa: ali mora *o que cada SEÇÃO diz*
//! (Stroke, Fill, Boolean, Arrange), aqui *como uma LINHA é desenhada* (o par slider+chip, o
//! rádio de três, o separador, o botão de ação, o par de meia-largura). Uma seção nova é escrita
//! com estes tijolos; um tijolo novo serve todas as seções — e foi a row de **Align** que levou o
//! arquivo ao teto.

use super::paint_sections::{BodyCtx, LABEL_COL_W};
use ph2d_editor_core::widget::showcase::paint_section_separator;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, paint_button, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;

impl BodyCtx<'_> {
    /// A full-width slider + linked value chip row; returns the advanced `y`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn slider_row(
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

    /// A labelled 3-across segmented button row (Cap / Join / text Align).
    ///
    /// ⚠️ **Delega ao [`Self::segmented`]**, que é a mesma fileira com um número de botões
    /// arbitrário: a aritmética de largura dos dois era literalmente a mesma expressão
    /// (`(inner_w − gap·(n−1))/n` colapsa em `(inner_w − gap·2)/3` para `n = 3`), escrita duas
    /// vezes. Duas respostas para *"onde cada botão desta fileira senta?"* divergem no dia em
    /// que uma delas ganhar wrap, e a outra continuar medindo pela regra velha — que é
    /// exatamente como o painel do impasto acabou pintando por cima dos próprios botões.
    /// Este método fica pelo tipo: um array de três não pode ter tamanho errado.
    pub(crate) fn segmented3(
        &mut self,
        label: &str,
        opts: [(ph2d_a11y::NodeId, &str, bool); 3],
        y: f32,
    ) -> f32 {
        self.segmented(label, &opts, y)
    }

    /// A linha canônica ENTRE seções (nunca dentro de uma).
    pub(crate) fn separator(&mut self, y: f32) -> f32 {
        paint_section_separator(self.scene, self.theme, self.inner_x, self.inner_w, y)
    }

    /// A full-width action button (Boolean / Vertex-delete / Duplicate).
    pub(crate) fn action_button(&mut self, id: ph2d_a11y::NodeId, label: &str, y: f32) -> f32 {
        self.action_button_kind(id, label, ButtonKind::Default, y)
    }

    /// The same, but with a chosen `kind` — an **Accent** button is how a *commit* action (o
    /// "Apply" da pilha de efeitos) se destaca das edições comuns à sua volta.
    pub(crate) fn action_button_kind(
        &mut self,
        id: ph2d_a11y::NodeId,
        label: &str,
        kind: ButtonKind,
        y: f32,
    ) -> f32 {
        let rect = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        let st = self.store.button_state(id).unwrap_or(ButtonState::Normal);
        let btn = Button::new(id, label).kind(kind).state(st);
        paint_button(&btn, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        y + self.row_h + Spacing::Xs.px()
    }

    /// A 2-column row of two half-width action buttons; returns the advanced `y`.
    pub(crate) fn row2(
        &mut self,
        w: f32,
        gap: f32,
        items: [(ph2d_a11y::NodeId, &str); 2],
        y: f32,
    ) -> f32 {
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
