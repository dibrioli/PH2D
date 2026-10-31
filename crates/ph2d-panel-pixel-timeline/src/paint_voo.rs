//! **O bloco VÊ-SE a mover** (smoke 31/10, D2) — a lei do cartão do Kanban
//! ([`ph2d_editor_core::paint_flight`]): onde o bloco vai cair abre-se o vão (a grelha pinta a ordem
//! prevista, [`crate::gestos::bloco_previsto`]) e o bloco voa com o rato, inclinado, com a sombra
//! de voo, por cima da grelha.

use crate::paint_grid::{Grelha, centrado, ponto};
use ph2d_editor_core::paint::{fill_rounded_rect, frame_radius, resolve, stroke_rounded_rect};
use ph2d_editor_core::paint_flight::{FLIGHT_TILT_DEG, flight_shadow, fly};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::pixel_panel::{PixelCel, PixelPanelInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr_with;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, StrokeToken, TypeToken};
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

fn escolhidas(info: &PixelPanelInfo) -> Vec<usize> {
    (0..info.frames.len())
        .filter(|&c| info.range.frames.contains(&info.frames[c].id))
        .collect()
}

/// O vão: as colunas do intervalo, na ordem prevista, ficam vazias por baixo da moldura.
pub(crate) fn vao(ctx: &mut PaintCtx, info: &PixelPanelInfo, g: &Grelha) {
    let theme = ctx.host.theme();
    let radius = frame_radius(theme, Radius::Sm.px());
    let alto = (g.bottom - g.top).max(0.0);
    for c in escolhidas(info) {
        if (g.first..g.end).contains(&c) {
            let r = Rect::new(g.x(c), g.top, g.col_w, alto);
            fill_rounded_rect(ctx.scene, r, radius, resolve(ColorToken::Bg2, theme));
        }
    }
}

/// O bloco em voo: as colunas do intervalo (na ordem de AGORA, juntas como `move_block` as junta),
/// os números, as durações e os cels, deslocado `dx` do sítio dele.
pub(crate) fn voa(ctx: &mut PaintCtx, info: &PixelPanelInfo, g: &Grelha, dx: f32) {
    let cols = escolhidas(info);
    let Some(&c0) = cols.first() else {
        return;
    };
    let pitch = ph2d_tokens::row_pitch_px();
    let mut filas = Vec::new();
    let mut y = g.rows_y;
    for row in &info.layers {
        if y >= g.top && y + ROW_H_PX <= g.bottom {
            filas.push((row, y - g.head_y));
        }
        y += pitch;
    }
    let fundo = filas.last().map_or(g.top - g.head_y, |(_, y)| y + ROW_H_PX);
    #[expect(clippy::cast_precision_loss, reason = "posições de coluna")]
    let (x0, w) = (
        g.gx + (c0 as f32 - g.first as f32) * g.col_w,
        cols.len() as f32 * g.col_w,
    );
    let r = Rect::new(x0 + dx, g.head_y, w, fundo);
    let cels: BTreeMap<(u32, u32), &PixelCel> =
        info.cels.iter().map(|c| ((c.layer, c.frame), c)).collect();

    let mut local = VectorScene::new();
    std::mem::swap(&mut *ctx.scene, &mut local);
    let theme = ctx.host.theme();
    let radius = frame_radius(theme, Radius::Sm.px());
    let caixa = Rect::new(0.0, 0.0, r.w, r.h);
    flight_shadow(ctx.scene, theme, caixa, radius);
    fill_rounded_rect(ctx.scene, caixa, radius, resolve(ColorToken::BgElev, theme));
    for (k, &c) in cols.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "a coluna no bloco")]
        let cx = k as f32 * g.col_w;
        let f = info.frames[c];
        let numero = (c + 1).to_string();
        let fonte = TypeToken::Sm;
        centrado(
            ctx,
            &numero,
            cx,
            0.0,
            g.col_w,
            ROW_H_PX,
            fonte,
            ColorToken::Text1,
        );
        let duracao = tr_with(
            "panel.pixel.timeline.duration_ms",
            &[("ms", &f.duration_ms)],
        );
        let fonte = TypeToken::Xs;
        let cor = ColorToken::Text3;
        centrado(ctx, &duracao, cx, ROW_H_PX, g.col_w, fonte.px(), fonte, cor);
        for &(row, y) in filas.iter().filter(|(row, _)| !row.group) {
            let cel = cels.get(&(row.id, f.id)).copied();
            ponto(ctx, Rect::new(cx, y, g.col_w, ROW_H_PX), cel);
        }
    }
    let traco = StrokeToken::Thick.px();
    let accent = resolve(ColorToken::Accent, theme);
    stroke_rounded_rect(ctx.scene, caixa, radius, traco, accent); // FRAME-RAW-OK: o bloco agarrado
    std::mem::swap(&mut *ctx.scene, &mut local);
    fly(ctx.scene, &local, r, FLIGHT_TILT_DEG);
}
