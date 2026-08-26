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
    motion: &crate::motion::UiMotion,
) {
    let Some(model) = store.command_palette_model() else {
        return;
    };
    command_palette::paint(
        scene,
        text_system,
        theme,
        hit_index,
        model,
        store.command_palette_query(),
        viewport,
        motion,
    );
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
        // ⭐ **A caixa da banda** (ADR-0166 / F3): vira o estado e AVISA quem abriu — a paleta
        // fica aberta, e é quem a abriu que reconstrói o modelo (só ele sabe o que a caixa quer
        // dizer). Fechar aqui seria um controlo que sai do ecrã ao ser usado.
        WidgetEvent::Click(id) if id == command_palette::CMD_PALETTE_SHOW_ALL => {
            hero.store.flip_command_palette_toggle();
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
            toggle: None,
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

    /// **The search text is typed, backspaced, ignores control chars, and clears on open/close.** The
    /// shell feeds these ops while the palette is modal; a fresh palette must never inherit a stale query.
    /// FALSIFIED by not clearing on open (the last search would linger) or by pushing control chars.
    #[test]
    fn the_search_query_is_typed_backspaced_and_cleared_on_open() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store.open_command_palette(model());
        assert_eq!(
            hero.store.command_palette_query(),
            "",
            "a fresh palette starts with an empty search"
        );

        hero.store.command_palette_push_char('b');
        hero.store.command_palette_push_char('o');
        hero.store.command_palette_push_char('\n'); // control char: ignored
        assert_eq!(hero.store.command_palette_query(), "bo");
        hero.store.command_palette_backspace();
        assert_eq!(hero.store.command_palette_query(), "b");

        // Re-open clears the query (no stale search leaks into a new palette).
        hero.store.open_command_palette(model());
        assert_eq!(hero.store.command_palette_query(), "");

        // Closed store: typing is a no-op (nothing to search into).
        hero.store.close_command_palette();
        hero.store.command_palette_push_char('z');
        assert_eq!(hero.store.command_palette_query(), "");
    }

    /// ⭐ **O QUADRO da abertura alveja ZERO — e sem este gate a ordem no tique pode inverter-se
    /// em silêncio.**
    ///
    /// O `every_card_is_targeted_at_zero_on_the_frame_the_palette_opens` prova a LEI (a função
    /// pura); este prova o **CHAMADOR**. Somando `dt` antes de alvejar, o quadro da abertura já
    /// traz `secs = dt`, o cartão 0 nasce alvejado em `1.0`, e pela lei do substrato a primeira
    /// vista CHEGA — ele aparece assente e a cascata começa no segundo cartão. A sonda apanhou-o
    /// (media 0,02 s de entrada, um quadro), e a suíte inteira estava VERDE sobre isso.
    ///
    /// *Mutação: `hero.palette_open_secs += dt;` antes do `let secs` ⇒ sangra.*
    #[test]
    fn the_first_frame_of_the_cascade_targets_zero() {
        let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
        hero.store.open_command_palette(model());
        hero.tick_motion(1.0 / 60.0);
        let t0 = hero
            .motion
            .get(crate::widget::command_palette::cascade_id(0))
            .expect("o cartao 0 tem track no primeiro tique");
        assert!(
            t0.abs() < 0.001,
            "o cartao 0 nasceu em {t0} — ele nao ENTRA, ele aparece"
        );
    }

    /// ⭐ **A caixa da banda vira o estado e a paleta FICA ABERTA** (ADR-0166 / F3).
    ///
    /// ⚠️ **As duas metades importam.** Fechar aqui seria um controlo que sai do ecrã ao ser usado
    /// — o artista liga *Show all* e a paleta desaparece. E o sinal tem de chegar a quem abriu: o
    /// widget não sabe o que «mostrar tudo» quer dizer, exactamente como não sabe o que um item
    /// significa.
    #[test]
    fn the_band_toggle_flips_and_keeps_the_palette_open() {
        let mut hero = HeroScreen::new(NodeId(1));
        let mut m = model();
        m.toggle = Some(crate::widget::command_palette::PaletteToggle {
            label: "Show all".into(),
            on: false,
        });
        hero.store.open_command_palette(m);
        assert!(apply(
            &mut hero,
            WidgetEvent::Click(crate::widget::command_palette::CMD_PALETTE_SHOW_ALL)
        ));
        assert!(
            hero.store.command_palette_open(),
            "a caixa fechou a paleta — o controlo sai do ecra ao ser usado"
        );
        assert!(hero.store.command_palette_toggle_on(), "a caixa nao virou");
        assert!(
            hero.store.take_command_palette_toggled(),
            "o sinal nao chegou a quem abriu a paleta"
        );
        assert!(
            !hero.store.take_command_palette_toggled(),
            "o sinal tem de ser consumido UMA vez — senao o modelo reconstroi-se todo o quadro"
        );
    }
}
