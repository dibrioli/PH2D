//! Wheel + scrollbar dispatch helpers.
//!
//! Extracted from [`super`] (Track A8). Two responsibilities:
//!
//! 1. [`dispatch_wheel`] — public entry point for wheel / trackpad
//!    events. Finds the panel under the cursor, adjusts its
//!    `panel_scroll`, and clamps against the painter-published
//!    `content_h` / `visible_h` so wheeling past the end doesn't
//!    produce a 1-frame jump.
//! 2. The two drags a scroll body answers — the bar (`begin_bar_drag`) and the body itself
//!    (`begin_body_drag`, with the inertia of `crate::interaction::fling`).
//!
//! ⚠️ **There is no routing table any more** (rolagem única, 2026-09-29). Until then
//!    `scrollbar_panel_for_id` mapped every bar id to its panel by hand — 26 arms, two of them for
//!    panels that no longer existed, and a new panel that forgot its arm painted a bar that could
//!    not be grabbed (the Input Map's was like that from 24/08 to 29/09). Today the owner is what the
//!    door `widget::scroll_area` PUBLISHES when it paints the bar ([`WidgetStore::scroll_bar_panel`]);
//!    a bar that was never painted cannot be pressed, so a published owner always exists by the time
//!    a press reaches it.

use super::super::{InteractiveState, WidgetEvent, WidgetStore};
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use ph2d_a11y::NodeId;

/// Wheel / trackpad scroll. Finds the panel under `(x, y)` via
/// [`WidgetStore::panel_at`] and adjusts that panel's
/// `panel_scroll` by `delta_y`. Caller (painter) is responsible
/// for clamping the offset against the panel's `content_h` —
/// dispatch only deltas, doesn't know content height.
pub fn dispatch_wheel<'frame>(
    store: &mut WidgetStore,
    event: ph2d_host::WheelEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    // Motion Nodes M0.T3 — a wheel over a graph surface is an anchored zoom,
    // consumed BEFORE any panel scroll so the graph zooms instead of scrolling
    // the panel underneath. The panel drains + applies the accumulated zoom.
    if let Some(surface) = store.graph_surface_at(event.x, event.y) {
        store.add_graph_zoom(surface, event.delta_y, event.x, event.y);
        return events.into_bump_slice();
    }
    // W2.E6 — a wheel over the timeline's dope-sheet drives its own view,
    // consumed BEFORE any panel scroll:
    //
    //   roda simples  -> ROLA as linhas de propriedade
    //   Ctrl/Cmd+roda -> zoom ancorado do eixo do tempo
    //   Shift+roda    -> pan horizontal do eixo do tempo
    //   eixo-x (trackpad) -> pan, sempre
    //
    // ⚠️ **A roda simples ZOOMAVA, e era a única superfície do app em que a roda sobre o corpo de
    // um painel não rolava o corpo.** Reportado assim mesmo por Enio (2026-08-15): *«a timeline
    // está sem o scroll … para as propriedades animadas»*. O scroll existia — em `Shift+roda`, que
    // ninguém descobre — e a barra existia e era **invisível** (o polegar era `border` sobre uma
    // pista `bg-2`: ΔL 0.11 numa faixa de 10 px). Duas metades do mesmo report, duas causas
    // distintas.
    //
    // ⚠️ **A lei escolhida é a do RESTO DO APP, não a de um editor de nós.** Este painel é uma
    // dope-sheet com uma LISTA de propriedades — a família do After Effects / Premiere, onde a roda
    // percorre a lista —, e não um grafo, que é de onde a convenção antiga foi herdada (o
    // comentário do `apply_wheel` ainda diz *"a mesma sensibilidade que o motion graph usa"*).
    // Vinte e quatro painéis deste app respondem *«a roda rola o corpo»*; a timeline ser a única
    // excepção **é** o report.
    //
    // ⚠️ **O `Alt` não entra nesta tabela de propósito:** o KDE rouba-o (precedente já pago pelo
    // `PH2D_STAGGER_SMOKE`, que usa Ctrl pela mesma razão). O `Ctrl+roda` é o modificador de zoom
    // universal (browser, VS Code, Figma) e o `Shift+roda` o de horizontal — nenhum dos dois é
    // invenção nossa.
    //
    // ⚠️ **O preço, nomeado:** com poucas propriedades as linhas cabem e a roda simples não faz
    // nada, exactamente como em qualquer outro painel cuja lista cabe. Quem zooma o tempo passa a
    // segurar Ctrl.
    if let Some(surface) = store.timeline_surface_at(event.x, event.y) {
        let m = event.modifiers;
        let (zoom, pan, scroll) = if m.ctrl || m.meta {
            (event.delta_y, event.delta_x, 0.0)
        } else if m.shift {
            (0.0, event.delta_y, 0.0)
        } else {
            (0.0, event.delta_x, event.delta_y)
        };
        store.add_timeline_wheel(surface, zoom, pan, scroll, event.x);
        return events.into_bump_slice();
    }
    // An OPEN dropdown popover scrolls first — it floats on top of any panel, and its rect lives in a
    // dedicated slot (not `panel_rects`) so `panel_at` isn't polluted. Its scroll value + heights use
    // the `panel_scroll`/`panel_*_h` tables keyed by the dropdown id.
    if let Some((dd, rect)) = store.dropdown_popover()
        && rect.contains(event.x, event.y)
        && matches!(
            store.get(dd),
            Some(InteractiveState::Dropdown { open: true, .. })
        )
    {
        store.wheel_panel(dd, event.delta_y);
        return events.into_bump_slice();
    }
    // ⭐⭐ **Uma SUB-REGIÃO rolável ganha à roda do painel que a contém** — a coluna de catálogos
    // do navegador é a primeira. ⚠️ Ela tem de vir ANTES do `panel_at`, que devolveria o painel
    // inteiro e rolaria a grade enquanto o polegar da coluna arrastava certo. Mesma forma do
    // popover acima, e a mesma razão de o rect viver num slot próprio.
    if let Some(sub) = store.sub_scroll_region_at(event.x, event.y) {
        store.wheel_panel(sub, event.delta_y);
        return events.into_bump_slice();
    }
    if let Some(panel) = store.panel_at(event.x, event.y) {
        store.wheel_panel(panel, event.delta_y);
    }
    events.into_bump_slice()
}

/// ⭐⭐ **Uma pressão numa BARRA arma o arrasto dela** — a metade da porta
/// [`crate::widget::scroll_area`] que vive no despacho.
///
/// ⚠️ **A trilha vem do que a porta PUBLICOU, nunca da altura do rect acertado.** Até 2026-09-29
/// era `track_h: rect.h`, e 14 painéis registavam o POLEGAR ⇒ a conta proporcional via uma trilha
/// do tamanho do polegar e o conteúdo andava várias vezes mais depressa do que o dedo (com o
/// polegar no mínimo o alcance colapsava a `1 px` e um pixel saltava ao fim). Uma barra que ainda
/// não passa pela porta cai no rect acertado, como antes.
///
/// ⭐ **Carregar na trilha FORA do polegar faz o polegar saltar para debaixo do dedo**
/// ([`crate::widget::scroll_area::scroll_for_track_press`]) e o arrasto continua dali — só para
/// barras publicadas, porque numa legada o rect acertado É o polegar.
pub(super) fn begin_bar_drag(
    store: &mut WidgetStore,
    id: NodeId,
    rect: crate::zones::Rect,
    cursor_y: f32,
) {
    let published = store.scroll_bar_track(id);
    let panel = if id == crate::widget::DROPDOWN_SCROLLBAR_ID {
        store.dropdown_popover().map(|(dd, _)| dd)
    } else {
        store.scroll_bar_panel(id)
    };
    let Some(panel) = panel else {
        return;
    };
    let (Some(content_h), Some(visible_h)) =
        (store.panel_content_h(panel), store.panel_visible_h(panel))
    else {
        return;
    };
    let track = published.unwrap_or(rect);
    store.stop_fling(panel);
    // ⚠️ O alvo passa a ser ONDE A SUPERFÍCIE ESTÁ: com a mola a meio caminho, o arrasto 1:1
    //    partiria do alvo e a lista saltaria debaixo do dedo no primeiro Move.
    let mut scroll_at_down = store.panel_scroll(panel);
    if published.is_some() {
        let thumb =
            crate::widget::scrollbar_thumb_rect(track, scroll_at_down, content_h, visible_h);
        if cursor_y < thumb.y || cursor_y > thumb.y + thumb.h {
            scroll_at_down = crate::widget::scroll_area::scroll_for_track_press(
                track, cursor_y, content_h, visible_h,
            );
        }
    }
    store.set_panel_scroll(panel, scroll_at_down);
    store.begin_scrollbar_drag(crate::interaction::drag::ScrollbarDragAnchor {
        panel,
        cursor_y_at_down: cursor_y,
        scroll_at_down,
        track_h: track.h,
        content_h,
        visible_h,
    });
}

/// ⭐⭐ **O dedo agarra o CORPO de um painel** — arma o arrasto 1:1 e começa a amostrar o dedo
/// para a inércia de quando ele largar ([`crate::interaction::fling`]).
pub(super) fn begin_body_drag(store: &mut WidgetStore, panel: NodeId, cursor_y: f32, t_ns: u128) {
    store.stop_fling(panel);
    let scroll_at_down = store.panel_scroll(panel);
    store.set_panel_scroll(panel, scroll_at_down);
    store.clear_body_scroll_samples();
    store.push_body_scroll_sample(t_ns, cursor_y);
    store.begin_body_scroll_drag(crate::interaction::drag::BodyScrollAnchor {
        panel,
        cursor_y_at_down: cursor_y,
        scroll_at_down,
    });
}

/// Uma pressão sobre um painel em voo SEGURA-O, acerte ela num widget ou no vazio.
pub(super) fn grab_fling_at(store: &mut WidgetStore, x: f32, y: f32) {
    if let Some(panel) = store.panel_at(x, y) {
        let live = store.panel_scroll(panel);
        if store.flings().any(|(p, _)| p == panel) {
            store.stop_fling(panel);
            store.set_panel_scroll(panel, live);
        }
    }
}
