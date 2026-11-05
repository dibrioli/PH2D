//! ⭐ **A tira de Frames em cartões**, o método da Timeline (`docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`):
//! a moldura da lista sem cartão por dentro, porque a altura é da tira; a barra em [`crate::barra_cartoes`];
//! as células nos pintores delas, com a tinta da lista.

use crate::state::{FlipStripSnapshot, FlipStripState};
use crate::{FlipFramesPanel, ids};
use ph2d_editor_core::panel::cards::{banda, censo};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::card_ink::{self, CardInk};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr_with;

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
    let tema = ctx.host.theme();
    let slot = ctx.slot;
    let store = ctx.host.store_mut();
    store.mark_card_list_close(panel, ids::FLIP_STRIP_CLOSE);
    crate::barra_cartoes::prepara(store, snap);
    let moldura = banda::pinta_moldura(ctx, panel, &titulo(snap), slot);
    // A faixa pinta-se fora de `paint_card_list`: o censo só a vê sob um id (que apaga o que já leu dele).
    let aberta = censo::na_secao_de(ids::FLIP_STRIP_SEC, || {
        let (y, aberta) = crate::barra_cartoes::paint(ctx, tema, moldura.cab, moldura.corpo, snap);
        let fundo = moldura.corpo.y + moldura.corpo.h;
        let celulas = Rect::new(moldura.corpo.x, y, moldura.corpo.w, (fundo - y).max(0.0));
        card_ink::com(CardInk::of_list(tema), || {
            crate::paint_cells::paint(state, ctx, tema, celulas, snap);
        });
        aberta
    });
    if let Some(chip) = aberta {
        let cur = if chip.id == ids::FLIP_TWEEN_EASE_DD {
            snap.tween_ease
        } else {
            snap.cycle
        };
        crate::paint_toolbar::paint_cycle_popover(ctx, tema, chip, cur);
    }
}
