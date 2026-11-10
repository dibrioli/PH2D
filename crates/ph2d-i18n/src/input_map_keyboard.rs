//! **B · Keyboard** da aba Input Map (`docs/InputMap/01_plano.md` §7): o teclado, o comando e a barra do curso.

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "input_map_keyboard.actions" => "Actions",
        "input_map_keyboard.yours" => "Yours",
        "input_map_keyboard.none" => "none",
        "input_map_keyboard.add" => "+ Add",
        "input_map_keyboard.add_tip" => "Make a new action: type its name, then press Enter.",
        "input_map_keyboard.remove" => "Remove",
        "input_map_keyboard.remove_tip" => "Delete the open action and all its keys.",
        "input_map_keyboard.remove_off_none" => "Open an action first, then Remove deletes it.",
        "input_map_keyboard.scrollbar" => "Actions scroll bar",
        "input_map_keyboard.scrollbar_tip" => "Drag to scroll the actions.",
        "input_map_keyboard.tile_x" => "Remove {key}",
        "input_map_keyboard.listen_label" => "Press a key to add it",
        "input_map_keyboard.bar_dead" => "Dead zone handle",
        "input_map_keyboard.bar_press" => "Press point handle",
        "input_map_keyboard.row_tip" => "Open this action to see and change its keys.",
        "input_map_keyboard.sub.keyboard" => "Keyboard",
        "input_map_keyboard.sub.keyboard_tip" => "Show a keyboard. Click a key to add it.",
        "input_map_keyboard.sub.gamepad" => "Gamepad",
        "input_map_keyboard.sub.gamepad_tip" => "Show a gamepad. Click a button to add it.",
        "input_map_keyboard.caption.keyboard" => {
            "Keyboard · click a key to add it to the open action"
        }
        "input_map_keyboard.caption.gamepad" => "Gamepad · click a button or a stick to add it",
        "input_map_keyboard.key_tip" => "Click to add this key to the open action.",
        "input_map_keyboard.pad_tip" => {
            "Click to add this to the open action. Around a stick, each arrow is one direction."
        }
        "input_map_keyboard.also" => "Also:",
        "input_map_keyboard.unused" => "no action uses it.",
        "input_map_keyboard.user" => "{action} ({who})",
        "input_map_keyboard.nobody" => "nobody reads it",
        "input_map_keyboard.sep" => " · ",
        "input_map_keyboard.colon" => ":",
        "input_map_keyboard.title.keys_one" => "{action} · 1 key",
        "input_map_keyboard.title.keys_many" => "{action} · {n} keys",
        "input_map_keyboard.title.inputs_many" => "{action} · {n} inputs",
        "input_map_keyboard.title.none" => "{action} · no keys",
        "input_map_keyboard.no_action" => "Pick an action on the left, or make one with + Add.",
        "input_map_keyboard.read_by" => "Read by {who}",
        "input_map_keyboard.read_by_nobody" => "Nobody reads it yet",
        "input_map_keyboard.device.keyboard" => "Keyboard",
        "input_map_keyboard.device.gamepad" => "Gamepad",
        "input_map_keyboard.tile_x_tip" => "Remove this from the action.",
        "input_map_keyboard.press_key" => "Press a key",
        "input_map_keyboard.press_button" => "Press a button",
        "input_map_keyboard.press_first" => "Click a key above, or press one",
        "input_map_keyboard.listen_tip" => {
            "Click, then press the key or gamepad button you want. Esc cancels."
        }
        "input_map_keyboard.nobody_reads" => {
            "Nobody reads {action} yet. A Trigger can turn it into a signal: type {action} in its Action."
        }
        "input_map_keyboard.shared_fine" => {
            "{key} is also in {others}. {who} reads that one; one object has one mover, so they never fire together."
        }
        "input_map_keyboard.shared_warn" => {
            "{key} is also in {others}. Both can fire on the same press: give one of them another key."
        }
        "input_map_keyboard.travel" => "The stick, from rest to full",
        "input_map_keyboard.travel_dead" => "Dead {v}",
        "input_map_keyboard.travel_press" => "Press {v}",
        "input_map_keyboard.zero" => "0",
        "input_map_keyboard.one" => "1",
        "input_map_keyboard.bar_dead_tip" => {
            "Drag to change how far the stick must move before it counts at all."
        }
        "input_map_keyboard.bar_press_tip" => {
            "Drag to change how far the stick must move to count as pressed."
        }
        "input_map_keyboard.pad.lt" => "LT",
        "input_map_keyboard.pad.rt" => "RT",
        "input_map_keyboard.pad.lb" => "LB",
        "input_map_keyboard.pad.rb" => "RB",
        "input_map_keyboard.pad.ls" => "LS",
        "input_map_keyboard.pad.rs" => "RS",
        "input_map_keyboard.pad.a" => "A",
        "input_map_keyboard.pad.b" => "B",
        "input_map_keyboard.pad.x" => "X",
        "input_map_keyboard.pad.y" => "Y",
        "input_map_keyboard.pad.back" => "Back",
        "input_map_keyboard.pad.start" => "Start",
        "input_map_keyboard.pad.dpad" => "D-pad",
        "input_map_keyboard.system.fn" => "Fn is handled inside the keyboard; games never see it.",
        "input_map_keyboard.system.windows" => {
            "The Windows key belongs to the system: it opens the app menu. Games never get it."
        }
        "input_map_keyboard.system.print_screen" => {
            "Print Screen belongs to the system: it takes a screenshot. Games never get it."
        }
        _ => return None,
    })
}
