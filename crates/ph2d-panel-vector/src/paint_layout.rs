//! **A seção LAYOUT** do painel (plano UI/UX W2, ADR-0153) — irmã de [`super::paint_frame`] pelo
//! teto de 600 LOC, e o corte é o mesmo: aqui mora a seção de UM assunto.
//!
//! # Duas metades, e nenhuma é pintada por engano
//!
//! - **a moldura DISPÕE** — pintada quando a seleção resolve para uma moldura (a MESMA porta da
//!   seção Frame, `state::frame_clip()`, que já sabe desfazer a expansão de seleção do W0);
//! - **o filho se COMPORTA** — pintada quando a seleção está dentro de um fluxo.
//!
//! ⚠️ Elas **coexistem** numa moldura aninhada, que empilha os próprios filhos *e* é um item no
//! fluxo do pai. E nenhuma delas aparece sozinha por acidente: a seção inteira só existe se pelo
//! menos uma tiver o que dizer, senão o cabeçalho *Layout* ficaria numa seleção que não tem layout
//! nenhum.
//!
//! # O que NÃO é pintado com a moldura parada
//!
//! ⚠️ Com a direção em **Off** só a fileira de direção é desenhada. Vão, recuo, alinhamento e
//! distribuição sobre uma moldura que não empilha são **cinco controlos que não mudam um pixel** —
//! e um controlo que o artista mexe e que não faz nada é pior do que um controlo que falta.

use ph2d_i18n::tr;
use ph2d_tokens::Spacing;

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::state::{self, LayoutFlow};

impl BodyCtx<'_> {
    /// **A seção LAYOUT** — a moldura que empilha, e o filho que cresce.
    pub(crate) fn layout_section(&mut self, y: f32) -> f32 {
        let frame = state::frame_clip().is_some();
        let item = state::layout_item();
        // Sem moldura E sem filho de fluxo não há assunto: o cabeçalho não é pintado.
        if !frame && item.is_none() {
            return y;
        }
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_LAYOUT,
            tr("panel.vector.section.layout"),
            y,
        );
        if collapsed {
            return y;
        }
        if frame {
            y = self.layout_frame_rows(y);
        }
        if let Some(it) = item {
            // O fora-do-fluxo vem PRIMEIRO porque é ele que decide se o resto existe.
            y = self.checkbox_row(
                ids::VECTOR_LAYOUT_ITEM_ABSOLUTE,
                tr("panel.vector.layout.absolute"),
                it.absolute,
                y,
            );
            // ⚠️ Os VALORES não são lidos aqui: eles são semeados no store pelo `paint.rs`, como
            // os campos do Transform — o campo numérico é dono do próprio buffer enquanto o
            // artista digita, e pintá-lo a partir do publicado apagaria a tecla que ele acabou de
            // premir. Aqui só se decide se as duas linhas EXISTEM.
            //
            // ⚠️ E elas NÃO existem para um filho absoluto: ele saiu do fluxo, logo não reparte
            // sobra nenhuma, e dois números que não movem um pixel são o controlo morto que esta
            // política existe para impedir.
            if !it.absolute {
                y = self.number_row(
                    "Grow",
                    ids::VECTOR_LAYOUT_ITEM_GROW,
                    "Shrink",
                    ids::VECTOR_LAYOUT_ITEM_SHRINK,
                    y,
                );
            }
        }
        y
    }

    /// As fileiras da MOLDURA: direção sempre, o resto só quando ela flui.
    fn layout_frame_rows(&mut self, y: f32) -> f32 {
        let flow = state::layout_flow();
        // O rádio de quatro: *esta moldura flui, e como?* — UMA pergunta (o `display` do CSS).
        let dir = flow.map(|f| f.dir);
        let mut y = self.segmented(
            tr("panel.vector.layout.dir"),
            &[
                (
                    ids::VECTOR_LAYOUT_DIR_OFF,
                    tr("panel.vector.layout.dir.off"),
                    dir.is_none(),
                ),
                (
                    ids::VECTOR_LAYOUT_DIR_ROW,
                    tr("panel.vector.layout.dir.row"),
                    dir == Some(ids::VECTOR_LAYOUT_DIR_ROW),
                ),
                (
                    ids::VECTOR_LAYOUT_DIR_COL,
                    tr("panel.vector.layout.dir.col"),
                    dir == Some(ids::VECTOR_LAYOUT_DIR_COL),
                ),
                (
                    ids::VECTOR_LAYOUT_DIR_WRAP,
                    tr("panel.vector.layout.dir.wrap"),
                    dir == Some(ids::VECTOR_LAYOUT_DIR_WRAP),
                ),
            ],
            y,
        );
        let Some(f) = flow else {
            return y;
        };
        y = self.layout_size_rows(&f, y);
        // ⚠️ **O vão TRANSVERSAL só é pintado no `Wrap`.** Em linha ou coluna há uma faixa só, e
        // o número entre faixas não tem entre o que ficar: seria um campo que o artista edita e
        // que não move um pixel. Ele nasce com o modo que o usa.
        // **OS TOKENS DE VÃO** (W4c.4): a chave presa em cada eixo, para a rachura sobre o número
        // que ela cobre e para a row do picker logo abaixo.
        let bound = state::token_bindings().unwrap_or_default();
        let wrap = f.dir == ids::VECTOR_LAYOUT_DIR_WRAP;
        let cw = self.half_cell_w();
        let main = self.number_cell(
            tr("panel.vector.layout.gap"),
            ids::VECTOR_LAYOUT_GAP_MAIN,
            self.inner_x,
            cw,
            y,
        );
        if bound.gap_main.is_some() {
            self.token_slash(main);
        }
        if wrap {
            let cross = self.number_cell(
                "Cross",
                ids::VECTOR_LAYOUT_GAP_CROSS,
                self.inner_x + cw + Spacing::Sm.px(),
                cw,
                y,
            );
            if bound.gap_cross.is_some() {
                self.token_slash(cross);
            }
        }
        y += self.row_h + self.row_gap;
        // ⚠️ **A row do vão transversal segue a mesma cerca do CAMPO dele** (só no `Wrap`): em
        // linha ou coluna não há entre-faixas, então prender um token ali seria uma escolha que
        // não move um pixel — a definição de controle morto que o campo já evita.
        y = self.token_row(ids::VECTOR_TOKEN_GAP_MAIN, bound.gap_main.as_deref(), y);
        if wrap {
            y = self.token_row(ids::VECTOR_TOKEN_GAP_CROSS, bound.gap_cross.as_deref(), y);
        }
        y = self.layout_padding_rows(y);
        y = self.layout_align_rows(&f, y);
        y
    }

    /// O recuo: um campo, ou quatro. O par de modo troca os campos PINTADOS.
    /// **O TAMANHO da moldura, e os limites** — o vocabulário de *sizing* do Figma.
    ///
    /// ⚠️ Um par por EIXO, e não um só: a barra que ocupa a largura toda e tem a altura do
    /// conteúdo é o caso de uso mais comum de uma UI, e ele precisa de responder diferente nos
    /// dois. Colapsá-los num controlo tornaria esse caso inexprimível.
    fn layout_size_rows(&mut self, f: &LayoutFlow, y: f32) -> f32 {
        let mut y = self.segmented(
            tr("panel.vector.layout.size.w"),
            &[
                (
                    ids::VECTOR_LAYOUT_SIZE_W_FIXED,
                    tr("panel.vector.layout.size.fixed"),
                    f.size[0] == ids::VECTOR_LAYOUT_SIZE_W_FIXED,
                ),
                (
                    ids::VECTOR_LAYOUT_SIZE_W_HUG,
                    tr("panel.vector.layout.size.hug"),
                    f.size[0] == ids::VECTOR_LAYOUT_SIZE_W_HUG,
                ),
            ],
            y,
        );
        y = self.segmented(
            tr("panel.vector.layout.size.h"),
            &[
                (
                    ids::VECTOR_LAYOUT_SIZE_H_FIXED,
                    tr("panel.vector.layout.size.fixed"),
                    f.size[1] == ids::VECTOR_LAYOUT_SIZE_H_FIXED,
                ),
                (
                    ids::VECTOR_LAYOUT_SIZE_H_HUG,
                    tr("panel.vector.layout.size.hug"),
                    f.size[1] == ids::VECTOR_LAYOUT_SIZE_H_HUG,
                ),
            ],
            y,
        );
        // ⚠️ Os valores são semeados no store pelo `paint.rs` (a mesma lei dos campos do
        // Transform); aqui só se decide que as linhas existem.
        y = self.number_row(
            tr("panel.vector.layout.min.w"),
            ids::VECTOR_LAYOUT_MIN_W,
            tr("panel.vector.layout.max.w"),
            ids::VECTOR_LAYOUT_MAX_W,
            y,
        );
        self.number_row(
            tr("panel.vector.layout.min.h"),
            ids::VECTOR_LAYOUT_MIN_H,
            tr("panel.vector.layout.max.h"),
            ids::VECTOR_LAYOUT_MAX_H,
            y,
        )
    }

    fn layout_padding_rows(&mut self, y: f32) -> f32 {
        let each = state::layout_pad_each();
        let mut y = self.segmented(
            tr("panel.vector.layout.padding"),
            &[
                (
                    ids::VECTOR_LAYOUT_PAD_ALL_MODE,
                    tr("panel.vector.layout.padding.all"),
                    !each,
                ),
                (
                    ids::VECTOR_LAYOUT_PAD_EACH_MODE,
                    tr("panel.vector.layout.padding.each"),
                    each,
                ),
            ],
            y,
        );
        if each {
            y = self.number_row(
                "T",
                ids::VECTOR_LAYOUT_PAD_T,
                "R",
                ids::VECTOR_LAYOUT_PAD_R,
                y,
            );
            y = self.number_row(
                "B",
                ids::VECTOR_LAYOUT_PAD_B,
                "L",
                ids::VECTOR_LAYOUT_PAD_L,
                y,
            );
        } else {
            y = self.lone_number_row("All", ids::VECTOR_LAYOUT_PAD_ALL, y);
        }
        y
    }

    /// As duas fileiras de alinhamento: a travessa e o eixo principal.
    fn layout_align_rows(&mut self, f: &LayoutFlow, y: f32) -> f32 {
        let y = self.segmented(
            tr("panel.vector.layout.align"),
            &[
                (
                    ids::VECTOR_LAYOUT_ALIGN_START,
                    tr("panel.vector.layout.align.start"),
                    f.align == ids::VECTOR_LAYOUT_ALIGN_START,
                ),
                (
                    ids::VECTOR_LAYOUT_ALIGN_CENTER,
                    tr("panel.vector.layout.align.center"),
                    f.align == ids::VECTOR_LAYOUT_ALIGN_CENTER,
                ),
                (
                    ids::VECTOR_LAYOUT_ALIGN_END,
                    tr("panel.vector.layout.align.end"),
                    f.align == ids::VECTOR_LAYOUT_ALIGN_END,
                ),
                (
                    ids::VECTOR_LAYOUT_ALIGN_STRETCH,
                    tr("panel.vector.layout.align.stretch"),
                    f.align == ids::VECTOR_LAYOUT_ALIGN_STRETCH,
                ),
            ],
            y,
        );
        self.segmented(
            tr("panel.vector.layout.justify"),
            &[
                (
                    ids::VECTOR_LAYOUT_JUSTIFY_START,
                    tr("panel.vector.layout.justify.start"),
                    f.justify == ids::VECTOR_LAYOUT_JUSTIFY_START,
                ),
                (
                    ids::VECTOR_LAYOUT_JUSTIFY_CENTER,
                    tr("panel.vector.layout.justify.center"),
                    f.justify == ids::VECTOR_LAYOUT_JUSTIFY_CENTER,
                ),
                (
                    ids::VECTOR_LAYOUT_JUSTIFY_END,
                    tr("panel.vector.layout.justify.end"),
                    f.justify == ids::VECTOR_LAYOUT_JUSTIFY_END,
                ),
                (
                    ids::VECTOR_LAYOUT_JUSTIFY_BETWEEN,
                    tr("panel.vector.layout.justify.between"),
                    f.justify == ids::VECTOR_LAYOUT_JUSTIFY_BETWEEN,
                ),
                (
                    ids::VECTOR_LAYOUT_JUSTIFY_AROUND,
                    tr("panel.vector.layout.justify.around"),
                    f.justify == ids::VECTOR_LAYOUT_JUSTIFY_AROUND,
                ),
            ],
            y,
        )
    }
}

/// **O modo do RECUO (All/Each)** — o único evento desta seção que o painel resolve sozinho.
///
/// ⚠️ Ele é panel-local porque só decide **quais campos são pintados**, e o documento não tem
/// opinião sobre isso: uma ida à shell seria uma porta sem trabalho do outro lado. Todo o resto
/// da seção — direção, alinhamento, distribuição, os nove números — atravessa o barramento,
/// porque o valor mora no COMPONENTE.
///
/// Mora aqui, e não no `apply_event`, pelo teto de 200 LOC por função dos painéis — o mesmo
/// motivo (e o mesmo padrão de delegação) das pontas e do conector.
pub(crate) fn apply_event(
    host: &mut dyn ph2d_editor_core::panel::PanelHostInternal,
    ev: ph2d_editor_core::interaction::WidgetEvent,
) -> bool {
    use ph2d_editor_core::interaction::WidgetEvent;
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    if id != ids::VECTOR_LAYOUT_PAD_ALL_MODE && id != ids::VECTOR_LAYOUT_PAD_EACH_MODE {
        return false;
    }
    ph2d_editor_core::panel::seam_reset_button(host, id);
    state::set_layout_pad_each(id == ids::VECTOR_LAYOUT_PAD_EACH_MODE);
    true
}
