//! ⭐ **O editor da curva Custom** no cartão da chave (dono 13/11: «ao escolher Custom deve abrir
//! abaixo a janela de edição»): o gráfico do quadrado unitário com a Bézier da chave e as duas alças
//! que se arrastam (o arrasto vive em `interaction::dispatch::key_card_drag`, e a timeline aplica-o).
//! Mora no lugar de Ease e Curve shape, que a Custom não usa.

use super::*;
use crate::interaction::state::BezArea;

/// A faixa de `y` que o editor mostra: o quadrado 0–1 no meio, folga para passar (overshoot).
const Y_MIN: f32 = -0.5; // LITERAL-PX-OK: faixa do gráfico em unidades da curva, não px
const Y_MAX: f32 = 1.5; // LITERAL-PX-OK: faixa do gráfico em unidades da curva, não px
/// O raio de uma alça, o dela sob o rato/agarrada, e a meia-largura do alvo de clique.
const ALCA_R: f32 = 6.0; // LITERAL-PX-OK: raio da alça do editor
const ALCA_R_QUENTE: f32 = 7.5; // LITERAL-PX-OK: raio da alça sob o rato
const ALCA_HIT: f32 = 12.0; // LITERAL-PX-OK: meia-largura do alvo da alça
/// O miolo vazado da alça (o anel tem esta espessura).
const ALCA_ANEL: f32 = 2.5; // LITERAL-PX-OK: espessura do anel da alça
/// A espessura da curva e a dos braços das alças.
const CURVA_W: f32 = 2.5; // LITERAL-PX-OK: traço da curva no editor
const BRACO_W: f32 = 1.25; // LITERAL-PX-OK: traço do braço da alça
/// A opacidade dos braços e das linhas 0 e 1 (as guias de dentro usam `GUIDE_A`).
const BRACO_A: f32 = 0.55; // LITERAL-PX-OK: opacidade do braço (não é medida de UI)
const LINHA_A: f32 = 0.3; // LITERAL-PX-OK: opacidade das linhas 0/1 (não é medida de UI)
/// Amostras da curva do editor.
const AMOSTRAS: usize = 64;

/// O ponto `t` da Bézier `P0 = (0,0) · P1 · P2 · P3 = (1,1)` no quadrado unitário.
pub(super) fn ponto(h: [f32; 4], t: f32) -> (f32, f32) {
    let s = 1.0 - t;
    let b = |p1: f32, p2: f32| 3.0 * s * s * t * p1 + 3.0 * s * t * t * p2 + t * t * t;
    (b(h[0], h[2]), b(h[1], h[3]))
}

/// Pinta o editor em `r` (o rótulo e a caixa do gráfico) e regista a área e as duas alças.
pub(super) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    hit: &mut HitIndex,
    store: &WidgetStore,
    r: Rect,
    ink: CardInk,
) {
    let rot = Rect::new(r.x, r.y + m::LABEL_TOP, r.w, m::LABEL_PX);
    let label = tr("chrome.timeline_menu.section_custom");
    crate::widget::card_field::paint_label(scene, ts, ink, rot.x, rot.y, rot.w, label);
    let top = rot.y + rot.h + m::LABEL_BOTTOM;
    let caixa = Rect::new(r.x, top, r.w, r.y + r.h - top);
    fill_rounded_rect(
        scene,
        caixa,
        m::FIELD_RADIUS,
        ink.color(ColorToken::CardField),
    );
    hit.register(c::CTX_MENU_TL_KEY_BEZ_AREA, caixa);
    let pad = Spacing::Lg.px();
    let area = BezArea {
        rect: Rect::new(
            caixa.x + pad,
            caixa.y + pad,
            caixa.w - pad * 2.0,
            caixa.h - pad * 2.0,
        ),
        y_min: Y_MIN,
        y_max: Y_MAX,
    };
    store.set_key_card_bez_area(Some(area));

    let fg = ink.color(ColorToken::KanbanCardFg);
    let acento = ink.color(ColorToken::KanbanBlueBold);
    let linha = |scene: &mut VectorScene, a: (f32, f32), b: (f32, f32), w: f32, cor| {
        stroke_polyline(scene, &[area.tela(a.0, a.1), area.tela(b.0, b.1)], w, cor);
    };
    // A grelha: quartos do tempo, o meio do valor; o 0 e o 1 mais fortes; a diagonal (a Linear).
    for u in [0.0, 0.25, 0.5, 0.75, 1.0] {
        linha(scene, (u, Y_MIN), (u, Y_MAX), 1.0, fg.with_alpha(GUIDE_A));
    }
    linha(scene, (0.0, 0.5), (1.0, 0.5), 1.0, fg.with_alpha(GUIDE_A));
    for v in [0.0, 1.0] {
        linha(scene, (0.0, v), (1.0, v), 1.0, fg.with_alpha(LINHA_A));
    }
    linha(scene, (0.0, 0.0), (1.0, 1.0), 1.0, fg.with_alpha(GUIDE_A));

    let h = store.key_card_bezier().unwrap_or(BEZ_LINEAR);
    let (p1, p2) = ((h[0], h[1]), (h[2], h[3]));
    linha(scene, (0.0, 0.0), p1, BRACO_W, fg.with_alpha(BRACO_A));
    linha(scene, (1.0, 1.0), p2, BRACO_W, fg.with_alpha(BRACO_A));
    let curva: Vec<(f32, f32)> = (0..=AMOSTRAS)
        .map(|i| {
            let (u, y) = ponto(h, i as f32 / AMOSTRAS as f32);
            area.tela(u, y)
        })
        .collect();
    stroke_polyline(scene, &curva, CURVA_W, acento);
    for (u, v) in [(0.0, 0.0), (1.0, 1.0)] {
        let (x, y) = area.tela(u, v);
        fill_circle(scene, x, y, DOT_R * 1.5, fg);
    }
    let fundo = ink.color(ColorToken::CardField);
    for (i, (id, p)) in [
        (c::CTX_MENU_TL_KEY_BEZ_P1, p1),
        (c::CTX_MENU_TL_KEY_BEZ_P2, p2),
    ]
    .into_iter()
    .enumerate()
    {
        let (x, y) = area.tela(p.0, p.1);
        let quente = store.hot_id() == Some(id) || store.key_card_bez_alca() == Some(i);
        let raio = if quente { ALCA_R_QUENTE } else { ALCA_R };
        fill_circle(scene, x, y, raio, acento);
        if !quente {
            fill_circle(scene, x, y, raio - ALCA_ANEL, fundo);
        }
        let alvo = Rect::new(x - ALCA_HIT, y - ALCA_HIT, ALCA_HIT * 2.0, ALCA_HIT * 2.0);
        hit.register(id, alvo);
    }
}
