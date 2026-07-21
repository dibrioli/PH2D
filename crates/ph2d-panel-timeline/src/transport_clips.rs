//! The clip cluster in the transport bar: `[ Main ▾ ] [+] [copy] [✎] [🗑]`.
//!
//! Its own module rather than more arms of `transport.rs`, which is at the 600-line cap
//! (HR-18): the cluster is one coherent thing — the chip, the four buttons that act on the
//! clip it names, and the option list — so it is the honest seam to cut on.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, DropdownState, paint_dropdown_chip};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_timeline::TimelineIntent;

use crate::ids;
use crate::state::{self, TimelinePanelState};
use crate::tab::Tab;
use crate::transport::{BTN_W, ClipChip, icon_button};

const CLIP_DD_W: f32 = 108.0; // LITERAL-PX-OK: clip dropdown chip width

/// How wide the cluster paints — the single source `transport`'s flow measures against.
pub(crate) fn width(snap: &TimelineViewSnapshot, tab: Tab) -> f32 {
    let half = Spacing::Sm.px() * 0.5;
    // The Containers tab leads with the chip that says WHICH container is being edited.
    let host = if shows_host_picker(tab) {
        CLIP_DD_W + half
    } else {
        0.0
    };
    // [ Main v ] [+] [copy] [pencil] [trash] — the trash only exists above one clip.
    // Duplicate sits beside the `+` that made the clip (Enio, 2026-07-16): they are the
    // two ways to get a clip, and the difference is only whether it starts empty.
    let trash = if snap.clips.len() > 1 {
        BTN_W + half
    } else {
        0.0
    };
    host + CLIP_DD_W + half + BTN_W + half + BTN_W + half + BTN_W + half + trash
}

/// **Whether the "which container am I editing" chip is on screen** — the Containers tab
/// only.
///
/// Everywhere else the question has a fixed answer the view already states: Keys is the
/// active clip's, and Arrange is always the scene's ([`Tab::scene_root`]).
pub(crate) fn shows_host_picker(tab: Tab) -> bool {
    tab == Tab::Containers
}

/// The clip cluster: `[ Main ▾ ] [+] [✎] [🗑]`.
///
/// Returns the `x` after it, and the chip rect when the dropdown is open (the
/// caller defers the popover paint — see [`paint_bar`]).
///
/// The TRASH is not painted, and — the part that matters —  **not hit-registered**,
/// while the document holds a single clip: a document must always have one to edit,
/// and a dimmed button that still dispatches is a click that silently does nothing
/// ([[feedback_disabled_button_still_dispatches]]).
pub(crate) fn cluster(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    snap: &TimelineViewSnapshot,
    view: crate::transport::BarView,
) -> Option<ClipChip> {
    let gap = Spacing::Sm.px();
    let mut x = x;

    // The host picker leads: *where am I* before *what will I place*.
    if shows_host_picker(view.tab) {
        let hc = Rect::new(x, y, CLIP_DD_W, ROW_H_PX);
        ctx.host.hit_index_mut().register(ids::TIMELINE_HOST_DD, hc);
        let (st, op) = match ctx.host.store().get(ids::TIMELINE_HOST_DD) {
            Some(InteractiveState::Dropdown { state, open, .. }) => (*state, *open),
            _ => (DropdownState::Normal, false),
        };
        let mut hd = Dropdown::new(ids::TIMELINE_HOST_DD, "", host_options(snap))
            .open(op)
            .state(st);
        if let Some(c) = view.open_container {
            hd = hd.selected(c);
        }
        paint_dropdown_chip(&hd, hc, ctx.scene, ctx.text_system, theme);
        x += CLIP_DD_W + gap * 0.5;
        if op {
            // Only one popover can be deferred; the open one wins. Two lists over the same
            // pixels would fight, and the artist opened exactly one of them.
            return Some(ClipChip {
                rect: hc,
                open: true,
                host: true,
            });
        }
    }

    let chip = Rect::new(x, y, CLIP_DD_W, ROW_H_PX);
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_CLIP_DD, chip);
    let (state, open) = match ctx.host.store().get(ids::TIMELINE_CLIP_DD) {
        Some(InteractiveState::Dropdown { state, open, .. }) => (*state, *open),
        _ => (DropdownState::Normal, false),
    };
    let dd = Dropdown::new(ids::TIMELINE_CLIP_DD, "", source_options(snap, view.tab))
        .selected(selected_source(snap, view))
        .open(open)
        .state(state);
    paint_dropdown_chip(&dd, chip, ctx.scene, ctx.text_system, theme);
    x += CLIP_DD_W + gap * 0.5;

    x = icon_button(ctx, theme, x, y, ids::TIMELINE_ADD_CLIP, IconId::Plus) + gap * 0.5;
    x = icon_button(ctx, theme, x, y, ids::TIMELINE_DUP_CLIP, IconId::Duplicate) + gap * 0.5;
    x = icon_button(ctx, theme, x, y, ids::TIMELINE_RENAME_CLIP, IconId::Text) + gap * 0.5;
    if snap.clips.len() > 1 {
        icon_button(ctx, theme, x, y, ids::TIMELINE_DELETE_CLIP, IconId::Trash);
    }

    Some(ClipChip {
        rect: chip,
        open,
        host: false,
    })
}

/// **What the dropdown offers, per tab** — the SOURCE selector (ADR-0133, amended
/// 2026-07-21).
///
/// A strip plays a clip *or* a container, so "what am I about to place" is one question with
/// two kinds of answer, and **every tab that shows LANES offers both** — a lanes tab is a
/// surface you place things on, and both kinds are placeable (ADR-0133: a container instance
/// IS a strip). Only **Keys** is clips-only: a container has no keys, so picking one there
/// would be a selection that makes the view show nothing.
///
/// ⚠️ The first cut filtered the CLIPS out of the Containers tab, reasoning that the tab is
/// "about containers". That broke the very thing the tab is for — building a container's
/// interior IS putting clips in its lanes (*"na aba containers o dropbox não está exibindo os
/// clips"*, Enio 2026-07-21).
///
/// Which container the tab is EDITING is a different question with its own chip
/// ([`ids::TIMELINE_HOST_DD`]): picking a SOURCE never navigates, because inside container A
/// picking B has to mean *"place B inside A"*.
///
/// Each row carries the glyph of its KIND ([`CLIP_GLYPH`]/[`CONTAINER_GLYPH`]) — mixed in one
/// list without a mark, the artist learns the kind by placing one and seeing what happens.
///
/// The VALUE is the option's position in this list — `source_at` reads the kind back off the
/// id array, never off arithmetic on the index. Each half truncates at its own id array: an
/// option past it could be painted but never clicked.
pub(crate) fn source_options(snap: &TimelineViewSnapshot, tab: Tab) -> Vec<DropdownOption<usize>> {
    let mut out: Vec<DropdownOption<usize>> = Vec::new();
    out.extend(
        snap.clips
            .iter()
            .enumerate()
            .take(ids::TIMELINE_CLIP_OPT.len())
            .map(|(i, name)| {
                DropdownOption::new(ids::TIMELINE_CLIP_OPT[i], 0, name.clone())
                    .with_icon(CLIP_GLYPH)
            }),
    );
    if tab != Tab::Keys {
        out.extend(
            snap.containers
                .iter()
                .enumerate()
                .take(ids::TIMELINE_CONT_OPT.len())
                .map(|(i, c)| {
                    DropdownOption::new(ids::TIMELINE_CONT_OPT[i], 0, c.name.clone())
                        .with_icon(CONTAINER_GLYPH)
                }),
        );
    }
    // The value IS the position, filled once the list is built — a `DropdownOption` carries
    // both, and letting each half guess its own offset is how the two halves come to disagree.
    for (i, opt) in out.iter_mut().enumerate() {
        opt.value = i;
    }
    out
}

/// Which row of [`source_options`] is the current selection — the chip's label.
pub(crate) fn selected_source(
    snap: &TimelineViewSnapshot,
    view: crate::transport::BarView,
) -> usize {
    let clips = snap.clips.len().min(ids::TIMELINE_CLIP_OPT.len());
    match view.source_container {
        Some(c) if view.tab != Tab::Keys => clips + c.min(ids::TIMELINE_CONT_OPT.len()),
        _ => snap.active_clip.min(clips),
    }
}

/// **The Containers tab's host list** — every container, as a place to GO rather than a thing
/// to place. Same glyph as the source list's container half: it is the same kind of thing.
pub(crate) fn host_options(snap: &TimelineViewSnapshot) -> Vec<DropdownOption<usize>> {
    snap.containers
        .iter()
        .enumerate()
        .take(ids::TIMELINE_HOST_OPT.len())
        .map(|(i, c)| {
            DropdownOption::new(ids::TIMELINE_HOST_OPT[i], i, c.name.clone())
                .with_icon(CONTAINER_GLYPH)
        })
        .collect()
}

/// **The two kinds, told apart by a discreet glyph** (Enio, 2026-07-21).
///
/// One sheet against a stack of them: a clip is one animation, a container is several packed
/// into a piece. They only have to be distinguishable from EACH OTHER — the list is where
/// they sit side by side, and that is the whole job of the mark.
const CLIP_GLYPH: IconId = IconId::Layer;
/// …and the stack. See [`CLIP_GLYPH`].
const CONTAINER_GLYPH: IconId = IconId::Layers;

/// **What a picked option IS** — read off the id ARRAY it belongs to, never off arithmetic.
///
/// Returns `Err(clip index)` / `Ok(container index)`, or `None` for an id that is neither.
pub(crate) fn source_at(id: ph2d_a11y::NodeId) -> Option<Result<usize, usize>> {
    if let Some(i) = ids::TIMELINE_CLIP_OPT.iter().position(|&o| o == id) {
        return Some(Err(i));
    }
    ids::TIMELINE_CONT_OPT.iter().position(|&o| o == id).map(Ok)
}

/// Whether `ev` is the clip cluster's to answer.
///
/// Every id here is one this module's [`apply_event`] has an arm for. A router that
/// enumerated them separately from the arms would drift, and the drift is silent in the
/// direction that matters: an event CLAIMED and not handled is a control that clicks and
/// does nothing. The seam gate clicks each of them for exactly that reason.
pub(crate) fn owns(ev: &WidgetEvent) -> bool {
    let id = match *ev {
        WidgetEvent::Click(id)
        | WidgetEvent::Submit(id)
        | WidgetEvent::Blur(id)
        | WidgetEvent::Cancel(id) => id,
        _ => return false,
    };
    id == ids::TIMELINE_CLIP_DD
        || id == ids::TIMELINE_ADD_CLIP
        || id == ids::TIMELINE_DUP_CLIP
        || id == ids::TIMELINE_REVERSE_CLIP
        || id == ids::TIMELINE_RENAME_CLIP
        || id == ids::TIMELINE_DELETE_CLIP
        || id == ids::TIMELINE_CLIP_RENAME_INPUT
        || id == ids::TIMELINE_HOST_DD
        || ids::TIMELINE_HOST_OPT.contains(&id)
        || source_at(id).is_some()
}

/// Answer one clip-cluster event. Only reached for events [`owns`] claimed.
pub(crate) fn apply_event(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        // ── Clip selector (W5) ──────────────────────────────────────────────
        // Picking a clip from the open list. The store's `selected_index` is set
        // too, so the chip reads right on the SAME frame — the document round-trip
        // only lands on the next one.
        // ── Source selector: picking a CLIP or a CONTAINER from the open list ────
        // The kind comes from `source_at` (which id array the option belongs to), so the two
        // halves cannot be confused for one another at the same list position.
        // ── Host picker: WHERE the Containers tab is looking ────────────────
        // Its own control because it answers the other question (see `TIMELINE_HOST_DD`).
        WidgetEvent::Click(id) if ids::TIMELINE_HOST_OPT.contains(&id) => {
            if let Some(i) = ids::TIMELINE_HOST_OPT.iter().position(|&o| o == id) {
                state::open_container_root(state, i);
                if let Some(InteractiveState::Dropdown {
                    open,
                    selected_index,
                    ..
                }) = host.store_mut().get_mut(ids::TIMELINE_HOST_DD)
                {
                    *open = false;
                    *selected_index = Some(i);
                }
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if source_at(id).is_some() => {
            let snap = state::current_snapshot();
            let clips = if state.tab == Tab::Containers {
                0
            } else {
                snap.clips.len().min(ids::TIMELINE_CLIP_OPT.len())
            };
            // Where the pick sits in the painted list. Computed from the PICK, not from
            // `selected_source`: the clip half only reaches the snapshot next frame, and
            // asking the derived answer now would show the previous selection for one frame.
            let mut row = None;
            match source_at(id) {
                Some(Err(index)) => {
                    // A clip. The active clip lives in the DOCUMENT (editing keys is an edit),
                    // so it round-trips as an intent — and the source's kind flips here.
                    state::push_intent(TimelineIntent::SetActiveClip { index });
                    state.source_container = None;
                    row = Some(index);
                }
                Some(Ok(index)) => {
                    // A container, as a thing to PLACE. Panel-local (picking a source is not
                    // an edit) — and it deliberately does NOT navigate: inside container A,
                    // picking B has to mean *"place B inside A"*, and a control that both
                    // selected and travelled could not express that at all. Travelling is
                    // the host picker's job.
                    state.source_container = Some(index);
                    row = Some(clips + index);
                }
                None => {}
            }
            if let Some(InteractiveState::Dropdown {
                open,
                selected_index,
                ..
            }) = host.store_mut().get_mut(ids::TIMELINE_CLIP_DD)
            {
                *open = false;
                *selected_index = row;
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::TIMELINE_ADD_CLIP => {
            state::push_intent(TimelineIntent::AddClip);
            EventOutcome::Consumed
        }
        // Duplicate the ACTIVE clip — the sibling of `+`, and the difference is only
        // whether the clip starts empty. Refused past `MAX_CLIPS` by the document.
        WidgetEvent::Click(id) if id == ids::TIMELINE_DUP_CLIP => {
            state::push_intent(TimelineIntent::DuplicateClip {
                index: state::current_snapshot().active_clip,
            });
            EventOutcome::Consumed
        }
        // **I** — play the ACTIVE clip backwards.
        WidgetEvent::Click(id) if id == ids::TIMELINE_REVERSE_CLIP => {
            state::push_intent(TimelineIntent::ReverseClip {
                index: state::current_snapshot().active_clip,
            });
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::TIMELINE_RENAME_CLIP => {
            crate::clip_rename::open(state, &state::current_snapshot());
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::TIMELINE_DELETE_CLIP => {
            // The SECOND barrier: the paint does not even hit-register the trash
            // while a single clip remains, but a dimmed control that still
            // dispatches is precisely the bug that guard is for — so refuse here
            // too, and let the document refuse a third time
            // ([[feedback_disabled_button_still_dispatches]]).
            let snap = state::current_snapshot();
            if snap.clips.len() > 1 {
                state::push_intent(TimelineIntent::DeleteClip {
                    index: snap.active_clip,
                });
            }
            EventOutcome::Consumed
        }
        // Clip rename field — same Enter/click-away/Esc contract as the marker's.
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id)
            if id == ids::TIMELINE_CLIP_RENAME_INPUT =>
        {
            crate::clip_rename::commit(state, host.store());
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if id == ids::TIMELINE_CLIP_RENAME_INPUT => {
            crate::clip_rename::cancel(state);
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_timeline::{ContainerView, TimelineViewSnapshot};

    fn snap() -> TimelineViewSnapshot {
        TimelineViewSnapshot {
            clips: vec!["Main".into(), "Run".into()],
            containers: vec![ContainerView {
                name: "Walk".into(),
                length: 2.0,
            }],
            active_clip: 1,
            ..TimelineViewSnapshot::default()
        }
    }

    fn names(tab: Tab) -> Vec<String> {
        source_options(&snap(), tab)
            .into_iter()
            .map(|o| o.label)
            .collect()
    }

    /// **Toda aba de LANES oferece os dois tipos; a Keys só clips.**
    ///
    /// É o pedido do Enio (*"no modo de Arrange o dropbox passa a mostrar Clip e Conteiners"*)
    /// — e a Keys fica de fora porque um container não tem keys: escolhê-lo lá seria uma
    /// seleção que faz a vista não mostrar nada.
    ///
    /// ⚠️ A primeira versão filtrava os CLIPS fora da aba Containers, raciocinando que
    /// aquela aba "é sobre containers". Isso quebrava exatamente o que ela serve para fazer:
    /// montar o interior de um container É pôr clips nas lanes dele (*"na aba containers o
    /// dropbox não está exibindo os clips"*, Enio 2026-07-21). Toda aba de lanes é uma
    /// superfície onde se COLOCA, e os dois tipos são colocáveis.
    #[test]
    fn each_tab_offers_what_it_operates_on() {
        assert_eq!(names(Tab::Keys), vec!["Main", "Run"], "Keys: só clips");
        for tab in [Tab::Containers, Tab::Arrange] {
            assert_eq!(
                names(tab),
                vec!["Main", "Run", "Walk"],
                "{tab:?}: os dois, clips primeiro"
            );
        }
    }

    /// **O VALOR de uma opção é a posição na lista PINTADA.**
    ///
    /// As duas metades são construídas separadamente; se cada uma calculasse o próprio
    /// deslocamento, a segunda apontaria para a linha da primeira — o clique selecionaria
    /// visivelmente a coisa errada.
    #[test]
    fn every_option_names_the_row_it_is_painted_on() {
        let opts = source_options(&snap(), Tab::Arrange);
        assert_eq!(
            opts.iter().map(|o| o.value).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }

    /// **O chip nomeia a fonte que o `+` colocaria** — e o container vence, porque escolher um
    /// container é também navegar até ele.
    #[test]
    fn the_chip_names_the_selected_source() {
        let s = snap();
        let view = |tab, c| crate::transport::BarView {
            tab,
            speed_view: false,
            source_container: c,
            open_container: None,
        };
        assert_eq!(
            selected_source(&s, view(Tab::Arrange, None)),
            1,
            "sem container escolhido, o chip é o CLIP ativo"
        );
        assert_eq!(
            selected_source(&s, view(Tab::Arrange, Some(0))),
            2,
            "com um container escolhido, é ele — depois dos dois clips"
        );
        assert_eq!(
            selected_source(&s, view(Tab::Containers, Some(0))),
            2,
            "a aba Containers lista clips TAMBÉM, então o container segue depois deles"
        );
    }

    /// **O chip de HOST só existe onde "qual container?" é pergunta** — e a barra abre
    /// espaço para ele.
    ///
    /// ⚠️ Substitui um gate que afirmava o oposto sobre os quatro botões de CLIP (eles
    /// sumiam na aba Containers). Aquela decisão CAIU junto com o filtro que tirava os clips
    /// de lá: a aba lista clips de novo, então o clip ativo está em vista e os botões dele
    /// voltam a significar algo.
    #[test]
    fn the_host_picker_only_exists_where_which_container_is_a_question() {
        assert!(shows_host_picker(Tab::Containers));
        assert!(!shows_host_picker(Tab::Keys) && !shows_host_picker(Tab::Arrange));
        let s = snap();
        assert!(
            width(&s, Tab::Containers) > width(&s, Tab::Arrange),
            "a barra tem de abrir espaço para ele"
        );
    }

    /// **Cada linha diz de que TIPO ela é** — um glifo discreto, e os dois nunca coincidem
    /// (Enio, 2026-07-21). Numa lista com clips e containers misturados sem marca, o artista
    /// descobre o tipo colocando um deles e vendo o que acontece.
    #[test]
    fn every_row_carries_the_glyph_of_its_kind() {
        let opts = source_options(&snap(), Tab::Arrange);
        assert_eq!(
            opts.iter().map(|o| o.icon).collect::<Vec<_>>(),
            vec![Some(CLIP_GLYPH), Some(CLIP_GLYPH), Some(CONTAINER_GLYPH)],
            "dois clips e um container"
        );
        assert_ne!(
            CLIP_GLYPH, CONTAINER_GLYPH,
            "distinguir é o trabalho inteiro"
        );
        // A lista de HOSTS é toda de containers, e leva o mesmo glifo: é a mesma coisa.
        assert!(
            host_options(&snap())
                .iter()
                .all(|o| o.icon == Some(CONTAINER_GLYPH))
        );
    }
}
