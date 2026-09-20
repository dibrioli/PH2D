//! `ph2d-panel-motion-graph` — the Motion Nodes graph-editor panel (M0.T9).
//!
//! Docked in the `motion_graph` region of [`HeroLayout`] (the graph half of the
//! center split), visible only while the `motion` tool is active — the shell's
//! `motion_bridge` drives `panel_visible("motion_graph")`.
//!
//! **M0 skeleton:** paints the opaque graph-canvas background (`graph-bg`, Fase A
//! of plan §2.1 — the fill covers the sprite render beneath the graph half) and
//! publishes its rect so pointer dispatch can route to it. Node cards, wiring,
//! pan/zoom and gestures land in M1 (the `GraphSurface` dispatch + snapshot come
//! from M0.T2/T3 + the bridge).

#![forbid(unsafe_code)]

mod backdrop;
mod flow;
mod geom;

mod hits;
mod interact;
/// **O custo de um CARTÃO pintado** — a medição do ciclo 1 (doc 103), irmã do
/// `measure_row_cost` do painel de params: sem os dois números não há orçamento para pôr
/// rows dentro dos cartões.
#[cfg(test)]
#[path = "measure_card_cost.rs"]
mod measure_card_cost;
mod paint;
mod paint_chrome;
/// **A LEI do realce de uma largada** — ver o cabeçalho do módulo.
///
/// ⚠️ **Ele NÃO se chama `paint_*` de propósito:** o censo de a11y (HR-12) varre todo ficheiro
/// `paint_*` de um painel à procura de fiação de acessibilidade, e esta é uma DECISÃO pura (que
/// força acende) sem um pixel dentro. *O nome de um ficheiro é a primeira coisa que um censo lê.*
mod realce;
mod split;
/// O id de hit de um chip da barra, para o gate de costura que mede a pintura.
pub use paint_chrome::chrome_hit_id_for_tests;
/// ⭐ A lei do arrasto do divisor — pública porque o gate dela é de **ida-e-volta** e tem de
/// atravessar as duas crates (a fórmula aqui, a aplicação no `HeroLayout`).
pub use split::split_fraction;
mod param_edit;
mod param_editor;
/// ⭐ O id de uma amostra do editor aberto sobre um cartão — lido pela SHELL, que faz a leitura
/// de volta do selector para dentro da string. Ver [`param_editor::card_editor_swatch_id`].
pub use param_editor::card_editor_swatch_id;
mod probe;
mod rename;
mod snapshot;
mod state;

/// The smallest a backdrop may be resized to (graph space). Published so the
/// shell — which owns the document and therefore the clamp — enforces exactly the
/// minimum the panel drew and hit-tested, instead of a second number that could
/// drift from it.
pub use backdrop::{MIN_H as BACKDROP_MIN_H, MIN_W as BACKDROP_MIN_W};
pub use snapshot::{
    CardChoices, CardParam, CardSection, ChoiceTarget, Crumb, GraphBackdropView, GraphEdgeView,
    GraphIntent, GraphNodeView, GraphViewSnapshot, HiddenPorts, NodeChoice, NodeViewKind,
    PROBE_SAMPLES, Piscada, PortChoice, PortView, PreviewThumb, ProbeView, RenameTarget, RowText,
    SUBGRAPH_VIEW_TAG, card_hidden_ports, current_graph_backdrop_selection, current_graph_flash,
    current_graph_param_scrub, current_graph_selection, drain_intents, is_subgraph_view,
    library_pick, pending_graph_selection, push_intent, request_graph_selection, set_card_choices,
    set_card_hidden_ports, set_card_texts, set_current_motion_graph, set_current_node_catalog,
    set_graph_backdrop_selection, set_graph_flash, set_graph_param_scrub, set_graph_selection,
    set_node_help, snapshot_from,
};
pub use state::MotionGraphPanelState;

/// ⭐ **O que um clique numa row do cartão FAZ** — a porta única do gesto, exposta porque o
/// **censo** do que o cartão ainda não alcança tem de ler a MESMA lei. Uma lista de espécies
/// escrita à mão do lado de fora deixaria uma espécie nova entrar no produto sem entrar na
/// conta, que é a forma do knob inalcançável.
pub use interact::param_row::{ClickDoes, click_does};

/// The `node-cat-*` colour token for a UI category — the single source shared by the graph menu row dot
/// and the shell's full-screen "Add Node" palette model (so the two never disagree on a category's hue).
pub use paint::{PortLabel, cat_token, input_label_budget_px};

/// **Is the add-menu open?** (seam gates only — the panel's state is private, and a test that
/// cannot see the menu cannot tell "it never opened" from "it opened and ate the click".)
pub fn menu_is_open(state: &MotionGraphPanelState) -> bool {
    state.menu_for_test().is_some()
}

/// The first row's rect, as the PAINT laid it out — the row the artist clicks. `None` when no
/// menu is open (or nothing matched the search).
/// Screen → graph, through the panel's OWN view (seam gates: a test that places a node "under
/// the cursor" has to use the same map the panel draws with, or it places it somewhere else).
pub fn graph_point(
    state: &MotionGraphPanelState,
    rect: zones::Rect,
    sx: f32,
    sy: f32,
) -> (f32, f32) {
    geom::View::new(rect, state.view_for_test()).graph(sx, sy)
}

pub fn first_menu_row(state: &MotionGraphPanelState, rect: zones::Rect) -> Option<zones::Rect> {
    let menu = state.menu_for_test()?;
    let snap = snapshot::current_snapshot();
    let rows = snapshot::menu_rows(&snap, menu);
    if rows.is_empty() {
        return None;
    }
    let panel = geom::menu_panel(menu, rows.len(), rect);
    Some(geom::menu_row(menu, panel, 0, menu.scroll))
}
use ph2d_editor_core::zones;

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// Zero-size marker implementing the typed graph-editor panel contract.
pub struct MotionGraphPanel;

impl Panel for MotionGraphPanel {
    type State = MotionGraphPanelState;

    const ID: &'static str = "motion_graph";
    const NODE_ID: NodeId = ids::MOTION_GRAPH_PANEL;
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.motion_graph.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::MotionNodes;
    /// ⭐⭐ **O ÚNICO painel que declara o CENTRO, e não é uma excepção — é a decisão D5.**
    ///
    /// O grafo não FLUTUA sobre a área de desenho: ele parte-a em duas regiões **irmãs**
    /// (`CenterSplit`), que é literalmente o que a D5 diz que uma região é. ⛔ Declarar
    /// `RightTop` — o que ele fazia enquanto o `DEFAULT_SLOT` tinha default — era a mentira que
    /// as abas iriam ler: ele apareceria como aba da coluna da direita, onde nunca esteve.
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::CENTER;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::Center;

    fn paint(state: &mut MotionGraphPanelState, ctx: &mut PaintCtx) {
        if !ctx.host.panel_visible(MotionGraphPanel::ID) {
            // Stale-rect cleanup so `panel_at` stops returning this panel once the
            // tool deactivates; also drop the graph keyboard focus.
            ctx.host
                .store_mut()
                .clear_panel_rect(ids::MOTION_GRAPH_PANEL);
            ctx.host.store_mut().set_graph_focused(None);
            return;
        }
        let rect = ctx.layout.motion_graph;
        if rect.w <= 0.0 || rect.h <= 0.0 {
            // No split (or a degenerate one) → nothing to draw.
            ctx.host
                .store_mut()
                .clear_panel_rect(ids::MOTION_GRAPH_PANEL);
            return;
        }
        // Publish the rect so wheel/click dispatch can route to this panel.
        ctx.host
            .store_mut()
            .set_panel_rect(ids::MOTION_GRAPH_PANEL, rect);
        // Fase A background + the M1 graph (cards / sockets / wires) + gesture
        // handling — reads the snapshot the shell bridge published this frame.
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut MotionGraphPanelState,
        _host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        // The graph's own gestures arrive on the `GraphSurface` dispatch channel, not here. The
        // node library is the shell's full-screen palette now (it owns its own search + Enter/Esc);
        // the one widget left in THIS panel is the inline rename box.
        match ev {
            // **Enter names the thing** (doc 61). The box is the other widget in this panel, and
            // it owns the keyboard while it is open, so Enter and Esc are ITS events too.
            WidgetEvent::Submit(id) if id == hits::rename_id() => {
                rename::commit(state, _host.store());
                EventOutcome::Consumed
            }
            // **Esc keeps the old name.** (`mark_cancel_on_escape` is what routes Esc here rather
            // than merely blurring the field.)
            WidgetEvent::Cancel(id) if id == hits::rename_id() => {
                state.rename = None;
                EventOutcome::Consumed
            }
            // ⭐⭐ **O TEXTO ESCRITO** (report do Enio, 2026-09-07: *«não funcionou. Tempo
            // permaneceu»*). Um `TextInput` comita por **`Submit`** — o `ValueChanged` abaixo é
            // do `NumberInput`, e é o que ele produz depois de ANALISAR o buffer como número.
            //
            // ⛔⛔ **A caixa de texto abria, aceitava, e o `Enter` caía no `_ => Ignored`.** O
            // gate que eu tinha chamava o `param_edit::commit` **directamente** — e o cabeçalho
            // deste ficheiro avisa exactamente contra isso: *um teste que empurra o gesto já
            // assumiu a resposta*. A lei estava certa; ninguém a chamava.
            WidgetEvent::Submit(id) if id == hits::param_edit_id() => {
                param_edit::commit(state, _host.store());
                EventOutcome::Consumed
            }
            // ⭐ **O NÚMERO ESCRITO** (report do Enio, 2026-09-05). Um `NumberInput` comita por
            // `ValueChanged` — é o que o `Enter` produz depois de analisar o buffer, e também o
            // que as setinhas e o arrasto DENTRO da caixa produzem.
            WidgetEvent::ValueChanged(id) if id == hits::param_edit_id() => {
                param_edit::commit(state, _host.store());
                EventOutcome::Consumed
            }
            // ⭐⭐ **O ARRASTO DE UMA ALÇA do editor rico** chega como `ValueChanged(raiz)` — o
            // despacho guardou o ponto normalizado e este braço dobra-o no texto. ⚠️ Ele vem
            // DEPOIS do braço da caixa de número de propósito: aquele compara com um id fixo,
            // este pergunta ao editor aberto.
            WidgetEvent::ValueChanged(id)
                if param_editor::on_drag(state, _host.store_mut(), id) =>
            {
                EventOutcome::Consumed
            }
            // ⭐ **Um botão do editor rico** (`+` / `−` / interp).
            WidgetEvent::Click(id) if param_editor::on_click(state, id) => EventOutcome::Consumed,
            // ⚠️ **O `Blur` é o ÚNICO fecho, e tem de ser** — ele é o que os TRÊS caminhos têm em
            // comum: o `Enter` (que manda `ValueChanged` e depois este), o `Esc` (que repõe o
            // buffer e manda **só** este) e o clique fora. Fechar no `ValueChanged` deixaria o
            // `Esc` sem quem o ouvisse, que é como uma caixa fica no ecrã a comer o teclado.
            WidgetEvent::Blur(id) if id == hits::param_edit_id() => {
                state.param_edit = None;
                EventOutcome::Consumed
            }
            _ => EventOutcome::Ignored,
        }
    }

    fn populate(_store: &mut WidgetStore) {
        // No focusable widgets in the M0 skeleton.
    }
}
