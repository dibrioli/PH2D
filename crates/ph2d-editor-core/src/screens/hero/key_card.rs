//! ⭐ **O CARTÃO da chave** — o menu do botão direito numa chave da timeline, no idioma dos cartões
//! do Inspector (dono 13/11: «veja nossos próprios cartões»): título que ARRASTA o cartão, rótulos
//! de secção, controlos segmentados (Interpolation · Ease), a forma da curva numa lista, o Rove numa
//! caixa de marcar e **Done** — as escolhas não o fecham.
//!
//! Só PINTA e diz o que cada clique quer ([`acao`]); quem aplica é o `chrome::timeline_segment`, pela
//! mesma `TimelineInterpPick` do menu em lista — a shell não muda. O estado (o que está aceso, o
//! título) vem do painel da timeline por `WidgetStore::set_menu_marks`; o arrasto vive em
//! `interaction::dispatch::key_card_drag`.

use crate::ids as c;
use crate::interaction::{ContextMenuKind, HitIndex, TimelineInterpScope, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text};
use crate::widget::card_ink::CardInk;
use crate::widget::lista_flutuante::Superficie;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_i18n::{TextKey, tr};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A largura do cartão: cinco segmentos de Interpolation cabem sem cortar «Straight».
const CARD_W: f32 = 352.0; // LITERAL-PX-OK: fotografado — a 300 «Straight», «Nearest» e «Custom» saíam com «…»
/// A largura do botão Done.
const DONE_W: f32 = 76.0; // LITERAL-PX-OK: o botão primário de um cartão, «Done» + folga

/// Interpolation: o id e o rótulo de cada segmento. Curve acende com qualquer ease.
const INTERP: [(NodeId, &str); 5] = [
    (c::CTX_MENU_TL_CURVE, "chrome.timeline_menu.curve"),
    (c::CTX_MENU_TL_LINEAR, "chrome.timeline_menu.straight"),
    (c::CTX_MENU_TL_HOLD, "chrome.timeline_menu.hold"),
    (c::CTX_MENU_TL_NEAREST, "chrome.timeline_menu.nearest"),
    (c::CTX_MENU_TL_CUSTOM, "chrome.timeline_menu.custom"),
];

/// Ease: o segmento, a linha do modo que o acende, o modo que ele pede, o rótulo.
const EASES: [(NodeId, NodeId, u8, &str); 3] = [
    (
        c::CTX_MENU_TL_CURVE_IN,
        c::CTX_MENU_TL_EASE_IN,
        c::TL_EASE_MODE_IN,
        "chrome.timeline_menu.seg_in",
    ),
    (
        c::CTX_MENU_TL_CURVE_INOUT,
        c::CTX_MENU_TL_EASE_INOUT,
        c::TL_EASE_MODE_INOUT,
        "chrome.timeline_menu.seg_in_out",
    ),
    (
        c::CTX_MENU_TL_CURVE_OUT,
        c::CTX_MENU_TL_EASE_OUT,
        c::TL_EASE_MODE_OUT,
        "chrome.timeline_menu.seg_out",
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

/// `id` é um alvo do cartão (o fundo, o título, um segmento, a forma, o Rove, o Done): um aperto nele
/// NÃO fecha o cartão.
#[must_use]
pub fn e_alvo_do_cartao(id: NodeId) -> bool {
    id == c::CTX_MENU_TL_KEY_CARD
        || id == c::CTX_MENU_TL_KEY_CARD_HEAD
        || c::TIMELINE_KEY_CARD.iter().any(|r| r.0 == id)
        || c::TIMELINE_SEGMENT_MENU.iter().any(|r| r.0 == id)
}

/// O que um clique no cartão pede.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Acao {
    /// Nada (o fundo, o título): o cartão fica aberto.
    Nada,
    /// A mesma escolha do menu em lista: `(item, mode)` da `TimelineInterpPick`. O cartão fica.
    Escolhe { item: NodeId, mode: u8 },
    /// Abre a lista das formas, no modo dado (escolher nela volta ao cartão).
    AbreFormas { mode: u8 },
    /// **Done**: fecha o cartão.
    Fecha,
}

/// A forma acesa (ou `Cubic`) e o modo aceso (ou In-Out): o que Curve e o ease mantêm.
fn forma_e_modo(store: &WidgetStore, kind: ContextMenuKind) -> (NodeId, u8) {
    let forma = c::TIMELINE_EASE_MENU
        .iter()
        .map(|&(id, ..)| id)
        .find(|&id| store.menu_row_is_marked(kind, id))
        .unwrap_or(c::CTX_MENU_TL_FAM_CUBIC);
    let modo = EASES
        .iter()
        .find(|e| store.menu_row_is_marked(kind, e.1))
        .map_or(c::TL_EASE_MODE_INOUT, |e| e.2);
    (forma, modo)
}

/// O que o clique em `id` pede, se `id` é do cartão. `None` = segue pelo caminho do menu em lista
/// (Straight, Hold, Nearest, Custom, Rove — os ids de sempre).
#[must_use]
pub fn acao(id: NodeId, kind: ContextMenuKind, store: &WidgetStore) -> Option<Acao> {
    if !e_cartao(kind) {
        return None;
    }
    let (forma, modo) = forma_e_modo(store, kind);
    match id {
        _ if id == c::CTX_MENU_TL_KEY_CARD || id == c::CTX_MENU_TL_KEY_CARD_HEAD => {
            Some(Acao::Nada)
        }
        _ if id == c::CTX_MENU_TL_KEY_DONE => Some(Acao::Fecha),
        _ if id == c::CTX_MENU_TL_CURVE => Some(Acao::Escolhe {
            item: forma,
            mode: modo,
        }),
        _ if id == c::CTX_MENU_TL_SHAPE => Some(Acao::AbreFormas { mode: modo }),
        _ => EASES.iter().find(|e| e.0 == id).map(|e| Acao::Escolhe {
            item: forma,
            mode: e.2,
        }),
    }
}

/// Os rectângulos do cartão ancorado em `(x, y)` — uma conta só para o desenho e o teste.
pub(super) struct Geo {
    pub cartao: Rect,
    pub cabeca: Rect,
    pub rotulo_interp: Rect,
    pub interp: Rect,
    pub rotulo_ease: Rect,
    pub ease: Rect,
    pub rotulo_forma: Rect,
    pub forma: Rect,
    pub rove: Rect,
    pub done: Rect,
}

pub(super) fn geo(x: f32, y: f32) -> Geo {
    let iw = CARD_W - m::CARD_PAD_X * 2.0;
    let x0 = x + m::CARD_PAD_X;
    let cabeca = Rect::new(x, y, CARD_W, m::CARD_PAD_TOP + m::CARD_HEAD_H);
    // Um grupo = o rótulo (LABEL_PX) e o campo (FIELD_H), com as folgas dos cartões.
    let grupo = |top: f32| {
        let rot = Rect::new(x0, top + m::LABEL_TOP, iw, m::LABEL_PX);
        let campo = Rect::new(x0, rot.y + rot.h + m::LABEL_BOTTOM, iw, m::FIELD_H);
        (rot, campo)
    };
    let (rotulo_interp, interp) = grupo(cabeca.y + cabeca.h + m::CARD_HEAD_BOTTOM - m::LABEL_TOP);
    let (rotulo_ease, ease) = grupo(interp.y + interp.h);
    let (rotulo_forma, forma) = grupo(ease.y + ease.h);
    let rove = Rect::new(x0, forma.y + forma.h + m::FIELD_ROW_GAP, iw, m::FIELD_H);
    let done = Rect::new(
        x0 + iw - DONE_W,
        rove.y + rove.h + m::CARD_GAP,
        DONE_W,
        m::FIELD_H,
    );
    let h = done.y + done.h + m::CARD_PAD_BOTTOM - y;
    Geo {
        cartao: Rect::new(x, y, CARD_W, h),
        cabeca,
        rotulo_interp,
        interp,
        rotulo_ease,
        ease,
        rotulo_forma,
        forma,
        rove,
        done,
    }
}

/// A moldura de um cartão do Inspector (sombra, `CARD_RADIUS`, `KanbanCard`), a flutuar.
fn moldura(scene: &mut VectorScene, r: Rect, ink: CardInk) {
    crate::paint_flight::shadow(
        scene,
        r,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanListShadow),
        m::LIST_SHADOW_FAR,
    );
    fill_rounded_rect(
        scene,
        r,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanCard).with_alpha(1.0),
    );
}

/// Pinta o cartão aberto e regista os alvos.
pub(super) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    req: crate::interaction::ContextMenuRequest,
    viewport: Rect,
) {
    let g0 = geo(0.0, 0.0);
    let r = crate::widget::panel_chrome::clamp_menu_to_viewport(
        req.x,
        req.y,
        g0.cartao.w,
        g0.cartao.h,
        viewport,
    );
    let g = geo(r.x, r.y);
    let ink = Superficie::de(theme)
        .tinta()
        .unwrap_or_else(|| CardInk::of_list(theme));
    crate::widget::card_ink::com(ink, || {
        corpo(scene, ts, theme, hit, store, req.kind, &g, ink)
    });
}

#[allow(clippy::too_many_arguments)]
fn corpo(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    kind: ContextMenuKind,
    g: &Geo,
    ink: CardInk,
) {
    moldura(scene, g.cartao, ink);
    hit.register(c::CTX_MENU_TL_KEY_CARD, g.cartao);
    hit.register(c::CTX_MENU_TL_KEY_CARD_HEAD, g.cabeca);
    let marcado = |id| store.menu_row_is_marked(kind, id);
    let quente = |id| store.hot_id() == Some(id);
    let fg = ink.color(ColorToken::KanbanCardFg);

    // O título: o do cartão do Inspector (CARD_TITLE_PX, mais forte), centrado na cabeça.
    let tpx = m::CARD_TITLE_PX;
    crate::paint::paint_text_title(
        ts,
        scene,
        store.menu_title(kind),
        g.cabeca.x + m::CARD_PAD_X,
        g.cabeca.y + m::CARD_PAD_TOP + (m::CARD_HEAD_H - tpx) * 0.5,
        tpx,
        g.cabeca.w - m::CARD_PAD_X * 2.0,
        fg,
    );

    let rotulo = |scene: &mut VectorScene, ts: &mut TextSystem, r: Rect, chave: &str| {
        crate::widget::card_field::paint_label(scene, ts, ink, r.x, r.y, r.w, tr(chave));
    };

    rotulo(
        scene,
        ts,
        g.rotulo_interp,
        "chrome.timeline_menu.section_interp",
    );
    let curva = EASES.iter().any(|e| marcado(e.1));
    let interp: Vec<(&str, bool, NodeId)> = INTERP
        .iter()
        .map(|&(id, chave)| {
            let on = if id == c::CTX_MENU_TL_CURVE {
                curva
            } else {
                marcado(id)
            };
            (tr(chave), on, id)
        })
        .collect();
    crate::widget::panel_chrome::paint_segmented_group(
        g.interp, &interp, scene, ts, theme, store, hit,
    );

    rotulo(
        scene,
        ts,
        g.rotulo_ease,
        "chrome.timeline_menu.section_ease",
    );
    let eases: Vec<(&str, bool, NodeId)> = EASES
        .iter()
        .map(|&(id, linha, _, chave)| (tr(chave), marcado(linha), id))
        .collect();
    crate::widget::panel_chrome::paint_segmented_group(
        g.ease, &eases, scene, ts, theme, store, hit,
    );

    rotulo(scene, ts, g.rotulo_forma, "chrome.timeline_menu.shape");
    let (forma, _) = forma_e_modo(store, kind);
    let nome = c::TIMELINE_EASE_MENU
        .iter()
        .find(|row| row.0 == forma)
        .map_or("", |row: &(NodeId, TextKey, _)| row.1.tr());
    crate::widget::paint_card_chip(scene, ts, ink, g.forma, nome, quente(c::CTX_MENU_TL_SHAPE));
    hit.register(c::CTX_MENU_TL_SHAPE, g.forma);

    // Rove: a caixa de marcar da casa e o rótulo ao lado; a linha inteira é o alvo.
    let caixa = crate::widget::card_check_rect(g.rove.x, g.rove.y, g.rove.h);
    crate::widget::paint_card_check(
        scene,
        ink,
        caixa,
        marcado(c::CTX_MENU_TL_ROVE),
        quente(c::CTX_MENU_TL_ROVE),
    );
    let px = TypeToken::Sm.px();
    let tx = caixa.x + caixa.w + m::FIELD_GAP;
    let meio = |r: Rect| r.y + (r.h - px) * 0.5;
    let rove = tr("chrome.timeline_menu.rove_check");
    paint_text(
        ts,
        scene,
        rove,
        tx,
        meio(g.rove),
        px,
        g.rove.x + g.rove.w - tx,
        fg,
    );
    hit.register(c::CTX_MENU_TL_ROVE, g.rove);

    // Done: o botão primário do cartão (o acento), no canto de baixo.
    let fundo = ink.color(ColorToken::KanbanBlueBold);
    let fundo = if quente(c::CTX_MENU_TL_KEY_DONE) {
        fundo.with_alpha(0.85)
    } else {
        fundo
    };
    fill_rounded_rect(scene, g.done, m::FIELD_RADIUS, fundo);
    let done = tr("chrome.timeline_menu.done");
    let w = ts.prefix_width(done, px).min(g.done.w);
    let cor = ink.color(ColorToken::AccentFg);
    paint_text(
        ts,
        scene,
        done,
        g.done.x + (g.done.w - w) * 0.5,
        meio(g.done),
        px,
        g.done.w,
        cor,
    );
    hit.register(c::CTX_MENU_TL_KEY_DONE, g.done);
}

#[cfg(test)]
#[path = "key_card_tests.rs"]
mod tests;
