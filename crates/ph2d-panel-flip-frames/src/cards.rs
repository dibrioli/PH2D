//! ⭐ **A tira de Frames em cartão: só o cromo** (`docs/CartoesUI/11_RECEITA`, «Canvas de teste»). A barra
//! e as células continuam nos pintores deles, com os mesmos ids: a tira mede a LARGURA da banda e não
//! cabe na grelha de duas colunas de um cartão. Um cartão só, sem cabeçalho (medido a 1930 × 1040: a
//! altura da célula é `98` no corpo de antes, `48` com cabeçalho, `78` sem), e o corpo pinta-se depois
//! da lista, com o `PaintCtx` que aqueles pintores pedem.

use crate::state::{FlipStripSnapshot, FlipStripState};
use crate::{FlipFramesPanel, ids};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::panel::cards::{CardDef, paint_card_list};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr_with;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use std::cell::Cell;

/// O que a lista gasta por cima e por baixo do corpo do cartão único.
const CROMO_H: f32 = m::LIST_MARGIN
    + m::LIST_HEAD_H
    + m::CARD_PAD_TOP
    + m::CARD_PAD_BOTTOM
    + m::LIST_PAD_BOTTOM
    + m::LIST_MARGIN;

fn titulo(snap: &FlipStripSnapshot) -> String {
    if snap.has_layer && !snap.layer_name.is_empty() {
        tr_with(
            "panel.flip_frames.title_with_layer",
            &[("layer", &snap.layer_name)],
        )
    } else {
        FlipFramesPanel::TITLE.tr().to_owned()
    }
}

pub(crate) fn paint(state: &mut FlipStripState, ctx: &mut PaintCtx, snap: &FlipStripSnapshot) {
    let panel = ph2d_editor_core::ids::FLIP_STRIP_PANEL;
    ctx.host
        .store_mut()
        .mark_card_list_close(panel, ids::FLIP_STRIP_CLOSE);
    // A faixa não encolhe por causa do cromo: o que falta sai da tira, e a costura do topo da banda
    // devolve-o ao artista.
    let corpo_h = (ctx.slot.h - CROMO_H).max(0.0);

    // O 1.º passe do corpo é o da lista.
    let corpo: Cell<Option<(Rect, Theme)>> = Cell::new(None);
    let nome = titulo(snap);
    {
        let corpo = &corpo;
        let cartao = CardDef {
            section: ids::FLIP_STRIP_SEC,
            // Sem cabeçalho: dobrar ou reordenar um cartão único não diz nada, e os 26 px são da tira.
            title: "",
            actions: Vec::new(),
            add: None,
            summary: None,
            body: Some(Box::new(move |c, y| {
                if corpo.get().is_none() {
                    corpo.set(Some((Rect::new(c.x, y, c.w, corpo_h), c.ink.theme)));
                }
                y + corpo_h
            })),
        };
        paint_card_list(ctx, panel, &nome, &[cartao]);
    }
    let Some((r, tema)) = corpo.get() else {
        return; // dobrado
    };
    let barra = Rect::new(r.x, r.y, r.w, ROW_H_PX);
    let filas = crate::toolbar_plan::rows(barra, ROW_H_PX, snap, ctx.text_system);
    let barra_h = crate::toolbar_plan::bar_height(filas, ROW_H_PX) + Spacing::Sm.px();
    let aberto = crate::paint_toolbar::paint(ctx, tema, barra, snap);
    let celulas_y = r.y + barra_h;
    let celulas = Rect::new(r.x, celulas_y, r.w, (r.h - barra_h).max(0.0));
    crate::paint_cells::paint(state, ctx, tema, celulas, snap);
    if let Some(chip) = aberto {
        let cur = if chip.id == ids::FLIP_TWEEN_EASE_DD {
            snap.tween_ease
        } else {
            snap.cycle
        };
        crate::paint_toolbar::paint_cycle_popover(ctx, tema, chip, cur);
    }
}
