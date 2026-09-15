//! ⭐⭐⭐ **O PINTOR de uma linha de propriedade — a porta que TODO painel usa.**
//!
//! ⛔⛔ **Ela mudou-se do `ph2d-panel-inspector` para cá em 2026-09-15, e o motivo é uma medição:**
//! sete painéis pintam campos numéricos (`flip-frames` · `grid-snap` · `motion-graph` ·
//! `motion-params` · `painter-layers` · `timeline` · `vector`) e **nenhum** passava pela porta —
//! cada um com a sua aritmética. Só o `painter-layers` tem **oito** larguras de coluna de rótulo
//! escritas à mão (`96` · `70` · `62` · `60` · `60` · `54` · `44` · `38`), e o cabeçalho do
//! `number_field.rs` dele declara-se *«the SAME number box as the Inspector's Transform (the app
//! standard) … X/Y pairs share one line with red X / green Y axis tags like the reference»*.
//!
//! ⚠️⚠️ **Isto é: ele é uma CÓPIA da linha do Transform, com os defeitos que o Transform acabou de
//! largar** — a largura fixa (que a spec §3 prova errada por construção: o dock é arrastável) e as
//! letras de eixo numa coluna própria (que custam a disposição que o dono aprovou). *Uma porta que
//! vive dentro de um painel é uma porta que o painel seguinte copia.*
//!
//! A lei que estas funções servem está em `docs/UI_New_and_Simple/spec/03_a_linha_de_propriedade.md`.

use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::resolve;
use crate::widget::showcase::read_number_input;
use crate::widget::{NumberInput, PropertyRow, Unit, paint_number_input_with_buffer};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// ⭐⭐⭐ **O NOME de uma linha, pintado à esquerda — e devolve ONDE o controlo vai.**
///
/// ⛔⛔ Para as linhas cujo controlo o chamador CONSTRÓI: um segmentado com «nenhum aceso», uma
/// grelha, quatro amostras de cor.
///
/// ⚠️ **O chamador pinta o ponto** (`paint_decorator_dot(scene, theme, row.dot)`) — ele não pode ser
/// pintado aqui porque a altura do controlo só se sabe depois de o desenhar, e há controlos que
/// refluem.
#[allow(clippy::too_many_arguments)]
pub fn paint_label_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    h: f32,
    label: &str,
) -> PropertyRow {
    let row = crate::widget::property_row_columns(x, w, y, h);
    let label_font = TypeToken::Sm.px();
    crate::widget::paint_property_label(
        text_system,
        scene,
        label,
        row.label.x,
        row.label.y + (row.label.h - label_font) * 0.5,
        label_font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
    row
}

/// ⭐⭐⭐ **A LINHA DE VÁRIAS COMPONENTES — o nome à ESQUERDA, as caixas na coluna do controlo.**
///
/// ⛔⛔ **Report do dono, 2026-09-14** (*«Label acima do campo numérico! Muito ruim!»*) e
/// **2026-09-15** (*«A disposição ficou diferente … Position X/Y Caixa Caixa»* · *«O painel ainda
/// largo com espaço à esquerda e as linhas já se quebram … Isso não pode acontecer»*). As três
/// respostas estão aqui dentro e em mais lado nenhum:
///
/// 1. o nome à esquerda, alinhado à direita, elidido (spec §3 e §4);
/// 2. o que não cabe ao piso do campo **desce** dentro da coluna do controlo (§6-bis);
/// 3. a coluna do nome **cede** ao controlo antes de deixar a linha quebrar (§6-ter).
///
/// ⚠️ **UM ponto por LINHA, nunca por campo:** um par `X`/`Y` é *uma* propriedade com duas
/// componentes, e dois pontos diriam que são duas.
///
/// ⚠️ **`campos_da_seccao` é da SECÇÃO e não desta linha** — ver a §6-ter da spec: se cada linha
/// cedesse pelo que ELA precisa, a coluna sairia esfarrapada. É **a linha que a secção não quer ver
/// quebrar**, e ⛔ não o máximo mecânico: uma secção com uma row de 4 campos que só cabe num painel
/// de `~700` declara `2`, senão ela deixa de ceder e o par `X`/`Y` volta a quebrar.
#[allow(clippy::too_many_arguments)]
pub fn paint_fields_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    field_ids: &[NodeId],
    step: f64,
    unit: Option<Unit>,
    campos_da_seccao: usize,
) -> f32 {
    let label_font = TypeToken::Sm.px();
    let gap = ph2d_tokens::control_gap_px();
    // ⭐⭐ **O que o rótulo PRECISA — medido, no peso em que pinta.** É ele o piso da cedência: a
    //    coluna encolhe para o controlo caber, e pára aqui. *Trocar uma linha quebrada por um nome
    //    cortado não é a cura que o dono pediu.*
    let quer = text_system.prefix_width(label, label_font);
    // ⭐⭐ **O que o CONTROLO precisa para não quebrar** — `n` caixas ao piso, com os vãos.
    let n = campos_da_seccao.max(field_ids.len()).max(1) as f32;
    let precisa = n * crate::widget::NUMBER_INPUT_MIN_W_PX + (n - 1.0) * gap;
    let row = crate::widget::property_row_columns_for(x, w, y, ROW_H_PX, Some(quer), Some(precisa));
    crate::widget::paint_property_label(
        text_system,
        scene,
        label,
        row.label.x,
        row.label.y + (row.label.h - label_font) * 0.5,
        label_font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
    let (por_linha, linhas, cw) =
        crate::widget::property_fields_layout(row.control.w, field_ids.len(), gap, 0.0);
    let passo = ph2d_tokens::row_pitch_px();
    for (i, &id) in field_ids.iter().enumerate() {
        let rect = Rect::new(
            row.control.x + (cw + gap) * (i % por_linha) as f32,
            row.control.y + passo * (i / por_linha) as f32,
            cw,
            ROW_H_PX,
        );
        hit_index.register(id, rect);
        let (state, value, buffer, caret, anchor) = read_number_input(store, id);
        let input = NumberInput::new(id, "", value)
            .step(step)
            .visual((state, store.hover_live(id)))
            .suffix(unit.map(Unit::suffix));
        paint_number_input_with_buffer(
            &input,
            Some(buffer),
            caret,
            anchor,
            rect,
            scene,
            text_system,
            theme,
        );
    }
    crate::widget::paint_decorator_dot(scene, theme, row.dot);
    y + passo * linhas as f32
}

/// ⭐⭐⭐ **CABE uma linha de propriedade com este nome dentro de `w`?**
///
/// ⛔⛔ **Report do dono, 2026-09-15, com duas fotos (painel largo e estreito):** *«Em Grain:
/// Voronoi : Metric e Edges os nomes somem ao estreitar o painel. Melhor seria quebrar a linha»*.
///
/// Aquelas duas vivem **emparelhadas**, cada uma numa METADE da largura do painel. Ao estreitar, a
/// coluna do nome de uma metade fica menor do que a reticência e o
/// [`crate::widget::paint_property_label`] devolve **string vazia** — que é a resposta certa dele
/// (*«a caixa fica só com o número, que é o degrau seguinte da escada do estreito»*) e a **errada**
/// para quem escolheu emparelhar: *o degrau a seguir a «não cabe o nome» não é apagar o nome, é
/// deixar de emparelhar.*
///
/// ⇒ quem empareelha pergunta ANTES. ⚠️ **Ela pergunta à PORTA, não a uma segunda aritmética:** o
/// veredito sai da [`crate::widget::property_row_columns_for`], a mesma que vai desenhar. *Duas
/// contas para «isto cabe?» divergem no dia em que uma das leis muda.*
#[must_use]
pub fn property_row_fits(w: f32, label_w: f32) -> bool {
    let piso = crate::widget::NUMBER_INPUT_MIN_W_PX;
    let row =
        crate::widget::property_row_columns_for(0.0, w, 0.0, ROW_H_PX, Some(label_w), Some(piso));
    row.label.w >= label_w - 0.01 && row.control.w >= piso - 0.01
}
