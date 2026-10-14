//! **As palavras da secção MACRO do Inspector** (ADR-0186). O rótulo da linha do modo é o do
//! descritor (`component.field.macro_fields.2`); os nomes dos modos chegam com quem os pinta (W5) —
//! o censo do painel reprova uma chave sem leitor.

/// A tradução de uma chave `panel.inspector.macro.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.inspector.macro.newer_version" => {
            "Recorded by a newer PH2D \u{2014} this version cannot play it."
        }
        "panel.inspector.macro.damaged" => "This recording cannot be read.",
        _ => return None,
    })
}
