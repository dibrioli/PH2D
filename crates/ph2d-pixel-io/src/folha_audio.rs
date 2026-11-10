//! `meta.audio` na folha JSON (P13 §4): as faixas de som do sprite viajam na folha, porque o
//! `.ase` não as leva (o Aseprite não tem som). A referência ao asset, nunca as amostras.

use ph2d_pixel_core::Sprite;

/// As faixas como JSON: `[{asset, start_ms, gain, muted}]`, na ordem do documento.
#[must_use]
pub fn audio_json(s: &Sprite) -> serde_json::Value {
    serde_json::Value::Array(
        s.audio()
            .iter()
            .map(|f| {
                serde_json::json!({
                    "asset": f.asset,
                    "start_ms": f.inicio_ms,
                    "gain": f.ganho,
                    "muted": f.mudo,
                })
            })
            .collect(),
    )
}

/// Põe `meta.audio` no JSON da folha quando o sprite tem som; sem som, o JSON de hoje, byte a byte.
#[must_use]
pub fn com_audio(s: &Sprite, json: &str) -> String {
    if s.audio().is_empty() {
        return json.to_owned();
    }
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(json) else {
        return json.to_owned();
    };
    if let Some(m) = v.get_mut("meta").and_then(|m| m.as_object_mut()) {
        m.insert("audio".to_owned(), audio_json(s));
    }
    serde_json::to_string_pretty(&v).unwrap_or_else(|_| json.to_owned())
}
