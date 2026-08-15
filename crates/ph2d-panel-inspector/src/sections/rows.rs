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
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y + (label_h - label_font) * 0.5,
        label_font,
        w,
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
        Rect::new(x, y + label_h, w, ROW_H_PX),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    y + label_h + seg_h + Spacing::Sm.px()
}

/// A altura que uma [`num_row`] consome — rótulo + caixa + respiro.
///
/// ⚠️ Existe porque o **card** (`card_frame`) precisa saber quanto medir ANTES
/// de as rows serem pintadas, e uma segunda cópia dessa aritmética é como uma
/// moldura passa a não caber no que ela emoldura: a próxima seção pintaria por
/// cima das caixas. Uma régua, dois consumidores — o mesmo argumento do
/// `segmented_row_counts` do Painter.
pub(super) fn num_row_h() -> f32 {
    TypeToken::Sm.px() + Spacing::Xs.px() + ROW_H_PX + Spacing::Sm.px()
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
/// assunto só"*. O que muda é a régua da linha — lá o rótulo fica à ESQUERDA da
/// caixa, aqui ele fica ACIMA dela, que é a forma de row do Inspector inteiro.
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
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, card, radius, resolve(ColorToken::Bg1, theme));
    stroke_rounded_rect(
        scene,
        card,
        radius,
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
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y + (label_h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    // ⚠️ A régua da altura é a `num_row_h`, e ela é PERGUNTADA — o `card_frame`
    // mede com a mesma função, então a moldura não pode deixar de caber.
    debug_assert!((num_row_h() - (label_h + ROW_H_PX + Spacing::Sm.px())).abs() < 1.0e-3);
    let row_y = y + label_h;
    let rect = Rect::new(x, row_y, w, ROW_H_PX);
    hit_index.register(id, rect);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value).visual((state, store.hover_live(id)));
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
    row_y + ROW_H_PX + Spacing::Sm.px()
}
