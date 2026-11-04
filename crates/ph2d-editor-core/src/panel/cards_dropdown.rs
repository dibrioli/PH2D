//! ⭐ **A lista suspensa de um cartão** — qual está aberta (um clique na caixa abre/fecha; qualquer
//! outro clique fecha) e as opções do quadro, pintadas por CIMA de tudo no fim da lista. Cada opção
//! leva o id do painel dono: o clique nela é o `Click` que o `apply_event` dele já trata.
//!
//! A lista nunca passa do canvas visível: o que não cabe rola (roda e polegar), com a rolagem e as
//! alturas nas tabelas do store sob o id da caixa, como a lista das secções.

use super::*;
use crate::widget::PaintedPopover;
use std::cell::RefCell;

type Opcoes = (Rect, CardInk, Vec<(NodeId, String, bool)>);

thread_local! {
    static ABERTA: RefCell<Option<NodeId>> = const { RefCell::new(None) };
    static PENDENTE: RefCell<Option<Opcoes>> = const { RefCell::new(None) };
    static CAIXAS: RefCell<Vec<NodeId>> = const { RefCell::new(Vec::new()) };
    /// As opções pintadas no último quadro: a pressão nelas não é «fora».
    static OPCOES: RefCell<Vec<NodeId>> = const { RefCell::new(Vec::new()) };
}

/// A pressão fora da caixa e da lista fecha (a caixa decide no clique dela). Dentro da lista, a barra
/// e a margem não fecham.
pub(super) fn ao_premir(store: &mut WidgetStore, hit_index: &HitIndex, x: f32, y: f32) {
    let dela = store.card_dropdown_at(x, y)
        || hit_index.hit_with_rect(x, y).is_some_and(|(id, _)| {
            CAIXAS.with(|c| c.borrow().contains(&id)) || OPCOES.with(|o| o.borrow().contains(&id))
        });
    if !dela {
        ABERTA.with(|a| *a.borrow_mut() = None);
        store.close_card_dropdown();
    }
}

#[must_use]
pub fn open() -> Option<NodeId> {
    ABERTA.with(|a| *a.borrow())
}

pub fn stash(r: Rect, ink: CardInk, options: Vec<(NodeId, String, bool)>) {
    PENDENTE.with(|p| *p.borrow_mut() = Some((r, ink, options)));
}

/// Declara `id` como caixa de lista suspensa (o painel, ao montar os cartões).
pub fn declare(id: NodeId) {
    CAIXAS.with(|c| {
        let mut c = c.borrow_mut();
        if !c.contains(&id) {
            c.push(id);
        }
    });
}

/// O clique: na caixa abre/fecha (consome); noutro sítio fecha e deixa o clique seguir.
pub fn on_click(id: NodeId) -> bool {
    if CAIXAS.with(|c| c.borrow().contains(&id)) {
        ABERTA.with(|a| {
            let mut a = a.borrow_mut();
            *a = if *a == Some(id) { None } else { Some(id) };
        });
        return true;
    }
    ABERTA.with(|a| *a.borrow_mut() = None);
    false
}

/// A rolagem que põe a opção escolhida a meio da lista quando ela abre.
fn escolhida_a_vista(opts: &[(NodeId, String, bool)], visivel: f32, max: f32) -> f32 {
    opts.iter()
        .position(|(_, _, on)| *on)
        .map_or(0.0, |i| {
            m::FIELD_H * i as f32 - (visivel - m::FIELD_H) * 0.5
        })
        .max(0.0)
        .min(max)
}

/// Pinta as opções da lista aberta por baixo da caixa, por cima de tudo.
pub fn paint_pending(ctx: &mut PaintCtx) {
    let (Some((r, ink, opts)), Some(dono)) = (PENDENTE.with(|p| p.borrow_mut().take()), open())
    else {
        // Cada painel de cartões passa por aqui: só quem já não tem lista aberta a dá por fechada.
        if open().is_none() {
            ctx.host.store_mut().close_card_dropdown();
        }
        return;
    };
    OPCOES.with(|o| *o.borrow_mut() = opts.iter().map(|(id, ..)| *id).collect());
    let conteudo = m::FIELD_H * opts.len() as f32 + m::SEG_PAD * 2.0;
    let visivel = conteudo.min(altura_das_janelinhas());
    // A lista tem a largura do nome mais longo (foto `r2_lista_longa_antes_nomes_cortados`), não a da
    // caixa, e fica dentro da janela.
    let texto = opts
        .iter()
        .map(|(_, t, _)| {
            ctx.text_system
                .prefix_width_weighted(t, m::VALUE_PX, ph2d_text::FontWeight::MEDIUM)
        })
        .fold(0.0_f32, f32::max);
    let medida = Rect::new(0.0, 0.0, r.w, visivel);
    let barra = medida.w - crate::widget::scrollbar_track_rect(medida).x;
    let janela = ctx.viewport;
    let largura =
        r.w.max((texto + m::FIELD_PAD_R * 2.0 + m::SEG_PAD * 2.0 + barra).ceil())
            .min(janela.w);
    let caixa = Rect::new(
        r.x.min(janela.x + janela.w - largura).max(janela.x),
        sobe_para_caber(r.y + r.h + m::SEG_PAD, visivel),
        largura,
        visivel,
    );
    let max = (conteudo - visivel).max(0.0);
    let store = ctx.host.store_mut();
    if store.card_dropdown() != Some(dono) {
        store.set_panel_scroll(dono, escolhida_a_vista(&opts, visivel, max));
    }
    let rolagem = store.panel_scroll(dono).max(0.0).min(max);
    let pintada = PaintedPopover {
        id: dono,
        panel: caixa,
        content_h: conteudo,
        visible_h: visivel,
        scroll: rolagem,
    };
    store.publish_card_dropdown(pintada);
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let (scene, text_system) = (&mut *ctx.scene, &mut *ctx.text_system);
    shadow(
        scene,
        caixa,
        m::FIELD_RADIUS,
        ink.color(ColorToken::KanbanListShadow),
        m::LIST_SHADOW_FAR,
    );
    fill_rounded_rect(
        scene,
        caixa,
        m::FIELD_RADIUS,
        ink.color(ColorToken::KanbanCardRaised),
    );
    let fim_da_linha = if max > 0.0 {
        crate::widget::scrollbar_track_rect(caixa).x
    } else {
        caixa.x + caixa.w - m::SEG_PAD
    };
    scene.push_clip(&crate::paint::rect_to_vello(caixa));
    hit_index.push_clip(caixa);
    for (i, (id, text, on)) in opts.iter().enumerate() {
        let lr = Rect::new(
            caixa.x + m::SEG_PAD,
            caixa.y + m::SEG_PAD + m::FIELD_H * i as f32 - rolagem,
            fim_da_linha - caixa.x - m::SEG_PAD,
            m::FIELD_H,
        );
        if lr.y + lr.h <= caixa.y || lr.y >= caixa.y + caixa.h {
            continue;
        }
        hit_index.register(*id, lr);
        if *on || store.hover_live(*id) > 0.0 {
            fill_rounded_rect(
                scene,
                lr,
                m::SEG_RADIUS,
                ink.color(ColorToken::CardFieldHover),
            );
        }
        crate::paint::paint_text_elided(
            text_system,
            scene,
            text,
            lr.x + m::FIELD_PAD_R,
            lr.y + (lr.h - m::VALUE_PX) * 0.5,
            m::VALUE_PX,
            lr.w - m::FIELD_PAD_R * 2.0,
            ink.color(ColorToken::Text1),
        );
    }
    hit_index.pop_clip();
    scene.pop_layer();
    crate::panel::paint_popover_scrollbar(&pintada, store, scene, ink.theme, hit_index);
}
