//! ⭐ **A legenda das barras no pé da fita** (doc 15 onda 2, maquete B1): curva · reta · segura, cada
//! uma com a amostra da barra que [`crate::linhas::paint_barras`] pinta. Só nos cartões e só pintura:
//! uma pílula no canto de baixo à direita da área das faixas, sem alvo (o clique passa às faixas).

use ph2d_anim::Interp;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};

use crate::linhas::{BARRA_TH, cor_da_barra, pinta_barra};

/// O comprimento da amostra de cada barra: três tracejados da segura.
const AMOSTRA_W: f32 = crate::linhas::BARRA_DASH * 5.0; // LITERAL-PX-OK: three hold dashes

/// As três entradas, na ordem da maquete.
const ENTRADAS: [(Interp, &str); 3] = [
    (
        Interp::Bezier {
            x1: 0.4,
            y1: 0.0,
            x2: 0.6,
            y2: 1.0,
        },
        "panel.timeline.card.legend_curve",
    ),
    (Interp::Linear, "panel.timeline.card.legend_linear"),
    (Interp::Hold, "panel.timeline.card.legend_hold"),
];

/// Pinta a legenda no canto de baixo à direita de `lanes` (a área das faixas, sem a barra de rolagem).
pub(crate) fn paint(ctx: &mut PaintCtx, theme: Theme, lanes: Rect) {
    let font = TypeToken::Xs.px();
    let (gap, pad) = (Spacing::Xs.px(), Spacing::Sm.px());
    let larguras: Vec<f32> = ENTRADAS
        .iter()
        .map(|(_, k)| ctx.text_system.prefix_width(ph2d_i18n::tr(k), font).ceil())
        .collect();
    let w = pad * 2.0
        + larguras.iter().map(|tw| AMOSTRA_W + gap + tw).sum::<f32>()
        + pad * (ENTRADAS.len() - 1) as f32;
    let h = font + gap * 2.0;
    let pilula = Rect::new(
        lanes.x + lanes.w - w - gap,
        lanes.y + lanes.h - h - gap,
        w,
        h,
    );
    if pilula.x < lanes.x || pilula.y < lanes.y {
        return;
    }
    fill_rounded_rect(
        ctx.scene,
        pilula,
        Radius::Xs.px(),
        resolve(ColorToken::BgElev, theme),
    );
    let cy = pilula.y + h * 0.5;
    let mut x = pilula.x + pad;
    for ((interp, chave), tw) in ENTRADAS.iter().zip(larguras) {
        pinta_barra(
            ctx,
            resolve(cor_da_barra(*interp), theme),
            cor_da_barra(*interp),
            (x, x + AMOSTRA_W),
            cy - BARRA_TH * 0.5,
        );
        x += AMOSTRA_W + gap;
        paint_text_elided(
            ctx.text_system,
            ctx.scene,
            ph2d_i18n::tr(chave),
            x,
            cy - font * 0.5,
            font,
            tw,
            resolve(ColorToken::Text2, theme),
        );
        x += tw + pad;
    }
}
