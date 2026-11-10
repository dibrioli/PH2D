//! ⭐ **A faixa «Markers»** (doc 17 item 2, maquete B6), no pé da lista, só nos cartões: o chip de cada
//! marcador (raio ⚡ com sinal, etiqueta só com nome), o nome ao lado (virado para a esquerda quando não
//! cabe à direita), o trecho armado como caixa entre os dois marcadores e o vão clicável entre vizinhos.
//! Os gestos do marcador (arrastar, Alt+clique, menu) moram no chip — os mesmos ids e `TimelineHitKind`
//! da bandeirola da régua do clássico. A régua dos cartões deixa de pintar marcadores.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, label_column_width, paint_icon, paint_text_elided, resolve,
    stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::chrome::{
    TIMELINE_MARCADOR_CHIP_PX as CHIP, TIMELINE_MARCADOR_ICONE_PX as ICONE,
    TIMELINE_MARCADOR_NOME_DX_PX as DX, TIMELINE_MARCADOR_RAIO_PX as RAIO,
    TIMELINE_MARCADOR_SO_NOME_FIO_ALPHA, TIMELINE_MARCADOR_SO_NOME_FUNDO_ALPHA,
    TIMELINE_TRECHO_FOLGA_PX as FOLGA,
};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, StrokeToken, Theme, TypeToken};

use crate::alca_estado::EstadoAlca;
use crate::graph::TimeView;
use crate::{geom, ids, ruler_trecho, tab::Tab};

/// A faixa existe onde a régua mostraria os marcadores (Arrange; Keys sem pilha), com ≥ 1 marcador.
pub(crate) fn mostra(snap: &TimelineViewSnapshot, tab: Tab) -> bool {
    ph2d_editor_core::panel::cards::ferramenta::cards_on()
        && !snap.markers.is_empty()
        && crate::ruler_clock::clock_for(tab, snap).markers
}

/// A altura dela: uma linha de faixa da aba Keys (`.c13ln`, 22 px), ou nada.
pub(crate) fn h(snap: &TimelineViewSnapshot, tab: Tab) -> f32 {
    if mostra(snap, tab) { ROW_H_PX } else { 0.0 }
}

/// `(nome à direita?, x do texto, largura máxima)`: à direita do chip até ao chip seguinte (ou à borda);
/// se o nome não cabe aí, à esquerda, até ao chip anterior (a `.c13ml.rl` da maquete).
fn lugar_do_nome(xs: &[f32], i: usize, (esq, dir): (f32, f32), w: f32) -> (f32, f32) {
    let x = xs[i];
    let prox = xs
        .iter()
        .filter(|&&o| o > x)
        .fold(dir, |f, &o| f.min(o - DX));
    let ant = xs
        .iter()
        .filter(|&&o| o < x)
        .fold(esq, |f, &o| f.max(o + DX));
    let (livre_d, livre_e) = (prox - x - DX, x - DX - ant);
    if w <= livre_d || livre_d >= livre_e {
        (x + DX, livre_d.max(0.0))
    } else {
        let m = w.min(livre_e.max(0.0));
        (x - DX - m, m)
    }
}

/// Pinta a faixa em `y` (logo a seguir à última faixa da lista) e regista o vão e os chips.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: TimeView,
    tab: Tab,
    snap: &TimelineViewSnapshot,
    y: f32,
) {
    let region = g.rows;
    if !mostra(snap, tab) || y >= region.y + region.h || y + ROW_H_PX <= region.y {
        return;
    }
    let amarelo = resolve(ColorToken::TimelineMarker, theme);
    // A coluna de nomes: o ícone da etiqueta e «Markers», na tinta dos marcadores (`.c13n.sg`).
    let x0 = region.x + Spacing::Sm.px();
    crate::linhas::paint_icone(ctx, IconId::Tag, (x0, y), amarelo);
    let tx = x0 + crate::linhas::icone_w(Some(IconId::Tag));
    let font = TypeToken::Sm.px();
    let (ts, sc) = (&mut *ctx.text_system, &mut *ctx.scene);
    let rotulo = ph2d_i18n::tr("panel.timeline.markers_row");
    let max = (region.x + g.label_w - tx).max(0.0);
    paint_text_elided(
        ts,
        sc,
        rotulo,
        tx,
        y + folga_do_texto(ROW_H_PX, font),
        font,
        max,
        amarelo,
    );

    let clock = crate::ruler_clock::clock_for(tab, snap);
    let (fundo, fio) = ruler_trecho::tintas(theme);
    let caixa =
        |x0: f32, x1: f32| Rect::new(x0, y + FOLGA, (x1 - x0).max(0.0), ROW_H_PX - FOLGA * 2.0);
    let fino = StrokeToken::Thin.px();
    if let Some((a, b)) = ruler_trecho::vale(snap, clock) {
        let r = caixa(view.x(a), view.x(b));
        fill_rounded_rect(ctx.scene, r, RAIO, fundo);
        stroke_rounded_rect(ctx.scene, r, RAIO, fino, fio);
    }
    // O vão entre dois vizinhos: o clique toca o trecho; sob o ponteiro, o fio da caixa que tocaria.
    if ruler_trecho::vista(snap, clock) {
        for (a, b) in ruler_trecho::vizinhos(snap) {
            let (xa, xb) = (view.x(snap.markers[a].0), view.x(snap.markers[b].0));
            let (h0, h1) = (xa + CHIP * 0.5, xb - CHIP * 0.5);
            if h1 <= h0 {
                continue;
            }
            let id = ids::timeline_marker_gap_hit_id(a);
            if EstadoAlca::nos_cartoes(ctx, id) != EstadoAlca::Repouso {
                stroke_rounded_rect(ctx.scene, caixa(xa, xb), RAIO, fino, fio);
            }
            let hit = Rect::new(h0, y, h1 - h0, ROW_H_PX);
            registra(ctx, id, TimelineHitKind::MarkerGap { a, b }, hit);
        }
    }
    chips(ctx, theme, (view, y), snap, amarelo);
}

/// O chip, o nome e o alvo de cada marcador (depois do vão: o chip ganha-lhe).
fn chips(
    ctx: &mut PaintCtx,
    theme: Theme,
    (view, y): (TimeView, f32),
    snap: &TimelineViewSnapshot,
    amarelo: ph2d_vector::Color,
) {
    let xs: Vec<f32> = snap.markers.iter().map(|m| view.x(m.0)).collect();
    let (texto, cy) = (resolve(ColorToken::Text1, theme), y + ROW_H_PX * 0.5);
    let font = TypeToken::Xs.px();
    let so_nome = crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::TextoValor, theme);
    let alvo_w = crate::ruler::MARKER_W + crate::ruler::MARKER_HIT_PAD * 2.0;
    for (index, (t, nome, sinal)) in snap.markers.iter().enumerate() {
        let x = xs[index];
        if x < view.time_x - CHIP || x > view.right + CHIP {
            continue;
        }
        let r = Rect::new(x - CHIP * 0.5, cy - CHIP * 0.5, CHIP, CHIP);
        let id = ids::timeline_marker_hit_id(index);
        let e = EstadoAlca::nos_cartoes(ctx, id);
        let (cor, icone, tinta) = if sinal.is_some() {
            (amarelo, IconId::Bolt, resolve(ColorToken::BgElev, theme))
        } else {
            let c = texto.multiply_alpha(TIMELINE_MARCADOR_SO_NOME_FUNDO_ALPHA);
            (c, IconId::Tag, texto)
        };
        if e == EstadoAlca::Repouso {
            fill_rounded_rect(ctx.scene, r, RAIO, cor);
        } else {
            crate::alca_estado::pinta(ctx, theme, e, cor, &[r], &[]);
            crate::alca_estado::pinta_leitura(ctx, theme, e, *t, r.x, cy, true);
        }
        if sinal.is_none() {
            let f = texto.multiply_alpha(TIMELINE_MARCADOR_SO_NOME_FIO_ALPHA);
            stroke_rounded_rect(ctx.scene, r, RAIO, StrokeToken::Thin.px(), f);
        }
        let ic = Rect::new(x - ICONE * 0.5, cy - ICONE * 0.5, ICONE, ICONE);
        paint_icon(ctx.scene, icone, ic, tinta, StrokeToken::Default.px());
        let w = label_column_width(ctx.text_system, font, [nome.as_str()]);
        let (nx, max) = lugar_do_nome(&xs, index, (view.time_x, view.right), w);
        let cor_nome = if sinal.is_some() { amarelo } else { so_nome };
        let ny = y + folga_do_texto(ROW_H_PX, font);
        let (ts, sc) = (&mut *ctx.text_system, &mut *ctx.scene);
        paint_text_elided(ts, sc, nome, nx, ny, font, max, cor_nome);
        let hit = Rect::new(x - alvo_w * 0.5, y, alvo_w, ROW_H_PX);
        registra(ctx, id, TimelineHitKind::Marker { index }, hit);
    }
}

fn registra(ctx: &mut PaintCtx, id: ph2d_a11y::NodeId, kind: TimelineHitKind, hit: Rect) {
    ctx.host.store_mut().register(
        id,
        InteractiveState::TimelineSurface {
            parent: ph2d_editor_core::ids::TIMELINE_PANEL,
            kind,
            canvas: hit,
        },
    );
    ctx.host.hit_index_mut().register(id, hit);
}

/// ⭐ **A pílula do trecho na barra** (`.c13` da B6): verde na tinta do trecho, «▷ takeoff → top ✕»;
/// o alvo é a pílula inteira (o ✕ e o texto devolvem o loop do clipe).
pub(crate) fn pilula_w(ctx: &mut PaintCtx, nome: &str) -> f32 {
    let tw = label_column_width(ctx.text_system, TypeToken::Sm.px(), [nome]);
    Spacing::Sm.px() * 2.0 + ICONE * 2.0 + Spacing::Xs.px() * 2.0 + tw
}

/// Pinta a pílula em `r` (de [`pilula_w`] de largura) e regista o alvo.
pub(crate) fn pilula(ctx: &mut PaintCtx, theme: Theme, r: Rect, nome: &str) {
    let font = TypeToken::Sm.px();
    let (pad, gap) = (Spacing::Sm.px(), Spacing::Xs.px());
    let tw = label_column_width(ctx.text_system, font, [nome]);
    let (fundo, fio) = ruler_trecho::tintas(theme);
    let id = ids::TIMELINE_STRETCH_CLEAR;
    let raio = r.h * 0.5;
    fill_rounded_rect(ctx.scene, r, raio, fundo);
    if ctx.host.store().hover_live(id) > 0.0 {
        fill_rounded_rect(ctx.scene, r, raio, fundo);
    }
    stroke_rounded_rect(ctx.scene, r, raio, StrokeToken::Thin.px(), fio);
    let verde = crate::folha_tinta::tinta(ph2d_tokens::TimelineTinta::LacoDaRegua, theme);
    let cy = r.y + r.h * 0.5;
    let icone = |x: f32| Rect::new(x, cy - ICONE * 0.5, ICONE, ICONE);
    let traco = StrokeToken::Default.px();
    paint_icon(ctx.scene, IconId::Play, icone(r.x + pad), verde, traco);
    let tx = r.x + pad + ICONE + gap;
    let texto = resolve(ColorToken::Text1, theme);
    let ty = r.y + folga_do_texto(r.h, font);
    paint_text_elided(ctx.text_system, ctx.scene, nome, tx, ty, font, tw, texto);
    paint_icon(ctx.scene, IconId::Close, icone(tx + tw + gap), texto, traco);
    ctx.host.hit_index_mut().register(id, r);
}

#[cfg(test)]
#[path = "faixa_marcadores_tests.rs"]
mod tests;
