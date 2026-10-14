//! **As palavras da secção MACRO do Inspector** (ADR-0186). O rótulo da linha do modo é o do
//! descritor (`component.field.macro_fields.2`).

/// A tradução de uma chave `panel.inspector.macro.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ⚠️ A ordem é a de `ModoDaMacro` (o índice que o painel envia).
        "panel.inspector.macro.mode.0" => "Faithful",
        "panel.inspector.macro.mode.1" => "Explain",
        "panel.inspector.macro.newer_version" => {
            "Recorded by a newer PH2D \u{2014} this version cannot play it."
        }
        "panel.inspector.macro.damaged" => "This recording cannot be read.",
        _ => return None,
    })
}
