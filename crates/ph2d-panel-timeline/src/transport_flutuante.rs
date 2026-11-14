//! ⭐ **Os controlos de tocar numa ABA do painel** (dono 13/11): colada ao topo da moldura, no canto
//! esquerdo, com a tinta da lista — painel e aba leem-se como UM objecto; não ocupa fileira nenhuma.
//! ● ⏮ ‹ ▶ ■ › ⏭: o ● vermelho é o Auto Key (a caixa saiu da barra; o clique é o do mesmo id), o ▶
//! verde, o ■ o Stop (pára e volta ao início).

use super::*;

/// As células, na ordem da fila.
const N: usize = 7;

/// A folga entre a fila e a borda da aba.
const FOLGA: f32 = m::SEG_PAD * 2.0;

/// As células e a tinta de cada uma: o gravar vermelho, o tocar verde; o resto, a de sempre.
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
        [gravar, inicio, antes, tocar, parar, depois, fim],
        [
            Some(ColorToken::Danger),
            None,
            None,
            Some(ColorToken::Success),
            None,
            None,
            None,
        ],
    )
}

/// A aba: em cima da lista do painel, encostada à esquerda dela (dentro da janela).
pub(crate) fn aba(painel: Rect, viewport: Rect) -> Rect {
    let lista = banda::geometria(painel).lista;
    // Sem folga em baixo: o fundo da caixa dos botões assenta na linha do topo da lista (dono 13/11).
    let (w, h) = (banda::icones_w(N) + FOLGA * 2.0, m::FIELD_H + FOLGA);
    Rect::new(lista.x, (lista.y - h).max(viewport.y), w, h)
}

/// Pinta a aba e a fila. A tinta é a da lista e a aba desce um raio para dentro da moldura: o canto
/// de cima-esquerdo do painel fica coberto e a junta não se vê.
pub(super) fn pinta(
    ctx: &mut PaintCtx,
    theme: Theme,
    painel: Rect,
    (snap, view): (&TimelineViewSnapshot, BarView),
) {
    let a = aba(painel, ctx.viewport);
    ph2d_editor_core::screens::hero::bottom_hud::publica_fila_flutuante(a);
    let raio = m::LIST_RADIUS;
    ph2d_editor_core::paint::fill_rounded_rect_radii(
        ctx.scene,
        Rect::new(a.x, a.y, a.w, a.h + raio),
        // A quina de cima-esquerda reta: as abas laterais sobem até ao topo desta, encostadas a ela.
        (0.0, raio, 0.0, 0.0),
        ph2d_editor_core::paint::resolve(ColorToken::KanbanList, theme),
    );
    let ink = CardInk::of_list(theme);
    let r = Rect::new(a.x + FOLGA, a.y + FOLGA, banda::icones_w(N), m::FIELD_H);
    let (icones, cores) = celulas(snap, view);
    // As quinas de baixo da caixa pontudas (dono 13/11); as de cima, as do campo.
    let f = m::FIELD_RADIUS;
    cartao(ctx, ink, r).icones_em_fila_tingidos(
        &icones,
        &cores,
        (r.x, r.y),
        (StrokeToken::Thick, (f, f, 0.0, 0.0)),
    );
    // ⭐ Dono 13/11: ligados, o ● (Auto Key) e o ▶ (a tocar) pulsam com brilho, em ciclo.
    let pulso = ctx.host.store().on_pulse();
    if snap.auto_key {
        brilho(ctx, ink.color(ColorToken::Danger), celula(r, 0), pulso);
    }
    if snap.playing {
        brilho(ctx, ink.color(ColorToken::Success), celula(r, 3), pulso);
    }
    // O gravar é uma bola CHEIA (o glifo do ponto é pequeno demais para ler-se como «gravar»).
    let d = m::ICON_GLYPH * 0.6; // LITERAL-PX-OK: a bola é uma razão do glifo, não uma medida nova
    let cx = r.x + m::FIELD_H * 0.5;
    let bola = Rect::new(cx - d * 0.5, r.y + (r.h - d) * 0.5, d, d);
    ph2d_editor_core::paint::fill_rounded_rect(
        ctx.scene,
        bola,
        d * 0.5,
        ink.color(ColorToken::Danger),
    );
}

/// A célula `i` da fila (cada ícone ocupa um campo quadrado).
fn celula(r: Rect, i: usize) -> Rect {
    Rect::new(r.x + m::FIELD_H * i as f32, r.y, m::FIELD_H, m::FIELD_H)
}

/// O halo de «ligado» sobre a célula `c`: um disco na cor do botão que cresce e acende com o pulso.
fn brilho(ctx: &mut PaintCtx, cor: ph2d_vector::Color, c: Rect, pulso: f32) {
    let alfa = ph2d_tokens::chrome::TIMELINE_LIGADO_BRILHO_ALPHA * pulso;
    let d = m::ICON_GLYPH + (c.w - m::ICON_GLYPH) * pulso;
    let disco = Rect::new(c.x + (c.w - d) * 0.5, c.y + (c.h - d) * 0.5, d, d);
    ph2d_editor_core::paint::fill_rounded_rect(ctx.scene, disco, d * 0.5, cor.multiply_alpha(alfa));
}
