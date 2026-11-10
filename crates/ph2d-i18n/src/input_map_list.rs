//! **A · List** da aba Input Map (`docs/InputMap/01_plano.md` §7): a lista, a busca e o cartão da acção.

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "input_map_list.new_name" => "New action name",
        "input_map_list.new_name_tip" => "Type a name for a new action, then press Add or Enter.",
        "input_map_list.add" => "Add",
        "input_map_list.add_tip" => "Create an action with the name you typed.",
        "input_map_list.empty_name" => "Type a name first: an action needs a name for the game to read it.",
        "input_map_list.duplicate" => "There is already an action with this name. Pick another name.",
        "input_map_list.search" => "Find an action, or press a key",
        "input_map_list.search_tip" => {
            "Type part of a name, or click here and press a key to see which actions use that key."
        }
        "input_map_list.listen_pad" => "Press a key or a gamepad button. Esc cancels.",
        "input_map_list.listen_key" => "Press a key. Esc cancels.",
        "input_map_list.listening" => "Listening for {action}.",
        "input_map_list.listening_rest" => "The next key goes to this row.",
        "input_map_list.add_key_tip" => "Add a key or a gamepad button: click here, then press it.",
        "input_map_list.remove_key_tip" => "Remove this key from the action.",
        "input_map_list.remove_action_tip" => "Delete this action.",
        "input_map_list.dead" => "Dead zone (0 to 1)",
        "input_map_list.dead_tip" => {
            "How far the stick or trigger must move before it counts at all. Drag to change it."
        }
        "input_map_list.press" => "Press at (0 to 1)",
        "input_map_list.press_tip" => {
            "How far the stick or trigger must move to count as pressed. Drag to change it."
        }
        "input_map_list.summary" => "{n} actions · {k} keys.",
        "input_map_list.shared_one" => {
            "{keys} is in two actions: a Platform Player and a Top-Down Player never run on the same object."
        }
        "input_map_list.shared_many" => {
            "{keys} are in two actions: a Platform Player and a Top-Down Player never run on the same object."
        }
        "input_map_list.shared_warn_lead" => "{key} is in {actions}.",
        "input_map_list.shared_warn_rest" => "Both react when it is pressed. Check that this is what you want.",
        "input_map_list.search_count" => "{key} is in {n} of {total} actions.",
        "input_map_list.read_by_first" => "{action} is read by {who}",
        "input_map_list.read_by_next" => "{action} by {who}",
        "input_map_list.comma" => ", ",
        "input_map_list.and" => " and ",
        "input_map_list.period" => ".",
        _ => return None,
    })
}
