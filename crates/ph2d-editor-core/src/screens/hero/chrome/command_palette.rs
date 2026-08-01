// ph2d-chrome-sync:z=185 (dispatch priority, ADR-0107; lower = earlier)
//! Command-palette chrome handler — the thin gate around the generic [`crate::widget::command_palette`].
//! Two halves, colocated (mirroring [`super::onion_modal`]):
//!
//! * [`paint_command_palette`] — full-screen render, gated on `store.command_palette_model()` being
//!   `Some`. Called from the hero paint pass, after the floating cards so it sits on top.
//! * [`apply`] — the dispatch half wired into `chrome::dispatch_all`: the close-X and a click on the
//!   dimmed **scrim** (outside the card) close the palette; a click on the card's dead space is consumed
//!   (no-op — it must not fall through the scrim to close); a click on an **item** records the pick
//!   (`store.set_command_pick`) and closes. The shell reads the pick back with `take_command_pick` and
//!   maps it to a real action — editor-core never learns what an item *means* (the colour-picker seam).

use crate::interaction::{HitIndex, WidgetEvent, WidgetStore};
use crate::screens::hero::HeroScreen;
use crate::widget::command_palette::{
    self, CMD_PALETTE_CARD, CMD_PALETTE_CLOSE, CMD_PALETTE_SCRIM,
};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Paint the full-screen command palette over `viewport`. No-op when closed (mirrors
/// [`super::onion_modal::paint_onion_modal`]'s open-gate).
pub fn paint_command_palette(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let Some(model) = store.command_palette_model() else {
        return;
    };
    command_palette::paint(scene, text_system, theme, hit_index, model, viewport);
}

/// Dispatch the command palette's widget events (wired into `chrome::dispatch_all`). Only acts while the
/// palette is open. FALSIFIED by not closing on the scrim (a click outside the card would fall through),
/// or by not recording the pick on an item click.
pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    if hero.store.command_palette_model().is_none() {
        return false;
    }
    match event {
        // The close-X, or a click on the dimmed area OUTSIDE the card, close the palette.
        WidgetEvent::Click(id) if id == CMD_PALETTE_CLOSE || id == CMD_PALETTE_SCRIM => {
            hero.store.close_command_palette();
            true
        }
        // A click on the card's dead space: consume so it never falls through the scrim to close.
        WidgetEvent::Click(id) if id == CMD_PALETTE_CARD => true,
        // A click on an item: record the pick (the shell routes it) and close.
        WidgetEvent::Click(id) => {
            let is_item = hero
                .store
                .command_palette_model()
                .is_some_and(|m| m.is_item(id));
            if is_item {
                hero.store.set_command_pick(id);
                hero.store.close_command_palette();
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    //! Gates for the command-palette chrome handler. Inline (not a `_tests.rs` sibling) because the
    //! chrome-sync scanner treats every `chrome/*.rs` as a handler.

    use super::*;
    use crate::widget::command_palette::{PaletteGroup, PaletteItem, PaletteModel, PaletteSub};
    use ph2d_a11y::NodeId;
    use ph2d_tokens::ColorToken;
    use ph2d_tool_registry::hash_node_id;

    fn model() -> PaletteModel {
        PaletteModel {
            title: "Add Node".into(),
            groups: vec![PaletteGroup {
                title: "Source".into(),
                color: ColorToken::NodeCatSource,
                subs: vec![PaletteSub {
                    title: None,
                    items: vec![
                        PaletteItem {
                            label: "Grid".into(),
                            id: hash_node_id("motion.grid"),
                        },
                        PaletteItem {
                            label: "Boids".into(),
                            id: hash_node_id("motion.boids"),
                        },
                    ],
                }],
            }],
        }
    }

    /// **A click on the scrim (outside the card) closes the palette; the card's dead space does NOT.**
    /// FALSIFIED by treating the card id like the scrim (a click inside would close), or by not closing
    /// on the scrim (a click outside would leak to the graph underneath).
    #[test]
    fn scrim_closes_but_card_dead_space_does_not() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store.open_command_palette(model());

        // A click on the card body is consumed but does NOT close.
        assert!(apply(&mut hero, WidgetEvent::Click(CMD_PALETTE_CARD)));
        assert!(
            hero.store.command_palette_open(),
            "a click on the card's dead space keeps the palette open"
        );

        // A click on the scrim (outside the card) closes.
        assert!(apply(&mut hero, WidgetEvent::Click(CMD_PALETTE_SCRIM)));
        assert!(
            !hero.store.command_palette_open(),
            "a click on the scrim closes the palette"
        );
    }

    /// **The close-X closes the palette.**
    #[test]
    fn close_x_closes() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store.open_command_palette(model());
        assert!(apply(&mut hero, WidgetEvent::Click(CMD_PALETTE_CLOSE)));
        assert!(!hero.store.command_palette_open());
    }

    /// **A click on an item records the pick and closes; the shell drains it exactly once.** FALSIFIED
    /// by not calling `set_command_pick` (the shell would never see the choice) or by leaving the palette
    /// open (the next click would pick again).
    #[test]
    fn item_click_records_the_pick_and_closes() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store.open_command_palette(model());
        let boids = hash_node_id("motion.boids");

        assert!(apply(&mut hero, WidgetEvent::Click(boids)));
        assert!(!hero.store.command_palette_open(), "picking closes");
        assert_eq!(
            hero.store.take_command_pick(),
            Some(boids),
            "the pick is the item the user clicked"
        );
        assert_eq!(
            hero.store.take_command_pick(),
            None,
            "the pick is drained exactly once"
        );
    }

    /// **Events are ignored while the palette is closed** (so it never steals a click that isn't its own).
    #[test]
    fn ignores_events_when_closed() {
        let mut hero = HeroScreen::new(NodeId(1));
        assert!(!apply(&mut hero, WidgetEvent::Click(CMD_PALETTE_SCRIM)));
        assert!(!apply(
            &mut hero,
            WidgetEvent::Click(hash_node_id("motion.grid"))
        ));
    }
}
