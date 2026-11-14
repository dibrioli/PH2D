//! ⭐ **A lei do ALVO do cartão da chave da Timeline** (dono 13/11): que menu é cartão, que ids são
//! dele, e o que está sob o cursor no editor da curva Custom. Mora com quem a CORRE (o despacho do
//! ponteiro, `dispatch::key_card_drag` e `pointer_down_menus`); o `screens::hero::key_card` só pinta.

use crate::ids as c;
use crate::interaction::state::{BezAlvo, BezArea};
use crate::interaction::{ContextMenuKind, TimelineInterpScope};
use ph2d_a11y::NodeId;
use ph2d_anim::{EaseSpline, HandleKind, SplineSide};

/// As alças da curva Linear: o editor Custom de uma chave sem Bézier normalizável parte daqui.
pub(crate) const BEZ_LINEAR: [f32; 4] = [1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0]; // LITERAL-PX-OK: terços do tempo da Bézier linear (matemática)

/// O raio do alvo de clique de um ponto ou alça do editor.
const ALVO_R: f32 = 10.0; // LITERAL-PX-OK: raio do alvo de um ponto ou alça
/// A distância (px, na vertical) até à curva em que um clique cria um ponto.
const CURVA_HIT: f32 = 8.0; // LITERAL-PX-OK: alcance do clique que cria um ponto

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

/// O escopo pinta-se como cartão (uma chave ou uma coluna do Summary; o fade continua em lista).
#[must_use]
pub fn e_cartao(kind: ContextMenuKind) -> bool {
    matches!(
        kind,
        ContextMenuKind::TimelineSegment {
            scope: TimelineInterpScope::Key { .. } | TimelineInterpScope::Column { .. }
        }
    )
}

/// `id` é um alvo do cartão (o fundo, o título, um botão, o Rove, o Done): um aperto nele NÃO fecha
/// o cartão.
#[must_use]
pub fn e_alvo_do_cartao(id: NodeId) -> bool {
    id == c::CTX_MENU_TL_KEY_CARD
        || id == c::CTX_MENU_TL_KEY_CARD_HEAD
        || c::TIMELINE_KEY_CARD.iter().any(|r| r.0 == id)
        || c::TIMELINE_SEGMENT_MENU.iter().any(|r| r.0 == id)
        || c::TIMELINE_EASE_MENU.iter().any(|r| r.0 == id)
        || id == c::CTX_MENU_TL_KEY_BEZ_AREA
        || TIPOS.iter().any(|t| t.0 == id)
}

/// A curva de partida de uma chave sem forma normalizável: a Linear.
#[must_use]
pub(crate) fn linear() -> EaseSpline {
    let [x1, y1, x2, y2] = BEZ_LINEAR;
    EaseSpline::from_bezier(x1, y1, x2, y2)
}

/// Os alvos que se desenham, na ordem em que se procuram: primeiro os pontos, depois as alças.
pub(crate) fn alvos(c: &EaseSpline) -> Vec<(BezAlvo, [f32; 2])> {
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
