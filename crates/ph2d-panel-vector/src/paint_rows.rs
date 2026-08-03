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
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::showcase::{paint_section_separator, read_number_input};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, Checkbox, CheckboxValue, NumberInput, paint_button,
    paint_checkbox, paint_number_input_with_buffer, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};

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

    /// **Uma linha de CHECKBOX** — caixa à esquerda, rótulo à direita.
    ///
    /// ⚠️ Este painel dizia booleano com um `segmented` Off/On (o *Clip* da moldura), e este é o
    /// primeiro checkbox de verdade dele. A diferença não é estética: um `segmented` é uma escolha
    /// entre MODOS nomeados, e um checkbox é uma PROPRIEDADE que o objeto tem ou não tem — e
    /// *Resize Box* é a segunda (o Enio pediu-o assim). O primitivo é o do design system
    /// ([`paint_checkbox`]), o mesmo que o Inspector e o painel do Painter usam, então não há uma
    /// segunda resposta a *"como é uma caixa de marcar neste app?"*.
    pub(crate) fn checkbox_row(
        &mut self,
        id: ph2d_a11y::NodeId,
        label: &str,
        checked: bool,
        y: f32,
    ) -> f32 {
        let value = if checked {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        };
        let cb = Checkbox::new(id, label).value(value);
        let rect = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        paint_checkbox(&cb, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        y + self.row_h + self.row_gap
    }

    /// **Uma linha de texto e nada mais** — um readout, sem id e sem hit-rect.
    ///
    /// ⚠️ Ela existe porque a alternativa seria pintar um botão desativado, e *dimmed que
    /// despacha mente* — ou pior, dimmed que **não** despacha e ainda parece clicável. Um facto
    /// que o artista precisa de ler (a instância cujo mestre sumiu) é uma FRASE; um facto sobre
    /// que ele pode agir é um botão. Sem id, o `architecture_panel_wiring_parity` não tem o que
    /// exigir aqui, e é correto: não há nada a registar.
    pub(crate) fn label_line(&mut self, text: &str, y: f32) -> f32 {
        paint_text(
            self.text_system,
            self.scene,
            text,
            self.inner_x,
            y + (self.row_h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            self.inner_w,
            resolve(ColorToken::Text2, self.theme),
        );
        y + self.row_h + Spacing::Xs.px()
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

    /// **Uma linha de DOIS campos numéricos rotulados** (X | Y, W | H, Gap principal | transversal).
    ///
    /// ⚠️ Ela nasceu privada no `paint_transform` e mudou-se para cá quando o AUTO LAYOUT virou o
    /// **segundo** consumidor — que é exatamente o momento em que um ajudante deve ir para a casa
    /// partilhada. Deixá-la lá obrigaria a seção nova a reescrever a aritmética de meia-largura, e
    /// duas respostas para *"onde este campo senta?"* divergem no dia em que uma delas ganhar um
    /// rótulo mais largo.
    pub(crate) fn number_row(
        &mut self,
        la: &str,
        ida: ph2d_a11y::NodeId,
        lb: &str,
        idb: ph2d_a11y::NodeId,
        y: f32,
    ) -> f32 {
        let cw = self.half_cell_w();
        self.number_cell(la, ida, self.inner_x, cw, y);
        self.number_cell(lb, idb, self.inner_x + cw + Spacing::Sm.px(), cw, y);
        y + self.row_h + self.row_gap
    }

    /// **A largura de UMA célula da grade de dois** — a mesma expressão para quem preenche as
    /// duas e para quem preenche só a da esquerda. Duas cópias divergem no dia em que o vão
    /// mudar, e o resultado é uma fileira desalinhada da de cima.
    fn half_cell_w(&self) -> f32 {
        ((self.inner_w - Spacing::Sm.px()) / 2.0).max(1.0)
    }

    /// **Um campo numérico SOZINHO**, na célula esquerda da mesma grade de dois.
    ///
    /// ⚠️ Ele ocupa meia largura e não a linha inteira, e isto é o report do Enio (2026-08-02:
    /// *"caixas de input numérico grande"*): um número curto — um vão, um recuo — numa caixa
    /// da largura do painel lê como o controlo mais importante da seção, quando é o menor. A
    /// grade já existe (X | Y, W | H); um campo solto senta numa célula dela.
    pub(crate) fn lone_number_row(&mut self, label: &str, id: ph2d_a11y::NodeId, y: f32) -> f32 {
        let cw = self.half_cell_w();
        self.number_cell(label, id, self.inner_x, cw, y);
        y + self.row_h + self.row_gap
    }

    /// Um campo numérico rotulado (`<rótulo> [ valor ]`) numa célula; regista o hit.
    ///
    /// ⚠️ **A calha do rótulo é MEDIDA, com `Spacing::Md` de PISO.** Ela era o piso e mais nada
    /// — oito pixels, o tamanho de um caractere —, e isso bastou enquanto todo rótulo desta
    /// função era `X`/`Y`/`W`/`H`. O AUTO LAYOUT trouxe `Gap`, `All`, `Grow` e `Shrink`, e o
    /// resultado foi o report do Enio: *"label sobreposta"* — o texto recortado em `G` e o
    /// campo desenhado por cima do resto (`paint_text` recebe a calha como largura máxima).
    ///
    /// Medir em vez de escolher uma constante maior é o que mantém `X`/`Y` **onde sempre
    /// estiveram**: um caractere mede menos que o piso, então a seção Transform não se move.
    pub(crate) fn number_cell(
        &mut self,
        label: &str,
        id: ph2d_a11y::NodeId,
        cx: f32,
        cw: f32,
        y: f32,
    ) {
        let lab_w = self
            .text_system
            .prefix_width(label, TypeToken::Sm.px())
            .max(Spacing::Md.px());
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
