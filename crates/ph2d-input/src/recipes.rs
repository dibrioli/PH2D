//! **AS RECEITAS DE COMEÇO** — «Start from…» (plano InputMap §2).
//!
//! Teclado = as teclas de fábrica ([`crate::InputMap::with_player_defaults`]); comando no idioma do
//! ecossistema; zona morta 0.2 só onde há manche (Godot e GDevelop usam 0.2).

use crate::action::Binding;
use crate::gamepad::{GamepadAxis, GamepadButton};
use crate::keyboard::Key;
use crate::map::{
    PLAYER_DASH, PLAYER_DOWN, PLAYER_GRAB, PLAYER_JUMP, PLAYER_MOVE_DOWN, PLAYER_MOVE_LEFT,
    PLAYER_MOVE_RIGHT, PLAYER_MOVE_UP,
};

/// O nome da acção de tiro do Shooter (ninguém a lê de fábrica: é do jogo do autor).
pub const RECIPE_FIRE: &str = "fire";
/// O nome da acção de recarregar do Shooter.
pub const RECIPE_RELOAD: &str = "reload";

/// A zona morta das acções com manche.
const STICK_DEAD_ZONE: f32 = 0.2;

/// Uma receita de começo.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Recipe {
    Platformer,
    TopDown,
    Shooter,
    Blank,
}

/// Uma acção de receita, ainda sem id (o mapa dá-o).
#[derive(Clone, Debug, PartialEq)]
pub struct RecipeAction {
    pub name: &'static str,
    pub bindings: Vec<Binding>,
    pub dead_zone: f32,
}

impl Recipe {
    /// Todas, pela ordem dos cartões.
    pub const ALL: [Self; 4] = [Self::Platformer, Self::TopDown, Self::Shooter, Self::Blank];

    /// A chave i18n do título do cartão.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Platformer => "input_map_try.recipe.platformer",
            Self::TopDown => "input_map_try.recipe.top_down",
            Self::Shooter => "input_map_try.recipe.shooter",
            Self::Blank => "input_map_try.recipe.blank",
        }
    }

    /// A chave i18n da frase do cartão.
    #[must_use]
    pub const fn blurb_key(self) -> &'static str {
        match self {
            Self::Platformer => "input_map_try.recipe.platformer.blurb",
            Self::TopDown => "input_map_try.recipe.top_down.blurb",
            Self::Shooter => "input_map_try.recipe.shooter.blurb",
            Self::Blank => "input_map_try.recipe.blank.blurb",
        }
    }

    /// As acções da receita, pela ordem do painel.
    #[must_use]
    pub fn actions(self) -> Vec<RecipeAction> {
        match self {
            Self::Platformer => vec![
                walk_left(),
                walk_right(),
                jump(),
                stick(
                    PLAYER_DOWN,
                    &[DOWN, Key(0x53)],
                    GamepadButton::DPadDown,
                    GamepadAxis::LeftStickY,
                    false,
                ),
                plain(
                    PLAYER_DASH,
                    &[Key(0x51)],
                    &[GamepadButton::West],
                ),
                plain(
                    PLAYER_GRAB,
                    &[Key(0x52)],
                    &[GamepadButton::East],
                ),
            ],
            Self::TopDown => vec![
                walk_left(),
                walk_right(),
                stick(
                    PLAYER_MOVE_UP,
                    &[UP, Key(0x57)],
                    GamepadButton::DPadUp,
                    GamepadAxis::LeftStickY,
                    true,
                ),
                stick(
                    PLAYER_MOVE_DOWN,
                    &[DOWN, Key(0x53)],
                    GamepadButton::DPadDown,
                    GamepadAxis::LeftStickY,
                    false,
                ),
            ],
            Self::Shooter => vec![
                walk_left(),
                walk_right(),
                jump(),
                plain(
                    RECIPE_FIRE,
                    &[Key(0x4A)],
                    &[GamepadButton::West, GamepadButton::RightTrigger],
                ),
                plain(
                    RECIPE_RELOAD,
                    &[Key(0x52)],
                    &[GamepadButton::North],
                ),
            ],
            Self::Blank => Vec::new(),
        }
    }
}

const LEFT: Key = Key(0xF702);
const RIGHT: Key = Key(0xF703);
const UP: Key = Key(0xF700);
const DOWN: Key = Key(0xF701);

fn walk_left() -> RecipeAction {
    stick(
        PLAYER_MOVE_LEFT,
        &[LEFT, Key(0x41)],
        GamepadButton::DPadLeft,
        GamepadAxis::LeftStickX,
        false,
    )
}

fn walk_right() -> RecipeAction {
    stick(
        PLAYER_MOVE_RIGHT,
        &[RIGHT, Key(0x44)],
        GamepadButton::DPadRight,
        GamepadAxis::LeftStickX,
        true,
    )
}

fn jump() -> RecipeAction {
    plain(PLAYER_JUMP, &[UP, Key(0x5A)], &[GamepadButton::South])
}

/// Teclas + meia manche + a cruz, com a zona morta do manche.
fn stick(
    name: &'static str,
    keys: &[Key],
    dpad: GamepadButton,
    axis: GamepadAxis,
    positive: bool,
) -> RecipeAction {
    let mut bindings: Vec<Binding> = keys.iter().copied().map(Binding::Key).collect();
    bindings.push(Binding::PadAxis { axis, positive });
    bindings.push(Binding::PadButton(dpad));
    RecipeAction {
        name,
        bindings,
        dead_zone: STICK_DEAD_ZONE,
    }
}

/// Teclas + botões, sem zona morta.
fn plain(name: &'static str, keys: &[Key], pad: &[GamepadButton]) -> RecipeAction {
    let bindings = keys
        .iter()
        .copied()
        .map(Binding::Key)
        .chain(pad.iter().copied().map(Binding::PadButton))
        .collect();
    RecipeAction {
        name,
        bindings,
        dead_zone: 0.0,
    }
}

#[cfg(test)]
#[path = "recipes_tests.rs"]
mod tests;
