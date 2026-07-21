//! **The lane area's ADD button** — exactly one, and WHICH one says where you are.
//!
//! Its own module (split out when `stack_lane_paint.rs` crossed the panel LOC cap): the
//! header is the one part of the lane area that exists over an EMPTY stack, because it is how
//! the first row is made.
//!
//! # One button, not two (Enio, 2026-07-21)
//!
//! It briefly carried "+ Lane" and "+ Container" side by side, and the pair had to be split
//! by label length so the long one was not crushed to a bare "+". The pair was the real
//! defect: they belong to different PLACES.
//!
//! - The **Containers tab, outside any container** is the level where containers are MADE:
//!   `+ Container`. There are no lanes here to add to.
//! - **Inside** a container — and on **Arrange** — you are looking at a stack: `+ Lane`.
//!   Making another container from in here is not a thing anyone asked for; PLACING one is,
//!   and that is the source dropdown plus the lane's own `+`.
//!
//! With one button the crush cannot happen at all: it takes the strip whole. The
//! proportional split it replaced is gone rather than kept "just in case" — a mechanism with
//! no caller is a mechanism that rots.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Theme;

use crate::ids;
use crate::tab::Tab;

/// What the lane area's ADD button makes here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AddKind {
    /// A row on the stack in view.
    Lane,
    /// A container asset — only where containers are made.
    Container,
}

impl AddKind {
    /// `(id, i18n key)` — the ONE table the paint and the router read, so a button cannot be
    /// drawn under one name and dispatch another.
    pub(crate) fn button(self) -> (ph2d_a11y::NodeId, &'static str) {
        match self {
            Self::Lane => (ids::TIMELINE_ADD_LANE, "panel.timeline.add_lane"),
            Self::Container => (ids::TIMELINE_ADD_CONTAINER, "panel.timeline.add_container"),
        }
    }
}

/// **Which ADD this place offers** — the whole rule, as a pure function of the two facts the
/// panel already has.
///
/// `inside` is *"the open container exists"* (the snapshot published a breadcrumb for it),
/// not *"the tab is Containers"*: the Containers tab has both states, and they are the two
/// levels — the LIST of containers, and the inside of one.
pub(crate) fn add_kind(tab: Tab, inside: bool) -> Option<AddKind> {
    match tab {
        // The Keys tab adds TRACKS; `tracks::paint_add_track` owns that header.
        Tab::Keys => None,
        Tab::Containers if !inside => Some(AddKind::Container),
        _ => Some(AddKind::Lane),
    }
}

/// Paint the lane area's ADD button across the label column's header strip.
pub(crate) fn paint_add_lane(
    ctx: &mut PaintCtx,
    theme: Theme,
    header: Rect,
    tab: Tab,
    inside: bool,
) {
    let Some(kind) = add_kind(tab, inside) else {
        return;
    };
    let (id, key) = kind.button();
    let st = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    paint_button(
        &Button::new(id, ph2d_i18n::tr(key).to_string()).state(st),
        header,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    // Painted AND hit-registered together: the button that is not offered here is neither,
    // so it can never be a control that clicks and does nothing
    // ([[feedback_disabled_button_still_dispatches]]).
    ctx.host.hit_index_mut().register(id, header);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab::ALL;

    /// **Cada lugar oferece UM botão, e o botão diz onde você está.**
    ///
    /// A aba Containers tem DOIS estados e eles são os dois níveis: a LISTA de containers
    /// (onde se faz um) e o INTERIOR de um (onde se fazem lanes). Oferecer "+ Container"
    /// dentro de um container convidaria a fazer um irmão de onde você não pode colocá-lo —
    /// colocar é o dropdown de fonte mais o `+` da lane (Enio, 2026-07-21).
    #[test]
    fn each_place_offers_exactly_the_add_that_belongs_to_it() {
        assert_eq!(add_kind(Tab::Containers, false), Some(AddKind::Container));
        assert_eq!(add_kind(Tab::Containers, true), Some(AddKind::Lane));
        assert_eq!(add_kind(Tab::Arrange, false), Some(AddKind::Lane));
        assert_eq!(add_kind(Tab::Arrange, true), Some(AddKind::Lane));
        assert_eq!(add_kind(Tab::Keys, false), None, "a Keys adiciona TRACKS");
        assert_eq!(add_kind(Tab::Keys, true), None);
    }

    /// **"+ Container" existe em exatamente UM lugar**, e "+ Lane" nunca aparece ao lado dele.
    ///
    /// É a lei que apagou a divisão proporcional: com um botão por lugar, o esmagamento do
    /// rótulo longo (o report anterior do Enio) não é sequer expressável.
    #[test]
    fn the_two_adds_are_never_offered_together() {
        let places: Vec<Option<AddKind>> = ALL
            .into_iter()
            .flat_map(|t| [add_kind(t, false), add_kind(t, true)])
            .collect();
        let makers = places
            .iter()
            .filter(|k| **k == Some(AddKind::Container))
            .count();
        assert_eq!(makers, 1, "um só lugar faz containers, veio {places:?}");
        assert!(
            places
                .iter()
                .flatten()
                .all(|k| matches!(k, AddKind::Lane | AddKind::Container)),
            "e cada lugar oferece no máximo um"
        );
    }

    /// **O que o botão PINTA é o que ele DESPACHA** — rótulo e id saem da mesma tabela.
    #[test]
    fn the_button_paints_and_dispatches_from_one_table() {
        assert_eq!(AddKind::Lane.button().0, ids::TIMELINE_ADD_LANE);
        assert_eq!(AddKind::Container.button().0, ids::TIMELINE_ADD_CONTAINER);
        assert_ne!(AddKind::Lane.button().1, AddKind::Container.button().1);
    }
}
