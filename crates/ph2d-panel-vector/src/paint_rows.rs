//! **Os PRIMITIVOS de LINHA do painel Vector** — irmão do [`super::paint_sections`] pelo teto de
//! 600 LOC dos painéis (o do `architecture_panel_loc_cap`, que o
//! `architecture_workspace_file_loc_cap` **não** cobre — o gotcha que a `line/physics` pagou).
//!
//! O corte é por RESPONSABILIDADE, e a fronteira é limpa: ali mora *o que cada SEÇÃO diz*
//! (Stroke, Fill, Boolean, Arrange), aqui *como uma LINHA é desenhada* (o par slider+chip, o
//! rádio de três, o separador, o botão de ação, o par de meia-largura). Uma seção nova é escrita
//! com estes tijolos; um tijolo novo serve todas as seções — e foi a row de **Align** que levou o
//! arquivo ao teto.

use super::paint_sections::{BodyCtx, label_col_w};
use ph2d_editor_core::paint::{paint_text, paint_text_block, resolve};
use ph2d_editor_core::widget::{
    Button, ButtonKind, Checkbox, CheckboxValue, paint_button, paint_checkbox,
    paint_slider_with_chip_layout_adaptive,
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
            label_col_w(self.inner_x, self.inner_w),
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
        self.with_rows(|r| r.separator(y))
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
        let cb = Checkbox::new(id, label)
            .visual(self.store.checkbox_visual(id))
            .value(value);
        let rect = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        paint_checkbox(&cb, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        y + self.row_h + self.row_gap
    }

    /// **Uma linha `rótulo … swatch de cor`** — a porta única das linhas de cor deste painel.
    ///
    /// ⚠️ `a11y` é separado do `label` de propósito: o rótulo é o que o artista LÊ na coluna da
    /// esquerda (*"Color"*, *"Color B"*, *"Colour (own)"*) e o `a11y` é o que o leitor de tela
    /// anuncia sobre a swatch. Colapsá-los faria um leitor de tela dizer *"Color"* para três
    /// swatches de assuntos diferentes.
    ///
    /// ⚠️ E ela **não regista o widget no store** — a swatch é alvo de PICKER
    /// (`register_picker_swatch`), o que é decisão do `populate`; aqui só se pinta e se põe o
    /// hit-rect.
    pub(crate) fn colour_swatch_row(
        &mut self,
        id: ph2d_a11y::NodeId,
        colour: [u8; 4],
        label: &str,
        y: f32,
    ) -> f32 {
        self.colour_swatch_row_rect(id, colour, label, y).0
    }

    /// ⭐⭐⭐ **A MESMA linha, devolvendo também o RECT que a swatch ocupou.**
    ///
    /// ⚠️ Ela existe porque duas das quatro linhas de cor deste painel desenham a **rachura do
    /// token** por cima da swatch, e para isso precisam do rectângulo. Sem esta metade as duas
    /// copiavam a montagem inteira — *que é exactamente o que elas faziam*, e é por isso que a lei
    /// da largura tinha **quatro** redacções e nenhuma régua.
    ///
    /// ⛔⛔ **A geometria é da PORTA da casa** ([`ph2d_editor_core::property_row::paint_color_row`])
    /// e nunca deste painel. O que havia aqui era um quadrado de `32 px` colado à direita, com a
    /// largura vinda do `SwatchSize::Md` — a aresta que o doc do `ColorSwatch` declara ser a
    /// **sugestão para uma amostra de PALETA** (*«callers may still hand any rect»*). Medido, todo
    /// vizinho deste painel começa na coluna do controlo e só a swatch começava `94 px` depois.
    ///
    /// ⚠️⚠️ **O `y` que a porta devolve é DESCARTADO, de propósito:** o passo dela é
    /// `ROW_H_PX + control_gap_px()` (`25`) e o deste painel é `row_h + row_gap` (`26`). Misturar
    /// dois passos dentro de uma secção é um defeito maior do que a largura que esta wave veio
    /// corrigir — *a porta decide a GEOMETRIA da fileira; o painel continua dono do RITMO dele.*
    pub(crate) fn colour_swatch_row_rect(
        &mut self,
        id: ph2d_a11y::NodeId,
        colour: [u8; 4],
        label: &str,
        y: f32,
    ) -> (f32, Rect) {
        ph2d_editor_core::property_row::paint_color_row(
            self.scene,
            self.text_system,
            self.theme,
            self.hit_index,
            self.store,
            self.inner_x,
            self.inner_w,
            y,
            label,
            id,
            colour,
            false,
            ph2d_editor_core::property_row::Seccao::apenas_campos(1),
        );
        (y + self.row_h + self.row_gap, self.caixa_da_swatch(y))
    }

    /// ⭐⭐ **O rect que a porta deu à swatch** — para quem precisa de desenhar POR CIMA dela.
    ///
    /// ⛔ Ela delega na porta: *uma segunda aritmética para a mesma coluna diverge no dia em que a
    /// porta mudar, e a rachura do token passa a cair ao lado da swatch.*
    pub(crate) fn caixa_da_swatch(&self, y: f32) -> Rect {
        ph2d_editor_core::property_row::caixa_do_controlo(
            self.inner_x,
            self.inner_w,
            y,
            ph2d_editor_core::property_row::Seccao::apenas_campos(1),
        )
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
        y + self.row_h + ph2d_tokens::control_gap_px()
    }

    /// **Uma DICA** — uma frase esmaecida que explica algo que o app mediu, e que pode quebrar em
    /// mais de uma linha.
    ///
    /// ⚠️⚠️ **É `paint_text_block` e não `paint_text`, e a diferença é um defeito que outro painel
    /// já pagou** (smoke do Enio sobre o 9-slice, 2026-08-22): uma dica é uma FRASE, e numa coluna
    /// estreita ela quebra — avançar a altura de UMA linha escreve a fileira seguinte por cima
    /// dela. É por isso que a `label_line` (um readout curto, de uma linha) não serve aqui.
    ///
    /// ⚠️ `Text3` e não `Text2`: o `ColorToken` diz-se *"tertiary/hints"*, e uma dica não pode
    /// competir com os rótulos dos controlos que ela comenta. ⛔ E **não** é `Warn` — o app está
    /// correcto e o padrão desenha o que foi pedido; o que se comenta é a ARTE.
    pub(crate) fn hint_line(&mut self, text: &str, y: f32) -> f32 {
        let font = TypeToken::Sm.px();
        let usado = paint_text_block(
            self.text_system,
            self.scene,
            text,
            self.inner_x,
            y + (self.row_h - font) * 0.5,
            font,
            self.inner_w,
            resolve(ColorToken::Text3, self.theme),
        );
        // O piso é a altura de uma row: uma dica de uma linha ocupa o que uma linha sempre ocupou,
        // e só o excedente da quebra é acrescentado.
        y + (self.row_h - font).mul_add(0.5, usado).max(self.row_h) + ph2d_tokens::control_gap_px()
    }

    /// **O mesmo readout, num RECT dado, e sem avançar o `y`** — para quando a linha partilha a
    /// altura com botões à direita.
    ///
    /// ⚠️ Ela delega no mesmo `paint_text` da irmã de propósito: duas leis de *onde o texto assenta
    /// na altura da row* divergiriam no primeiro ajuste de tipografia, e o artista veria duas
    /// linhas vizinhas desalinhadas por um pixel.
    pub(crate) fn label_line_in(&mut self, text: &str, rect: Rect) {
        paint_text(
            self.text_system,
            self.scene,
            text,
            rect.x,
            rect.y + (rect.h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            rect.w,
            resolve(ColorToken::Text2, self.theme),
        );
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
        self.with_rows(|r| r.action_button_kind(id, label, kind, y))
    }

    // ⛔ **O `labeled_action_button` saiu daqui** (2026-09-09): o único chamador era a secção do
    // esqueleto, que passou a ter painel próprio. A composição continua a existir **uma vez**, na
    // porta partilhada ([`ph2d_editor_core::panel::RowCtx`]) — o que sai é a delegação sem
    // chamador, que é a espécie de código morto que o `dead_code` nomeia.

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
        // ⭐⭐⭐ **A fileira de DUAS células PARTE quando o nome deixa de caber** — spec §6-quater,
        // report do dono de 2026-09-15 sobre a fileira gémea do Painter: *«os nomes somem ao
        // estreitar o painel. Melhor seria quebrar a linha»*. ⛔ A pergunta vai à PORTA.
        let cw = self.half_cell_w();
        if self.par_cabe(la, lb, cw) {
            self.number_cell(la, ida, self.inner_x, cw, y);
            self.number_cell(lb, idb, self.inner_x + cw + Spacing::Sm.px(), cw, y);
            return y + self.row_h + self.row_gap;
        }
        let y = self.lone_number_row_full(la, ida, y);
        self.lone_number_row_full(lb, idb, y)
    }

    /// **Os dois nomes cabem, cada um na sua metade?** — ver a spec §6-quater.
    fn par_cabe(&mut self, la: &str, lb: &str, cw: f32) -> bool {
        let font = TypeToken::Sm.px();
        let wa = self.text_system.prefix_width(la, font);
        let wb = self.text_system.prefix_width(lb, font);
        ph2d_editor_core::property_row::property_row_fits(cw, wa)
            && ph2d_editor_core::property_row::property_row_fits(cw, wb)
    }

    /// Um campo numérico sozinho, na LINHA INTEIRA — o degrau a que a fileira de duas cai.
    fn lone_number_row_full(&mut self, label: &str, id: ph2d_a11y::NodeId, y: f32) -> f32 {
        self.number_cell(label, id, self.inner_x, self.inner_w, y);
        y + self.row_h + self.row_gap
    }

    /// ⭐⭐⭐ **A LINHA DE PROPRIEDADE do app — um nome, N caixas.**
    ///
    /// ⛔⛔ **Ordem do dono, 2026-09-15:** *«funciona mas não segue o mesmo padrão inspector para o
    /// layout. Coloque no padrão: Position X/Y Quadro Quadro. Rotation com nome completo. Veja no
    /// inspector»*.
    ///
    /// ⚠️ A [`Self::number_row`] ao lado é OUTRA coisa e fica: ela põe **duas propriedades
    /// diferentes** lado a lado, cada uma com o seu nome (a grade de dois deste painel). Esta põe
    /// **uma** propriedade com N componentes, que é o padrão do Inspector.
    pub(crate) fn fields_row(
        &mut self,
        label: &str,
        ids: &[ph2d_a11y::NodeId],
        unit: Option<ph2d_editor_core::widget::Unit>,
        y: f32,
        // ⭐⭐ **A coluna é da SECÇÃO** — ver [`ph2d_editor_core::property_row::Seccao`].
        sec: ph2d_editor_core::property_row::Seccao,
    ) -> f32 {
        ph2d_editor_core::property_row::paint_fields_row(
            self.scene,
            self.text_system,
            self.theme,
            self.hit_index,
            self.store,
            self.inner_x,
            self.inner_w,
            y,
            label,
            ids,
            1.0, // LITERAL-PX-OK: passo de scrub de um campo do vector
            unit,
            sec,
        )
    }

    /// **A largura de UMA célula da grade de dois** — a mesma expressão para quem preenche as
    /// duas e para quem preenche só a da esquerda. Duas cópias divergem no dia em que o vão
    /// mudar, e o resultado é uma fileira desalinhada da de cima.
    pub(crate) fn half_cell_w(&self) -> f32 {
        ((self.inner_w - Spacing::Sm.px()) / 2.0).max(1.0)
    }

    /// **Um campo numérico SOZINHO**, na célula esquerda da mesma grade de dois.
    ///
    /// ⚠️ Ele ocupa meia largura e não a linha inteira, e isto é o report do Enio (2026-08-02:
    /// *"caixas de input numérico grande"*): um número curto — um vão, um recuo — numa caixa
    /// da largura do painel lê como o controlo mais importante da seção, quando é o menor. A
    /// grade já existe (X | Y, W | H); um campo solto senta numa célula dela.
    pub(crate) fn lone_number_row(&mut self, label: &str, id: ph2d_a11y::NodeId, y: f32) -> f32 {
        // ⚠️ **E ele também cai para a linha inteira quando a metade não chega** — a decisão de
        // 2026-08-02 era sobre a caixa ser GRANDE demais, e uma metade onde o nome não cabe já não
        // é um campo pequeno: é um campo sem nome.
        let cw = self.half_cell_w();
        let font = TypeToken::Sm.px();
        let quer = self.text_system.prefix_width(label, font);
        if ph2d_editor_core::property_row::property_row_fits(cw, quer) {
            self.number_cell(label, id, self.inner_x, cw, y);
            return y + self.row_h + self.row_gap;
        }
        self.lone_number_row_full(label, id, y)
    }

    /// Um campo numérico rotulado numa célula — **pela PORTA do app**; devolve o rect do campo.
    ///
    /// ⛔⛔ **A calha do rótulo era MEDIDA POR LINHA, e isso é a coluna ESFARRAPADA que o manual
    /// proíbe** (spec §6: *«uma coluna por linha põe cada controlo num `x` diferente»*). Ela nasceu
    /// assim para o `X`/`Y`/`W`/`H` — um caractere — e o AUTO LAYOUT trouxe `Gap`, `All`, `Grow`,
    /// `Shrink`, cada um a empurrar o campo dele para outro sítio.
    ///
    /// ⇒ hoje a coluna é a do app (`property_row`), e com ela vêm as três leis que o dono pediu em
    /// 14 e 15 de Setembro: o nome **à direita**, a coluna que **cede** ao controlo antes de a linha
    /// quebrar, e o ponto da coluna de animação.
    ///
    /// ⚠️ **Devolve o retângulo do CAMPO** (W4c.4): quem precisa desenhar por cima dele — a
    /// rachura de *«um token cobre este número»* — tem de receber a caixa que o pintor usou, e ela
    /// vem da MESMA chamada que a desenhou.
    pub(crate) fn number_cell(
        &mut self,
        label: &str,
        id: ph2d_a11y::NodeId,
        cx: f32,
        cw: f32,
        y: f32,
    ) -> Rect {
        // ⏳⏳ **DÍVIDA NOMEADA: aqui a coluna é a da LINHA, não a da secção.**
        //
        // ⛔ Report do dono, 2026-09-15 (*«a caixa recua quando na verdade o nome deveria criar as
        // colunas»*): a cura é a [`ph2d_editor_core::property_row::Seccao`], e ela pede que a
        // secção declare os nomes que pinta. ⚠️ **Este painel não tem secções no sentido do
        // Inspector:** a `number_cell` é uma CÉLULA de uma grade de duas, chamada de ~33 sítios em
        // 39 secções, e metade delas escolhe entre meia largura e a linha inteira **por linha**
        // (ver [`Self::lone_number_row`]). *Declarar a secção aqui seria declarar uma que não
        // existe* — a conversão é uma wave própria, com as secções deste painel definidas primeiro.
        let sec = ph2d_editor_core::property_row::Seccao::medida(self.text_system, 1, &[label]);
        ph2d_editor_core::property_row::paint_field_row(
            self.scene,
            self.text_system,
            self.theme,
            self.hit_index,
            self.store,
            cx,
            cw,
            y,
            label,
            id,
            1.0, // LITERAL-PX-OK: passo de scrub de um campo do vector
            None,
            sec,
        )
        .1
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
            let bstate = self.store.button_visual(*id);
            let btn = Button::new(*id, *label)
                .kind(ButtonKind::Default)
                .visual(bstate);
            paint_button(&btn, rect, self.scene, self.text_system, self.theme);
            self.hit_index.register(*id, rect);
        }
        y + self.row_h + self.row_gap
    }
}
