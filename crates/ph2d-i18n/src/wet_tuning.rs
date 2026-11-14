//! **O painel Wet Paint (Tuning)** (doc 22): título, grupos e knobs básicos. Cortado da `lib.rs` por ASSUNTO
//! (tecto de 700 LOC, 13/11); os grupos avançados moram na irmã `wet_tuning_avancado.rs`.

/// A tradução de uma chave `panel.wet_tuning.*` básica, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // Wet Tuning side panel (doc 22) — labels are the model app's own.
        // ⛔⛔ **Este valor era "Wet Tuning" e a ABA dizia "Wet Paint"** — duas superfícies do MESMO
        // painel, no ecrã ao mesmo tempo, com nomes diferentes (medido 2026-09-17: cinco painéis
        // assim). A partir da migração do `Panel::TITLE` para `TextKey` há **uma** fonte: esta
        // chave é o que a aba lê E o que o cabeçalho do painel pinta. Ganhou a palavra da ABA,
        // porque é a que o menu *Window* também diz e há gate a atá-las
        // (`the_tab_and_the_menu_call_a_panel_the_same_thing`).
        "panel.wet_tuning.title" => "Wet Paint",
        "panel.wet_tuning.group.paint" => "Paint",
        "panel.wet_tuning.group.water" => "Water",
        "panel.wet_tuning.group.solver" => "Solver (advanced)",
        "panel.wet_tuning.group.tools" => "Tools",
        "panel.wet_tuning.group.experimental" => "Experimental",
        "panel.wet_tuning.knob.emboss" => "Emboss",
        // O Glaze do PINCEL (prompt 42): guardado com o pincel, vale para a tinta dele — a nota do cartão Experimental
        // diz que as duas caixas valem para a tinta da OBRA inteira e que o do pincel age por cima delas.
        "panel.wet_tuning.knob.glaze" => "Brush glaze",
        // O PÓ do pincel (prompt 43): guardado com o pincel — a tinta cai em gotas de 2 px (o aerógrafo).
        "panel.wet_tuning.knob.dust" => "Brush dust",
        "panel.wet_tuning.knob.pressureSize" => "Pressure size",
        "panel.wet_tuning.knob.pressureOpacity" => "Pressure opacity",
        "panel.wet_tuning.knob.pressureOpacityCurve" => "Pressure opacity curve",
        "panel.wet_tuning.knob.pressureSizeCurve" => "Pressure size curve",
        "panel.wet_tuning.knob.pressureFloor" => "Pressure floor",
        "panel.wet_tuning.knob.pressureCeiling" => "Pressure ceiling",
        "panel.wet_tuning.knob.pressureSmoothing" => "Pressure smoothing",
        "panel.wet_tuning.knob.pulledString" => "Pulled string",
        "panel.wet_tuning.knob.dabRotation" => "Dab rotation (none · follow · pen tilt)",
        "panel.wet_tuning.knob.tiltStart" => "Tilt start",
        "panel.wet_tuning.knob.tiltFull" => "Tilt full",
        "panel.wet_tuning.knob.tiltWidth" => "Tilt width",
        "panel.wet_tuning.knob.tiltLength" => "Tilt length",
        "panel.wet_tuning.knob.tiltSmooth" => "Tilt smooths the grain",
        "panel.wet_tuning.knob.tiltLag" => "Tilt lag",
        "panel.wet_tuning.inerte.real_oil_on" => "Acts when Real Oil is on",
        "panel.wet_tuning.inerte.texture_across_own" => {
            "Acts when Texture across the stroke is Own exponent"
        }
        "panel.wet_tuning.km_glaze" => "Glaze layering (K-M)",
        "panel.wet_tuning.note" => {
            "Glaze layering (Kubelka-Munk subtractive color) applies to the next strokes: paint already on the canvas keeps how it was made. It makes a new film filter the paint below, and is saved with the brush preset. Pigment mixing lives in the brush's Wet Paint card. Brush glaze (Paint) is saved with the brush too and acts on top of them. The advanced groups (Bristles, Stroke, Wet effects) start folded: they are for building new brush presets."
        }
        "panel.wet_tuning.knob.pigmentPerDab" => "Pigment per dab",
        "panel.wet_tuning.knob.paperGate" => "Paper gate",
        "panel.wet_tuning.knob.felt" => "Felt (pores)",
        "panel.wet_tuning.knob.bristleCount" => "Bristle count",
        "panel.wet_tuning.knob.drag" => "Drag",
        "panel.wet_tuning.knob.pickup" => "Pickup",
        "panel.wet_tuning.knob.intensity" => "Intensity",
        "panel.wet_tuning.knob.bristleStrength" => "Bristle strength",
        "panel.wet_tuning.knob.bristleSize" => "Bristle size",
        "panel.wet_tuning.knob.tipClean" => "Tip clean",
        "panel.wet_tuning.knob.blendForce" => "Blend force",
        "panel.wet_tuning.knob.gateSaturation" => "Gate saturation",
        "panel.wet_tuning.knob.waterPerDab" => "Water per dab",
        "panel.wet_tuning.knob.waterCap" => "Water cap",
        "panel.wet_tuning.knob.evaporation" => "Evaporation",
        "panel.wet_tuning.knob.rewet" => "Re-wet",
        "panel.wet_tuning.knob.retention" => "Retention",
        "panel.wet_tuning.knob.edgeDarkening" => "Edge darkening",
        "panel.wet_tuning.knob.baseEvaporation" => "Base evaporation",
        "panel.wet_tuning.knob.leveling" => "Leveling",
        "panel.wet_tuning.knob.capillary" => "Capillary",
        "panel.wet_tuning.knob.brake" => "Brake",
        "panel.wet_tuning.knob.gravity" => "Gravity",
        "panel.wet_tuning.knob.levelClamp" => "Level clamp",
        "panel.wet_tuning.knob.viscosity" => "Viscosity",
        "panel.wet_tuning.knob.maxVelocity" => "Max velocity",
        "panel.wet_tuning.knob.projection" => "Projection",
        "panel.wet_tuning.knob.brakeReach" => "Brake reach",
        "panel.wet_tuning.knob.capillaryGate" => "Capillary gate",
        "panel.wet_tuning.knob.eraser" => "Eraser",
        "panel.wet_tuning.knob.dryer" => "Dryer",
        "panel.wet_tuning.knob.blow" => "Blow",
        "panel.wet_tuning.knob.smear" => "Smear",
        "panel.wet_tuning.knob.wetLift" => "Rewet lift",
        "panel.wet_tuning.knob.extStaining" => "Staining",
        _ => return None,
    })
}
