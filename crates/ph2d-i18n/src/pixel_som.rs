//! **As palavras do som do sprite** (P13 T): as linhas de som sob os quadros da Timeline do Pixel.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.pixel.timeline.add_audio_tip" => {
            "Add Audio… Put a sound under the frames. It plays with the animation, and Play \
             follows it."
        }
        "panel.pixel.timeline.audio_mute_tip" => "Mute this sound (it stays in the sprite).",
        "panel.pixel.timeline.audio_remove_tip" => "Remove this sound from the sprite.",
        "panel.pixel.timeline.audio_kind" => "Sound files",
        "panel.pixel.timeline.audio_missing" => "{asset} (not found)",
        _ => return None,
    })
}
