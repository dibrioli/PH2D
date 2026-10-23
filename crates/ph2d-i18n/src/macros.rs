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
        "panel.macros.delete_step" => "Delete step",
        "panel.macros.hint" => "Balloon text\u{2026}",
        "panel.macros.field.wait" => "Wait before",
        "panel.macros.field.size" => "Size",
        "panel.macros.field.value" => "Value",
        "panel.macros.field.color" => "Colour",
        "panel.macros.step.stroke" => "Stroke {n}",
        "panel.macros.step.fill" => "Fill {n}",
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
        "panel.macros.error.before_start" => "A step cannot start before the clip does.",
        "panel.macros.error.changes_what_undo_takes_back" => {
            "Moving this step would change which stroke a recorded Undo takes back."
        }
        "panel.macros.error.not_a_stroke" => "Only a stroke has a colour and a size.",
        "panel.macros.error.not_a_value" => "Only a parameter step has a value.",
        "panel.macros.error.no_such_step" => "That step is no longer in the recording.",
        "panel.macros.error.unknown_name" => "This build has no {detail}.",
        "panel.macros.error.target" => "A step points at something the graph does not have.",
        "panel.macros.error.document" => "A recorded graph cannot be read: {detail}",
        "panel.macros.error.no_graph" => "This build has no Motion graph to replay it.",
        "panel.macros.error.other_size" => {
            "This image is not the size the tutorial was recorded on ({detail})."
        }
        // O balão do tutorial «fazer» (W6).
        "panel.macros.tutorial.step" => "Step {n} of {total}",
        "panel.macros.tutorial.before" => "Before you start",
        "panel.macros.tutorial.done_title" => "Done!",
        "panel.macros.tutorial.done" => "You made every step of this tutorial.",
        "panel.macros.tutorial.edit" => {
            "Edit the tutorial object: select it and press Tab (or pick Edit in its Mode menu)."
        }
        "panel.macros.tutorial.add" => {
            "Add a {node} node: right-click the node editor and pick it from the list."
        }
        "panel.macros.tutorial.connect" => {
            "Connect {from} to {to}: drag from the ringed output of {from} to the ringed input of \
             {to}."
        }
        "panel.macros.tutorial.disconnect" => "Disconnect the ringed input of {to}.",
        "panel.macros.tutorial.value" => "Set {param} on {node} to {value}.",
        "panel.macros.tutorial.delete" => "Delete one {node} node.",
        "panel.macros.tutorial.open_paint" => {
            "Paint on the tutorial image: select it and pick Paint in its Mode menu."
        }
        "panel.macros.tutorial.medium" => "Pick {medium} in the Paint Mode dropdown.",
        "panel.macros.tutorial.paint" => {
            "Paint over the marked area: fill most of it, with any colour and brush."
        }
        "panel.macros.tutorial.fill_colour" => {
            "Pick this colour on the ringed C&F button: {colour} (click it, then type the code or \
             pick it by eye)."
        }
        "panel.macros.tutorial.fill_drop" => {
            "Drag from the ringed C&F button onto the marked spot on the image and let go."
        }
        "panel.macros.tutorial.preparing" => "Preparing this tutorial\u{2026}",
        "panel.macros.tutorial.show_me" => "Show me",
        "panel.macros.tutorial.restart" => "Restart",
        _ => return None,
    })
}
