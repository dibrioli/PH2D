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
        "panel.inspector.motion_input.graph_reads" => "Graph Reads",
        "panel.inspector.motion_input.name_hint" => "input name\u{2026}",
        "panel.inspector.motion_input.empty_name" => {
            "No name \u{2014} the graph reads nothing from this object."
        }
        "panel.inspector.motion_input.graph_has_no_input" => {
            "The Motion graph has no Input node yet \u{2014} add one with this name."
        }
        "panel.inspector.motion_input.name_not_read" => {
            "No Input node in the Motion graph has this name \u{2014} pick one below."
        }
        "panel.inspector.motion_input.no_health" => {
            "This object has no Health \u{2014} the graph reads nothing until you add it."
        }
        "panel.inspector.motion_input.more_names" => {
            "{n} more in the graph \u{2014} type the name you need."
        }
        _ => return None,
    })
}
