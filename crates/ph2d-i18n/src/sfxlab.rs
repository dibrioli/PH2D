//! O que o SFX LAB diz (W3): a aba do som.

/// A tradução de uma chave do SFX Lab, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "sfxlab.cmd.new" => "New Sound",
        "sfxlab.cmd.open" => "Open Sound \u{25B8} {name}",
        "sfxlab.default_name" => "Sound {n}",
        "sfxlab.play" => "Play",
        "sfxlab.stop" => "Stop",
        "sfxlab.play_on_change" => "Play on change",
        "sfxlab.mix" => "Mix",
        "sfxlab.ms" => "{n} ms",
        "sfxlab.layer.noise" => "Noise",
        "sfxlab.layer.tone" => "Tone",
        "sfxlab.layer.body" => "Struck body",
        "sfxlab.layer.recording" => "Recording",
        "sfxlab.layer.mute" => "M",
        "sfxlab.layer.solo" => "S",
        "sfxlab.add_layer" => "Add layer",
        "sfxlab.add.no_recording" => "Recording \u{2014} no recorded sound in this project",
        "sfxlab.empty" => "Add a layer to hear something",
        "sfxlab.missing" => "Recording not in this project",
        _ => return None,
    })
}
