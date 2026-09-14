//! The two labelled rows the physics sections are built from.
//!
//! They live here rather than in either section because BOTH use them: §11
//! (Physics Body) and §12 (Physics Joint) are the same shape — a label, then a
//! segmented control or a number box. A second copy would be two answers to
//! "what does a physics row look like", and they would drift by a pixel.

use super::*;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};

/// A labelled segmented control. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
pub(super) fn seg_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    group_id: NodeId,
    option_ids: &[NodeId],
    labels: &[&str],
    selected: u8,
) -> f32 {
    // ⭐⭐ **Irmã da [`num_row`]: o rótulo fica à ESQUERDA, pela mesma porta.** Convertê-la no mesmo
    // commit não é zelo — ela e a `num_row` partilham as mesmas secções, e deixar uma à esquerda e a
    // outra por cima **é** a queixa do dono («muito ruim!») uma linha mais abaixo.
    //
    // ⚠️ **O controlo REFLUI e a coluna do rótulo não:** com três opções compridas
    // (*Dynamic · Static · Kinematic*) o grupo segmentado quebra em mais de uma fileira dentro da
    // coluna dele — e é por isso que a altura devolvida é a do que ele de facto ocupou
    // (`seg_h`), nunca `ROW_H_PX`. *O que mantém a secção legível é a coluna da esquerda estar
    // sempre no mesmo `x`; o lado direito pode crescer.*
    let row = ph2d_editor_core::widget::property_row_columns(x, w, y, ROW_H_PX);
    let label_font = TypeToken::Sm.px();
    ph2d_editor_core::paint::paint_text_elided(
        text_system,
        scene,
        label,
        row.label.x,
        row.label.y + (row.label.h - label_font) * 0.5,
        label_font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
    let seg = SegmentedAdaptive::new(
        group_id,
        label,
        option_ids
            .iter()
            .zip(labels)
            .map(|(&id, &l)| SegmentedOption::new(id, l))
            .collect(),
    )
    .selected((selected as usize).min(labels.len() - 1));
    let seg_h = paint_segmented_adaptive(
        &seg,
        row.control,
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, row.dot);
    y + seg_h.max(ROW_H_PX) + ph2d_tokens::control_gap_px()
}

/// A altura que uma [`num_row`] consome — **uma linha**.
///
/// ⚠️ Existe porque o **card** (`card_frame`) precisa saber quanto medir ANTES
/// de as rows serem pintadas, e uma segunda cópia dessa aritmética é como uma
/// moldura passa a não caber no que ela emoldura: a próxima seção pintaria por
/// cima das caixas. Uma régua, dois consumidores — o mesmo argumento do
/// `segmented_row_counts` do Painter.
/// ⚠️ **Encolheu em 2026-09-14** — de `rótulo + caixa + respiro` para **o passo de uma linha**.
/// O rótulo deixou de ter faixa própria (ele é a coluna da esquerda, pela
/// `widget::property_row_columns`), logo uma row mede uma LINHA, como em todo painel desta casa.
pub(super) fn num_row_h() -> f32 {
    ph2d_tokens::row_pitch_px()
}

/// Quanto um card **inteiro** mede.
///
/// ⚠️ **A régua é `pub(crate)` de propósito.** O pintor tem de avançar pela
/// ALTURA DA MOLDURA, nunca pela soma das rows — as duas diferem por uma
/// respiração, e somar as rows faz a moldura do card seguinte nascer POR CIMA da
/// borda do anterior. É um defeito que só a geometria vê, então a geometria é o
/// oráculo: o gate compara onde as rows de fato caíram contra este passo.
pub(crate) fn card_h(n_rows: usize) -> f32 {
    let pad = Spacing::Sm.px();
    let title_h = TypeToken::Sm.px() + Spacing::Sm.px();
    pad + title_h + n_rows as f32 * num_row_h() + pad
}

/// O passo de um card ao próximo — a moldura mais o respiro entre elas.
pub(crate) fn card_pitch(n_rows: usize) -> f32 {
    card_h(n_rows) + Spacing::Sm.px()
}

/// Uma **moldura titulada** para um grupo de [`num_row`] — a subseção.
///
/// Devolve `(inner_x, inner_w, first_row_y, y_after_card)`: o chamador pinta as
/// rows em `[inner_x, inner_w]` a partir de `first_row_y` e segue em
/// `y_after_card`.
///
/// ⚠️ **Molde do card do Painter** (`ph2d-panel-painter-layers::card_frame`), e
/// não uma invenção: é a caixa em que este app já diz *"estes números são um
/// assunto só"*. ⚠️ **E desde 2026-09-14 a régua da linha é a MESMA dos dois lados**: o rótulo fica
/// à ESQUERDA aqui também, pela `widget::property_row_columns`. *Esta nota dizia o contrário — ela
/// é de quando o Inspector era a excepção, e o report do dono fechou essa excepção.*
#[allow(clippy::too_many_arguments)]
pub(super) fn card_frame(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    title: &str,
    n_rows: usize,
) -> (f32, f32, f32, f32) {
    let pad = Spacing::Sm.px();
    let font = TypeToken::Sm.px();
    let title_h = font + Spacing::Sm.px();
    let card_h = card_h(n_rows);
    let card = Rect::new(x, y, w, card_h);
    // ⭐ Raio e moldura pela porta do TEMA: o cartão é plano num tema moderno.
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(
        scene,
        card,
        radius,
        resolve(
            ph2d_editor_core::widget::section_cards::CardDepth::Subsection.token(),
            theme,
        ),
    );
    ph2d_editor_core::paint::stroke_frame(
        scene,
        card,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        ph2d_tokens::StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );
    paint_text(
        text_system,
        scene,
        title,
        x + pad,
        y + pad,
        font,
        w - 2.0 * pad,
        resolve(ColorToken::Text2, theme),
    );
    (
        x + pad,
        w - 2.0 * pad,
        y + pad + title_h,
        y + card_pitch(n_rows),
    )
}

/// A labelled number box, seeded from the store (which `sync` mirrors from
/// the snapshot). Returns the next `y`.
#[allow(clippy::too_many_arguments)]
pub(super) fn num_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id: NodeId,
) -> f32 {
    // ⭐⭐⭐ **O rótulo fica à ESQUERDA, pela porta** — report do dono, 2026-09-14, com foto da
    // secção LEG do Platform Player: *«Label acima do campo numérico! Muito ruim!»*. Até aqui esta
    // função empilhava (uma faixa só para o rótulo, o campo por baixo em largura cheia) e era a
    // única das cinco do Inspector com o campo a ocupar a linha inteira.
    //
    // ⚠️ **A altura de uma row cai de `label_h + passo` para o PASSO** (~16 px por linha), e é por
    // isso que a [`num_row_h`] encolheu no mesmo commit: a `card_h` deriva dela, e as duas têm de
    // continuar a ser a mesma resposta — senão a moldura do card nasce por cima das caixas.
    let row = ph2d_editor_core::widget::property_row_columns(x, w, y, ROW_H_PX);
    let label_font = TypeToken::Sm.px();
    // ⚠️ **ELIDIDO, nunca transbordado:** a coluna do rótulo é uma FRACÇÃO da linha (a coluna
    // docada é arrastável), logo um rótulo comprido numa coluna estreita passaria por cima do campo.
    ph2d_editor_core::paint::paint_text_elided(
        text_system,
        scene,
        label,
        row.label.x,
        row.label.y + (row.label.h - label_font) * 0.5,
        label_font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
    hit_index.register(id, row.control);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value).visual((state, store.hover_live(id)));
    paint_number_input_with_buffer(
        &input,
        Some(buffer),
        caret,
        anchor,
        row.control,
        scene,
        text_system,
        theme,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, row.dot);
    y + ph2d_tokens::row_pitch_px()
}
