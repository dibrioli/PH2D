//! ⭐ **A DICA ILUSTRADA** sobre o item sob o rato (GameEngineFuncional N1, plano 05 §4) — título, desenho e
//! a explicação completa, num cartão ao lado do item. A 5.ª responsabilidade da paleta (desenha · mede ·
//! entra · banda · explica), irmã pelo tecto de 500 LOC dos primitivos.
//!
//! ⚠️ **Só pinta — não regista alvo nenhum:** o clique continua a ser do item por baixo do rato, e o cartão
//! nunca o cobre (abre ao lado, virando para o lado que cabe).
//!
//! ⚠️ **Mede pintando:** o texto é pintado numa cena à parte (o [`paint_text_block`] devolve a altura que de
//! facto ocupou) e só depois o fundo, do tamanho certo — medir e pintar são a mesma passada de layout.

use super::{PaletteModel, PaletteTip};
use crate::paint::{fill_rounded_rect, paint_text_block, resolve};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, VectorScene};

/// A largura do cartão da dica.
const TIP_W: f32 = 320.0; // LITERAL-PX-OK: largura do cartão da dica (geometria do modal)
/// A altura da moldura do desenho, em fracção da largura útil (uma moldura 2:1).
const PICTURE_ASPECT: f32 = 0.5;

/// ⭐ **Pinta a dica do item `hot`**, se ele tiver uma e estiver registado neste quadro (`rect_for`, o
/// rect do item no índice de alvos de quem chama). Devolve o cartão.
pub(super) fn paint_tip(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    rect_for: impl Fn(NodeId) -> Option<Rect>,
    model: &PaletteModel,
    hot: Option<NodeId>,
    viewport: Rect,
) -> Option<Rect> {
    let hot = hot?;
    let tip = model.tips.iter().find(|t| t.id == hot)?;
    let item = rect_for(hot)?;
    Some(paint_card(scene, ts, theme, tip, item, viewport))
}

fn paint_card(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    tip: &PaletteTip,
    item: Rect,
    viewport: Rect,
) -> Rect {
    let pad = Spacing::Lg.px();
    let gap = Spacing::Md.px();
    let inner_w = TIP_W - pad * 2.0;

    // ── O texto, em coordenadas do cartão, numa cena à parte. ──
    let mut text = VectorScene::new();
    let title_h = paint_text_block(
        ts,
        &mut text,
        &tip.title,
        pad,
        pad,
        TypeToken::Sm.px(),
        inner_w,
        resolve(ColorToken::Text1, theme),
    );
    let picture = Rect::new(pad, pad + title_h + gap, inner_w, inner_w * PICTURE_ASPECT);
    let body_y = picture.y + picture.h + gap;
    let body_h = paint_text_block(
        ts,
        &mut text,
        &tip.body,
        pad,
        body_y,
        TypeToken::Xs.px(),
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    let card_h = body_y + body_h + pad;

    let card = place(item, TIP_W, card_h, gap, viewport);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, card, radius, resolve(ColorToken::Bg2, theme));
    crate::paint::stroke_frame(
        scene,
        card,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );
    let frame = Rect::new(card.x + picture.x, card.y + picture.y, picture.w, picture.h);
    fill_rounded_rect(
        scene,
        frame,
        crate::paint::frame_radius(theme, Radius::Sm.px()),
        resolve(ColorToken::Bg3, theme),
    );
    crate::widget::illustration::paint_illustration(scene, frame, tip.picture, theme);
    scene.append_transformed(
        &text,
        Affine::translate((f64::from(card.x), f64::from(card.y))),
    );
    card
}

/// **Onde o cartão cai:** à direita do item, ou à esquerda se ali não cabe; preso dentro do `viewport` na
/// vertical, alinhado ao topo do item.
pub(super) fn place(item: Rect, w: f32, h: f32, gap: f32, viewport: Rect) -> Rect {
    let right = viewport.x + viewport.w - gap;
    let x = if item.x + item.w + gap + w <= right {
        item.x + item.w + gap
    } else {
        (item.x - gap - w).max(viewport.x + gap)
    };
    let bottom = viewport.y + viewport.h - gap;
    let y = item.y.min(bottom - h).max(viewport.y + gap);
    Rect::new(x, y, w, h)
}

#[cfg(test)]
#[path = "tip_tests.rs"]
mod tests;
