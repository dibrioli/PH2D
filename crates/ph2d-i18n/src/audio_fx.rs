//! **O RACK DO AUDIO EDITOR** — os nomes dos 42 efeitos, os rótulos dos parâmetros, os presets de
//! fábrica e as unidades da leitura — e o resto do que a ponte do editor (`ph2d-app-audio`) publica
//! para o painel: a entrega, as plataformas, as variações e a secção espectral (`audio.editor.*`).
//!
//! ⚠️ **O texto mora aqui e a IDENTIDADE não:** cada efeito tem um `FxKind::id` estável (é o que um
//! preset do utilizador grava) e cada parâmetro uma `const` de chave em
//! `ph2d-app-audio/src/fx_param_keys.rs`. Traduzir uma linha desta tabela nunca muda qual efeito
//! um preset carrega nem qual barra um preset de fábrica mexe.
//!
//! ⚠️ **`audio.fx.param.*` é partilhado entre efeitos** (`Freq`, `Mix`, `Q`…): uma língua que precise
//! de duas palavras para o mesmo rótulo em dois efeitos pede uma chave nova, não um `if`.
//!
//! ⚠️ As **unidades** recebem o número JÁ formatado (`{v}`) — sinal e casas decimais são do valor.

/// A tradução de uma chave do rack, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // Os EFEITOS — o nome que o seletor do rack mostra.
        "audio.fx.kind.low_pass" => "Low-Pass",
        "audio.fx.kind.high_pass" => "High-Pass",
        "audio.fx.kind.peak_eq" => "Peak EQ",
        "audio.fx.kind.low_shelf" => "Low Shelf",
        "audio.fx.kind.high_shelf" => "High Shelf",
        "audio.fx.kind.de_hum" => "De-Hum",
        "audio.fx.kind.compress" => "Compress",
        "audio.fx.kind.multiband" => "Multiband",
        "audio.fx.kind.gate_expander" => "Gate / Expander",
        "audio.fx.kind.de_esser" => "De-Esser",
        "audio.fx.kind.de_plosive" => "De-Plosive",
        "audio.fx.kind.de_click" => "De-Click",
        "audio.fx.kind.de_clip" => "De-Clip",
        "audio.fx.kind.limiter" => "Limiter",
        "audio.fx.kind.leveler" => "Leveler",
        "audio.fx.kind.transient" => "Transient",
        "audio.fx.kind.saturate" => "Saturate",
        "audio.fx.kind.distortion" => "Distortion",
        "audio.fx.kind.bitcrush" => "Bitcrush",
        "audio.fx.kind.widen" => "Widen",
        "audio.fx.kind.haas" => "Haas",
        "audio.fx.kind.exciter" => "Exciter",
        "audio.fx.kind.reverb" => "Reverb",
        "audio.fx.kind.conv_reverb" => "Conv Reverb",
        "audio.fx.kind.echo" => "Echo",
        "audio.fx.kind.ping_pong" => "Ping-Pong",
        "audio.fx.kind.comb" => "Comb",
        "audio.fx.kind.chorus" => "Chorus",
        "audio.fx.kind.flanger" => "Flanger",
        "audio.fx.kind.vibrato" => "Vibrato",
        "audio.fx.kind.phaser" => "Phaser",
        "audio.fx.kind.auto_wah" => "Auto-Wah",
        "audio.fx.kind.tremolo" => "Tremolo",
        "audio.fx.kind.auto_pan" => "Auto-Pan",
        "audio.fx.kind.trance_gate" => "Trance Gate",
        "audio.fx.kind.doubler" => "Doubler",
        "audio.fx.kind.ring_mod" => "Ring Mod",
        "audio.fx.kind.pitch_shift" => "Pitch Shift",
        "audio.fx.kind.formant_shift" => "Formant Shift",
        "audio.fx.kind.vocoder" => "Vocoder",
        "audio.fx.kind.granular" => "Granular",
        "audio.fx.kind.harmonizer" => "Harmonizer",
        // Os PARÂMETROS — o rótulo à esquerda de cada barra (partilhado entre efeitos).
        "audio.fx.param.cutoff" => "Cutoff",
        "audio.fx.param.q" => "Q",
        "audio.fx.param.freq" => "Freq",
        "audio.fx.param.gain" => "Gain",
        "audio.fx.param.depth" => "Depth",
        "audio.fx.param.harmonics" => "Harmonics",
        "audio.fx.param.threshold" => "Threshold",
        "audio.fx.param.ratio" => "Ratio",
        "audio.fx.param.attack" => "Attack",
        "audio.fx.param.release" => "Release",
        "audio.fx.param.ceiling" => "Ceiling",
        "audio.fx.param.target" => "Target",
        "audio.fx.param.amount" => "Amount",
        "audio.fx.param.speed" => "Speed",
        "audio.fx.param.sustain" => "Sustain",
        "audio.fx.param.drive" => "Drive",
        "audio.fx.param.tone" => "Tone",
        "audio.fx.param.bits" => "Bits",
        "audio.fx.param.downsample" => "Downsample",
        "audio.fx.param.width" => "Width",
        "audio.fx.param.delay" => "Delay",
        "audio.fx.param.resonance" => "Resonance",
        "audio.fx.param.mix" => "Mix",
        "audio.fx.param.base" => "Base",
        "audio.fx.param.sens" => "Sens",
        "audio.fx.param.room" => "Room",
        "audio.fx.param.damp" => "Damp",
        "audio.fx.param.tail" => "Tail",
        "audio.fx.param.time" => "Time",
        "audio.fx.param.feedback" => "Feedback",
        "audio.fx.param.rate" => "Rate",
        "audio.fx.param.smooth" => "Smooth",
        "audio.fx.param.detune" => "Detune",
        "audio.fx.param.semitones" => "Semitones",
        "audio.fx.param.shift" => "Shift",
        "audio.fx.param.voice_1" => "Voice 1",
        "audio.fx.param.voice_2" => "Voice 2",
        "audio.fx.param.carrier" => "Carrier",
        "audio.fx.param.bands" => "Bands",
        "audio.fx.param.breath" => "Breath",
        "audio.fx.param.grain" => "Grain",
        "audio.fx.param.scatter" => "Scatter",
        "audio.fx.param.pitch" => "Pitch",
        "audio.fx.param.sensitivity" => "Sensitivity",
        // Os PRESETS de fábrica.
        "audio.fx.preset.voice_cleanup" => "Voice Cleanup",
        "audio.fx.preset.podcast" => "Podcast",
        "audio.fx.preset.telephone" => "Telephone",
        "audio.fx.preset.radio" => "Radio",
        "audio.fx.preset.helmet" => "Helmet",
        "audio.fx.preset.lo_fi" => "Lo-Fi",
        "audio.fx.preset.master_bus" => "Master Bus",
        "audio.fx.preset.wide_and_bright" => "Wide & Bright",
        "audio.fx.preset.gate_glue" => "Gate + Glue",
        "audio.fx.preset.robot" => "Robot",
        "audio.fx.preset.vocoder" => "Vocoder",
        "audio.fx.preset.vocoder_whisper" => "Vocoder Whisper",
        "audio.fx.preset.megaphone" => "Megaphone",
        "audio.fx.preset.underwater" => "Underwater",
        "audio.fx.preset.sci_fi_comm" => "Sci-Fi Comm",
        "audio.fx.preset.air" => "Air",
        "audio.fx.preset.wobble" => "Wobble",
        "audio.fx.preset.voice_eq" => "Voice EQ",
        "audio.fx.preset.whisper" => "Whisper",
        "audio.fx.preset.shout" => "Shout",
        "audio.fx.preset.restore" => "Restore",
        "audio.fx.preset.giant" => "Giant",
        "audio.fx.preset.choir" => "Choir",
        // As UNIDADES da leitura — o valor já vem formatado, com o sinal e as casas certas.
        "audio.fx.unit.khz" => "{v} kHz",
        "audio.fx.unit.hz" => "{v} Hz",
        "audio.fx.unit.ms" => "{v} ms",
        "audio.fx.unit.s" => "{v} s",
        "audio.fx.unit.db" => "{v} dB",
        "audio.fx.unit.st" => "{v} st",
        "audio.fx.unit.x" => "{v}\u{d7}",
        // ── O que a ponte do EDITOR publica para o painel (entrega, plataformas, variações, espectral).
        "audio.editor.delivery.quality" => "Quality",
        "audio.editor.delivery.bitrate" => "Bitrate",
        "audio.editor.delivery.pricing_export" => "Pricing the export",
        "audio.editor.delivery.ram_of_budget" => "RAM {ram} \u{b7} {pct}% of budget",
        "audio.editor.platforms.cost" => "{disk} \u{b7} RAM {ram} ({pct}%)",
        "audio.editor.platforms.pricing" => "Pricing shipping targets",
        "audio.editor.variation.row" => "{stem}  \u{00d7}{weight}",
        "audio.editor.variation.row_off" => "(off) {stem}  \u{00d7}{weight}",
        "audio.editor.spectral.switch_to_spectrogram" => {
            "Switch to Spectrogram to select a frequency band"
        }
        "audio.editor.spectral.band" => {
            "Band {lo}-{hi} Hz \u{b7} drag in the spectrogram to change"
        }
        "audio.editor.spectral.drag_a_box" => "Drag a box in the spectrogram to select a region",
        "audio.editor.spectral.profile_learned" => "{status}\nNoise profile learned",
        "audio.editor.spectral.profile_needed" => {
            "{status}\nDenoise needs a noise profile: select silence, then Learn"
        }
        "audio.editor.spectral.ai_denoise" => "AI Denoise (Voice)",
        _ => return None,
    })
}
