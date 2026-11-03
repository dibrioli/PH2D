//! A rolagem da lista de cartões nas bordas durante um arrasto — a do Kanban
//! (`ph2d_app_kanban::view::edge_scroll`): mais depressa quanto mais fundo na faixa.

use crate::interaction::WidgetStore;
use ph2d_a11y::NodeId;

/// A mão que arrasta um cartão a `y` perto do topo (`top`) ou do fundo da banda visível rola a
/// lista de `panel` (o alvo da rolagem; o tique leva o vivo até lá).
pub(super) fn na_borda(store: &mut WidgetStore, panel: NodeId, y: f32, top: f32) {
    let (Some(conteudo), Some(visivel)) =
        (store.panel_content_h(panel), store.panel_visible_h(panel))
    else {
        return;
    };
    let max = (conteudo - visivel).max(0.0);
    if max <= 0.0 || !(top..=top + visivel).contains(&y) {
        return;
    }
    let e = ph2d_card_stack::edge_depth(y, top, top + visivel);
    if e == 0.0 {
        return;
    }
    let dt = store
        .card_list_mut(panel)
        .last
        .map_or(0.0, |t| t.elapsed().as_secs_f64().min(0.1));
    #[allow(clippy::cast_possible_truncation)]
    let alvo = (f64::from(store.panel_scroll_target(panel))
        + ph2d_card_stack::edge_scroll_px(e, dt)) as f32;
    store.set_panel_scroll(panel, alvo.clamp(0.0, max));
}

/// A largura dos cartões: a lista menos as margens e, quando ela transborda (o quadro anterior), a
/// faixa da barra de rolagem — a barra nunca pinta sobre a beirada de um cartão.
pub(super) fn largura_dos_cartoes(store: &WidgetStore, panel: NodeId, x: f32, w: f32) -> f32 {
    let m = ph2d_card_stack::metrics::LIST_PAD_X;
    let transborda = match (store.panel_content_h(panel), store.panel_visible_h(panel)) {
        (Some(c), Some(v)) => crate::widget::scrollbar_is_needed(c, v),
        _ => false,
    };
    if !transborda {
        return w - m * 2.0;
    }
    let barra = crate::widget::scrollbar_track_rect(crate::zones::Rect::new(x, 0.0, w, 0.0));
    barra.x - ph2d_card_stack::metrics::LIST_BAR_GAP - (x + m)
}

/// A altura que a rolagem conta: a lista que ainda CABE abraça o conteúdo e não rola, por
/// construção — sem isto a nota que chega um quadro depois (ou meio pixel de diferença) punha a
/// barra num quadro, a barra estreitava os cartões, e a barra piscava (smoke do dono 02/11, «Look at»).
pub(super) fn conteudo(real: f32, cabe: bool) -> f32 {
    if cabe { 0.0 } else { real }
}
