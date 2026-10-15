//! **As palavras da secção MACRO do Inspector** (ADR-0186). O rótulo da linha do modo é o do
//! descritor (`component.field.macro_fields.2`); «editada desde» e os erros são as frases do painel
//! MACROS (`panel.macros.*`), a mesma frase nos dois sítios.

/// A tradução de uma chave `panel.inspector.macro.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.inspector.macro.newer_version" => {
            "Recorded by a newer PH2D \u{2014} this version cannot play it."
        }
        "panel.inspector.macro.damaged" => "This recording cannot be read.",
        // ⚠️ A ordem é a de `ModoDaMacro` (o índice que o painel envia).
        "panel.inspector.macro.mode.0" => "Faithful",
        "panel.inspector.macro.mode.1" => "Explanatory",
        "panel.inspector.macro.mode.2" => "Do it",
        "panel.inspector.macro.mode_hint.0" => "Plays back exactly what you saw while recording.",
        "panel.inspector.macro.mode_hint.1" => {
            "Plays back the effect of each node, one step at a time."
        }
        "panel.inspector.macro.mode_hint.2" => {
            "A tutorial: you do each step yourself, and a balloon shows which one."
        }
        "panel.inspector.macro.empty" => "Nothing recorded yet.",
        "panel.inspector.macro.summary" => {
            "{tracks} tracks \u{b7} {steps} steps \u{b7} {seconds} s"
        }
        "panel.inspector.macro.edit_steps" => "Edit steps\u{2026}",
        _ => return None,
    })
}
