//! **A TELA DE CONTROLES DO JOGADOR** — o que o autor decide sobre ela (plano InputMap §2).

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::action::ActionId;
use crate::keyboard::{F1, Key};

/// O que o jogador vê e pode mudar no Play.
///
/// ⚠️ Guarda-se o ESCONDIDO, não o mostrado: uma acção nova entra visível sem segunda escrita.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerControls {
    /// O jogador pode trocar as teclas na tela de controles.
    pub players_can_change_keys: bool,
    /// As acções que a tela do jogador NÃO mostra.
    pub hidden_in_game: BTreeSet<ActionId>,
    /// A tecla que abre a tela no Play.
    pub open_key: Key,
}

impl Default for PlayerControls {
    fn default() -> Self {
        Self {
            players_can_change_keys: false,
            hidden_in_game: BTreeSet::new(),
            open_key: Key(F1),
        }
    }
}

impl PlayerControls {
    /// A acção aparece na tela do jogador («Shown in the game»).
    #[must_use]
    pub fn shown_in_game(&self, id: ActionId) -> bool {
        !self.hidden_in_game.contains(&id)
    }

    /// Mostra ou esconde a acção na tela do jogador.
    pub fn set_shown_in_game(&mut self, id: ActionId, shown: bool) {
        if shown {
            self.hidden_in_game.remove(&id);
        } else {
            self.hidden_in_game.insert(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_player_screen_starts_closed_to_changes_shows_all_and_opens_on_f1() {
        let c = PlayerControls::default();
        assert!(!c.players_can_change_keys);
        assert!(c.hidden_in_game.is_empty());
        assert_eq!(c.open_key.label(), "F1");
    }

    #[test]
    fn shown_is_the_negation_of_hidden() {
        let mut c = PlayerControls::default();
        let id = ActionId(3);
        assert!(c.shown_in_game(id), "uma acção nova entra visível");
        c.set_shown_in_game(id, false);
        assert!(!c.shown_in_game(id));
        assert_eq!(c.hidden_in_game.len(), 1);
        c.set_shown_in_game(id, true);
        assert!(c.hidden_in_game.is_empty());
    }

    #[test]
    fn removing_an_action_forgets_that_it_was_hidden() {
        let mut m = crate::InputMap::with_player_defaults();
        let jump = m.id("jump").expect("fábrica");
        let dash = m.id("dash").expect("fábrica");
        m.controls_mut().set_shown_in_game(jump, false);
        m.controls_mut().set_shown_in_game(dash, false);
        m.remove(jump);
        assert_eq!(
            m.controls().hidden_in_game.iter().copied().collect::<Vec<_>>(),
            [dash]
        );
    }
}
