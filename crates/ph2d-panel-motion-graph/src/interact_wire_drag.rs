//! **PUXAR UM FIO** — a máquina de estados do fio vivo: onde ele nasce, o alvo magnético que
//! procura enquanto anda, e o que acontece quando é largado longe de um pino.
//!
//! ⚠️ Irmão de [`super`] (`interact`) por RESPONSABILIDADE e não por contagem: o pai decide
//! *que gesto é este*, e este executa o único que tem estado próprio entre as fases.

use super::*;

/// Output-socket gestures: drag begins a wire; the ghost tracks the pointer and
/// snaps its validity to the hovered input; the drop emits `Connect`.
pub(super) fn apply_socket_out(
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
                detached: None,
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
            {
                let Some((to_node, to_port, _compat)) = target else {
                    // The wire landed on no input socket: a collapsed card, a regular node's
                    // BODY, or empty canvas — resolved in that order (doc 45/57/63.3).
                    let view = View::new(rect, state.view);
                    drop_gesture::resolve_loose_output_drop(
                        state, snap, &view, from_node, from_port, g.x, g.y,
                    );
                    return;
                };
                // Emit regardless of the local compatibility flag — the shell is
                // the authority (cycle / occupied / typing / membrane) and raises
                // the refusal toast.
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
pub(crate) fn target_socket(
    snap: &GraphViewSnapshot,
    view: &View,
    from_node: u32,
    from_port: u16,
    x: f32,
    y: f32,
) -> Option<(u32, u16, bool)> {
    let (to_node, to_port) = geom::nearest_input_socket(snap, view, x, y)?;
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
