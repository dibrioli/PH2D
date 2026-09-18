//! **AS STRINGS DOS PAINÉIS DE ÁUDIO** — `panel.audio_editor.*` e `panel.audio_mixer.*`.
//!
//! ⚠️ **Um corte por ASSUNTO**, como o `vector.rs` e o `painter_layers.rs`: dois painéis do mesmo
//! módulo, e nenhum dos dois partilha um `match` com outra linha.
//!
//! # De onde isto veio
//!
//! Migrado em 2026-09-16 por `scripts/migrar-texto-pintado.py` (mapas
//! `docs/UI_New_and_Simple/ferramentas/seccoes_audio_editor.tsv` e `…_audio_mixer.tsv`): os dois
//! painéis escreviam **124** textos no fonte e não dependiam da `ph2d-i18n`.
//!
//! ⚠️ **Os braços entre os marcadores são do script** — uma chave nova escreve-se à mão FORA deles.

/// A tradução de uma chave dos painéis de áudio, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // À MÃO: a régua lexical lê «UI» como identificador (maiúsculas), e ele é um nome de faixa.
        "panel.audio_mixer.bus.ui" => "UI",
        // ph2d-migrar-texto:begin
        "panel.audio_editor.title" => "Audio Editor",
        "panel.audio_editor.delivery.quality" => "Quality",
        "panel.audio_editor.delivery.ram" => "RAM \u{2014}",
        "panel.audio_editor.delivery.drops_loop_points_and_markers" => {
            "Drops loop points and markers"
        }
        "panel.audio_editor.delivery.export_set" => "Export Set",
        "panel.audio_editor.delivery.export_pieces" => "Export Pieces",
        "panel.audio_editor.edit.select" => "Select",
        "panel.audio_editor.edit.move" => "Move",
        "panel.audio_editor.edit.scale" => "Scale",
        "panel.audio_editor.edit.cut" => "Cut",
        "panel.audio_editor.edit.copy" => "Copy",
        "panel.audio_editor.edit.paste" => "Paste",
        "panel.audio_editor.edit.split" => "Split",
        "panel.audio_editor.edit.clear_cuts" => "Clear Cuts",
        "panel.audio_editor.edit.undo" => "Undo",
        "panel.audio_editor.edit.redo" => "Redo",
        "panel.audio_editor.edit.normalize" => "Normalize",
        "panel.audio_editor.edit.norm_lufs" => "Norm LUFS",
        "panel.audio_editor.edit.reverse" => "Reverse",
        "panel.audio_editor.edit.rm_dc" => "Rm DC",
        "panel.audio_editor.edit.gain_down" => "Gain \u{2212}",
        "panel.audio_editor.edit.gain_up" => "Gain +",
        "panel.audio_editor.edit.invert" => "Invert",
        "panel.audio_editor.edit.force_mono" => "Force Mono",
        "panel.audio_editor.edit.trim" => "Trim",
        "panel.audio_editor.edit.silence" => "Silence",
        "panel.audio_editor.edit.fade_in" => "Fade In",
        "panel.audio_editor.edit.fade_out" => "Fade Out",
        "panel.audio_editor.effects.apply" => "Apply",
        "panel.audio_editor.effects.save" => "Save",
        "panel.audio_editor.effects.load" => "Load",
        "panel.audio_editor.effects.load_ir" => "Load IR\u{2026}",
        "panel.audio_editor.effects.change_ir" => "Change IR\u{2026}",
        "panel.audio_editor.effects.no_room_loaded" => "no room loaded",
        "panel.audio_editor.effects.chain" => "Chain",
        "panel.audio_editor.effects.bypass" => "Bypass",
        "panel.audio_editor.effects.cancel" => "Cancel",
        "panel.audio_editor.loop.set_loop" => "Set Loop",
        "panel.audio_editor.loop.clear" => "Clear",
        "panel.audio_editor.loop.crossfade" => "Crossfade",
        "panel.audio_editor.loop.crossfade_loop" => "Crossfade Loop",
        "panel.audio_editor.loop.no_loop" => "No loop",
        "panel.audio_editor.loop.add_marker" => "Add Marker",
        "panel.audio_editor.loop.delete" => "Delete",
        "panel.audio_editor.loop.split_at_markers" => "Split at Markers",
        "panel.audio_editor.loop.no_markers" => "No markers",
        "panel.audio_editor.loop.one_marker" => "1 marker",
        "panel.audio_editor.loop.n_markers" => "{n} markers",
        "panel.audio_editor.transport.transport" => "Transport",
        "panel.audio_editor.transport.loop" => "Loop",
        "panel.audio_editor.transport.edit" => "Edit",
        "panel.audio_editor.transport.spectral" => "Spectral",
        "panel.audio_editor.transport.effects" => "Effects",
        "panel.audio_editor.transport.markers" => "Markers",
        "panel.audio_editor.transport.variations" => "Variations",
        "panel.audio_editor.transport.delivery" => "Delivery",
        "panel.audio_editor.transport.no_clip_loaded" => "No clip loaded",
        "panel.audio_editor.transport.pause" => "Pause",
        "panel.audio_editor.transport.play" => "Play",
        "panel.audio_editor.transport.stop" => "Stop",
        "panel.audio_editor.transport.load" => "Load\u{2026}",
        "panel.audio_editor.transport.export_wav" => "Export WAV\u{2026}",
        "panel.audio_editor.transport.batch_lufs" => "Batch LUFS\u{2026}",
        "panel.audio_editor.spectral.spectrogram" => "Spectrogram",
        "panel.audio_editor.spectral.repair_selection" => "Repair Selection",
        "panel.audio_editor.spectral.learn_noise" => "Learn Noise",
        "panel.audio_editor.spectral.denoise" => "Denoise",
        "panel.audio_editor.spectral.ai_denoise_voice" => "AI Denoise (Voice)",
        "panel.audio_editor.spectral.amount" => "Amount",
        "panel.audio_editor.spectral.ai_denoise_running" => {
            "AI Denoise is running \u{b7} see the progress bar at the top"
        }
        "panel.audio_editor.spectral.waveform" => "Waveform",
        "panel.audio_editor.variations.add" => "Add\u{2026}",
        "panel.audio_editor.variations.add_folder" => "Add Folder\u{2026}",
        "panel.audio_editor.variations.enabled" => "Enabled",
        "panel.audio_editor.variations.disabled" => "Disabled",
        "panel.audio_editor.variations.remove" => "Remove",
        "panel.audio_editor.variations.play_variation" => "Play Variation",
        "panel.audio_editor.variations.weight_half" => "Weight \u{00f7}2",
        "panel.audio_editor.variations.weight_double" => "Weight \u{00d7}2",
        "panel.audio_editor.variations.pitch_jitter" => "Pitch jitter",
        "panel.audio_editor.variations.gain_jitter" => "Gain jitter",
        "panel.audio_editor.variations.save" => "Save\u{2026}",
        "panel.audio_editor.variations.load" => "Load\u{2026}",
        "panel.audio_editor.variations.add_clips_to_build_a_set" => "Add clips to build a set",
        "panel.audio_editor.variations.no_clips" => "No clips",
        "panel.audio_editor.variations.one_clip" => "1 clip",
        "panel.audio_editor.variations.n_clips" => "{n} clips",
        "panel.audio_editor.variations.shuffle" => "Shuffle",
        "panel.audio_mixer.bus.music" => "Music",
        "panel.audio_mixer.bus.sfx" => "SFX",
        "panel.audio_mixer.bus.voice" => "Voice",
        "panel.audio_mixer.title" => "Audio Mixer",
        "panel.audio_mixer.strip.master" => "Master",
        "panel.audio_mixer.strip.mute" => "Mute",
        // ⭐⭐ **O PAR DO `dB` FALTAVA AO LADO DO PAR DO `LUFS` — e o que os separou foi a RÉGUA.**
        // O `master.inf_lufs`/`master.lufs_value` migrou em 2026-09-16 e este, no MESMO pintor,
        // ficou: `is_language` aceita `-inf LUFS` (GRITADO) e recusa `-inf` e `{:.0} dB` (`dB` não
        // é Capitalizado nem GRITADO). *Um censo lexical não vê a UNIDADE que o artista lê* — quem
        // o viu foi o idioma de teste, na 2.ª fotografia do dono.
        "panel.audio_mixer.strip.inf_db" => "-inf",
        "panel.audio_mixer.strip.db_value" => "{db} dB",
        // ⚠️ `M` e `S` são a convenção da mesa de mistura e continuam a ser TEXTO que o artista lê
        // — uma letra solta é invisível à régua (ela exige duas letras adjacentes), e é por isso que
        // o irmão de largura inteira (`strip.mute`) migrou e estes dois não.
        "panel.audio_mixer.strip.mute_short" => "M",
        "panel.audio_mixer.strip.solo_short" => "S",
        "panel.audio_mixer.master.stop" => "Stop",
        "panel.audio_mixer.master.play_test" => "Play Test",
        "panel.audio_mixer.master.inf_lufs" => "-inf LUFS",
        "panel.audio_mixer.master.lufs_value" => "{lufs} LUFS",
        "panel.audio_mixer.master.limiter" => "Limiter",
        "panel.audio_mixer.master.low" => "Low",
        "panel.audio_mixer.master.mid" => "Mid",
        "panel.audio_mixer.master.high" => "High",
        "panel.audio_mixer.master.reverb" => "Reverb",
        "panel.audio_mixer.master.size" => "Size",
        "panel.audio_mixer.master.return" => "Return",
        "panel.audio_mixer.master.delay" => "Delay",
        "panel.audio_mixer.master.time" => "Time",
        "panel.audio_mixer.master.fbk" => "Fbk",
        "panel.audio_mixer.master.comp" => "Comp",
        "panel.audio_mixer.master.ducking" => "Ducking",
        "panel.audio_mixer.master.key" => "Key: {bus}",
        "panel.audio_mixer.master.depth" => "Depth",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
