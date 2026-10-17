//! ⭐⭐ **As ABAS do título** — «Drawing» / «Interface» (F17, dono 16/10: *«duas abas NO PRÓPRIO
//! TÍTULO»*, sem faixa nova — a recusa de 31/08).
//!
//! ⚠️ **As abas SÃO o título**, como no Figma: na doca mínima (`210` px) sobram `132` px entre a
//! borda e o X, e «Vector» em `Lg` mais duas abas não cabem (`28_plano_F17…` §1). O nome do painel
//! fica no `TabList` (a11y) e na aba da doca. A banda é a do X — a altura do painel não muda.
//!
//! Pintar e registar andam pela MESMA lista ([`abas`]); a marca na aba Interface diz que a seleção
//! tem algo que só lá se edita ([`crate::state::seleccao_tem_interface`]).

use crate::ids;
use crate::state::Aba;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::paint::{fill_circle, resolve};
use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal, seam_reset_button};
use ph2d_editor_core::widget::panel_chrome::{PANEL_HEAD_PAD, panel_close_button_rect};
use ph2d_editor_core::widget::{TabItem, Tabs, paint_tabs_with_hover};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing, Theme};

/// As abas, pela ordem em que se pintam e registam: `(id, aba, chave do rótulo)`.
pub(crate) const ABAS: [(ph2d_a11y::NodeId, Aba, &str); 2] = [
    (
        ids::VECTOR_TAB_DRAWING,
        Aba::Drawing,
        "panel.vector.tab.drawing",
    ),
    (
        ids::VECTOR_TAB_INTERFACE,
        Aba::Interface,
        "panel.vector.tab.interface",
    ),
];

/// A aba de um id de aba — `None` = não é uma aba.
#[must_use]
pub(crate) fn aba_do_id(id: ph2d_a11y::NodeId) -> Option<Aba> {
    ABAS.iter().find(|(t, _, _)| *t == id).map(|(_, a, _)| *a)
}

/// ⭐⭐ **O clique numa aba ou no interruptor da ①** — a ÚNICA porta de [`crate::state::set_aba`].
/// O interruptor alterna; a aba abre-se a si. Trocar de aba volta o corpo ao topo (o que estava à
/// vista era outra lista).
pub(crate) fn apply_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let nova = match aba_do_id(id) {
        Some(a) => a,
        None if id == ids::VECTOR_VIEW_INTERFACE => match crate::state::aba() {
            Aba::Drawing => Aba::Interface,
            Aba::Interface => Aba::Drawing,
        },
        None => return false,
    };
    seam_reset_button(host, id);
    if nova != crate::state::aba() {
        crate::state::set_aba(nova);
        let store = host.store_mut();
        store.set_panel_scroll(core_ids::VECTOR_PANEL, 0.0);
        store.set_panel_scroll_live(core_ids::VECTOR_PANEL, 0.0);
    }
    true
}

/// O rect das abas: da borda esquerda até ao X, na banda dele.
#[must_use]
pub(crate) fn tabs_rect(panel: Rect) -> Rect {
    let close = panel_close_button_rect(panel);
    let x = panel.x + PANEL_HEAD_PAD;
    Rect::new(
        x,
        close.y,
        (close.x - Spacing::Sm.px() - x).max(0.0),
        close.h,
    )
}

/// A fila, com a aba aberta escolhida.
#[must_use]
pub(crate) fn abas() -> Tabs {
    let aberta = crate::state::aba();
    let items = ABAS
        .iter()
        .map(|(id, _, chave)| TabItem::new(*id, tr(chave)))
        .collect();
    let idx = ABAS.iter().position(|(_, a, _)| *a == aberta).unwrap_or(0);
    Tabs::new(
        ids::VECTOR_TABS,
        <crate::VectorPanel as ph2d_editor_core::panel::Panel>::TITLE.tr(),
        items,
    )
    .selected(idx)
}

/// Pinta as abas no título e regista o alvo de cada uma. Chamado DEPOIS do corpo: o último a
/// registar ganha o clique, e nenhum alvo do corpo pode ficar por cima da banda do título.
pub(crate) fn paint(ctx: &mut PaintCtx, panel: Rect, theme: Theme) {
    let tabs = abas();
    let r = tabs_rect(panel);
    let hot = ctx.host.store().hot_id();
    let hovered = tabs.items.iter().position(|t| hot == Some(t.id));
    paint_tabs_with_hover(&tabs, hovered, r, ctx.scene, ctx.text_system, theme);
    for i in 0..tabs.items.len() {
        ctx.host
            .hit_index_mut()
            .register(tabs.items[i].id, tabs.tab_rect(r, i));
    }
    if crate::state::aba() == Aba::Drawing && crate::state::seleccao_tem_interface() {
        let t = tabs.tab_rect(r, 1);
        fill_circle(
            ctx.scene,
            t.x + t.w - Spacing::Sm.px(),
            t.y + Spacing::Sm.px(),
            Spacing::Xs.px(),
            resolve(ColorToken::Accent, theme),
        );
    }
}
