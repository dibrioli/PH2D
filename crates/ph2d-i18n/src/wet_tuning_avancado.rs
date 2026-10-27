//! ⭐ **Os grupos AVANÇADOS do Tuning do Wet Paint** (25/10, dono: «os escondidos do Wet Paint podem
//! mostrar dentro do Tuning»): Bristles · Stroke · Wet effects · Look, e os dois knobs novos do Tools.
//! Tabela IRMÃ do bloco `panel.wet_tuning.*` da `lib.rs` por isolamento (`CLAUDE.md` §0.2).

/// A tradução de uma chave dos grupos avançados do Tuning, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.wet_tuning.group.bristles" => "Bristles",
        "panel.wet_tuning.group.stroke" => "Stroke",
        "panel.wet_tuning.group.wet_effects" => "Wet effects",
        "panel.wet_tuning.group.look" => "Look",
        "panel.wet_tuning.group.material" => "Material",
        // Material — o Principled simplificado que o pincel pousa na tinta, por célula (W3, nomes do Blender).
        "panel.wet_tuning.knob.metallic" => "Metallic",
        "panel.wet_tuning.knob.roughness" => "Roughness",
        "panel.wet_tuning.knob.iorLevel" => "IOR Level",
        "panel.wet_tuning.knob.coat" => "Coat",
        "panel.wet_tuning.knob.coatRoughness" => "Coat Roughness",
        "panel.wet_tuning.knob.sheen" => "Sheen",
        "panel.wet_tuning.knob.subsurface" => "Subsurface",
        "panel.wet_tuning.knob.subsurfaceRadius" => "Subsurface Radius",
        "panel.wet_tuning.knob.lockMaterial" => "Lock material (off · on)",
        "panel.wet_tuning.knob.knifeRidge" => "Knife ridge",
        // Bristles — os fios com física própria (lidos quando a Shape é Bristles).
        "panel.wet_tuning.knob.bristlePressure" => "Pressure per bristle",
        "panel.wet_tuning.knob.bristleNoiseLength" => "Pressure change length",
        "panel.wet_tuning.knob.bristleLaneWidth" => "Bristle width",
        "panel.wet_tuning.knob.bristleWave" => "Width wave",
        "panel.wet_tuning.knob.bristleWaveLength" => "Width wave length",
        "panel.wet_tuning.knob.bristleOscillation" => "Brush wobble",
        "panel.wet_tuning.knob.bristleSpan" => "Random span",
        "panel.wet_tuning.knob.bristleSpanLength" => "Random span length",
        "panel.wet_tuning.knob.bristleNoiseHue" => "Hue jitter per bristle",
        "panel.wet_tuning.knob.bristleNoiseSat" => "Saturation jitter per bristle",
        "panel.wet_tuning.knob.bristleNoiseLight" => "Lightness jitter per bristle",
        "panel.wet_tuning.knob.bristleLift" => "Light touch lifts bristles",
        "panel.wet_tuning.knob.bristleHeight" => "Bristle relief",
        "panel.wet_tuning.knob.bristleRemove" => "Bristles that dig",
        "panel.wet_tuning.knob.bristleTipRagged" => "Ragged bristle tips",
        "panel.wet_tuning.knob.bristleStreak" => "Dark streaks",
        "panel.wet_tuning.knob.bristleStreakLight" => "Light streaks",
        "panel.wet_tuning.knob.bristleStreakLength" => "Streak length",
        "panel.wet_tuning.knob.bristleStreakCut" => "Streak rarity",
        // Stroke — como o traço pousa a tinta.
        "panel.wet_tuning.knob.extDryBrush" => "Dry brush",
        "panel.wet_tuning.knob.extWetSoften" => "Softer on wet paint",
        "panel.wet_tuning.knob.levelingOwnStroke" => "Strokes keep apart (off · on)",
        "panel.wet_tuning.knob.tipReservoir" => "Tip picks up and smears",
        "panel.wet_tuning.knob.pressureGrain" => "Light pressure breaks into grain",
        "panel.wet_tuning.knob.strokeFilm" => "Smooth wash body (off · on)",
        "panel.wet_tuning.knob.dustDrop" => "Dust grain size",
        // Wet effects — o que a água faz depois.
        "panel.wet_tuning.knob.extDiffusion" => "Pigment diffusion",
        "panel.wet_tuning.knob.extBackrun" => "Backruns (blooms)",
        "panel.wet_tuning.knob.extFingering" => "Drip fingers",
        "panel.wet_tuning.knob.extRivulets" => "Number of fingers",
        "panel.wet_tuning.knob.extGranulation" => "Granulation",
        "panel.wet_tuning.knob.wetOpen" => "Wet-in-wet bloom",
        "panel.wet_tuning.knob.wetOpenPush" => "Bloom pushes paint away",
        // Look — só a aparência.
        "panel.wet_tuning.knob.extRakeLight" => "Raking light",
        "panel.wet_tuning.knob.extValleyGran" => "Grain in paper valleys",
        "panel.wet_tuning.knob.extWetSheen" => "Wet sheen",
        "panel.wet_tuning.knob.extGrainDither" => "Grain noise",
        "panel.wet_tuning.knob.extEdgeTint" => "Edge tone (lighter · darker)",
        // Tools — a água e o secador.
        "panel.wet_tuning.knob.waterPush" => "Water pushes pigment",
        "panel.wet_tuning.knob.dryerEdge" => "Dryer leaves an edge",
        _ => return None,
    })
}
