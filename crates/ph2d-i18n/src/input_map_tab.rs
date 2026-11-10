//! **A ABA Input Map** (`docs/InputMap/01_plano.md` §7): o topo, o comum e o diálogo. As vistas e a tela do
//! jogador têm cada uma o seu ficheiro (um dono por ficheiro); o `lib.rs` (tecto de 700 linhas) conhece só esta.

#[path = "input_map_keyboard.rs"]
mod keyboard;
#[path = "input_map_list.rs"]
mod list;
#[path = "player_controls.rs"]
mod player_controls;
#[path = "input_map_try.rs"]
mod try_it;

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    if let Some(t) = list::tr(key)
        .or_else(|| keyboard::tr(key))
        .or_else(|| try_it::tr(key))
        .or_else(|| player_controls::tr(key))
    {
        return Some(t);
    }
    Some(match key {
        "input_map_tab.title" => "Input Map",
        "input_map_tab.view.list" => "List",
        "input_map_tab.reader.platform_player" => "Platform Player",
        "input_map_tab.reader.top_down_player" => "Top-Down Player",
        "input_map_tab.view.keyboard" => "Keyboard",
        "input_map_tab.view.try" => "Try it",
        "input_map_tab.view.list_tip" => "See every action as a list, with its keys and buttons.",
        "input_map_tab.view.keyboard_tip" => {
            "See which keys and gamepad buttons each action uses, on a drawn keyboard and gamepad."
        }
        "input_map_tab.view.try_tip" => {
            "Press keys or move the sticks and watch which actions react, without starting the game."
        }
        _ => return None,
    })
}
