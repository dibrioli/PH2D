//! **B · Keyboard** (plano `docs/InputMap/01_plano.md` §3, maquete B, no molde da aba do Animated Sprite): à
//! esquerda as acções por quem as lê, em cima o teclado (ou o comando) com uma barra por acção em cada tecla, em
//! baixo a acção aberta com as ligações em ladrilhos, a escuta no lugar e — com manche — a barra do curso.

pub mod geometry;
pub use geometry::half_icon;
pub mod hscroll;
pub(crate) mod input;
pub(crate) mod keys;
pub mod keys_fit;
mod list;
mod pad;
pub mod state;
mod tiles;
pub(crate) mod travel;

pub(crate) use input::{key, motion, press, raw_key};
pub use input::{tip_here, tip_of};

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_input::readers::Reader;
use ph2d_input::{ActionId, Binding};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

use crate::doc::InputMapDoc;

/// Os três grupos da lista, por quem lê a acção.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    PlatformPlayer,
    TopDownPlayer,
    Yours,
}

/// O grupo de uma acção: lida pelos dois movers vai para o Platform Player (como a maquete).
#[must_use]
pub fn group_of(doc: &InputMapDoc, name: &str) -> Group {
    let rs = crate::view_list::footer::readers_of(&doc.snap, name);
    if rs.contains(&Reader::PlatformPlayer) {
        Group::PlatformPlayer
    } else if rs.contains(&Reader::TopDownPlayer) {
        Group::TopDownPlayer
    } else {
        Group::Yours
    }
}

/// A acção aberta: a escolhida se ainda existe, senão a primeira.
#[must_use]
pub fn open(doc: &InputMapDoc) -> Option<ActionId> {
    let map = &doc.snap.map;
    doc.kb
        .open
        .filter(|a| map.get(*a).is_some())
        .or_else(|| map.actions().first().map(|a| a.id))
}

/// A ligação `b` é a que o alvo desenhado `part` mostra? O gatilho é o mesmo como botão e como eixo.
pub(crate) fn same_input(part: Binding, b: Binding) -> bool {
    use ph2d_input::{GamepadAxis as A, GamepadButton as P};
    part == b
        || matches!(
            (part, b),
            (
                Binding::PadButton(P::LeftTrigger),
                Binding::PadAxis {
                    axis: A::LeftTrigger,
                    ..
                }
            ) | (
                Binding::PadButton(P::RightTrigger),
                Binding::PadAxis {
                    axis: A::RightTrigger,
                    ..
                }
            )
        )
}

/// As acções que usam `b`, pela ordem do mapa.
pub(crate) fn users(doc: &InputMapDoc, b: Binding) -> Vec<ActionId> {
    doc.snap
        .map
        .actions()
        .iter()
        .filter(|a| a.bindings.iter().any(|x| same_input(b, *x)))
        .map(|a| a.id)
        .collect()
}

/// Pinta a vista dentro do cartão `body` (o cartão já está pintado).
pub fn paint(
    doc: &mut InputMapDoc,
    body: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let ink = CardInk::of_list(theme);
    // A B como a A: o fundo é a superfície MAIS FUNDA (kanban-list), os cartões sobem (kanban-card) e as teclas
    // sobem mais (kanban-card-raised). Da cor do quadro, o fundo não deixa faixa vazia por baixo do conteúdo.
    fill_rounded_rect(
        scene,
        body,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanList),
    );
    doc.kb.targets.clear();
    doc.kb.bars.clear();
    doc.kb.live_dot = None;
    doc.kb.bar = None;
    if let Some(n) = doc.kb.select_name.clone()
        && let Some(id) = doc.snap.map.id(&n)
    {
        doc.kb.open = Some(id);
        doc.kb.reveal = Some(id);
        doc.kb.select_name = None;
    }
    doc.kb.open = open(doc);
    let inner = Rect::new(
        body.x + m::CARD_PAD_X,
        body.y + m::CARD_PAD_TOP,
        (body.w - m::CARD_PAD_X * 2.0).max(0.0),
        (body.h - m::CARD_PAD_TOP - m::CARD_PAD_BOTTOM).max(0.0),
    );
    let list_w = list::width(doc, ts).min(inner.w * 0.5);
    let side_w = (inner.w - list_w - m::CARD_GAP).max(0.0);
    // ⭐ Plano §13.2 (a 1280): a coluna do teclado não segura o passo mínimo ⟹ empilha — o teclado na largura
    // inteira por cima, a lista e a acção aberta lado a lado por baixo. A mesma regra no comando.
    let stacked = !wide_fits(ts, side_w);
    doc.kb.stacked = stacked;
    let (top, below) = if stacked {
        let top = Rect::new(
            inner.x,
            inner.y,
            inner.w,
            keys::top_height(doc, ts, inner.w),
        );
        (top, top.y + top.h + m::CARD_GAP)
    } else {
        let x = inner.x + list_w + m::CARD_GAP;
        (
            Rect::new(x, inner.y, side_w, keys::top_height(doc, ts, side_w)),
            inner.y,
        )
    };
    fill_rounded_rect(
        scene,
        top,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanCard),
    );
    keys::paint_top(doc, top, scene, ts, ink);
    // A região que rola: a lista (lado a lado) ou, empilhada, a lista e a acção aberta como UMA região por baixo do
    // desenho — o teclado nunca rola.
    // Empilhada, o fundo da região guarda o mesmo recuo do topo (`CARD_PAD_TOP`): o último cartão acaba onde o recorte
    // acaba, sem faixa de outro cartão por baixo.
    let region = if stacked {
        let floor = body.y + body.h - m::CARD_PAD_TOP;
        Rect::new(inner.x, below, inner.w, (floor - below).max(0.0))
    } else {
        Rect::new(inner.x, below, list_w, (inner.y + inner.h - below).max(0.0))
    };
    // A calha da barra é o recuo direito da LISTA (nenhum texto lá), empilhada ou lado a lado: só a lista rola.
    let gutter_w = m::LIST_PAD_X;
    let gutter = Rect::new(inner.x + list_w - gutter_w, region.y, gutter_w, region.h);
    // A acção aberta que a pintura anterior não mostrou (clique, cena, porta de smoke) rola para a vista, pela conta
    // da lista (já na 1.ª pintura), e a rolagem em repouso pára numa fronteira de linha.
    let opened = doc.kb.open;
    if opened != doc.kb.painted_open {
        doc.kb.reveal = doc.kb.reveal.or(opened);
        doc.kb.painted_open = opened;
    }
    let reveal = doc.kb.reveal.take();
    if let Some((y0, h)) = reveal.and_then(|a| list::row_top(doc, a)) {
        let lo = (y0 + h - region.h).max(0.0);
        let s = doc.kb.scroll.clamp(lo, y0.max(lo));
        let step = list::ROW_STEP;
        doc.kb.scroll = if s > lo + f32::EPSILON {
            (s / step).floor() * step
        } else {
            (s / step).ceil() * step
        };
        doc.kb.scroll = doc.kb.scroll.min(y0);
    }
    if doc.kb.list_total > 0.0 {
        doc.kb.scroll = doc
            .kb
            .scroll
            .clamp(0.0, (doc.kb.list_total - region.h).max(0.0));
    }
    doc.kb.rows.clear();
    let y0 = region.y - doc.kb.scroll;
    let tall = f32::MAX * 0.25; // LITERAL-PX-OK: a região rolada não tem fundo — o recorte corta
    doc.hits.push_clip(region);
    let mark = doc.ui.controls.len();
    let mut over = VectorScene::new();
    over.push_clip(&ph2d_editor_core::paint::rect_to_vello(region));
    let list_r = Rect::new(inner.x, y0, list_w, if stacked { tall } else { region.h });
    let list_end = fitted(&mut over, ink, list_r, region, |s| {
        list::paint(doc, list_r, s, ts, ink)
    });
    let rx = list_r.x + list_r.w + m::CARD_GAP;
    let rw = (inner.x + inner.w - rx).max(0.0);
    let (mut end, mut card_end) = (list_end, region.y);
    if stacked {
        // O cartão da aberta fica no topo da região e só rola o que passa do fundo: revelar uma linha do fundo da
        // lista não o tira da vista, e rolar alcança o fim dele (o curso e os números do comando a 1280).
        let shift = doc.kb.scroll.min((doc.kb.card_h - region.h).max(0.0));
        let bottom = Rect::new(rx, region.y - shift, rw, tall);
        card_end = fitted(&mut over, ink, bottom, region, |s| {
            tiles::paint(doc, bottom, s, ts, ink)
        });
        doc.kb.card_h = card_end - bottom.y;
        end = end.max(y0 + doc.kb.card_h);
    }
    over.pop_layer();
    doc.hits.pop_clip();
    doc.kb.scrolled = doc.ui.controls[mark..].iter().map(|c| c.id).collect();
    scene.append_scene(&over);
    let total = end - y0;
    doc.kb.list_total = total;
    doc.kb.list_view_h = region.h;
    doc.kb.scroll = doc.kb.scroll.clamp(0.0, (total - region.h).max(0.0));
    if total > region.h + f32::EPSILON {
        doc.census.clip = Some(region);
        // O desvanecer só sobre o cartão que o recorte corta: a lista (rola) e, empilhada, o da aberta se passa do fundo.
        let floor = region.y + region.h;
        let list_col = Rect::new(inner.x, region.y, list_w, region.h);
        let list_cut = (
            doc.kb.scroll > f32::EPSILON,
            list_end > floor + f32::EPSILON,
        );
        fade(doc, scene, ink, &[list_col], list_cut);
        if stacked && card_end > floor + f32::EPSILON {
            let card_col = Rect::new(rx, region.y, rw, region.h);
            fade(doc, scene, ink, &[card_col], (false, true));
        }
        list::scrollbar(doc, (region, gutter), total, scene, ink);
    }
    if !stacked {
        let by = top.y + top.h + m::CARD_GAP;
        let bottom = Rect::new(rx, by, rw, (inner.y + inner.h - by).max(0.0));
        fitted(scene, ink, bottom, bottom, |s| {
            tiles::paint(doc, bottom, s, ts, ink)
        });
    }
}

/// O desvanecer das bordas cortadas (`top`, `bottom`) da região que rola: bandas da cor do cartão, cada vez mais
/// leves para dentro, sobre cada coluna de cartão — o conteúdo diz que continua.
fn fade(
    doc: &mut InputMapDoc,
    scene: &mut VectorScene,
    ink: CardInk,
    cols: &[Rect],
    (top, bottom): (bool, bool),
) {
    const BANDS: usize = 4;
    let band = m::FIELD_H / BANDS as f32;
    let card = ink.color(ColorToken::KanbanCard);
    for c in cols {
        for (edge, on) in [(true, top), (false, bottom)] {
            if !on {
                continue;
            }
            for i in 0..BANDS {
                let alpha = 1.0 - (i as f32 + 0.5) / BANDS as f32; // LITERAL-PX-OK: alfa da banda, não px
                let y = if edge {
                    c.y + band * i as f32
                } else {
                    c.y + c.h - band * (i as f32 + 1.0)
                };
                let r = Rect::new(c.x, y, c.w, band);
                fill_rounded_rect(scene, r, 0.0, card.multiply_alpha(alpha)); // LITERAL-PX-OK: banda recta
            }
            let h = band * BANDS as f32;
            let y = if edge { c.y } else { c.y + c.h - h };
            doc.census.fades.push(Rect::new(c.x, y, c.w, h));
        }
    }
}

/// A coluna de largura `w` segura o teclado sem rolar (o passo mínimo, com os dois recuos do cartão e do desenho
/// que o `top_height` desconta).
fn wide_fits(ts: &mut TextSystem, w: f32) -> bool {
    let f = keys_fit::fit(
        ts,
        w - m::CARD_PAD_X * 4.0,
        None,
        geometry::PITCH_MAX_ROWS_B,
    );
    !f.scrolls && f.legible
}

/// A roda rola a lista das acções (só quando ela não cabe); com o Shift, o teclado na horizontal. `true` = rolou.
pub(crate) fn wheel(doc: &mut InputMapDoc, dy: f32, shift: bool) -> bool {
    if shift {
        return hscroll::wheel(doc, dy);
    }
    let before = doc.kb.scroll;
    let max = (doc.kb.list_total - doc.kb.list_view_h).max(0.0);
    doc.kb.scroll = (doc.kb.scroll - dy).clamp(0.0, max);
    (doc.kb.scroll - before).abs() > f32::EPSILON
}

/// Um cartão que cabe no conteúdo: `f` pinta numa cena à parte e devolve onde acabou; o fundo entra por baixo,
/// até ao fundo de `view` (o recorte). Devolve o fim do conteúdo.
fn fitted(
    scene: &mut VectorScene,
    ink: CardInk,
    r: Rect,
    view: Rect,
    f: impl FnOnce(&mut VectorScene) -> f32,
) -> f32 {
    let mut over = VectorScene::new();
    let end = f(&mut over);
    let card = Rect::new(
        r.x,
        r.y,
        r.w,
        (end - r.y).clamp(0.0, (view.y + view.h - r.y).max(0.0)),
    );
    fill_rounded_rect(
        scene,
        card,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanCard),
    );
    scene.append_scene(&over);
    end
}
