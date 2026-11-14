//! ⭐ **O CARTÃO da chave** — o menu do botão direito numa chave da timeline, no idioma dos cartões
//! do Inspector: título que ARRASTA o cartão, rótulos de secção, **toda escolha num botão com a curva
//! desenhada** (dono 13/11: «todas as escolhas em botões com as curvas de animação desenhadas
//! neles») — Interpolation (5), Ease (3) e as dez formas —, o Rove numa caixa de marcar e **Done**;
//! as escolhas não o fecham.
//!
//! Só PINTA e diz o que cada clique quer ([`acao`]); quem aplica é o `chrome::timeline_segment`, pela
//! mesma `TimelineInterpPick` do menu em lista — a shell não muda. O estado (o que está aceso, o
//! título) e a régua dos eases ([`crate::interaction::EaseSampler`]) vêm do painel da timeline; o
//! arrasto vive em `interaction::dispatch::key_card_drag`.

use crate::ids as c;
use crate::interaction::key_card_alvo::e_cartao;
use crate::interaction::{ContextMenuKind, HitIndex, WidgetStore};
use crate::paint::{fill_circle, fill_rounded_rect, paint_text, stroke_polyline};
use crate::widget::card_ink::CardInk;
use crate::widget::lista_flutuante::Superficie;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_i18n::{TextKey, tr};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A largura do cartão: cinco botões por fileira cabem sem cortar «Straight».
const CARD_W: f32 = 352.0; // LITERAL-PX-OK: fotografado — a 300 «Straight», «Nearest» e «Custom» saíam com «…»
/// A altura de um botão desenhado: a curva (~32 px) e o rótulo por baixo.
const TILE_H: f32 = 58.0; // LITERAL-PX-OK: o bloco 76×58 da maquete do dono (fotos/31)
/// Botões por fileira nas formas (2 fileiras de 5) e na Interpolation.
const TILE_COLS: usize = 5;
/// A espessura da curva num botão e a do contorno do escolhido.
const CURVE_W: f32 = 1.75; // LITERAL-PX-OK: traço da curva desenhada
const SEL_W: f32 = 1.5; // LITERAL-PX-OK: contorno do botão escolhido (o do anel da chave)
/// O raio das bolinhas das pontas e das alças do Custom.
const DOT_R: f32 = 2.0; // LITERAL-PX-OK: ponta da curva desenhada
/// A opacidade das guias 0 e 1 e da curva num botão não escolhido.
const GUIDE_A: f32 = 0.14; // LITERAL-PX-OK: opacidade das guias 0/1 (não é medida de UI)
const CURVE_A: f32 = 0.72; // LITERAL-PX-OK: opacidade da curva apagada (não é medida de UI)
/// A opacidade do véu sobre Ease e Curve shape quando a chave não é curva (Straight, Hold, Nearest).
const VEU_A: f32 = 0.6; // LITERAL-PX-OK: opacidade do véu (não é medida de UI)
/// O Done sob o ponteiro: o acento um pouco recuado.
const DONE_QUENTE_A: f32 = 0.85; // LITERAL-PX-OK: opacidade do hover (não é medida de UI)
/// Amostras por curva desenhada (o Elastic tem três ondas num botão de ~50 px).
const SAMPLES: usize = 48;
/// O Custom desenha uma Bézier em S com as duas alças à vista (P1, P2 no quadrado unitário).
const BEZ_P1: (f32, f32) = (0.62, 0.0); // LITERAL-PX-OK: alça do desenho do Custom (quadrado unitário)
const BEZ_P2: (f32, f32) = (0.38, 1.0); // LITERAL-PX-OK: alça do desenho do Custom (quadrado unitário)
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

/// O que um clique no cartão pede.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Acao {
    /// Nada (o fundo, o título): o cartão fica aberto.
    Nada,
    /// A mesma escolha do menu em lista: `(item, mode)` da `TimelineInterpPick`. O cartão fica.
    Escolhe { item: NodeId, mode: u8 },
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
        // Uma das dez formas: ela, no modo aceso (ou In-Out).
        _ if c::TIMELINE_EASE_MENU.iter().any(|r| r.0 == id) => Some(Acao::Escolhe {
            item: id,
            mode: modo,
        }),
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
    /// Os cinco botões de [`INTERP`], na ordem dela.
    pub interp: Vec<Rect>,
    pub rotulo_ease: Rect,
    /// Os três botões de [`EASES`].
    pub ease: Vec<Rect>,
    pub rotulo_forma: Rect,
    /// Os dez botões de `ids::TIMELINE_EASE_MENU`, cinco por fileira.
    pub formas: Vec<Rect>,
    /// Onde o editor da curva Custom mora quando a chave é Custom: o lugar de Ease e Curve shape.
    pub editor: Rect,
    pub rove: Rect,
    pub done: Rect,
}

/// `n` botões em fileiras de `cols`, de largura igual, a começar em `(x, y)` na largura `w`.
fn botoes(x: f32, y: f32, w: f32, cols: usize, n: usize) -> Vec<Rect> {
    let gap = m::FIELD_GAP;
    let bw = (w - gap * (cols - 1) as f32) / cols as f32;
    (0..n)
        .map(|i| {
            let (col, lin) = ((i % cols) as f32, (i / cols) as f32);
            Rect::new(x + col * (bw + gap), y + lin * (TILE_H + gap), bw, TILE_H)
        })
        .collect()
}

/// A base (o fundo) de uma lista de botões.
fn base(rs: &[Rect]) -> f32 {
    rs.iter().map(|r| r.y + r.h).fold(0.0, f32::max)
}

pub(super) fn geo(x: f32, y: f32) -> Geo {
    let iw = CARD_W - m::CARD_PAD_X * 2.0;
    let x0 = x + m::CARD_PAD_X;
    let cabeca = Rect::new(x, y, CARD_W, m::CARD_PAD_TOP + m::CARD_HEAD_H);
    // Um grupo = o rótulo (LABEL_PX) e os botões por baixo, com as folgas dos cartões.
    let rotulo = |top: f32| Rect::new(x0, top + m::LABEL_TOP, iw, m::LABEL_PX);
    let abaixo = |r: Rect| r.y + r.h + m::LABEL_BOTTOM;
    let rotulo_interp = rotulo(cabeca.y + cabeca.h + m::CARD_HEAD_BOTTOM - m::LABEL_TOP);
    let interp = botoes(x0, abaixo(rotulo_interp), iw, TILE_COLS, INTERP.len());
    let rotulo_ease = rotulo(base(&interp));
    let ease = botoes(x0, abaixo(rotulo_ease), iw, EASES.len(), EASES.len());
    let rotulo_forma = rotulo(base(&ease));
    let formas = botoes(
        x0,
        abaixo(rotulo_forma),
        iw,
        TILE_COLS,
        c::TIMELINE_EASE_MENU.len(),
    );
    // Rove à esquerda e Done à direita, na mesma fileira.
    let fy = base(&formas) + m::CARD_GAP;
    let done = Rect::new(x0 + iw - DONE_W, fy, DONE_W, m::FIELD_H);
    let rove = Rect::new(x0, fy, iw - DONE_W - m::FIELD_GAP, m::FIELD_H);
    let h = done.y + done.h + m::CARD_PAD_BOTTOM - y;
    let editor = Rect::new(x0, rotulo_ease.y, iw, base(&formas) - rotulo_ease.y);
    Geo {
        cartao: Rect::new(x, y, CARD_W, h),
        cabeca,
        rotulo_interp,
        interp,
        rotulo_ease,
        ease,
        rotulo_forma,
        formas,
        editor,
        rove,
        done,
    }
}

/// O que um botão desenha.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Desenho {
    /// Straight: a diagonal.
    Reta,
    /// Hold: o valor fica e salta no fim.
    Segura,
    /// Nearest: salta a meio.
    Proxima,
    /// Custom: uma Bézier em S com as duas alças.
    Bezier,
    /// Uma forma (`item` de `TIMELINE_EASE_MENU`) num modo, pela régua publicada.
    Ease { item: NodeId, mode: u8 },
}

/// Os pontos `(u, y)` do desenho no quadrado unitário (`y` pode passar de 0–1: Back, Elastic).
/// Vazio = sem régua publicada para esta forma (o botão fica só com o rótulo).
pub(super) fn pontos(
    d: Desenho,
    sampler: Option<crate::interaction::EaseSampler>,
) -> Vec<(f32, f32)> {
    let us = (0..=SAMPLES).map(|i| i as f32 / SAMPLES as f32);
    match d {
        Desenho::Reta => vec![(0.0, 0.0), (1.0, 1.0)],
        Desenho::Segura => vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)],
        Desenho::Proxima => vec![(0.0, 0.0), (0.5, 0.0), (0.5, 1.0), (1.0, 1.0)],
        Desenho::Bezier => us
            .map(|t| {
                let s = 1.0 - t;
                let b = |p0: f32, p1: f32, p2: f32, p3: f32| {
                    s * s * s * p0 + 3.0 * s * s * t * p1 + 3.0 * s * t * t * p2 + t * t * t * p3 // LITERAL-PX-OK: coeficientes de Bernstein da cúbica
                };
                (
                    b(0.0, BEZ_P1.0, BEZ_P2.0, 1.0),
                    b(0.0, BEZ_P1.1, BEZ_P2.1, 1.0),
                )
            })
            .collect(),
        Desenho::Ease { item, mode } => {
            let Some(f) = sampler else {
                return Vec::new();
            };
            us.map_while(|u| f(item, mode, u).map(|y| (u, y))).collect()
        }
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
    crate::widget::card_ink::com(ink, || corpo(scene, ts, hit, store, req.kind, &g, ink));
}

/// Um botão desenhado: o fundo do campo de cartão (o escolhido sobe a `KanbanCardRaised` com o
/// contorno no acento), as guias 0 e 1, a curva com as pontas e o rótulo por baixo.
#[allow(clippy::too_many_arguments)]
fn botao(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
    r: Rect,
    rotulo: &str,
    d: Desenho,
    (aceso, quente): (bool, bool),
    sampler: Option<crate::interaction::EaseSampler>,
) {
    let fundo = if aceso {
        ColorToken::KanbanCardRaised
    } else if quente {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    let acento = ink.color(ColorToken::KanbanBlueBold);
    // O escolhido: um anel no acento por preenchimento (o fundo por cima, recuado SEL_W).
    let dentro = if aceso {
        fill_rounded_rect(scene, r, m::FIELD_RADIUS, acento);
        Rect::new(
            r.x + SEL_W,
            r.y + SEL_W,
            r.w - SEL_W * 2.0,
            r.h - SEL_W * 2.0,
        )
    } else {
        r
    };
    fill_rounded_rect(scene, dentro, m::FIELD_RADIUS - SEL_W, ink.color(fundo));
    let fg = ink.color(ColorToken::KanbanCardFg);
    let px = TypeToken::Xs.px();
    let pad = Spacing::Sm.px();
    let area = Rect::new(
        r.x + pad,
        r.y + pad,
        r.w - pad * 2.0,
        r.h - pad * 2.0 - px - Spacing::Xs.px(),
    );
    let pts = pontos(d, sampler);
    // O 0 e o 1 ocupam a área; o que passa (Back, Elastic) encolhe-os para caber.
    let (lo, hi) = pts
        .iter()
        .fold((0.0_f32, 1.0_f32), |(lo, hi), p| (lo.min(p.1), hi.max(p.1)));
    let ny = |y: f32| area.y + area.h - (y - lo) / (hi - lo) * area.h;
    let nx = |u: f32| area.x + u * area.w;
    let guia = fg.with_alpha(GUIDE_A);
    for v in [0.0, 1.0] {
        stroke_polyline(
            scene,
            &[(area.x, ny(v)), (area.x + area.w, ny(v))],
            1.0,
            guia,
        );
    }
    let tinta = if aceso {
        acento
    } else {
        fg.with_alpha(CURVE_A)
    };
    if d == Desenho::Bezier {
        let (p0, p3) = ((nx(0.0), ny(0.0)), (nx(1.0), ny(1.0)));
        let (p1, p2) = ((nx(BEZ_P1.0), ny(BEZ_P1.1)), (nx(BEZ_P2.0), ny(BEZ_P2.1)));
        stroke_polyline(scene, &[p0, p1], 1.0, tinta);
        stroke_polyline(scene, &[p3, p2], 1.0, tinta);
        fill_circle(scene, p1.0, p1.1, DOT_R, tinta);
        fill_circle(scene, p2.0, p2.1, DOT_R, tinta);
    }
    let tela: Vec<(f32, f32)> = pts.iter().map(|&(u, y)| (nx(u), ny(y))).collect();
    stroke_polyline(scene, &tela, CURVE_W, tinta);
    if let (Some(a), Some(b)) = (tela.first(), tela.last()) {
        fill_circle(scene, a.0, a.1, DOT_R, tinta);
        fill_circle(scene, b.0, b.1, DOT_R, tinta);
    }
    let cor = if aceso {
        fg
    } else {
        ink.color(ColorToken::CardLabel)
    };
    let w = ts.prefix_width(rotulo, px).min(r.w - pad);
    crate::paint::paint_text_elided(
        ts,
        scene,
        rotulo,
        r.x + (r.w - w) * 0.5,
        r.y + r.h - pad * 0.5 - px,
        px,
        r.w - pad,
        cor,
    );
}

fn corpo(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
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
    let sampler = store.ease_sampler();

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
    let (forma, modo) = forma_e_modo(store, kind);
    let curva = EASES.iter().any(|e| marcado(e.1));

    rotulo(
        scene,
        ts,
        g.rotulo_interp,
        "chrome.timeline_menu.section_interp",
    );
    for (&(id, chave), &r) in INTERP.iter().zip(&g.interp) {
        let (d, on) = match id {
            _ if id == c::CTX_MENU_TL_CURVE => (
                Desenho::Ease {
                    item: forma,
                    mode: modo,
                },
                curva,
            ),
            _ if id == c::CTX_MENU_TL_LINEAR => (Desenho::Reta, marcado(id)),
            _ if id == c::CTX_MENU_TL_HOLD => (Desenho::Segura, marcado(id)),
            _ if id == c::CTX_MENU_TL_NEAREST => (Desenho::Proxima, marcado(id)),
            _ => (Desenho::Bezier, marcado(id)),
        };
        botao(scene, ts, ink, r, tr(chave), d, (on, quente(id)), sampler);
        hit.register(id, r);
    }

    // Custom: o editor da curva abre no lugar de Ease e Curve shape (dono 13/11).
    store.set_key_card_bez_area(None);
    if marcado(c::CTX_MENU_TL_CUSTOM) {
        editor::paint(scene, ts, hit, store, g.editor, ink);
    } else {
        // Straight, Hold e Nearest não têm ease nem forma: as duas secções ficam apagadas (um
        // clique nelas continua a escolher a curva).
        let apagado = [
            c::CTX_MENU_TL_LINEAR,
            c::CTX_MENU_TL_HOLD,
            c::CTX_MENU_TL_NEAREST,
        ]
        .into_iter()
        .any(marcado);
        let veu = |scene: &mut VectorScene, r: Rect| {
            if apagado {
                let cor = ink.color(ColorToken::KanbanCard).with_alpha(VEU_A);
                fill_rounded_rect(scene, r, m::FIELD_RADIUS, cor);
            }
        };
        rotulo(
            scene,
            ts,
            g.rotulo_ease,
            "chrome.timeline_menu.section_ease",
        );
        for (&(id, linha, mode, chave), &r) in EASES.iter().zip(&g.ease) {
            let d = Desenho::Ease { item: forma, mode };
            botao(
                scene,
                ts,
                ink,
                r,
                tr(chave),
                d,
                (marcado(linha), quente(id)),
                sampler,
            );
            veu(scene, r);
            hit.register(id, r);
        }

        rotulo(scene, ts, g.rotulo_forma, "chrome.timeline_menu.shape");
        for (row, &r) in c::TIMELINE_EASE_MENU.iter().zip(&g.formas) {
            let row: &(NodeId, TextKey, _) = row;
            let id = row.0;
            let d = Desenho::Ease {
                item: id,
                mode: modo,
            };
            // A forma acende só quando a chave está em curva (Straight não tem forma).
            let on = curva && marcado(id);
            botao(scene, ts, ink, r, row.1.tr(), d, (on, quente(id)), sampler);
            veu(scene, r);
            hit.register(id, r);
        }
    }

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
    let meio = |r: Rect| r.y + crate::paint::folga_do_texto(r.h, px);
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
        fundo.with_alpha(DONE_QUENTE_A)
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

#[path = "key_card_editor.rs"]
pub(crate) mod editor;

#[cfg(test)]
#[path = "key_card_tests.rs"]
mod tests;
