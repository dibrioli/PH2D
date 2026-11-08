//! **A faixa do onion** (P12 R4, Aseprite): com o onion ligado, `[` no quadro actual − `prev` e `]`
//! no actual + `next`, sobre os números dos quadros; segue o quadro actual e corta-se no 1.º e no
//! último. Arrastar um colchete muda `prev`/`next` das preferências do DOCUMENTO (pelo canal da
//! janela Grid & Background: sem passo de desfazer, a família marca a revisão).

use crate::paint_grid::Grelha;
use crate::{event, ids};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::pixel_panel::PixelPanelInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_prefs::canal::{self, PrefsRequest, Scope};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, StrokeToken};

/// As colunas `(de, até)` da faixa: `None` sem onion, sem documento ou sem quadro actual.
pub(crate) fn faixa(info: &PixelPanelInfo) -> Option<(usize, usize)> {
    let o = canal::view()
        .doc
        .filter(|_| info.onion && info.editing)?
        .onion;
    let a = event::activo(info)?;
    let n = info.frames.len();
    let de = a.saturating_sub(usize::try_from(o.prev).unwrap_or(usize::MAX));
    let ate = a.saturating_add(usize::try_from(o.next).unwrap_or(usize::MAX));
    Some((de, ate.min(n.saturating_sub(1))))
}

/// O colchete arrastado está agora sobre a coluna `col`: o alcance novo, ≥ 0 (a coluna já vem
/// presa à grelha, logo cortada no 1.º e no último quadro).
pub(crate) fn arrasta(info: &PixelPanelInfo, right: bool, col: usize) {
    let (Some(mut p), Some(a)) = (canal::view().doc, event::activo(info)) else {
        return;
    };
    let v = u32::try_from(if right {
        col.saturating_sub(a)
    } else {
        a.saturating_sub(col)
    })
    .unwrap_or(u32::MAX);
    let alvo = if right {
        &mut p.onion.next
    } else {
        &mut p.onion.prev
    };
    if *alvo != v {
        *alvo = v;
        canal::request(PrefsRequest::Set(Scope::Document, p));
    }
}

pub(crate) fn pinta(ctx: &mut PaintCtx, info: &PixelPanelInfo, g: &Grelha) {
    let Some((de, ate)) = faixa(info) else {
        return;
    };
    let theme = ctx.host.theme();
    let cor = resolve(ColorToken::Accent, theme);
    let (traco, aba, pega) = (StrokeToken::Thick.px(), Spacing::Xs.px(), Spacing::Sm.px());
    let y = g.head_y;
    for (right, c, id) in [
        (false, de, ids::ONION_EDGES[0]),
        (true, ate, ids::ONION_EDGES[1]),
    ] {
        if c < g.first || c >= g.end {
            continue;
        }
        let x = if right {
            g.x(c) + g.col_w - traco
        } else {
            g.x(c)
        };
        let dentro = if right { x + traco - aba } else { x };
        for r in [
            Rect::new(x, y, traco, ROW_H_PX),
            Rect::new(dentro, y, aba, traco),
            Rect::new(dentro, y + ROW_H_PX - traco, aba, traco),
        ] {
            fill_rounded_rect(ctx.scene, r, 0.0, cor);
        }
        let px = if right { x + traco - pega } else { x };
        ctx.host
            .hit_index_mut()
            .register(id, Rect::new(px, y, pega, ROW_H_PX));
    }
}
