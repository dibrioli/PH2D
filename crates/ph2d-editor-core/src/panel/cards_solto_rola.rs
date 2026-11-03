//! ⭐ T57 — o cartão SOLTO mais alto que o canvas rola por dentro (smoke do dono 02/11, Platform
//! Player). O cabeçalho fica (é a pega); o corpo passa pela porta da rolagem
//! ([`crate::widget::scroll_area`]) com o rect publicado do solto como dono — a roda sobre ele já
//! lhe chega por `panel_at` — e a barra tem faixa própria: os campos estreitam-se quando ela
//! aparece, como na lista ([`super::super::rolagem::largura_dos_cartoes`]).

use super::super::*;
use crate::widget::scroll_area::{self, Pending, ScrollArea};
use std::cell::RefCell;

thread_local! {
    /// As alturas a publicar (o pintor só tem o store imutável): [`publica`] esvazia-as.
    static PENDENTES: RefCell<Vec<Pending>> = const { RefCell::new(Vec::new()) };
}

/// O corpo do solto `secao` rola entre o topo dele e `fundo` (o fundo do canvas).
#[derive(Clone, Copy)]
pub(in crate::panel) struct Rola {
    pub(in crate::panel) dono: NodeId,
    pub(in crate::panel) fundo: f32,
    /// T63: a chave do tema, do contorno e das notas da cópia.
    pub(in crate::panel) face: NodeId,
}

/// O id da barra de rolagem do solto `section`.
fn barra_of(section: NodeId) -> NodeId {
    const SAL: u64 = 0x5B19_E7A2_C640_8D3F;
    NodeId(section.0 ^ SAL)
}

impl Rola {
    pub(in crate::panel) fn de(section: NodeId, canvas: Rect) -> Self {
        Self {
            dono: crate::interaction::loose_rect_of(section),
            fundo: canvas.y + canvas.h - m::LIST_PAD_BOTTOM,
            face: crate::interaction::loose_face_of(section),
        }
    }
}

/// O canvas do solto pára acima do HUD de baixo (a faixa de estado pinta-se por cima do canvas).
pub(in crate::panel) fn acima_do_hud(canvas: Rect, hud: Rect) -> Rect {
    if hud.h <= 0.0 || hud.y >= canvas.y + canvas.h {
        return canvas;
    }
    Rect {
        h: (hud.y - canvas.y).max(0.0),
        ..canvas
    }
}

/// Pinta o corpo de `card` (campos e notas) a partir de `topo`, no cartão `r`, e devolve o `y`
/// onde ele acaba. Com `rola`, o corpo pára no fundo do canvas e rola por dentro.
#[allow(clippy::too_many_arguments)]
pub(in crate::panel) fn corpo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    panel: NodeId,
    theme: Theme,
    ink: CardInk,
    r: Rect,
    card: &CardDef<'_>,
    topo: f32,
    rola: Option<Rola>,
) -> f32 {
    let ix = r.x + m::CARD_PAD_X;
    let mut pinta = |scene: &mut VectorScene, hit_index: &mut HitIndex, w: f32, y: f32| {
        let Some(body) = card.body.as_ref() else {
            return y;
        };
        let fim = {
            let mut c = CardCtx {
                scene: &mut *scene,
                text_system: &mut *text_system,
                store,
                hit_index: &mut *hit_index,
                ink,
                x: ix,
                w,
                first: true,
            };
            corpo::com(ink, || body(&mut c, y))
        };
        crate::widget::showcase::notes_chrome::notas_do_cartao(
            scene,
            text_system,
            hit_index,
            store,
            panel,
            rola.map_or(card.section, |k| k.face),
            ix,
            w,
            fim,
        )
    };
    let iw = r.w - m::CARD_PAD_X * 2.0;
    let Some(k) = rola else {
        return pinta(scene, hit_index, iw, topo);
    };
    let max = (k.fundo - m::CARD_PAD_BOTTOM - topo).max(0.0);
    let faixa = Rect::new(r.x, topo, r.w, max);
    let transborda = matches!(
        (store.panel_content_h(k.dono), store.panel_visible_h(k.dono)),
        (Some(c), Some(v)) if crate::widget::scrollbar_is_needed(c, v)
    );
    let w = if transborda {
        crate::widget::scrollbar_track_rect(faixa).x - m::LIST_BAR_GAP - ix
    } else {
        iw
    };
    let barra = barra_of(card.section);
    let area = scroll_area::open_with(scene, hit_index, store, k.dono, barra, faixa);
    let fim = pinta(scene, hit_index, w, area.top());
    let conteudo = fim - area.top();
    let visivel = conteudo.min(max);
    let area = ScrollArea::opened(
        k.dono,
        barra,
        Rect {
            h: visivel,
            ..faixa
        },
        area.scroll(),
    );
    let pendente = scroll_area::close_parts(
        area,
        scene,
        hit_index,
        store,
        super::super::rolagem::conteudo(conteudo, conteudo <= max),
        theme,
    );
    PENDENTES.with(|p| p.borrow_mut().push(pendente));
    topo + visivel
}

/// Publica as alturas e a barra de cada solto pintado neste quadro.
pub(in crate::panel) fn publica(store: &mut WidgetStore) {
    for p in PENDENTES.with(|p| std::mem::take(&mut *p.borrow_mut())) {
        p.publish(store);
    }
}
