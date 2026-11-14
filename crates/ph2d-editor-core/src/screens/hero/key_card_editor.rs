//! ⭐ **O editor da curva Custom** no cartão da chave (dono 13/11: «ao escolher Custom deve abrir
//! abaixo a janela de edição» · «permita criar e deletar pontos na curva custom»): o gráfico do
//! quadrado unitário com a curva da chave ([`ph2d_anim::EaseSpline`], a do documento), os pontos e
//! as alças. Clique na curva cria um ponto; botão direito num ponto apaga-o; pontos e alças
//! arrastam-se (`interaction::dispatch::key_card_drag`; a timeline aplica). Mora no lugar de Ease e
//! Curve shape, que a Custom não usa.

use super::*;
use crate::interaction::state::{BezAlvo, BezArea};
use ph2d_anim::{EaseSpline, HandleKind, SplineSide};

/// Os tipos de alça do ponto escolhido, na ordem e com os nomes do cartão de ponto do Vector
/// (`ph2d-panel-vector cards_ponto`; as mesmas chaves de texto).
pub(crate) const TIPOS: [(NodeId, HandleKind, &str); 3] = [
    (
        c::CTX_MENU_TL_KEY_HANDLE_CORNER,
        HandleKind::Corner,
        "panel.vector.vertex.corner",
    ),
    (
        c::CTX_MENU_TL_KEY_HANDLE_SMOOTH,
        HandleKind::Smooth,
        "panel.vector.vertex.smooth",
    ),
    (
        c::CTX_MENU_TL_KEY_HANDLE_SYMM,
        HandleKind::Symmetric,
        "panel.vector.vertex.symm",
    ),
];

/// A faixa de `y` que o editor mostra: o quadrado 0–1 no meio, folga para passar (overshoot).
const Y_MIN: f32 = -0.5; // LITERAL-PX-OK: faixa do gráfico em unidades da curva, não px
const Y_MAX: f32 = 1.5; // LITERAL-PX-OK: faixa do gráfico em unidades da curva, não px
/// O raio de uma alça e o de um ponto, o de quem está agarrado, e o raio do alvo de clique.
const ALCA_R: f32 = 5.0; // LITERAL-PX-OK: raio da alça do editor
const PONTO_R: f32 = 6.0; // LITERAL-PX-OK: raio do ponto da curva
const AGARRADO_R: f32 = 7.5; // LITERAL-PX-OK: raio do que está agarrado
const ALVO_R: f32 = 10.0; // LITERAL-PX-OK: raio do alvo de um ponto ou alça
/// A distância (px, na vertical) até à curva em que um clique cria um ponto.
const CURVA_HIT: f32 = 8.0; // LITERAL-PX-OK: alcance do clique que cria um ponto
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

/// A curva de partida de uma chave sem forma normalizável: a Linear.
#[must_use]
pub(crate) fn linear() -> EaseSpline {
    let [x1, y1, x2, y2] = BEZ_LINEAR;
    EaseSpline::from_bezier(x1, y1, x2, y2)
}

/// Os alvos que se desenham, na ordem em que se procuram: primeiro os pontos, depois as alças.
fn alvos(c: &EaseSpline) -> Vec<(BezAlvo, [f32; 2])> {
    let n = c.node_count();
    let mut v: Vec<(BezAlvo, [f32; 2])> = (1..n - 1)
        .map(|i| (BezAlvo::Ponto(i), c.node(i).p))
        .collect();
    for i in 0..n {
        let no = c.node(i);
        if i > 0 {
            v.push((BezAlvo::Alca(i, SplineSide::In), no.h_in));
        }
        if i < n - 1 {
            v.push((BezAlvo::Alca(i, SplineSide::Out), no.h_out));
        }
    }
    v
}

/// O que está sob `(x, y)` no editor: o ponto ou a alça mais perto, dentro de [`ALVO_R`].
#[must_use]
pub(crate) fn alvo_em(c: &EaseSpline, area: &BezArea, x: f32, y: f32) -> Option<BezAlvo> {
    alvos(c)
        .into_iter()
        .map(|(a, q)| {
            let (px, py) = area.tela(q[0], q[1]);
            (a, (px - x).powi(2) + (py - y).powi(2))
        })
        .filter(|(_, d)| *d <= ALVO_R * ALVO_R)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(a, _)| a)
}

/// `Some(u)` se `(x, y)` está em cima da curva (na vertical, a [`CURVA_HIT`]): onde criar um ponto.
#[must_use]
pub(crate) fn na_curva(c: &EaseSpline, area: &BezArea, x: f32, y: f32) -> Option<f64> {
    let (u, _) = area.unidade(x, y);
    let (_, cy) = area.tela(u, c.eval(f64::from(u)) as f32);
    ((cy - y).abs() <= CURVA_HIT).then_some(f64::from(u))
}

/// Pinta o editor em `r` (o rótulo e a caixa do gráfico) e regista a área.
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
    // Os tipos de alça numa fileira por baixo do gráfico.
    let fila = Rect::new(r.x, r.y + r.h - m::FIELD_H, r.w, m::FIELD_H);
    let caixa = Rect::new(r.x, top, r.w, fila.y - m::FIELD_GAP - top);
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
    let linha = |scene: &mut VectorScene, a: [f32; 2], b: [f32; 2], w: f32, cor| {
        stroke_polyline(
            scene,
            &[area.tela(a[0], a[1]), area.tela(b[0], b[1])],
            w,
            cor,
        );
    };
    // A grelha: quartos do tempo, o meio do valor; o 0 e o 1 mais fortes; a diagonal (a Linear).
    const QUARTOS: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0]; // LITERAL-PX-OK: quartos do tempo normalizado
    for u in QUARTOS {
        linha(scene, [u, Y_MIN], [u, Y_MAX], 1.0, fg.with_alpha(GUIDE_A));
    }
    linha(scene, [0.0, 0.5], [1.0, 0.5], 1.0, fg.with_alpha(GUIDE_A));
    for v in [0.0, 1.0] {
        linha(scene, [0.0, v], [1.0, v], 1.0, fg.with_alpha(LINHA_A));
    }
    linha(scene, [0.0, 0.0], [1.0, 1.0], 1.0, fg.with_alpha(GUIDE_A));

    let curva = store.key_card_curve().unwrap_or_else(linear);
    // Os braços das alças, debaixo de tudo.
    let n = curva.node_count();
    for i in 0..n {
        let no = curva.node(i);
        if i > 0 {
            linha(scene, no.p, no.h_in, BRACO_W, fg.with_alpha(BRACO_A));
        }
        if i < n - 1 {
            linha(scene, no.p, no.h_out, BRACO_W, fg.with_alpha(BRACO_A));
        }
    }
    let tela: Vec<(f32, f32)> = (0..=AMOSTRAS)
        .map(|k| {
            let u = k as f32 / AMOSTRAS as f32;
            area.tela(u, curva.eval(f64::from(u)) as f32)
        })
        .collect();
    stroke_polyline(scene, &tela, CURVA_W, acento);
    for q in [[0.0, 0.0], [1.0, 1.0]] {
        let (x, y) = area.tela(q[0], q[1]);
        fill_circle(scene, x, y, DOT_R * 1.5, fg); // LITERAL-PX-OK: a alça é o ponto vezes uma razão, não uma medida nova
    }
    let agarrado = store.key_card_bez_alvo();
    let escolhido = store.key_card_bez_escolhido();
    let fundo = ink.color(ColorToken::CardField);
    for (a, q) in alvos(&curva) {
        let (x, y) = area.tela(q[0], q[1]);
        let preso = agarrado == Some(a);
        match a {
            // Um ponto: no acento, com o miolo claro (a forma que se apaga com o direito); o
            // Corner é quadrado, o liso redondo; o escolhido maior e de miolo no acento.
            BezAlvo::Ponto(i) => {
                let raio = if preso || escolhido == Some(i) {
                    AGARRADO_R
                } else {
                    PONTO_R
                };
                let miolo = if escolhido == Some(i) { acento } else { fg };
                if curva.kind(i) == Some(HandleKind::Corner) {
                    let q = |r: f32| Rect::new(x - r, y - r, r * 2.0, r * 2.0);
                    fill_rounded_rect(scene, q(raio), BRACO_W, acento);
                    fill_rounded_rect(scene, q(raio - ALCA_ANEL), BRACO_W, miolo);
                } else {
                    fill_circle(scene, x, y, raio, acento);
                    fill_circle(scene, x, y, raio - ALCA_ANEL, miolo);
                }
            }
            // Uma alça: um anel no acento (cheio enquanto agarrada).
            BezAlvo::Alca(..) => {
                let raio = if preso { AGARRADO_R } else { ALCA_R };
                fill_circle(scene, x, y, raio, acento);
                if !preso {
                    fill_circle(scene, x, y, raio - ALCA_ANEL, fundo);
                }
            }
        }
    }

    // Corner · Smooth · Symmetric do ponto escolhido; sem ponto escolhido, apagados e sem nenhum aceso.
    let tipo = escolhido.and_then(|i| curva.kind(i));
    let segs: Vec<(&str, bool, NodeId)> = TIPOS
        .iter()
        .map(|&(id, k, chave)| (tr(chave), tipo == Some(k), id))
        .collect();
    crate::widget::panel_chrome::paint_segmented_group(
        fila, &segs, scene, ts, ink.theme, store, hit,
    );
    if tipo.is_none() {
        let cor = ink.color(ColorToken::KanbanCard).with_alpha(super::VEU_A);
        fill_rounded_rect(scene, fila, m::FIELD_RADIUS, cor);
    }
}
