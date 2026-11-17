//! **As palavras da secção MOTION INPUT do Inspector** (ciclo 19 do Motion).
//!
//! Um ficheiro próprio pela razão do [`super::inspector_nav`]: o `inspector_game` está perto do
//! tecto de 700 linhas. Os rótulos das linhas são os do descritor (`component.field.*`).

/// A tradução de uma chave `panel.inspector.motion_input.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ⚠️ A ordem é a de `MotionSource::ALL` (o índice que o painel envia).
        "panel.inspector.motion_input.source.0" => "Value",
        "panel.inspector.motion_input.source.1" => "Color",
        "panel.inspector.motion_input.source.2" => "Health",
        "panel.inspector.motion_input.source.3" => "Damage",
        "panel.inspector.motion_input.source.4" => "Speed",
        "panel.inspector.motion_input.source.5" => "Grounded",
        "panel.inspector.motion_input.source.6" => "Action",
        "panel.inspector.motion_input.source.7" => "Property",
        "panel.inspector.motion_input.source.8" => "Velocity",
        "panel.inspector.motion_input.source.9" => "Move",
        "panel.inspector.motion_input.new" => "New\u{2026}",
        "panel.inspector.motion_input.name_label" => "Name",
        "panel.inspector.motion_input.add_health" => "Add Health",
        "panel.inspector.motion_input.reads_health" => "Reads Health here: {v}",
        "panel.inspector.motion_input.reads_property" => "Reads {c} here: {v}",
        "panel.inspector.motion_input.sends" => "Sends {v}",
        "panel.inspector.motion_input.a_color" => "this color",
        "panel.inspector.motion_input.fired" => "Fired {n} times",
        "panel.inspector.motion_input.fired_once" => "Fired once",
        // O cartão do objecto Motion (MI-8): um nó «Send signal» por linha, nesta corrida.
        "panel.inspector.motion.sent" => "Sent {s} {n} times",
        "panel.inspector.motion.sent_once" => "Sent {s} once",
        "panel.inspector.motion.not_sent" => "{s} not sent yet",
        "panel.inspector.motion_input.not_fired" => {
            "Not fired yet \u{2014} it fires each time Health drops."
        }
        "panel.inspector.motion_input.covered_by_script" => {
            "A script sets this name first ({v}) \u{2014} the graph reads that."
        }
        "panel.inspector.motion_input.covered_by_line" => {
            "The command line sets this name first ({v}) \u{2014} the graph reads that."
        }
        "panel.inspector.motion_input.covered_by_object" => {
            "The Motion Input on {o} sets this name first ({v}) \u{2014} the graph reads that."
        }
        "panel.inspector.motion_input.name_hint" => "input name\u{2026}",
        "panel.inspector.motion_input.empty_name" => {
            "No name \u{2014} the graph reads nothing from this object."
        }
        "panel.inspector.motion_input.graph_has_no_input" => {
            "The Motion graph has no Input node yet \u{2014} add one with this name."
        }
        "panel.inspector.motion_input.name_not_read" => {
            "No Input node in the Motion graph has this name \u{2014} pick one above."
        }
        "panel.inspector.motion_input.no_health" => {
            "{o} has no Health \u{2014} the graph reads nothing until you add it."
        }
        "panel.inspector.motion_input.no_from" => {
            "No object has the name in From \u{2014} pick another one, or clear it to read {o}."
        }
        "panel.inspector.motion_input.no_body" => {
            "{o} has no body to read \u{2014} Speed and Velocity need a Rigid Body, Grounded a Platform Player."
        }
        "panel.inspector.motion_input.unknown_action" => {
            "The Input Map has no action with this name \u{2014} pick one from the list."
        }
        "panel.inspector.motion_input.no_property" => {
            "No property chosen \u{2014} pick a component and a field to read."
        }
        "panel.inspector.motion_input.property_gone" => {
            "This object does not have that field \u{2014} pick one it has."
        }
        // GEF 08 onda 2: From, as listas, o grupo «Shape» e as saídas das queixas.
        "panel.inspector.motion_input.from_this" => "{o} (parent)",
        "panel.inspector.motion_input.the_parent" => "the parent",
        "panel.inspector.motion_input.parent" => "Parent",
        "panel.inspector.motion_input.from_tip" => {
            "The object this input reads. Empty reads the parent. Type a name or use the eyedropper."
        }
        "panel.inspector.motion_input.no_move" => {
            "The Input Map has no move_left, move_right, move_up or move_down action \u{2014} add them in the Input Map."
        }
        "panel.inspector.motion_input.reads_move" => "Reads {a} from the Input Map",
        "panel.inspector.motion_input.reads_pair" => "Reads {c} here: {v}",
        "panel.inspector.motion_input.pair_field" => "{c}  (x, y)",
        "panel.inspector.motion_input.pair" => "({x}, {y})",
        "panel.inspector.motion_input.remap_summary" => "{a}\u{2013}{b} \u{2192} 0\u{2013}1",
        "panel.inspector.motion_input.from_pick_tip" => {
            "Click an object on the stage or in the Hierarchy to read it."
        }
        "panel.inspector.motion_input.action_pick" => "Pick an action",
        "panel.inspector.motion_input.action_tip" => {
            "The Input Map action this sends: 0 when let go, up to 1 when fully pressed (a stick sends the part in between)."
        }
        "panel.inspector.motion_input.property_pick" => "Pick a field",
        "panel.inspector.motion_input.property_tip" => {
            "The field this reads from the From object, every frame: a number, or a pair like Position."
        }
        "panel.inspector.motion_input.shape" => "Shape",
        "panel.inspector.motion_input.shape_tip" => {
            "Turn the number into 0 to 1, bend it with a curve, and smooth it over time."
        }
        "panel.inspector.motion_input.shape_off" => "Off",
        "panel.inspector.motion_input.remap_tip" => {
            "Map Remap from to 0 and Remap to to 1. Numbers outside stay at 0 or 1."
        }
        "panel.inspector.motion_input.remap_range_tip" => {
            "The number that becomes 0, and the one that becomes 1."
        }
        "panel.inspector.motion_input.curve_tip" => "How the 0 to 1 number bends on its way.",
        "panel.inspector.motion_input.curve_linear" => "Linear",
        "panel.inspector.motion_input.curve_ease_in" => "Ease In",
        "panel.inspector.motion_input.curve_ease_out" => "Ease Out",
        "panel.inspector.motion_input.curve_smooth" => "Smooth",
        "panel.inspector.motion_input.curve_custom" => "Custom curve",
        "panel.inspector.motion_input.smooth_tip" => {
            "Seconds to catch up halfway with a new number. 0 is off: it jumps at once."
        }
        "panel.inspector.motion_input.smooth_s" => "Smooth {s} s",
        "panel.inspector.motion_input.add_body" => "Add Physics Body",
        "panel.inspector.motion_input.pick_object" => "Pick object",
        "panel.inspector.motion_input.open_input_map" => "Open Input Map",
        "panel.inspector.motion_input.more_names" => {
            "{n} more in the graph \u{2014} type the name you need."
        }
        _ => return None,
    })
}
