//! ⭐ **Os controlos de tocar FLUTUAM acima do painel** (dono 13/11): no canto superior esquerdo, colados
//! ao lado de fora da moldura — não ocupam fileira nenhuma. ⏮ ‹ ■ ▶ › ⏭ ●: o ■ é o Stop (pára e volta
//! ao início), o ▶ verde, o ● vermelho é o Auto Key (a caixa saiu da barra; o clique é o do mesmo id).

use super::*;

/// As células, na ordem da fila.
const N: usize = 7;

/// As células e a tinta de cada uma: o tocar verde, o gravar vermelho; o resto, a de sempre.
fn celulas(snap: &TimelineViewSnapshot, view: BarView) -> ([Icone; N], [Option<ColorToken>; N]) {
    let [inicio, antes, tocar, depois, fim] = fileira_do_transporte(snap, view);
    let vivo = |id, icon, on| Icone {
        id: Some(id),
        icon,
        on,
    };
    let gravar = vivo(TIMELINE_AUTOKEY, IconId::CardDot, snap.auto_key);
    let parar = vivo(ids::TIMELINE_STOP, IconId::Stop, false);
    (
        [inicio, antes, parar, tocar, depois, fim, gravar],
        [
            None,
            None,
            None,
            Some(ColorToken::Success),
            None,
            None,
            Some(ColorToken::Danger),
        ],
    )
}

/// Onde a fila flutua: à esquerda do painel, logo acima da moldura (dentro da janela).
pub(super) fn rect(painel: Rect, viewport: Rect) -> Rect {
    let w = banda::icones_w(N);
    let y = (painel.y - m::FIELD_GAP - m::FIELD_H).max(viewport.y);
    Rect::new(painel.x, y, w, m::FIELD_H)
}

/// Pinta a fila em [`rect`]: o fundo do cartão por baixo, para ler-se sobre o canvas.
pub(super) fn pinta(
    ctx: &mut PaintCtx,
    theme: Theme,
    painel: Rect,
    (snap, view): (&TimelineViewSnapshot, BarView),
) {
    let r = rect(painel, ctx.viewport);
    ph2d_editor_core::screens::hero::bottom_hud::publica_fila_flutuante(r);
    let ink = CardInk::of_list(theme);
    let fundo = Rect::new(
        r.x - m::SEG_PAD,
        r.y - m::SEG_PAD,
        r.w + m::SEG_PAD * 2.0,
        r.h + m::SEG_PAD * 2.0,
    );
    ph2d_editor_core::paint::fill_rounded_rect(
        ctx.scene,
        fundo,
        m::FIELD_RADIUS + m::SEG_PAD,
        ink.color(ColorToken::KanbanList),
    );
    let (icones, cores) = celulas(snap, view);
    cartao(ctx, ink, r).icones_em_fila_tingidos(&icones, &cores, (r.x, r.y), StrokeToken::Thick);
    // O gravar é uma bola CHEIA (o glifo do ponto é pequeno demais para ler-se como «gravar»).
    let d = m::ICON_GLYPH * 0.6;
    let cx = r.x + m::FIELD_H * (N - 1) as f32 + m::FIELD_H * 0.5;
    let bola = Rect::new(cx - d * 0.5, r.y + (r.h - d) * 0.5, d, d);
    ph2d_editor_core::paint::fill_rounded_rect(
        ctx.scene,
        bola,
        d * 0.5,
        ink.color(ColorToken::Danger),
    );
}
