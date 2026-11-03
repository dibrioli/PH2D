//! ⭐⭐ **O NOME de uma linha de marcar** — onde ele é pousado, e o pintor que o compõe com a marca.
//!
//! ⚠️ **Irmão por RESPONSABILIDADE do [`super::mark`], forçado pelo tecto de 500 LOC do widget**
//! (2026-09-15, quando a marca ganhou CAIXA por ordem do dono): aquele ficheiro responde *onde a
//! marca assenta e que tinta ela leva*; este responde *onde o texto do nome é pousado*. ⛔ Os dois
//! crescem por motivos diferentes — o primeiro por decisões de disposição, o segundo por defeitos
//! de medição de texto. **É a mesma partição que o `property_box` já tem** (`row.rs` / `label.rs`).
//!
//! ⛔ A re-exportação fica no [`super`], então nenhum chamador muda de caminho.

use super::mark::{BooleanMark, paint_boolean_mark};
use super::{Checkbox, CheckboxState};
use crate::paint::{paint_text, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

impl Checkbox {
    /// ⭐ **O orçamento do NOME** — na linha de formulário, a coluna do nome da secção; fora dela,
    /// até à marca menos um recuo de cada lado (clássico: o nome à direita da marca).
    /// É a régua que o pintor usa, pública para quem dispõe a linha (o nome EM CIMA quando não
    /// cabe — galeria em cartões, doc `CartoesUI/08`).
    #[must_use]
    pub fn free_label_budget(&self, rect: Rect) -> f32 {
        let pad = Spacing::Md.px();
        let box_size = self.box_px.unwrap_or(super::CHECKBOX_BOX_PX).min(rect.h);
        if !crate::paint::ui_is_redesign() {
            return (rect.w - box_size - pad).max(0.0);
        }
        if let Some(sec) = self.seccao {
            // Na linha de formulário o nome vai à coluna do nome da secção.
            return crate::widget::property_box::colunas_da_linha(
                rect.x, rect.w, rect.y, rect.h, sec,
            )
            .label
            .w;
        }
        let mark_x = crate::widget::property_box::value_column(rect, box_size, self.decorator).x;
        (mark_x - rect.x - pad * 2.0).max(0.0)
    }

    /// ⭐ **Quantas linhas (`row_pitch` cada) o NOME ocupa** — `2` quando a secção quebra o nome
    /// ([`crate::widget::Seccao::que_quebra_o_nome`]) e ele não cabe na coluna. Quem empilha a linha
    /// avança por isto, e o alvo e a camada que esmaece cobrem as duas.
    #[must_use]
    pub fn linhas_do_nome(&self, rect: Rect, text_system: &mut TextSystem) -> usize {
        match (crate::paint::ui_is_redesign(), self.seccao) {
            (true, Some(sec)) if sec.quebra_o_nome() && !self.label.is_empty() => {
                let col = crate::widget::property_box::colunas_da_linha(
                    rect.x, rect.w, rect.y, rect.h, sec,
                )
                .label
                .w;
                let duas =
                    crate::widget::nome_em_duas_linhas(text_system, &self.label, label_font(), col);
                if duas.is_some() { 2 } else { 1 }
            }
            _ => 1,
        }
    }

    /// O nome cabe INTEIRO ao lado da marca em `rect`, na letra em que o pintor o escreve?
    #[must_use]
    pub fn label_fits_beside(&self, rect: Rect, text_system: &mut TextSystem) -> bool {
        text_system
            .layout(&self.label, label_font(), f32::INFINITY)
            .width()
            <= self.free_label_budget(rect)
    }
}

/// A letra do nome: `Sm` no redesenho (a da linha de propriedade), `Base` no clássico.
fn label_font() -> f32 {
    if crate::paint::ui_is_redesign() {
        TypeToken::Sm.px()
    } else {
        TypeToken::Base.px()
    }
}

/// Square box + label. Box fills with `Accent` when Checked, paints a check glyph; Indeterminate
/// paints a dash glyph.
///
/// ⚠️ **A marca vai à DIREITA e o rótulo à esquerda** desde o redesenho de 2026-09 — ver
/// [`paint_boolean_mark`], que é onde a geometria vive.
pub fn paint_checkbox(
    cb: &Checkbox,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let pintada = paint_boolean_mark(
        rect,
        BooleanMark {
            value: cb.value,
            state: cb.state,
            hover_t: cb.hover_t,
            box_px: cb.box_px,
            decorator: cb.decorator,
            pinta_coluna: !cb.coluna_de_fora,
            linha: cb.seccao,
        },
        scene,
        theme,
    );
    let box_rect = pintada.marca;

    // ⭐⭐⭐ **A PALAVRA do valor, à direita da marca e DENTRO da caixa** — ordem do dono,
    // 2026-09-15, com a foto do inspector do Godot: *«Godot define a checkbox como uma palavra
    // (On) à direita. Isso me parece bom»*.
    //
    // ⚠️ **Ela não muda com o valor, e é assim no alvo:** na foto dele três linhas dizem *On* e só
    // uma está marcada. A palavra nomeia o que a marca LIGA — como o rótulo de um interruptor de
    // parede —, e quem diz se está ligado é a marca. ⛔ Eu tinha-a deixado de fora por achar que
    // ela mentia, e devolvi-lhe a decisão; ele decidiu.
    //
    // ⚠️ **Só na linha de FORMULÁRIO**: fora dela não há caixa onde a pôr (a pele de canvas é o que
    // o artista desenhou), e o interruptor nem por aqui passa.
    if let Some(caixa) = pintada.caixa {
        let fonte = TypeToken::Sm.px();
        let x = box_rect.x + box_rect.w + Spacing::Sm.px();
        // ⚠️ O orçamento acaba no mesmo recuo em que o VALOR de um campo acaba.
        let orcamento = (caixa.x + caixa.w - crate::widget::field_pad_x() - x).max(0.0);
        paint_text(
            text_system,
            scene,
            ph2d_i18n::tr("chrome.checkbox.on"),
            x,
            rect.y + (rect.h - fonte) * 0.5,
            fonte,
            orcamento,
            resolve(
                if cb.state == CheckboxState::Disabled {
                    ColorToken::TextDisabled
                } else {
                    ColorToken::Text1
                },
                theme,
            ),
        );
    }

    if !cb.label.is_empty() {
        let label_color = if cb.state == CheckboxState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        // ⚠️ **`Sm`, e não `Base`** — medido em 2026-09-03: a linha de propriedade escreve o
        // rótulo a `12 px` e esta escrevia a `13`. Num formulário as duas alternam, e **1 px de
        // corpo de letra entre linhas vizinhas lê-se como desalinho**, não como ênfase.
        // ⚠️ **No clássico volta a `Base`, à DIREITA da caixa** — é a linha de sempre.
        let redesign = crate::paint::ui_is_redesign();
        let font_size = label_font();
        let ly = rect.y + (rect.h - font_size) * 0.5;
        if let (true, Some(sec)) = (redesign, cb.seccao) {
            // ⭐⭐⭐ **O NOME VAI À COLUNA DO NOME DA SECÇÃO** — report do dono, 2026-09-14
            // (*«as labels alinhadas todas à direita»*) levado até à linha de MARCAR.
            //
            // ⛔⛔ Até 2026-09-15 ele era encostado à esquerda da faixa e corria até à marca. Num
            // formulário que alterna números e marcas isso dá **duas colunas de nome**, alternando
            // linha sim linha não — *a mesma doença do rótulo por cima do campo, meia volta
            // adiante*.
            //
            // ⚠️ **A marca NÃO se move**, e isso é uma lei com gate: ela partilha a
            // [`super::super::property_box::value_column`] com o número, e é isso que dá ao
            // formulário **uma** margem direita (`the_mark_is_anchored_to_the_right_edge`). ⇒ entre
            // o nome e a marca fica um vão, e ele é o mesmo em todas as linhas de marcar.
            let row =
                crate::widget::property_box::colunas_da_linha(rect.x, rect.w, rect.y, rect.h, sec);
            // ⭐ A secção que QUEBRA o nome: a 2.ª linha desce um passo (ver [`Checkbox::linhas_do_nome`]).
            if sec.quebra_o_nome() {
                crate::widget::property_box::paint_property_label_em_linhas(
                    text_system,
                    scene,
                    &cb.label,
                    row.label.x,
                    ly,
                    font_size,
                    row.label.w,
                    ph2d_tokens::row_pitch_px(),
                    resolve(label_color, theme),
                );
            } else if let Some(ink) = crate::widget::card_ink::card_body() {
                // Num cartão não há coluna de nomes à direita: alinha com os rótulos do corpo,
                // na lei do rótulo dele.
                let px = ph2d_card_stack::metrics::LABEL_PX;
                crate::widget::card_field::paint_label(
                    scene,
                    text_system,
                    ink,
                    rect.x,
                    rect.y + (rect.h - px) * 0.5,
                    row.label.x + row.label.w - rect.x,
                    &cb.label,
                );
            } else {
                crate::widget::property_box::paint_property_label(
                    text_system,
                    scene,
                    &cb.label,
                    row.label.x,
                    ly,
                    font_size,
                    row.label.w,
                    resolve(label_color, theme),
                );
            }
        } else if redesign {
            // ⛔ **Fora do formulário** (a pele de canvas): o nome fica onde o artista o pôs.
            let lx = rect.x + Spacing::Md.px();
            let budget = cb.free_label_budget(rect);
            let cut =
                crate::widget::property_box::fit_label(text_system, &cb.label, font_size, budget);
            if !cut.is_empty() {
                paint_text(
                    text_system,
                    scene,
                    &cut,
                    lx,
                    ly,
                    font_size,
                    f32::INFINITY,
                    resolve(label_color, theme),
                );
            }
        } else if rect.w > box_rect.w + Spacing::Md.px() {
            let lx = rect.x + box_rect.w + Spacing::Md.px();
            paint_text(
                text_system,
                scene,
                &cb.label,
                lx,
                ly,
                font_size,
                (rect.x + rect.w - lx).max(0.0),
                resolve(label_color, theme),
            );
        }
    }
}
