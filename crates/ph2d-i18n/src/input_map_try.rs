//! **C · Try it** da aba Input Map (`docs/InputMap/01_plano.md` §7): as barras ao vivo, as receitas e o círculo.

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "input_map_try.start_from" => "Start from",
        "input_map_try.recipe.platformer" => "Platformer",
        "input_map_try.recipe.platformer.blurb" => "Run, jump, dash and grab, seen from the side.",
        "input_map_try.recipe.top_down" => "Top-down",
        "input_map_try.recipe.top_down.blurb" => "Walk in four directions, seen from above.",
        "input_map_try.recipe.shooter" => "Shooter",
        "input_map_try.recipe.shooter.blurb" => "Run and jump, fire and reload.",
        "input_map_try.recipe.blank" => "Blank",
        "input_map_try.recipe.blank.blurb" => "No actions. You add them.",
        "input_map_try.recipe_tip" => "Replace your actions with this set. Undo brings yours back.",
        "input_map_try.recipe_caption" => {
            "A recipe makes the actions and their keys. Platform Player and Top-Down Player read them by name, so they work at once."
        }
        "input_map_try.confirm_one" => {
            "Replace your {n} action with {recipe}? Undo brings it back."
        }
        "input_map_try.confirm_many" => {
            "Replace your {n} actions with {recipe}? Undo brings them back."
        }
        "input_map_try.replace" => "Replace",
        "input_map_try.replace_tip" => "Swap your actions for the recipe. Undo brings them back.",
        "input_map_try.cancel" => "Cancel",
        "input_map_try.cancel_tip" => "Keep your actions as they are.",
        "input_map_try.try_it" => "Try it",
        "input_map_try.live" => "\u{25CF} LIVE",
        "input_map_try.stopped" => "The game is not running",
        "input_map_try.running" => "The game is running",
        "input_map_try.idle" => {
            "Press keys on your keyboard to see which actions turn on. Nothing moves, because nothing runs."
        }
        "input_map_try.idle_running" => {
            "The game is running: this shows what it reads now. Your keys go to the game."
        }
        "input_map_try.pressing" => "You are pressing {keys}:",
        "input_map_try.on_one" => "{actions} is on. Nothing moves, because nothing runs.",
        "input_map_try.on_many" => "{actions} are on. Nothing moves, because nothing runs.",
        "input_map_try.on_none" => "no action uses it. Nothing moves, because nothing runs.",
        "input_map_try.and" => " and ",
        "input_map_try.list_sep" => ", ",
        "input_map_try.stick" => "The stick",
        "input_map_try.no_stick" => {
            "No action uses a stick yet. Open the Keyboard view, pick an action, switch to Gamepad and click a stick arrow."
        }
        "input_map_try.stick_of" => "{action} \u{00B7} {stick}",
        "input_map_try.left_stick" => "Left Stick",
        "input_map_try.right_stick" => "Right Stick",
        "input_map_try.dead_mark" => "D",
        "input_map_try.press_mark" => "P",
        "input_map_try.pushed" => "Pushed",
        "input_map_try.strength" => "Strength",
        "input_map_try.pressed" => "Pressed",
        "input_map_try.not_pressed" => "Not pressed",
        "input_map_try.stick_caption" => {
            "Inner disc: too small to count. Dashed ring: where it counts as pressed. Move the stick and watch the dot."
        }
        "input_map_try.controls" => "Controls",
        "input_map_try.players_can_change" => "Players can change keys",
        "input_map_try.players_can_change_tip" => {
            "Let people who play your game open a Controls screen and pick their own keys."
        }
        "input_map_try.opens_with" => "Opens with",
        "input_map_try.opens_with_tip" => {
            "Click, then press the key that opens the Controls screen in the game."
        }
        "input_map_try.press_open_key" => "Press a key\u{2026}",
        "input_map_try.refused_f5" => "F5 cannot open it: F5 starts and stops the game.",
        "input_map_try.shown" => "Shown in the game",
        "input_map_try.shown_tip" => {
            "Show this action on the Controls screen, so players can change its keys."
        }
        "input_map_try.changed_one" => "On this computer, a player changed {n} key.",
        "input_map_try.changed_many" => "On this computer, a player changed {n} keys.",
        "input_map_try.forget" => "Forget them",
        "input_map_try.forget_tip" => {
            "Delete the keys a player picked on this computer. The game's keys stay."
        }
        "input_map_try.forget_off" => {
            "Nothing to forget: no player changed a key on this computer."
        }
        "input_map_try.start_from_named" => "Start from {recipe}",
        "input_map_try.opens_with_named" => "Opens with {key}",
        "input_map_try.scrollbar" => "Scroll the Try it view",
        "input_map_try.bar_button_name" => "Controls",
        "input_map_try.bar_button" => "Show or hide the Controls screen. Key: {key}",
        "input_map_try.bar_button_off" => {
            "Press Play first. Then this shows the Controls screen. Key: {key}"
        }
        "input_map_try.bar_button_locked" => {
            "Players can't change keys in this game. To allow it: Input Map \u{25b8} Try it \u{25b8} \
             Players can change keys."
        }
        _ => return None,
    })
}
