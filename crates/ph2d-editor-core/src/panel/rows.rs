//! ⭐⭐⭐ **O VOCABULÁRIO DE LINHAS de um painel de propriedades — uma lei, N hospedeiros.**
//!
//! Um cabeçalho de secção, um botão de acção, um botão rotulado, um campo numérico rotulado e uma
//! fileira de segmentos. Cada um é uma *composição* dos primitivos do design system
//! ([`crate::widget`]) com o **ritmo** desta casa: a coluna de rótulo, o vão entre controlos, a
//! altura de linha.
//!
//! # ⛔⛔ Por que ele saiu do painel de vetor
//!
//! Ele nasceu lá dentro, como `BodyCtx`, e ficou correcto enquanto **um** painel o usava. Em
//! 2026-09-09 o esqueleto ganhou painel próprio (ordem do dono) e passaram a ser **dois** — e a
//! escolha era duplicar ~200 linhas de composição ou nomear a lei uma vez.
//!
//! ⚠️ **O que se duplicaria não é «código»: é o RITMO.** Duas cópias divergem na primeira vez que
//! alguém mexe num vão, e o sintoma é um painel que *parece* de outro app — a lição que o
//! `ph2d-tokens` já escreveu (*«a cara de um app é a tabela de tokens»*).
//!
//! # ⚠️ Ele vive em `panel/` e NÃO em `widget/`
//!
//! Um `widget` é um primitivo com desenho próprio e entra no showcase (há gate). Isto é
//! **composição** — nada aqui desenha uma forma que os primitivos já não desenhem —, e o sítio dela
//! é ao lado do [`super::PaintCtx`], que é a outra metade da infra-estrutura de painel.
//!
//! # ⚠️ Os 147 chamadores do painel de vetor NÃO mudaram
//!
//! O `BodyCtx` de lá manteve os métodos e passou a **delegar**. *Uma extracção que obriga 147
//! sítios a mudar de nome no mesmo commit é uma extracção que colide com toda linha viva.*

use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::resolve;
use crate::widget::panel_chrome::SECTION_LABEL_TO_CONTROL_PX;
use crate::widget::section_cards::close_section;
use crate::widget::showcase::read_number_input;
use crate::widget::{
    Button, ButtonKind, NumberInput, SectionFold, SectionHeader, paint_button,
    paint_number_input_with_buffer, paint_section_header,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// **A coluna de RÓTULO** de uma linha rotulada — ela **pergunta a porta**, e deixou de ser um
/// número escrito aqui (2026-09-14).
///
/// ⛔⛔ **Era `64.0`, e a mesma pergunta tinha ONZE respostas no app** (`96` · `78` · `64` · `84` ·
/// `76` · `72` · `150` · `176`), cada uma um literal com dispensa. Uma largura FIXA está errada por
/// construção: a coluna docada é arrastável (`WidgetStore::DOCK_W_MIN`..`720`), logo um número
/// afinado à largura de omissão come o controlo numa coluna estreita e deixa-o absurdo numa larga.
///
/// ⭐ **É esta função que serve vários painéis de uma vez**, porque o [`RowCtx`] é a linha
/// partilhada — converter o número aqui alinha todos os que passam por ele.
///
/// ⚠️ **Ela pede `x` e `w` e mais nada** (2026-09-14): nasceu a pedir também `y`/`row_h` por
/// simetria com a porta grande, e o resultado nunca os leu. *Um parâmetro que o resultado ignora é
/// um convite a supor que ele importa* — e foi essa suposição que deixou o painel de vetor fora da
/// conversão por um dia, com «muitos sítios sem `y`/`row_h` em alcance» escrito como se fosse preço.
///
/// Ver [`crate::widget::property_label_col_w`] para a derivação da fracção.
#[must_use]
pub fn label_col_w(inner_x: f32, inner_w: f32) -> f32 {
    crate::widget::property_label_col_w(inner_x, inner_w)
}

/// ⭐⭐⭐ **A COLUNA DO VALOR** de uma linha rotulada — o rect onde o controlo começa e acaba.
///
/// ⛔⛔ **Ordem do dono (2026-09-21): *«quanto ao alinhamento precisamos melhorar em todos os
/// lugares»*.** Medido pela varredura `onde_comeca_o_valor` à largura dele (`300 px`): o painel de
/// vetor punha os marcadores a `124` e as fileiras vizinhas a `128`, **4 px de degrau dentro do
/// mesmo painel** — e a causa era uma só, repetida à mão: `inner_x + label_col_w + Spacing::Xs`.
/// A porta da linha ([`crate::widget::property_row_columns`]) usa o vão `Spacing::Md` **e**
/// desconta a coluna de animação à direita; cada cópia acertava a metade de uma coisa e nenhuma
/// das duas.
///
/// ⇒ **o valor começa onde a porta diz, e acaba onde a porta diz.** Esta função é essa porta com
/// o vertical que a linha já tem à mão — *uma coluna escrita em N sítios não é uma coluna, é N
/// palpites que coincidem enquanto ninguém mexe num vão*.
#[must_use]
pub fn value_col(inner_x: f32, inner_w: f32, y: f32, row_h: f32) -> Rect {
    crate::widget::property_row_columns(inner_x, inner_w, y, row_h).control
}

/// **O contexto de uma linha** — os alvos mutáveis do quadro mais as métricas partilhadas.
///
/// ⚠️ **A `store` é `&` e não `&mut`, de propósito:** o passe de pintura de um painel **não escreve
/// estado de widget**. Quem precisa de escrever fá-lo no passe diferido, que é o único sítio com a
/// loja mutável na mão.
pub struct RowCtx<'a> {
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
    pub store: &'a WidgetStore,
    pub hit_index: &'a mut HitIndex,
    pub theme: Theme,
    pub inner_x: f32,
    pub inner_w: f32,
    pub row_h: f32,
    pub row_gap: f32,
    pub font: f32,
    /// **A dobra da secção que está a ser pintada AGORA**, guardada entre o
    /// [`RowCtx::section_header`] que a abre e o [`RowCtx::close_fold`] que a fecha.
    pub open_fold: Option<SectionFold>,
}

impl RowCtx<'_> {
    /// **O cabeçalho de uma secção.** Devolve `(y do corpo, está FECHADA E PARADA)`.
    ///
    /// ⚠️ O `bool` não é o `is_collapsed`: é *fechada **e parada***. Ao clicar para fechar, o flag
    /// semântico vira neste quadro e o `t` ainda desce — um corpo gateado no flag sumiria de repente
    /// por baixo de um chevron a rodar.
    pub fn section_header(&mut self, id: NodeId, label: &str, y: f32) -> (f32, bool) {
        let header_h = TypeToken::Md.px() + Spacing::Md.px();
        let collapsed = self.store.is_collapsed(id);
        let header = SectionHeader::new(id, label)
            .collapsible(!collapsed)
            .open_t(self.store.section_open_live(id));
        let rect = Rect::new(self.inner_x, y, self.inner_w, header_h);
        paint_section_header(&header, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        let body_top = y + header_h + SECTION_LABEL_TO_CONTROL_PX;
        self.open_fold = SectionFold::begin(
            self.store,
            id,
            self.inner_x,
            self.inner_w,
            body_top,
            self.scene,
            self.hit_index,
        );
        (body_top, self.open_fold.is_none())
    }

    /// **Fecha a dobra que a secção abriu.** ⛔ Esquecê-la deixa a secção seguinte a herdar a dobra
    /// desta, que já foi fechada.
    pub fn close_fold(&mut self, after: f32) -> f32 {
        match self.open_fold.take() {
            Some(fold) => fold.finish(self.store, self.scene, self.hit_index, after),
            None => after,
        }
    }

    /// A linha canónica que fecha uma secção.
    pub fn separator(&mut self, y: f32) -> f32 {
        close_section(self.scene, self.theme, self.inner_x, self.inner_w, y)
    }

    /// **Um botão de acção de largura inteira.**
    pub fn action_button(&mut self, id: NodeId, label: &str, y: f32) -> f32 {
        self.action_button_kind(id, label, ButtonKind::Default, y)
    }

    /// O mesmo, com um `kind` escolhido — um **Accent** é como uma acção de *commit* se destaca.
    ///
    /// ⭐⭐ **Onde ele fica é a porta [`crate::property_row::caixa_do_botao`]** (ordem do dono,
    /// 2026-09-24: *«Arrumar o painel Vector»*, depois de aprovada a regra no Inspector e nos outros
    /// painéis): a coluna do valor quando o rótulo lá cabe, a linha inteira quando não cabe, e a
    /// altura de um campo. Este ajudante pintava os `52` botões de acção do painel Vector e os do
    /// esqueleto a atravessar a linha, e o censo `an_action_button_asks_the_door_where_it_goes` não
    /// o via — ele vive no núcleo e não num painel.
    ///
    /// ⛔ **Um `Accent` ATRAVESSA a linha, de propósito:** é o *commit* (o *Apply* da pilha de efeitos,
    /// o da simetria) — o mesmo lugar que o *Apply Mask* do Painter e o par `Cancel | Apply` das
    /// ferramentas de imagem têm, e a razão por que eles se destacam das edições à volta.
    pub fn action_button_kind(&mut self, id: NodeId, label: &str, kind: ButtonKind, y: f32) -> f32 {
        let rect = if kind == ButtonKind::Accent {
            Rect::new(self.inner_x, y, self.inner_w, ph2d_tokens::ROW_H_PX)
        } else {
            crate::property_row::caixa_do_botao(
                self.text_system,
                self.inner_x,
                self.inner_w,
                y,
                label,
            )
        };
        let st = self.store.button_visual(id);
        let btn = Button::new(id, label).kind(kind).visual(st);
        paint_button(&btn, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        crate::property_row::abaixo_do_botao(rect)
    }

    /// **Um botão ROTULADO** (`<rótulo> [ botão ]`) — a geometria do [`Self::labeled_number_field`]
    /// com um botão no lugar da caixa.
    ///
    /// ⚠️ Ele existe porque um botão que é também o **readout** de uma escolha precisa da coluna de
    /// rótulo dos vizinhos: sem ela, o nome do que foi escolhido lê-se como mais um verbo.
    /// ⚠️ O `on` **acende** o botão (`Accent`): ele é o readout de uma escolha, e um readout que
    /// não distingue *escolhido* de *por escolher* obriga a abrir outra coisa para saber.
    pub fn labeled_action_button(
        &mut self,
        label: &str,
        id: NodeId,
        texto: &str,
        on: bool,
        y: f32,
    ) -> f32 {
        self.label_cell(label, y);
        let rect = value_col(self.inner_x, self.inner_w, y, self.row_h);
        let st = self.store.button_visual(id);
        let btn = Button::new(id, texto)
            .kind(if on {
                ButtonKind::Accent
            } else {
                ButtonKind::Default
            })
            .visual(st);
        paint_button(&btn, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        // ⚠️ **O vão é o de CONTROLO, não o de linha** — ele é um botão, e a fileira de botões
        // desta casa respira com o `control_gap_px`. Trocá-lo aqui desalinharia esta linha das
        // vizinhas por uns poucos px, que é a espécie de deriva que ninguém reporta e todos veem.
        y + self.row_h + ph2d_tokens::control_gap_px()
    }

    /// **Um campo numérico ROTULADO** (`<rótulo> [ 12,0 ]`).
    pub fn labeled_number_field(&mut self, label: &str, id: NodeId, step: f64, y: f32) -> f32 {
        self.label_cell(label, y);
        let rect = value_col(self.inner_x, self.inner_w, y, self.row_h);
        self.hit_index.register(id, rect);
        let (st, value, buffer, caret, anchor) = read_number_input(self.store, id);
        let input = NumberInput::new(id, "", value)
            .step(step)
            .visual((st, self.store.hover_live(id)));
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

    /// **Uma escolha entre MODOS nomeados — pela porta da ESCOLHA.**
    ///
    /// ⛔⛔ Até 2026-09-23 esta fileira pintava o nome POR CIMA e o grupo a toda a largura, em
    /// todos os painéis que a usam (Vector e o Esqueleto) — a varredura geométrica
    /// `nenhum_nome_por_cima_do_controlo` acusava `9 + 1` grupos a começar na borda. Hoje ela
    /// delega em [`crate::property_row::paint_choice_row`], que põe o nome AO LADO quando o grupo
    /// cabe numa fileira da coluna do valor e só faz PALETA quando não cabe.
    ///
    /// ⚠️ **A coluna é a de omissão** (`Seccao::apenas_campos(1)`), que é exactamente a
    /// [`label_col_w`] das outras linhas deste contexto — senão o nome de uma escolha e o de um
    /// campo vizinho cairiam em `x` diferentes.
    ///
    /// ⚠️ **O ritmo é o do PAINEL:** a porta acaba com o vão da casa (`control_gap_px`) e este
    /// contexto separa as linhas pelo `row_gap` dele; troca-se um pelo outro para que uma escolha
    /// não mude a distância à linha seguinte.
    pub fn segmented(&mut self, label: &str, opts: &[(NodeId, &str, bool)], y: f32) -> f32 {
        let segs: Vec<(&str, bool, NodeId)> =
            opts.iter().map(|(id, lbl, on)| (*lbl, *on, *id)).collect();
        let fim = crate::property_row::paint_choice_row(
            self.scene,
            self.text_system,
            self.theme,
            self.hit_index,
            self.store,
            self.inner_x,
            self.inner_w,
            y,
            label,
            &segs,
            crate::widget::Seccao::apenas_campos(1),
        );
        fim - ph2d_tokens::control_gap_px() + self.row_gap
    }

    /// A célula de rótulo das duas linhas rotuladas — uma porta, para as duas nunca desalinharem.
    fn label_cell(&mut self, label: &str, y: f32) {
        // ⚠️ **ELIDIDO:** a coluna é uma fracção da linha, e a coluna docada estreita-se.
        crate::widget::paint_property_label(
            self.text_system,
            self.scene,
            label,
            self.inner_x,
            y + (self.row_h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            label_col_w(self.inner_x, self.inner_w),
            resolve(ColorToken::Text2, self.theme),
        );
    }
}
