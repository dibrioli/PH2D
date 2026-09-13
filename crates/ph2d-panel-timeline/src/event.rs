//! Timeline panel event router (W2.E2).
//!
//! The transport controls are document commands, not tool edits — the panel
//! translates each `WidgetEvent` into an [`EditorAction::TimelinePanelEvent`]
//! carrying a tool-agnostic [`PanelEvent`] (NodeId + payload); the shell drains
//! it, maps the id to a `ph2d_timeline::TimelineIntent`, and applies it (see
//! `render_loop::timeline_bridge::intent_for_transport`). The close (X) button
//! hides the panel directly through the host.

use crate::event_stack_menus::{lane_menu_click, strip_menu_click};
use crate::ids;
use crate::ids::{
    TIMELINE_ADD_MARKER, TIMELINE_AUTOKEY, TIMELINE_CLOSE, TIMELINE_FRAME_NUM, TIMELINE_GO_END,
    TIMELINE_GO_START, TIMELINE_LOOP, TIMELINE_MOTION_PATH, TIMELINE_NEXT_FRAME,
    TIMELINE_ONION_SETTINGS, TIMELINE_PHYSICS, TIMELINE_PINGPONG, TIMELINE_PLAY,
    TIMELINE_PREV_FRAME, TIMELINE_RECORD, TIMELINE_RULER, TIMELINE_SNAP, TIMELINE_TIME_NUM,
};
use crate::state;
use crate::{TimelinePanel, state::TimelinePanelState};
use ph2d_a11y::NodeId;
/// How long a strip of an EMPTY clip is: a clip with no keys has no duration, and
/// a strip of zero seconds paints as nothing and cannot be grabbed to fix.
///
/// It floors ONLY the empty case. Padding a short clip's span to a second used to
/// look harmless and was not: a 0.4 s clip in a 1 s box is a strip playing at 0.4x
/// before anyone asked it to (`slice == span * speed`), and the first stretch would
/// have snapped its rate to match.
const MIN_NEW_STRIP_S: f64 = 1.0;

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_timeline::TimelineIntent;

/// The transport + "+Track" buttons (Click → `PanelEvent::Click`; the shell
/// maps transport ids to a Playhead command and "+Track" ids to a Bind of the
/// selected sprite).
fn is_button(id: NodeId) -> bool {
    id == TIMELINE_PLAY
        || id == TIMELINE_ADD_MARKER
        || id == TIMELINE_GO_START
        || id == TIMELINE_GO_END
        || id == TIMELINE_PREV_FRAME
        || id == TIMELINE_NEXT_FRAME
        // Onion Settings (ADR-0142 W3b): a plain button whose Click reaches the shell, which owns
        // the `hero.store` the settings card lives in (the panel cannot open hero chrome).
        || id == TIMELINE_ONION_SETTINGS
        || ids::ADDPROP_BUTTONS.iter().any(|(bid, _)| *bid == id)
}

/// The two transport chips (ValueChanged → `PanelEvent::SetValue`). The Dur(s)
/// chip is NOT here: its routing needs the panel's own dropdown state
/// (`source_container`), so it pushes its intent directly — see the dedicated
/// arm in `apply_event`.
fn is_chip(id: NodeId) -> bool {
    id == TIMELINE_TIME_NUM || id == TIMELINE_FRAME_NUM
}

/// The transport toggles routed to the shell (Toggled → `PanelEvent::Toggle`).
/// `TIMELINE_SPEED` is deliberately absent — it is a panel-local VIEW toggle,
/// handled in `apply_event` without reaching the shell.
fn is_toggle(id: NodeId) -> bool {
    id == TIMELINE_LOOP
        || id == TIMELINE_PINGPONG
        || id == TIMELINE_PHYSICS
        || id == TIMELINE_AUTOKEY
        || id == TIMELINE_RECORD
        // Motion Path (ADR-0141): per-object, so it MUST reach the shell — the panel
        // has no selection; the shell resolves the entity and converts it.
        || id == TIMELINE_MOTION_PATH
        || id == TIMELINE_SNAP
}

pub(crate) fn apply_event(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // The clip stack answers first, through ONE door (`stack_event`): its chrome,
    // its two menus and its weight field all speak to the STACK rather than to the
    // sheet, and folding them into one guard is also what keeps `apply_event` under
    // its LOC cap — the cap noticing that the stack had grown into its own subject.
    if let Some(out) = stack_event(state, ev, host) {
        return out;
    }
    match ev {
        // Close (X) — hide the panel (mirror of the other docked panels).
        WidgetEvent::Click(id) if id == TIMELINE_CLOSE => {
            host.set_panel_visible(TimelinePanel::ID, false);
            EventOutcome::Consumed
        }
        // Ruler scrub: the slider value (0..1 over the visible span) maps back to
        // an absolute time via the span `paint` stored; forward it as a Scrub.
        WidgetEvent::ValueChanged(id) if id == TIMELINE_RULER => {
            let v = host
                .store()
                .slider(id)
                .map(|(_, v)| f64::from(v))
                .unwrap_or(0.0);
            let local = state.view_start_s + v * state.view_span_s;
            // **O eixo da régua É o relógio que o arrasto busca — IDENTIDADE em todo modo**
            // (Enio, 2026-07-22). O shell entrega o playhead do modo (cena · clip · CONTAINER)
            // e a régua é desenhada nesse mesmo relógio, então o valor cru já é o tempo. Dentro
            // de um container o transporte é o relógio DELE (`container_open`), não mais a cena
            // mapeada — foi a troca de relógios (ler num, escrever noutro) que fazia o playback
            // ser o do Arrange. Na aba KEYS o eixo é o clip; na cena raiz é a timeline.
            let time = local;
            host.bus_mut()
                .push(EditorAction::TimelinePanelEvent(PanelEvent::SetValue(
                    id, time,
                )));
            EventOutcome::Consumed
        }
        // Scrollbar drag: dispatch's vertical slider reads `1.0` at the TOP of
        // its track, so the scroll fraction is `1 - v` (panel-local; no intent).
        WidgetEvent::ValueChanged(id) if id == ids::TIMELINE_SCROLLBAR => {
            let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(1.0);
            state.scroll_y = crate::scrollbar::value_to_fraction(v) * state.scroll_max;
            EventOutcome::Consumed
        }
        // "+Track" opens/closes the property dropdown (panel-local; no intent).
        WidgetEvent::Click(id) if id == ids::TIMELINE_ADD_TRACK => {
            state.add_track_open = !state.add_track_open;
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if is_button(id) => {
            // Picking a property from the dropdown closes it.
            if ids::ADDPROP_BUTTONS.iter().any(|(bid, _)| *bid == id) {
                state.add_track_open = false;
            }
            host.bus_mut()
                .push(EditorAction::TimelinePanelEvent(PanelEvent::Click(id)));
            EventOutcome::Consumed
        }
        // **The Dur(s) chip writes THE VIEW on screen** (Enio, 2026-07-23): the
        // open container, the active clip (Keys), the scene (Arrange). Routed
        // HERE and shared with the painter (`transport::length_scope`), so what
        // you read and what your typing edits cannot diverge. 0 clears.
        WidgetEvent::ValueChanged(id) if id == ids::TIMELINE_LENGTH_NUM => {
            let v = host.store().number_value(id).unwrap_or(0.0);
            let len = (v > 0.0).then_some(v);
            let snap = crate::state::current_snapshot();
            // `snap.keys_mode`, NOT `state.tab`: the same number the box READ
            // (`view_end_seconds(keys_mode)`), so reading and writing cannot diverge —
            // which they did on a single-clip Keys tab (box showed the scene, typed the
            // clip).
            let intent = match crate::transport::length_scope(snap.container_open, snap.keys_mode) {
                crate::transport::LengthScope::Container(c) => {
                    ph2d_timeline::TimelineIntent::SetContainerLength { container: c, len }
                }
                crate::transport::LengthScope::Clip => {
                    ph2d_timeline::TimelineIntent::SetClipLength { len }
                }
                crate::transport::LengthScope::Scene => {
                    ph2d_timeline::TimelineIntent::SetSceneLength { len }
                }
            };
            crate::state::push_intent(intent);
            EventOutcome::Consumed
        }
        WidgetEvent::ValueChanged(id) if is_chip(id) => {
            let v = host.store().number_value(id).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::TimelinePanelEvent(PanelEvent::SetValue(
                    id, v,
                )));
            EventOutcome::Consumed
        }
        // The view tabs — panel-local VIEW state, never a document command, so they
        // are answered here and never reach the shell (mirror of `TIMELINE_SPEED`
        // below). Switching takes the rows out from under any in-flight gesture, so
        // it is cleaned up exactly as hiding the panel is: same cause, same door.
        //
        // The arm walks `tab::TABS` rather than naming ids, so a third tab is a row
        // in that table and nothing else — a hand-written match here would paint a
        // tab that clicks into the void.
        WidgetEvent::Click(id) if crate::tab::TABS.iter().any(|(tid, _)| *tid == id) => {
            let want = crate::tab::TABS
                .iter()
                .position(|(tid, _)| *tid == id)
                .map_or(crate::tab::Tab::default(), crate::tab::Tab::from_index);
            // **Tapping the tab you are already on returns it to its root**, and that is the
            // way back out of a container to the LIST (the phone tab-bar convention, and the
            // Vector pill's toggle is the same idiom here). `set_tab` is a no-op when the tab
            // does not change, so the pop is the whole effect.
            //
            // It has to be BEFORE the switch, not after: after, the comparison would always
            // be "equal" and every tab click would drop the trail — losing the animator's
            // place on a plain Keys→Containers switch, which is the opposite of the rule.
            if want == crate::tab::Tab::Containers && state.tab == want {
                crate::state::pop_to_depth(0);
            }
            crate::state::set_tab(state, want);
            EventOutcome::Consumed
        }
        // Speed-graph view toggle (W5) — panel-local view state, NOT a document
        // command: flip it here (no bus/intent), like the +Track dropdown. The
        // transport bar mirrors `speed_view` back into the store's switch each
        // paint, so the painted toggle follows this flip. Any in-flight band
        // gesture maps its pointer through the OLD view's value range — drop it
        // (and close its undo bracket) rather than let it resolve as garbage
        // (mirror of the hide-panel cleanup in `paint`).
        WidgetEvent::Toggled(id) if id == ids::TIMELINE_SPEED => {
            state.speed_view = !state.speed_view;
            let handle = state.handle_drag.take().is_some();
            let anchor = state.anchor_drag.take().is_some();
            if handle || anchor {
                crate::state::push_intent(ph2d_timeline::TimelineIntent::EndEdit);
            }
            EventOutcome::Consumed
        }
        // Onion (ADR-0142) — estado de vista GLOBAL (não per-objeto como o Motion Path),
        // então vai direto pelo canal de intent (como o Dur(s)), sem passar pela shell:
        // lê o onion autoritativo do snapshot, vira o bit, e reenvia o struct inteiro.
        WidgetEvent::Toggled(id) if id == ids::TIMELINE_ONION => {
            let mut onion = crate::state::current_snapshot().onion;
            onion.enabled = !onion.enabled;
            crate::state::push_intent(ph2d_timeline::TimelineIntent::SetOnion(onion));
            EventOutcome::Consumed
        }
        WidgetEvent::Toggled(id) if id == ids::TIMELINE_ONION_MODE => {
            let mut onion = crate::state::current_snapshot().onion;
            onion.mode = match onion.mode {
                ph2d_timeline::OnionMode::Keys => ph2d_timeline::OnionMode::Frames,
                ph2d_timeline::OnionMode::Frames => ph2d_timeline::OnionMode::Keys,
            };
            crate::state::push_intent(ph2d_timeline::TimelineIntent::SetOnion(onion));
            EventOutcome::Consumed
        }
        WidgetEvent::Toggled(id) if is_toggle(id) => {
            let on = host.store().toggle(id).map(|(_, on)| on).unwrap_or(false);
            host.bus_mut()
                .push(EditorAction::TimelinePanelEvent(PanelEvent::Toggle(id, on)));
            EventOutcome::Consumed
        }
        // O menu do botão direito numa track row (Delete Track · Auto-Orient), que
        // mora inteiro no `event_track_menu`: as duas linhas fazem a MESMA dança sobre
        // a requisição parqueada, e duas cópias divergiriam na próxima linha do menu.
        WidgetEvent::Click(id) if crate::event_track_menu::route(host, id).is_some() => {
            EventOutcome::Consumed
        }
        // O menu do botão direito num MARKER (Rename / Set Signal / Delete, ADR-0143).
        // Mora no `marker_menu`, que precisa do `state` — as duas edições abrem o campo
        // inline (`marker_rename`), estado do painel, não intent.
        WidgetEvent::Click(id) if crate::marker_menu::route(state, host, id).is_some() => {
            EventOutcome::Consumed
        }
        // Marker rename field (W4.T3). Enter → Submit, click-away → Blur both
        // commit (the `take` inside makes the Enter→Submit+Blur pair idempotent);
        // Esc → Cancel abandons it.
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id)
            if id == ids::TIMELINE_MARKER_RENAME_INPUT =>
        {
            crate::marker_rename::commit(state, host.store());
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if id == ids::TIMELINE_MARKER_RENAME_INPUT => {
            crate::marker_rename::cancel(state);
            EventOutcome::Consumed
        }

        // The clip cluster (chip, +, duplicate, I, pencil, trash, rename field) answers
        // in its own module — it is one coherent thing, and it is what pushed this
        // function past its cap.
        ev if crate::transport_clips::owns(&ev) => {
            crate::transport_clips::apply_event(state, host, ev)
        }
        _ => EventOutcome::Ignored,
    }
}

/// Everything the clip stack answers: its chrome, its two right-click menus, and
/// the lane weight field. `None` means "not ours" — the caller falls through to
/// the sheet.
fn stack_event(
    state: &mut TimelinePanelState,
    ev: WidgetEvent,
    host: &mut dyn PanelHostInternal,
) -> Option<EventOutcome> {
    match ev {
        WidgetEvent::Click(id) => stack_click(state, id)
            .or_else(|| strip_menu_click(state, id, host))
            .or_else(|| lane_menu_click(state, id, host)),
        // Grabbing (or clicking into) the weight field OPENS the undo bracket.
        //
        // Dispatch emits a `ValueChanged` for every Move of a number body-drag, and
        // each one, unbracketed, is its own atomic undo step: sliding the weight
        // across its range left dozens of Ctrl+Z steps behind it. Every other
        // document-mutating gesture in this panel brackets (`strip_drag`, `key_drag`,
        // `anchor_drag`); this was the one that did not.
        WidgetEvent::Focus(id) => {
            let lane = ids::TIMELINE_LANE_WEIGHT.iter().position(|&w| w == id)?;
            if state.weight_edit.is_none() {
                state.weight_edit = Some(lane);
                state::push_intent(TimelineIntent::BeginEdit);
            }
            Some(EventOutcome::Consumed)
        }
        // …and letting go closes it. Dispatch guarantees the Blur: it fires on
        // pointer-up for a body drag and on click-away for a typed edit. A gesture
        // that changed nothing commits no step (`commit_if_changed`).
        WidgetEvent::Blur(id) | WidgetEvent::Submit(id)
            if ids::TIMELINE_LANE_WEIGHT.contains(&id) =>
        {
            if state.weight_edit.take().is_some() {
                state::push_intent(TimelineIntent::EndEdit);
            }
            Some(EventOutcome::Consumed)
        }
        // The lane weight is a bounded field, so its edit arrives as a ValueChanged.
        WidgetEvent::ValueChanged(id) => {
            let lane = ids::TIMELINE_LANE_WEIGHT.iter().position(|&w| w == id)?;
            // A lane gone from the snapshot addresses nothing — the field is
            // registered for all MAX_LANES, because the store is populated once.
            if lane < crate::state::current_snapshot().lanes.len() {
                let weight = host.store().number_value(id).unwrap_or(1.0);
                state::push_intent(TimelineIntent::SetLaneWeight { lane, weight });
            }
            Some(EventOutcome::Consumed)
        }
        _ => None,
    }
}

/// The clip stack's chrome (ADR-0115): "+ Lane", and each lane's mute and
/// "+ Strip". `None` means "not one of ours" — the caller falls through.
fn stack_click(
    state: &mut TimelinePanelState,
    id: ph2d_editor_core::NodeId,
) -> Option<EventOutcome> {
    if id == ids::TIMELINE_ADD_LANE {
        crate::state::push_intent(ph2d_timeline::TimelineIntent::AddLane);
        return Some(EventOutcome::Consumed);
    }
    // **New container** — the asset, and NOTHING else.
    //
    // ⚠️ It used to open the new container too, and that was wrong twice over: it made the
    // Containers tab jump into edit mode on a press that said "make one", and it meant the
    // tab could never be seen as what Enio asked it to be — *"uma lista de containers
    // criados"* (2026-07-21). Making a thing and going into it are two acts; the second one
    // is the double-click on its bar.
    if id == ids::TIMELINE_ADD_CONTAINER {
        crate::state::push_intent(ph2d_timeline::TimelineIntent::AddContainer);
        return Some(EventOutcome::Consumed);
    }
    // The pencil on a container's row — the second of the list's three verbs.
    if let Some(index) = ids::TIMELINE_CONT_RENAME.iter().position(|&b| b == id) {
        // A container the snapshot no longer has raises nothing: the action expires with its
        // target, exactly as the lane mute's does.
        if index < crate::state::current_snapshot().containers.len() {
            crate::clip_rename::open_container(state, index);
        }
        return Some(EventOutcome::Consumed);
    }
    // The trash — the third verb (Enio, 2026-07-21: *"as funções normais de renomear,
    // deletar e entrar"*). One intent; the document cascades (asset + instances) inside it.
    if let Some(index) = ids::TIMELINE_CONT_DELETE.iter().position(|&b| b == id) {
        if index < crate::state::current_snapshot().containers.len() {
            // The source selection follows the ASSET the artist picked, not the slot number
            // it happened to occupy — deleted, it is no selection at all; above the hole, it
            // steps down with its asset (the mirror of the document's own remap).
            match state.source_container {
                Some(s) if s == index => state.source_container = None,
                Some(s) if s > index => state.source_container = Some(s - 1),
                _ => {}
            }
            // An open container rename holds an index into the same list; shifted, it would
            // commit onto a neighbour. Abandon it rather than guess.
            if state
                .clip_rename
                .is_some_and(|c| c.kind == crate::state::RenameKind::Container)
            {
                state.clip_rename = None;
            }
            crate::state::push_intent(ph2d_timeline::TimelineIntent::RemoveContainer { index });
        }
        return Some(EventOutcome::Consumed);
    }
    // The breadcrumb: segment 0 is the scene root (leave everything), segment `n` pops OUT to
    // the container at that depth. Popping to where you already are is a no-op, which is what
    // makes the trailing segment safe to paint as part of the trail rather than special-cased.
    if let Some(depth) = ids::TIMELINE_CRUMB.iter().position(|&b| b == id) {
        // The button's POSITION is not the depth once the trail is longer than the id array
        // (it elides from the outside, so you always see where you are). `trail` answers both
        // "what does this segment say" and "what depth is it" — the paint and the click read
        // the same list, so a segment can never pop somewhere other than what it shows.
        if let Some(d) = crate::breadcrumb::depth_of_slot(depth, crate::state::edit_path().len()) {
            crate::state::pop_to_depth(d);
        }
        // Every segment names a STACK, so clicking one also lands the panel on the tab that
        // shows that stack: the root is the scene (**Arrange**), every other level is a
        // container (**Containers**). It is what makes the trail a way BACK from the Keys tab
        // (Enio, 2026-07-20: *"em keys não consigo voltar direto para Jump"*): the trailing
        // segment's pop is a no-op, and without the tab switch the click did nothing at all.
        let want = if crate::state::trail_len() == 0 {
            crate::tab::Tab::Arrange
        } else {
            crate::tab::Tab::Containers
        };
        crate::state::set_tab(state, want);
        return Some(EventOutcome::Consumed);
    }
    if let Some(lane) = ids::TIMELINE_LANE_MUTE.iter().position(|&b| b == id) {
        // A lane the snapshot no longer has (deleted since the paint that
        // registered this button) raises nothing: the action expires with its
        // target, exactly as Delete Track's does.
        if let Some(v) = crate::state::current_snapshot().lanes.get(lane) {
            crate::state::push_intent(ph2d_timeline::TimelineIntent::SetLaneMuted {
                lane,
                muted: !v.muted,
            });
        }
        return Some(EventOutcome::Consumed);
    }
    if let Some(lane) = ids::TIMELINE_LANE_ADD_STRIP.iter().position(|&b| b == id) {
        let snap = crate::state::current_snapshot();
        if lane < snap.lanes.len() {
            // **The ACTIVE SOURCE, dropped AT THE PLAYHEAD** — what the dropdown names, where
            // you are looking. A clip or a container: placing is the one gesture both kinds
            // share (ADR-0133), and it is what a container was missing.
            let t = snap.time_seconds.max(0.0);
            let (source, len) = match state.source_container {
                Some(c) => (
                    ph2d_timeline::StripSource::Container(u16::try_from(c).unwrap_or(u16::MAX)),
                    snap.containers.get(c).map_or(0.0, |v| v.length),
                ),
                None => (
                    ph2d_timeline::StripSource::Clip(
                        u16::try_from(snap.active_clip).unwrap_or(u16::MAX),
                    ),
                    // The clip's EFFECTIVE length (last key, not authored duration) — the
                    // same number `add_strip` sizes the slice to, so the strip is born at
                    // speed 1 instead of crammed.
                    snap.clip_length_seconds,
                ),
            };
            // Only an EMPTY source (no keys, no strips inside) falls back to a grabbable
            // minimum: a zero-width strip is one nobody can grab to fix.
            let len = if len > 0.0 { len } else { MIN_NEW_STRIP_S };
            crate::state::push_intent(ph2d_timeline::TimelineIntent::AddStrip {
                lane,
                source,
                t_start: t,
                t_end: t + len,
            });
        }
        return Some(EventOutcome::Consumed);
    }
    None
}
