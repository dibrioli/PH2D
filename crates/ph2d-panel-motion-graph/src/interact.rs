//! Gesture interpretation (Motion Nodes M1.E4–E7, Phase 1b). Drains the
//! `GraphSurface` channel the M0 dispatch fills (pointer gestures, anchored zoom,
//! graph keys) and turns it into ephemeral view/selection/drag/menu state, plus
//! the [`GraphIntent`]s the shell applies (doc mutations only).
//!
//! Coverage: pan (drag empty canvas), anchored wheel zoom, F = fit, Esc =
//! deselect / cancel, click/shift-select, multi-drag (one `MoveNodes` undo at
//! End), socket→socket **connect** (with a live compatibility ghost; the shell
//! validates for real), alt-click a wire = **disconnect**, R-press (anywhere) /
//! `A` = **add-node** menu, and Delete = **delete selection**.

use crate::geom::{self, View};
use crate::snapshot::{GraphIntent, GraphViewSnapshot, current_catalog, push_intent};
use crate::state::{AddMenu, Interaction, MotionGraphPanelState};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{
    GesturePhase, GraphGesture, GraphHitKind, GraphKey, GraphZoom,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerButton;

/// Drain this frame's graph input and fold it into `state` (+ push doc intents).
/// Called before drawing so the render reflects the latest gestures. `snap` is
/// the snapshot `paint` already fetched — reused for socket/wire hit-testing.
pub(crate) fn process(
    state: &mut MotionGraphPanelState,
    ctx: &mut PaintCtx,
    rect: Rect,
    center: Rect,
    snap: &GraphViewSnapshot,
) {
    let panel = ids::MOTION_GRAPH_PANEL;

    if let Some(z) = ctx.host.store_mut().take_graph_zoom(panel) {
        apply_zoom(state, rect, z);
    }
    let keys: Vec<GraphKey> = ctx.host.store_mut().drain_graph_keys().collect();
    for k in keys {
        apply_key(state, k, rect);
    }
    let gestures: Vec<GraphGesture> = ctx.host.store_mut().drain_graph_gestures().collect();
    for g in gestures {
        apply_gesture(state, g, rect, center, snap);
    }
}

// Wheel-zoom tuning (canvas-interaction constants, not chrome tokens).
const ZOOM_WHEEL_DIV: f32 = 240.0; // LITERAL-PX-OK: wheel-notch → zoom-factor sensitivity divisor
const ZOOM_MIN: f32 = 0.2; // LITERAL-PX-OK: min graph zoom
const ZOOM_MAX: f32 = 2.5; // LITERAL-PX-OK: max graph zoom

/// Anchored zoom: keep the cursor's graph point fixed while scaling.
fn apply_zoom(state: &mut MotionGraphPanelState, rect: Rect, z: GraphZoom) {
    let old = state.view.zoom;
    let factor = (z.delta / ZOOM_WHEEL_DIV).exp();
    let new = (old * factor).clamp(ZOOM_MIN, ZOOM_MAX); // CLAMP-OK: const bounds, min<max, non-NaN
    let f = new / old;
    // screen = base + pan + graph*zoom ⇒ hold `anchor` ⇒ pan' = (anchor-base)(1-f) + pan*f.
    state.view.pan_x = (z.anchor_x - rect.x) * (1.0 - f) + state.view.pan_x * f;
    state.view.pan_y = (z.anchor_y - rect.y) * (1.0 - f) + state.view.pan_y * f;
    state.view.zoom = new;
}

fn apply_key(state: &mut MotionGraphPanelState, k: GraphKey, rect: Rect) {
    match k {
        // Re-fit on the next paint (the draw pass owns the fit math).
        GraphKey::Fit => state.fitted = false,
        GraphKey::Escape => {
            state.selected.clear();
            state.interaction = Interaction::Idle;
            state.add_menu = None;
        }
        // Delete the selection (orphan edges go with the nodes, shell-side).
        // Empty selection → no intent (idempotent against the double key
        // dispatch: M0 focus gate + the shell's cursor push).
        GraphKey::Delete => {
            if !state.selected.is_empty() {
                push_intent(GraphIntent::DeleteSelection {
                    nodes: state.selected.iter().copied().collect(),
                });
                state.selected.clear();
            }
            state.interaction = Interaction::Idle;
        }
        // `A` opens the add-node menu at the canvas center (the keyboard verb
        // carries no cursor position). Idempotent: a second `A` (menu already
        // open) falls through to the no-op arm below.
        GraphKey::Add if state.add_menu.is_none() => {
            let center = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
            let spawn = View::new(rect, state.view).graph(center.0, center.1);
            state.add_menu = Some(AddMenu {
                screen: center,
                spawn,
            });
        }
        // Duplicate / knife / probe land later.
        _ => {}
    }
}

fn apply_gesture(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    rect: Rect,
    center: Rect,
    snap: &GraphViewSnapshot,
) {
    // Right-button opens the add-node menu at the cursor — on the PRESS (Begin),
    // over ANY hit (background, node, socket, wire). Doing it on Begin (not the
    // release Click) makes it movement-independent: a right-click that drifts a
    // pixel is classified End by the dispatch, which would otherwise never open
    // (or would dismiss) the menu. All secondary phases are absorbed here so a
    // right-drag / right-release never pans, selects, or dismisses.
    if g.button == PointerButton::Secondary {
        if g.phase == GesturePhase::Begin {
            let spawn = View::new(rect, state.view).graph(g.x, g.y);
            state.add_menu = Some(AddMenu {
                screen: (g.x, g.y),
                spawn,
            });
            state.interaction = Interaction::Idle;
        }
        return;
    }
    match g.kind {
        GraphHitKind::Background => apply_background(state, g, rect),
        GraphHitKind::Node { node } => apply_node(state, g, node as u32),
        GraphHitKind::SocketOut { node, port } => {
            apply_socket_out(state, g, node as u32, port, rect, snap)
        }
        // A wire: alt-click removes the edge (identified by its unique target
        // input, decoded from the opaque handle). Plain clicks are inert for now
        // (they fall through to the no-op arm below).
        GraphHitKind::Wire { edge } if g.phase == GesturePhase::Click && g.mods.alt => {
            let (to_node, to_port) = crate::paint::wire_target(edge);
            push_intent(GraphIntent::Disconnect { to_node, to_port });
        }
        // Split divider (E9): drag maps the pointer to a split fraction against
        // the full center band (scene `center` + graph `rect`). Begin/Update both
        // emit so the split tracks the cursor live; the shell clamps `t`.
        GraphHitKind::SplitDivider
            if matches!(g.phase, GesturePhase::Begin | GesturePhase::Update) =>
        {
            let vertical = rect.x > center.x + 0.5;
            let t = if vertical {
                (g.x - center.x) / (rect.x + rect.w - center.x).max(1.0)
            } else {
                (g.y - center.y) / (rect.y + rect.h - center.y).max(1.0)
            };
            push_intent(GraphIntent::SetSplit { t });
        }
        // Toolbar chips (E9): SplitH / SplitV flip orientation; Fit re-fits.
        GraphHitKind::Chrome { id } if g.phase == GesturePhase::Click => match id {
            crate::paint_chrome::CHROME_SPLIT_H => {
                push_intent(GraphIntent::SetSplitVertical { vertical: false })
            }
            crate::paint_chrome::CHROME_SPLIT_V => {
                push_intent(GraphIntent::SetSplitVertical { vertical: true })
            }
            crate::paint_chrome::CHROME_FIT => state.fitted = false,
            _ => {}
        },
        // Input sockets (reverse-drag of an occupied input) + backdrops land
        // later; ignore for now.
        _ => {}
    }
}

/// Empty-canvas gestures (primary button only — the secondary button is fully
/// handled in [`apply_gesture`]): pan, selection clear, and resolving the
/// add-node menu the right-press opened.
fn apply_background(state: &mut MotionGraphPanelState, g: GraphGesture, rect: Rect) {
    match g.phase {
        GesturePhase::Begin => {
            // A press while the menu is open consumes (no pan); the release
            // resolves it. Otherwise begin a pan.
            if state.add_menu.is_some() {
                state.interaction = Interaction::Idle;
            } else {
                state.interaction = Interaction::Pan { last: (g.x, g.y) };
            }
        }
        GesturePhase::Update => {
            if let Interaction::Pan { last } = &mut state.interaction {
                state.view.pan_x += g.x - last.0;
                state.view.pan_y += g.y - last.1;
                *last = (g.x, g.y);
            }
        }
        GesturePhase::Click => {
            if let Some(menu) = state.add_menu.take() {
                // A primary click while the menu is open closes it; a click on a
                // row also adds that node at the menu's spawn point.
                resolve_add_menu(&menu, rect, g.x, g.y);
            } else {
                // A plain tap on empty canvas clears the selection.
                state.selected.clear();
            }
            state.interaction = Interaction::Idle;
        }
        GesturePhase::End | GesturePhase::DoubleClick => {
            // A primary drag over empty canvas dismisses an open menu.
            state.add_menu = None;
            state.interaction = Interaction::Idle;
        }
    }
}

/// Resolve a primary click at `(x, y)` against the open add-menu: if it lands on
/// a catalog row, emit `AddNode` at the menu's graph-space spawn point.
fn resolve_add_menu(menu: &AddMenu, rect: Rect, x: f32, y: f32) {
    let catalog = current_catalog();
    let panel = geom::add_menu_panel(menu, catalog.len(), rect);
    for (i, c) in catalog.iter().enumerate() {
        if geom::add_menu_row(panel, i).contains(x, y) {
            push_intent(GraphIntent::AddNode {
                type_name: c.type_name,
                x: menu.spawn.0,
                y: menu.spawn.1,
            });
            return;
        }
    }
}

/// Node-body gestures: select on press, multi-drag with a live `MoveNodes`.
fn apply_node(state: &mut MotionGraphPanelState, g: GraphGesture, node: u32) {
    match g.phase {
        GesturePhase::Begin => {
            select_on_press(state, node, g.mods.shift);
            state.interaction = Interaction::DragNodes {
                nodes: state.selected.iter().copied().collect(),
                last: (g.x, g.y),
                started: false,
            };
        }
        GesturePhase::Update => {
            let zoom = state.view.zoom;
            if let Interaction::DragNodes {
                nodes,
                last,
                started,
            } = &mut state.interaction
            {
                let (dx, dy) = ((g.x - last.0) / zoom, (g.y - last.1) / zoom);
                *last = (g.x, g.y);
                if dx != 0.0 || dy != 0.0 {
                    if !*started {
                        push_intent(GraphIntent::BeginDrag);
                        *started = true;
                    }
                    // Applied live by the shell → the node tracks the cursor (no
                    // end-jump); one undo step for the whole drag.
                    push_intent(GraphIntent::MoveNodes {
                        nodes: nodes.clone(),
                        dx,
                        dy,
                    });
                }
            }
        }
        GesturePhase::End => {
            if let Interaction::DragNodes { started, .. } = std::mem::take(&mut state.interaction)
                && started
            {
                push_intent(GraphIntent::EndDrag);
            }
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            state.interaction = Interaction::Idle;
        }
    }
}

/// Output-socket gestures: drag begins a wire; the ghost tracks the pointer and
/// snaps its validity to the hovered input; the drop emits `Connect`.
fn apply_socket_out(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    node: u32,
    port: u16,
    rect: Rect,
    snap: &GraphViewSnapshot,
) {
    match g.phase {
        GesturePhase::Begin => {
            state.interaction = Interaction::DrawWire {
                from_node: node,
                from_port: port,
                cur: (g.x, g.y),
                target: None,
            };
        }
        GesturePhase::Update => {
            let view = View::new(rect, state.view);
            let target = target_socket(snap, &view, node, port, g.x, g.y);
            if let Interaction::DrawWire { cur, target: t, .. } = &mut state.interaction {
                *cur = (g.x, g.y);
                *t = target;
            }
        }
        GesturePhase::End => {
            if let Interaction::DrawWire {
                from_node,
                from_port,
                target,
                ..
            } = std::mem::take(&mut state.interaction)
                && let Some((to_node, to_port, _compat)) = target
            {
                // Emit regardless of the local compatibility flag — the shell is
                // the authority (cycle / occupied / typing / membrane) and raises
                // the refusal toast. Dropping in empty space (target None) is a
                // no-op (smart-connect is deferred).
                push_intent(GraphIntent::Connect {
                    from_node,
                    from_port,
                    to_node,
                    to_port,
                });
            }
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            state.interaction = Interaction::Idle;
        }
    }
}

/// The input socket under `(x, y)`, with whether it is locally type-compatible
/// with the source output (domain + dim + clock — `connects_directly` minus the
/// membrane, which the shell checks). `None` when the pointer is over no input.
fn target_socket(
    snap: &GraphViewSnapshot,
    view: &View,
    from_node: u32,
    from_port: u16,
    x: f32,
    y: f32,
) -> Option<(u32, u16, bool)> {
    let (to_node, to_port) = geom::hit_input_socket(snap, view, x, y)?;
    let out = snap
        .nodes
        .iter()
        .find(|n| n.id == from_node)
        .and_then(|n| n.outputs.get(from_port as usize));
    let inp = snap
        .nodes
        .iter()
        .find(|n| n.id == to_node)
        .and_then(|n| n.inputs.get(to_port as usize));
    let compat = match (out, inp) {
        (Some(o), Some(i)) => o.domain == i.domain && o.dim == i.dim && o.clock == i.clock,
        _ => false,
    };
    Some((to_node, to_port, compat))
}

/// Selection on node press: plain click selects only this node (unless it is
/// already part of the selection — then keep it, so a multi-drag works); Shift
/// toggles it into/out of the selection.
fn select_on_press(state: &mut MotionGraphPanelState, node: u32, shift: bool) {
    if shift {
        if !state.selected.insert(node) {
            state.selected.remove(&node);
        }
    } else if !state.selected.contains(&node) {
        state.selected.clear();
        state.selected.insert(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{GraphNodeView, GraphViewSnapshot, PortView, drain_intents};
    use ph2d_a11y::NodeId as A11yNodeId;
    use ph2d_editor_core::interaction::GestureMods;
    use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
    use ph2d_nodegraph::port::{Clock, Dim, Domain};

    const RECT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 800.0,
        h: 600.0,
    };
    // Scene half of the split (unused by the node/socket/menu tests; a valid
    // arg for `apply_gesture`).
    const CENTER: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 800.0,
        h: 300.0,
    };

    fn port(domain: Domain) -> PortView {
        PortView {
            name: "p",
            domain,
            dim: Dim::Scalar,
            clock: Clock::Frame,
        }
    }

    /// A → B with a matching output/input (`Instances/Scalar/Frame`), B's input
    /// socket 0 at screen (200, 37) under the identity view.
    fn two_node_snapshot() -> GraphViewSnapshot {
        let node = |id: u32, x: f32, ins: Vec<PortView>, outs: Vec<PortView>| GraphNodeView {
            id,
            display_name: "n".into(),
            category: NodeUiCategory::Utility,
            silhouette: NodeSilhouette::Rect,
            x,
            y: 0.0,
            inputs: ins,
            outputs: outs,
        };
        GraphViewSnapshot {
            nodes: vec![
                node(1, 0.0, vec![], vec![port(Domain::Instances)]),
                node(2, 200.0, vec![port(Domain::Instances)], vec![]),
            ],
            edges: vec![],
        }
    }

    fn gesture(kind: GraphHitKind, phase: GesturePhase, x: f32, y: f32) -> GraphGesture {
        GraphGesture {
            surface: A11yNodeId(0),
            kind,
            phase,
            x,
            y,
            button: PointerButton::Primary,
            mods: GestureMods::default(),
        }
    }

    #[test]
    fn socket_drag_over_compatible_input_emits_connect() {
        let _ = drain_intents(); // isolate this test thread's intent queue
        let snap = two_node_snapshot();
        let mut st = MotionGraphPanelState::default();
        let out = GraphHitKind::SocketOut { node: 1, port: 0 };
        // Begin on A's output, drag to B's input (200, 37), release there.
        apply_gesture(
            &mut st,
            gesture(out, GesturePhase::Begin, 10.0, 37.0),
            RECT,
            CENTER,
            &snap,
        );
        apply_gesture(
            &mut st,
            gesture(out, GesturePhase::Update, 200.0, 37.0),
            RECT,
            CENTER,
            &snap,
        );
        // The live ghost snapped to a compatible target.
        assert!(matches!(
            st.interaction,
            Interaction::DrawWire {
                target: Some((2, 0, true)),
                ..
            }
        ));
        apply_gesture(
            &mut st,
            gesture(out, GesturePhase::End, 200.0, 37.0),
            RECT,
            CENTER,
            &snap,
        );
        let intents = drain_intents();
        assert_eq!(
            intents,
            vec![GraphIntent::Connect {
                from_node: 1,
                from_port: 0,
                to_node: 2,
                to_port: 0,
            }]
        );
        assert!(matches!(st.interaction, Interaction::Idle));
    }

    #[test]
    fn socket_drag_into_empty_space_emits_nothing() {
        let _ = drain_intents();
        let snap = two_node_snapshot();
        let mut st = MotionGraphPanelState::default();
        let out = GraphHitKind::SocketOut { node: 1, port: 0 };
        apply_gesture(
            &mut st,
            gesture(out, GesturePhase::Begin, 10.0, 37.0),
            RECT,
            CENTER,
            &snap,
        );
        apply_gesture(
            &mut st,
            gesture(out, GesturePhase::Update, 500.0, 500.0),
            RECT,
            CENTER,
            &snap,
        );
        apply_gesture(
            &mut st,
            gesture(out, GesturePhase::End, 500.0, 500.0),
            RECT,
            CENTER,
            &snap,
        );
        assert!(drain_intents().is_empty());
    }

    #[test]
    fn alt_click_on_wire_emits_disconnect() {
        let _ = drain_intents();
        let snap = two_node_snapshot();
        let mut st = MotionGraphPanelState::default();
        let handle = crate::paint::wire_handle(2, 0);
        let mut g = gesture(
            GraphHitKind::Wire { edge: handle },
            GesturePhase::Click,
            100.0,
            37.0,
        );
        // Plain click: inert.
        apply_gesture(&mut st, g, RECT, CENTER, &snap);
        assert!(drain_intents().is_empty());
        // Alt-click: disconnect the edge into (2, 0).
        g.mods.alt = true;
        apply_gesture(&mut st, g, RECT, CENTER, &snap);
        assert_eq!(
            drain_intents(),
            vec![GraphIntent::Disconnect {
                to_node: 2,
                to_port: 0,
            }]
        );
    }

    #[test]
    fn delete_key_emits_delete_selection_and_is_idempotent() {
        let _ = drain_intents();
        let mut st = MotionGraphPanelState::default();
        st.selected.extend([1, 2]);
        apply_key(&mut st, GraphKey::Delete, RECT);
        assert_eq!(
            drain_intents(),
            vec![GraphIntent::DeleteSelection { nodes: vec![1, 2] }]
        );
        assert!(st.selected.is_empty());
        // A second Delete (double-dispatch) with the now-empty selection is inert.
        apply_key(&mut st, GraphKey::Delete, RECT);
        assert!(drain_intents().is_empty());
    }

    #[test]
    fn right_click_background_opens_menu_then_left_pick_adds_node() {
        let _ = drain_intents();
        crate::snapshot::set_current_node_catalog(vec![crate::snapshot::NodeChoice {
            type_name: "motion.grid",
            display: "Grid",
            category: NodeUiCategory::Source,
        }]);
        let mut st = MotionGraphPanelState::default();
        // R-press opens the menu at the cursor (on Begin, movement-independent).
        let mut rc = gesture(GraphHitKind::Background, GesturePhase::Begin, 120.0, 90.0);
        rc.button = PointerButton::Secondary;
        apply_gesture(&mut st, rc, RECT, CENTER, &two_node_snapshot());
        let menu = st.add_menu.expect("menu opened");
        assert_eq!(menu.spawn, (120.0, 90.0)); // identity view → graph == screen
        // Left-click the first (only) row → AddNode at the spawn point.
        let panel = geom::add_menu_panel(&menu, 1, RECT);
        let row = geom::add_menu_row(panel, 0);
        let pick = gesture(
            GraphHitKind::Background,
            GesturePhase::Click,
            row.x + 2.0,
            row.y + 2.0,
        );
        apply_gesture(&mut st, pick, RECT, CENTER, &two_node_snapshot());
        assert_eq!(
            drain_intents(),
            vec![GraphIntent::AddNode {
                type_name: "motion.grid",
                x: 120.0,
                y: 90.0,
            }]
        );
        assert!(st.add_menu.is_none()); // picking closes the menu
    }

    #[test]
    fn right_press_over_a_node_opens_menu_and_release_keeps_it() {
        let _ = drain_intents();
        let mut st = MotionGraphPanelState::default();
        // A right-press whose hit resolved to a node still opens the add-menu
        // (movement-independent, over any hit) — the node is not selected/dragged.
        let mut down = gesture(
            GraphHitKind::Node { node: 7 },
            GesturePhase::Begin,
            300.0,
            150.0,
        );
        down.button = PointerButton::Secondary;
        apply_gesture(&mut st, down, RECT, CENTER, &two_node_snapshot());
        assert!(
            st.add_menu.is_some(),
            "right-press opens the menu over a node"
        );
        assert!(st.selected.is_empty(), "the node is not selected");
        // A right-release classified as End (the click drifted) must NOT dismiss.
        let mut up = gesture(
            GraphHitKind::Node { node: 7 },
            GesturePhase::End,
            305.0,
            152.0,
        );
        up.button = PointerButton::Secondary;
        apply_gesture(&mut st, up, RECT, CENTER, &two_node_snapshot());
        assert!(
            st.add_menu.is_some(),
            "the right-release keeps the menu open"
        );
    }
}
