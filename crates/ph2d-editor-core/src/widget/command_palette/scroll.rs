//! **Quanto o conteúdo mede, e o que dele SE VÊ** — a metade de ROLAGEM da paleta.
//!
//! Irmã de [`super`] pelo teto de 500 LOC dos primitivos, e a quinta responsabilidade do widget
//! (desenha · mede o arranjo · entra · anuncia · **cabe**).
//!
//! ⚠️ **Ela nasceu de um report:** com *Show all* ligado a lista transbordava o ecrã e o que
//! sobrava era **inalcançável por gesto nenhum** (Enio, 2026-08-25). O arranjo em masonry cortou o
//! excedente de `345` para `217` px no caso dele — e os `217` que ficam só a roda alcança, o que
//! torna a rolagem obrigatória e não um conforto.

use super::{
    CARD_MAX_H_FRAC, CARD_MAX_W, CardLayout, EDGE_MARGIN, HEADER_H, MIN_COL_W, PILL_H,
    PaletteModel, arrange, filter_model,
};
use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// **Quanto o cartão mede, e onde cada coisa cai** — a PORTA da geometria (F3 / ADR-0166).
///
/// ⚠️ **Ela existe porque a rolagem obrigou a MESMA conta a ser feita duas vezes:** o pintor
/// precisa dela para desenhar, e a [`max_scroll`] para dizer à roda até onde ir. Duas cópias
/// divergiriam no dia em que uma fosse ajustada — e a que decide se há rolagem é a que o artista
/// sente: metade da lista invisível e a roda inerte.
pub(super) struct Metrics {
    pub(super) card: Rect,
    pub(super) content_x: f32,
    pub(super) content_w: f32,
    /// A altura que o conteúdo PEDE (pode passar da janela).
    pub(super) content_h: f32,
    /// A altura que o cartão tem para o conteúdo — o que **cabe**.
    pub(super) view_h: f32,
    pub(super) pad: f32,
    pub(super) font: f32,
    pub(super) placed: Vec<(CardLayout, f32, f32, f32)>,
}

pub(super) fn metrics(
    ts: &mut TextSystem,
    model: &PaletteModel,
    viewport: Rect,
    no_match: bool,
) -> Metrics {
    // ── Card geometry independent of content height: width (centred) + the content box. ──
    let card_w = CARD_MAX_W
        .min(viewport.w - EDGE_MARGIN * 2.0)
        .max(MIN_COL_W);
    let card_x = viewport.x + (viewport.w - card_w) * 0.5;
    let pad = Spacing::Md.px();
    let content_x = card_x + pad;
    let content_w = card_w - pad * 2.0;
    let font = TypeToken::Md.px();

    let (placed, measured) = arrange(ts, model, content_x, content_w);
    // With no matches the arrangement is empty (height 0); reserve a row for the "No matches" message so
    // the card doesn't collapse to just the header.
    let content_h = if no_match {
        TypeToken::Md.px() + Spacing::Lg.px()
    } else {
        measured
    };
    let chrome_h = pad * 2.0 + HEADER_H + Spacing::Sm.px();
    let card_h = (chrome_h + content_h)
        .min(viewport.h * CARD_MAX_H_FRAC)
        .min(viewport.h - EDGE_MARGIN * 2.0);
    let card_y = viewport.y + (viewport.h - card_h) * 0.5;
    Metrics {
        card: Rect::new(card_x, card_y, card_w, card_h),
        content_x,
        content_w,
        content_h,
        // ⚠️ **O que CABE**, e nunca o que se pediu: é esta a diferença que a roda percorre.
        view_h: (card_h - chrome_h).max(0.0),
        pad,
        font,
        placed,
    }
}

/// ⭐ **Até onde a roda pode levar a lista** — `0` quando tudo cabe.
///
/// ⚠️ **O teto vem de QUEM MEDE, não do chamador**, ao contrário da janela do Input Map: ali o
/// conteúdo é uma lista de linhas de altura fixa e a shell sabe contá-las; aqui a altura depende de
/// **medir texto**, e um teto adivinhado levaria a lista para longe com o artista sem saber voltar.
#[must_use]
pub fn max_scroll(ts: &mut TextSystem, model: &PaletteModel, query: &str, viewport: Rect) -> f32 {
    let filtered;
    let model = if query.is_empty() {
        model
    } else {
        filtered = filter_model(model, query);
        &filtered
    };
    let m = metrics(ts, model, viewport, model.groups.is_empty());
    (m.content_h - m.view_h).max(0.0)
}

/// ⭐ **O traço que diz que há mais** — um indicador fino na borda direita do corpo.
///
/// ⚠️ **Sem ele a rolagem é uma feature invisível**, que é a mesma doença do botão morto sob o
/// dedo: o artista não descobre por adivinhação que a roda faz alguma coisa. Ele aparece **só**
/// quando há transbordo, e o comprimento dele é a fração visível — a régua que toda barra de
/// rolagem usa, e que diz de relance *quanto* falta.
///
/// ⛔ Não é um `Scrollbar` registado: ele não se arrasta. Um alvo de arrasto aqui competiria com
/// as pílulas do cartão pelo mesmo `x`, e a roda já resolve — *um segundo caminho para o mesmo
/// gesto é a divergência que esta linha passou a semana a apagar*.
pub(super) fn paint_scroll_hint(
    scene: &mut VectorScene,
    theme: Theme,
    body: Rect,
    content_h: f32,
    view_h: f32,
    scroll: f32,
) {
    if content_h <= view_h || view_h <= 0.0 {
        return;
    }
    let w = 3.0; // LITERAL-PX-OK: espessura do traço indicador
    let frac = (view_h / content_h).clamp(0.0, 1.0); // CLAMP-OK: fracao, limites literais
    let thumb_h = (view_h * frac).max(PILL_H);
    let travel = (view_h - thumb_h).max(0.0);
    let t = (scroll / (content_h - view_h)).clamp(0.0, 1.0); // CLAMP-OK: fracao, limites literais
    fill_rounded_rect(
        scene,
        Rect::new(body.x + body.w - w, body.y + travel * t, w, thumb_h),
        w * 0.5,
        resolve(ColorToken::Border, theme),
    );
}
