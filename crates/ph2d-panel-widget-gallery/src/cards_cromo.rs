//! ⭐ **O cromo na galeria**: a barra de ferramentas e o que flutua, com os pintores do app. A geometria é
//! a deles ([`entry_rects`], [`ContextMenu::entry_rect`]); aqui só se diz o que há em cada um.

use crate::controlos::{card, find, lit};
use crate::ids as g;
use crate::state::Choices;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef};
use ph2d_editor_core::widget::cromo_barra::uniao;
use ph2d_editor_core::widget::{
    Barra, ButtonState, ContextMenu, ContextMenuEntry, Divider, ListItem, ListItemState, RailAxis,
    Superficie, ToolRail, ToolRailEntry, Tooltip, entry_rects, horizontal_lines, line_pitch,
    paint_context_menu, paint_tool_rail_axis, paint_tooltip, size_row_pulldowns,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;

/// `(id, ícone, chave da legenda)` dos chips da barra.
const CHIPS: [(NodeId, IconId, &str); 3] = [
    (g::BAR_MOVE, IconId::Move, "chrome.rail.caption.move"),
    (g::BAR_ROTATE, IconId::Rotate, "chrome.rail.caption.rot"),
    (
        g::BAR_TRANSFORM,
        IconId::Transform,
        "chrome.rail.caption.xform",
    ),
];

/// `(id, ícone, chave do nome, chave do atalho, chave da dica)` das linhas da lista.
const LINHAS: [(NodeId, IconId, &str, &str, &str); 3] = [
    (
        g::LIST_COPY,
        IconId::Copy,
        "panel.widget_gallery.cromo.copy",
        "panel.widget_gallery.cromo.copy_key",
        "panel.widget_gallery.cromo.tip.copy",
    ),
    (
        g::LIST_DUPLICATE,
        IconId::Duplicate,
        "panel.widget_gallery.cromo.duplicate",
        "panel.widget_gallery.cromo.duplicate_key",
        "panel.widget_gallery.cromo.tip.duplicate",
    ),
    (
        g::LIST_DELETE,
        IconId::Trash,
        "panel.widget_gallery.cromo.delete",
        "panel.widget_gallery.cromo.delete_key",
        "panel.widget_gallery.cromo.tip.delete",
    ),
];

fn aceso(id: NodeId, ch: &Choices, store: &WidgetStore) -> bool {
    find(id).is_some_and(|ctl| lit(ctl, ch, store))
}

/// A barra de ferramentas: a fila de sempre sobre o painel flutuante dela.
pub(crate) fn tool_bar(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_TOOL_BAR,
        tr("panel.widget_gallery.card.tool_bar"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let theme = c.ink.theme;
            let entries = CHIPS
                .iter()
                .map(|&(id, icon, sub)| {
                    let chip = ToolRailEntry::icon(id, tr(sub), icon).with_sub(tr(sub));
                    if aceso(id, &ch, c.store) {
                        chip.active()
                    } else {
                        chip
                    }
                })
                .collect();
            let mut rail = ToolRail::new(
                g::BAR_RAIL,
                tr("panel.widget_gallery.card.tool_bar"),
                entries,
            );
            size_row_pulldowns(&mut rail, c.text_system);
            let size = c.store.rail_button_size();
            let folga = m::FIELD_GAP;
            let largura = c.w - folga * 2.0;
            let linhas = horizontal_lines(&rail, largura, size) as f32;
            let h = linhas * line_pitch(size.chip_px());
            let fila = Rect::new(c.x + folga, y + folga, largura, h);
            let slots = entry_rects(&rail, fila, size, RailAxis::Horizontal);
            // A barra abraça os chips, como a que flutua sobre o canvas.
            let chips = uniao(slots.iter().map(|s| s.rect)).map_or(largura, |u| u.w);
            let painel = Rect::new(c.x, y, chips + folga * 2.0, h + folga * 2.0);
            if let Some(barra) = Barra::de(theme) {
                barra.painel(c.scene, painel);
            }
            paint_tool_rail_axis(
                &rail,
                fila,
                c.scene,
                c.text_system,
                theme,
                c.store,
                &|_| None,
                false,
                RailAxis::Horizontal,
            );
            for slot in slots {
                if let Some(id) = slot.id {
                    c.hit_index.register(id, slot.rect);
                }
            }
            painel.y + painel.h
        }),
    )
}

fn estado(id: NodeId, store: &WidgetStore) -> ListItemState {
    match store.button_state(id) {
        Some(ButtonState::Hovered) => ListItemState::Hovered,
        Some(ButtonState::Pressed) => ListItemState::Pressed,
        Some(ButtonState::Focused) => ListItemState::Focused,
        _ => ListItemState::Normal,
    }
}

/// O menu do cartão, com a linha escolhida marcada.
#[must_use]
pub fn menu(ch: &Choices, store: &WidgetStore) -> ContextMenu {
    let item = |i: usize| {
        let (id, icon, nome, atalho, _) = LINHAS[i];
        ContextMenuEntry::Item(
            ListItem::new(id, tr(nome))
                .icon(icon)
                .value(tr(atalho))
                .selected(aceso(id, ch, store))
                .state(estado(id, store)),
        )
    };
    ContextMenu::new(
        g::LIST_MENU,
        tr("panel.widget_gallery.card.floating_list"),
        vec![
            item(0),
            item(1),
            ContextMenuEntry::Separator(Divider::new(g::LIST_SEPARATOR)),
            item(2),
        ],
    )
}

/// O que flutua: o menu do app, e por baixo a dica do que está escolhido nele.
pub(crate) fn floating_list(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_FLOATING_LIST,
        tr("panel.widget_gallery.card.floating_list"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let theme = c.ink.theme;
            let menu = menu(&ch, c.store);
            let h = menu.preferred_height(m::FIELD_H);
            let caixa = Rect::new(c.x, y, c.w, h);
            let superficie = Superficie::de(theme);
            {
                let _veste = superficie.veste();
                paint_context_menu(&menu, caixa, c.scene, c.text_system, theme, m::FIELD_H);
            }
            for (i, entrada) in menu.entries.iter().enumerate() {
                if let (ContextMenuEntry::Item(linha), Some(r)) =
                    (entrada, menu.entry_rect(caixa, i, m::FIELD_H))
                {
                    c.hit_index.register(linha.id, r);
                }
            }
            let escolhida = LINHAS
                .iter()
                .find(|l| aceso(l.0, &ch, c.store))
                .map_or("", |l| tr(l.4));
            let px = ph2d_tokens::TypeToken::Sm.px();
            let medida = c.text_system.prefix_width(escolhida, px);
            let w = (medida + ph2d_tokens::Spacing::Xl.px()).min(c.w);
            let topo = caixa.y + caixa.h + m::FIELD_ROW_GAP;
            let balao = Rect::new(c.x, topo, w, m::FIELD_H);
            paint_tooltip(
                &Tooltip::new(g::LIST_TIP, escolhida),
                balao,
                c.scene,
                c.text_system,
                theme,
            );
            balao.y + balao.h
        }),
    )
}
