//! **As palavras das MACROS** (ADR-0186, W5d): o painel MACROS (Gravar ▸ Parar, as etapas, as
//! propriedades, as recusas e os erros que chegam ao artista) e o nome do clipe que o Parar cria.

/// A tradução de uma chave `panel.macros.*` / `app.macros.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "app.macros.clip_name" => "Macro",
        "panel.macros.title" => "Macros",
        "panel.macros.record" => "\u{25cf} Record",
        "panel.macros.stop" => "\u{25a0} Stop",
        "panel.macros.family.painter" => "the Painter",
        "panel.macros.family.motion" => "the Motion graph",
        "panel.macros.ready" => "Record captures {family}.",
        "panel.macros.recording" => "Recording {family} \u{2014} Stop keeps it on the object.",
        "panel.macros.empty" => {
            "Select an object with a macro \u{2014} or paint in Wet Paint, or edit a Motion object, \
             and press Record."
        }
        "panel.macros.summary" => "{steps} steps \u{b7} {seconds} s",
        "panel.macros.edited_since" => "Edited from step {n}",
        "panel.macros.rename" => "Rename",
        "panel.macros.field.wait" => "Wait before (s)",
        "panel.macros.field.size" => "Size (px)",
        "panel.macros.field.value" => "Value",
        "panel.macros.field.color" => "Colour",
        "panel.macros.step.stroke" => "Stroke {n}",
        "panel.macros.step.add" => "Add {nodes}",
        "panel.macros.step.connect" => "{from} \u{2192} {to}",
        "panel.macros.step.disconnect" => "Disconnect {to}",
        "panel.macros.step.delete" => "Delete {n} nodes",
        "panel.macros.step.move" => "Move nodes",
        "panel.macros.step.rename" => "Rename to {name}",
        "panel.macros.step.param" => "{param}: {from} \u{2192} {to}",
        "panel.macros.step.graph" => "Graph edit",
        "panel.macros.step.direct" => "Direct edit",
        "panel.macros.step.refused" => "Refused: {why}",
        "panel.macros.step.unknown_node" => "?",
        "panel.macros.step.default_value" => "default",
        "panel.macros.refusal.wet_paint_only" => "Only Wet Paint records in this version.",
        "panel.macros.refusal.no_canvas" => "The Painter has no canvas to record on.",
        "panel.macros.refusal.already_recording" => "Already recording.",
        "panel.macros.refusal.nothing_to_record" => {
            "Nothing to record here: paint in Wet Paint, or edit a Motion object."
        }
        "panel.macros.error.newer_version" => {
            "Recorded by a newer PH2D \u{2014} this version cannot play it."
        }
        "panel.macros.error.damaged" => "This recording cannot be read.",
        "panel.macros.error.initial_state" => {
            "The state the recording starts from cannot be rebuilt."
        }
        "panel.macros.error.missing_textures" => "Missing textures: {detail}",
        "panel.macros.error.refused_door" => {
            "The recording went through something a macro does not replay: {detail}"
        }
        "panel.macros.error.newer_step" => "A step was recorded by a newer PH2D.",
        "panel.macros.error.diverged" => "The replay stopped matching the recording.",
        "panel.macros.error.missing_preset" => "Missing brush preset: {detail}",
        "panel.macros.error.edit_does_not_fit" => "That edit does not fit the recording.",
        "panel.macros.error.wait_longer_than_gap" => {
            "Cannot shorten the wait past the end of the previous step."
        }
        "panel.macros.error.not_a_stroke" => "Only a stroke has a colour and a size.",
        "panel.macros.error.not_a_value" => "Only a parameter step has a value.",
        "panel.macros.error.no_such_step" => "That step is no longer in the recording.",
        "panel.macros.error.unknown_name" => "This build has no {detail}.",
        "panel.macros.error.target" => "A step points at something the graph does not have.",
        "panel.macros.error.document" => "A recorded graph cannot be read: {detail}",
        "panel.macros.error.no_graph" => "This build has no Motion graph to replay it.",
        _ => return None,
    })
}
