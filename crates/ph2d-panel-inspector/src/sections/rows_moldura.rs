//! A moldura titulada de um grupo de [`super::rows::num_row`] e a régua da altura dela —
//! saiu de `rows.rs` (tecto de 600 linhas) sem mudar uma linha.

use super::rows::num_row_h;
use super::*;

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
pub(crate) fn card_frame(
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
    let title_h = TypeToken::Sm.px() + Spacing::Sm.px();
    moldura(
        scene,
        text_system,
        theme,
        Rect::new(x, y, w, card_h(n_rows)),
        title,
    );
    (
        x + pad,
        w - 2.0 * pad,
        y + pad + title_h,
        y + card_pitch(n_rows),
    )
}

/// ⭐ T59: a moldura com as linhas DENTRO — no cartão cada linha leva o rótulo em cima (e ele pode
/// quebrar), logo a altura dela não é a [`num_row_h`]: as linhas pintam-se primeiro numa cena à
/// parte (`linhas(cena, x, w, y) → y` final) e a moldura nasce com a altura MEDIDA, por baixo delas.
/// Fora do cartão é a [`card_frame`] de sempre. Devolve o `y` depois da moldura.
#[allow(clippy::too_many_arguments)]
pub(crate) fn card_frame_with(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    title: &str,
    n_rows: usize,
    linhas: impl FnOnce(&mut VectorScene, &mut TextSystem, f32, f32, f32) -> f32,
) -> f32 {
    if ph2d_editor_core::widget::card_ink::card_body().is_none() {
        let (ix, iw, ry, next) = card_frame(scene, text_system, theme, x, w, y, title, n_rows);
        let _ = linhas(scene, text_system, ix, iw, ry);
        return next;
    }
    let pad = Spacing::Sm.px();
    let title_h = TypeToken::Sm.px() + Spacing::Sm.px();
    let mut cena = VectorScene::new();
    let fim = linhas(
        &mut cena,
        text_system,
        x + pad,
        w - 2.0 * pad,
        y + pad + title_h,
    );
    let h = fim - y + pad;
    moldura(scene, text_system, theme, Rect::new(x, y, w, h), title);
    scene.append_transformed(&cena, ph2d_vector::Affine::IDENTITY);
    y + h + ph2d_tokens::control_gap_px()
}

/// O fundo, a borda e o título de uma moldura em `card`.
fn moldura(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    card: Rect,
    title: &str,
) {
    let (x, y, w) = (card.x, card.y, card.w);
    let pad = Spacing::Sm.px();
    let font = TypeToken::Sm.px();
    // ⭐ Raio e moldura pela porta do TEMA: o cartão é plano num tema moderno.
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(
        scene,
        card,
        radius,
        // O degrau de subsecção do tema em que o cartão se pinta (T59, aprovado), não a caixa dele.
        ph2d_editor_core::paint::resolve(
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
}
