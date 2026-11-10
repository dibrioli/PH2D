//! **A tela «Controls» do jogador** no Play (`docs/InputMap/01_plano.md` §4 e §7).

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "player_controls.title" => "Controls",
        "player_controls.listening" => "Press a key",
        "player_controls.add" => "Add",
        "player_controls.reset" => "Reset to defaults",
        "player_controls.hint_choose" => "Up and Down choose. Enter changes the key. Esc closes.",
        "player_controls.hint_listening" => "Press a key. Esc cancels.",
        "player_controls.none" => "This game has no controls to change.",
        _ => return None,
    })
}
