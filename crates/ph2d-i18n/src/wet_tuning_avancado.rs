//! ⭐ **Os grupos AVANÇADOS do Tuning do Wet Paint** (25/10, dono: «os escondidos do Wet Paint podem
//! mostrar dentro do Tuning»): Bristles · Stroke · Wet effects · Look, e os dois knobs novos do Tools.
//! Tabela IRMÃ do bloco `panel.wet_tuning.*` da `lib.rs` por isolamento (`CLAUDE.md` §0.2).

/// A tradução de uma chave dos grupos avançados do Tuning, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.wet_tuning.group.bristles" => "Bristles",
        "panel.wet_tuning.group.stroke" => "Stroke",
        "panel.wet_tuning.group.wet_effects" => "Wet effects",
        "panel.wet_tuning.group.material" => "Material",
        // Material — o Principled simplificado que o pincel pousa na tinta, por célula (W3, nomes do Blender).
        // Depth e Gloss do Óleo Real (dono 28/10): saíram do cartão Real Oil — os nomes que o dono já conhece.
        "panel.wet_tuning.knob.oilDepth" => "Depth",
        "panel.wet_tuning.knob.oilGloss" => "Gloss",
        "panel.wet_tuning.knob.metallic" => "Metallic",
        "panel.wet_tuning.knob.roughness" => "Roughness",
        "panel.wet_tuning.knob.iorLevel" => "IOR Level",
        "panel.wet_tuning.knob.coat" => "Coat",
        "panel.wet_tuning.knob.coatRoughness" => "Coat Roughness",
        "panel.wet_tuning.knob.sheen" => "Sheen",
        "panel.wet_tuning.knob.subsurface" => "Subsurface",
        "panel.wet_tuning.knob.subsurfaceRadius" => "Subsurface Radius",
        // W3b: a Transparency é o `1 − Alpha` do Principled (mistura de cobertura); a Transmission, o vidro colorido.
        "panel.wet_tuning.knob.transparency" => "Transparency",
        "panel.wet_tuning.knob.transmission" => "Transmission",
        // O modo dela (dono 28/10): o do Blender (o Principled, mais escuro; omissão) ou o vidro colorido claro.
        "panel.wet_tuning.knob.transmissionMode" => "Transmission mode",
        "panel.wet_tuning.option.transmissionMode.0" => "Blender",
        "panel.wet_tuning.option.transmissionMode.1" => "Clear glass",
        "panel.wet_tuning.knob.paperRelief" => "Paper Relief",
        // Dono 28/10 (PT-BR «Amassar o papel»): 1 = a tinta prensa o dente do papel; 0 = segue-o.
        "panel.wet_tuning.knob.paperPress" => "Paper Press",
        "panel.wet_tuning.knob.lockMaterial" => "Lock material",
        "panel.wet_tuning.knob.knifeRidge" => "Knife ridge",
        "panel.wet_tuning.knob.knifeLevel" => "Knife leveling",
        // Bristles — os fios com física própria (lidos quando a Shape é Bristles).
        "panel.wet_tuning.knob.bristlePressure" => "Pressure per bristle",
        "panel.wet_tuning.knob.bristleNoiseLength" => "Pressure change length",
        "panel.wet_tuning.knob.bristleLaneWidth" => "Bristle width",
        "panel.wet_tuning.knob.bristleLaneSizeExp" => "Bristles follow brush size",
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
        // O leque (31/10): os tufos abrem-se num arco.
        "panel.wet_tuning.knob.tuftArc" => "Tufts fan out in an arc",
        // B35.13: as dicas das cerdas no cartão Shape, em Plain English.
        "card.painter.shape.bristle_noise_length" => "Pressure change length (px)",
        "card.painter.shape.bristle_wave_length" => "Width wave length (px)",
        "card.painter.shape.bristle_oscillation" => "Brush wobble (px)",
        "card.painter.shape.bristle_span" => "Random span (px)",
        "card.painter.shape.bristle_span_length" => "Random span length (px)",
        "card.painter.shape.bristle_streak_length" => "Streak length (px)",
        "panel.wet_tuning.tip.bristlePressure" => {
            "How much each bristle presses on its own. Higher gives uneven, streaky paint."
        }
        "panel.wet_tuning.tip.bristleNoiseLength" => {
            "How far you paint before each bristle's pressure changes."
        }
        "panel.wet_tuning.tip.bristleWave" => "Bristles get wider and thinner as you paint.",
        "panel.wet_tuning.tip.bristleWaveLength" => "How far you paint for one wide-thin wave.",
        "panel.wet_tuning.tip.bristleOscillation" => "The brush wobbles side to side as you paint.",
        "panel.wet_tuning.tip.bristleSpan" => "Bristles drift apart and back at random.",
        "panel.wet_tuning.tip.bristleSpanLength" => "How far you paint before the drift changes.",
        "panel.wet_tuning.tip.bristleNoiseHue" => "Each bristle shifts the colour's hue a little.",
        "panel.wet_tuning.tip.bristleNoiseSat" => {
            "Each bristle makes the colour a little more or less vivid."
        }
        "panel.wet_tuning.tip.bristleNoiseLight" => {
            "Each bristle makes the colour a little lighter or darker."
        }
        "panel.wet_tuning.tip.bristleLift" => {
            "With a light touch, some bristles leave the paper and skip."
        }
        "panel.wet_tuning.tip.bristleRemove" => {
            "Bristles dig into the paint below. Needs Bristle relief."
        }
        "panel.wet_tuning.tip.bristleStreak" => "Some bristles leave darker lines in the stroke.",
        "panel.wet_tuning.tip.bristleStreakLight" => {
            "Some bristles leave lighter lines in the stroke."
        }
        "panel.wet_tuning.tip.bristleStreakLength" => "How long each streak runs before it fades.",
        "panel.wet_tuning.tip.bristleStreakCut" => "Higher means fewer streaks.",
        "panel.wet_tuning.tip.tuftArc" => "Tufts spread out in a fan instead of a straight row.",
        // Stroke — como o traço pousa a tinta.
        "panel.wet_tuning.knob.extDryBrush" => "Dry brush",
        "panel.wet_tuning.knob.extWetSoften" => "Softer on wet paint",
        "panel.wet_tuning.knob.tipReservoir" => "Tip picks up and smears",
        "panel.wet_tuning.knob.pressureGrain" => "Light pressure breaks into grain",
        "panel.wet_tuning.knob.strokeFilm" => "Smooth wash body",
        "panel.wet_tuning.knob.strokeFilmDome" => "Wash body: loaded centre, soft sides",
        "panel.wet_tuning.knob.strokeFilmEnds" => "Wash body: soft ends",
        "panel.wet_tuning.knob.dustDrop" => "Dust grain size",
        "panel.wet_tuning.knob.textureSizeExp" => "Texture follows brush size",
        "panel.wet_tuning.knob.textureSizeRef" => "Texture size reference",
        "panel.wet_tuning.knob.textureAcrossOwn" => "Texture across the stroke",
        "panel.wet_tuning.option.textureAcrossOwn.0" => "Same as along",
        "panel.wet_tuning.option.textureAcrossOwn.1" => "Own exponent",
        "panel.wet_tuning.knob.textureAcrossExp" => "Texture follows size across",
        // Water — a borda graduada da secagem (31/10).
        "panel.wet_tuning.knob.edgeSoftMass" => "Edge darkening fades with the paint",
        // Wet effects — o que a água faz depois.
        "panel.wet_tuning.knob.extDiffusion" => "Pigment diffusion",
        "panel.wet_tuning.knob.extBackrun" => "Backruns (blooms)",
        "panel.wet_tuning.knob.extFingering" => "Drip fingers",
        "panel.wet_tuning.knob.extRivulets" => "Number of fingers",
        "panel.wet_tuning.knob.extGranulation" => "Granulation",
        "panel.wet_tuning.knob.wetOpen" => "Wet-in-wet bloom",
        "panel.wet_tuning.knob.wetOpenPush" => "Bloom pushes paint away",
        "panel.wet_tuning.knob.wetOpenEdge" => "Bloom edge softness",
        "panel.wet_tuning.knob.wetCover" => "Covers wet paint underneath",
        // Tools — a água e o secador.
        "panel.wet_tuning.knob.waterPush" => "Water pushes pigment",
        "panel.wet_tuning.knob.dryerEdge" => "Dryer leaves an edge",
        "panel.wet_tuning.knob.dryerLevel" => "Dryer evens the paint (brush radii)",
        "panel.wet_tuning.knob.blendReach" => "Blender reach (brush radii)",
        _ => return None,
    })
}
