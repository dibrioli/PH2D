//! ⭐ **B35.3 — a linha cujo controlo-pai está desligado não aparece** (PA-11, decisão 4 do dono 10/11: esconder,
//! não esmaecer). Cada pai é a MESMA pergunta que o leitor do motor faz: o fio só lê `bristleWaveLength` com
//! `bristleWave > 0`, o pó só lê `dustDrop` com `dust > 0`… Lista e porquê: `docs/Painter/missao_4k/fontes/PA_parametros_limites.md`
//! §LISTAS 4. ⛔ Fora daqui, MEDIDO: os 13 pesos do Material sem o Real Oil (a célula guarda-os, `trail/transfer.rs`,
//! e aparecem quando ele liga) e o `extFingering` sem tilt (o `gravity_override` move a água sem o tilt).

use ph2d_tool_painter::BrushSettings;
use ph2d_wet_paint::tuning::KNOB_DEFS;

/// O que liga uma linha.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pai {
    /// Outro knob do registo, ligado com valor `> 0`.
    Knob(&'static str),
    /// A ponta do Shape é Bristles (`dab_route.rs`: sem ela a ponta é LISA e os fios não se leem).
    CerdasNoShape,
    /// O Real Oil ligado ([`ph2d_tool_painter::Dependente::MaterialSemOleo`]).
    OleoReal,
}

/// `(filha, pai)`. Uma filha pode ter vários pais: aparece só com TODOS ligados.
pub const PAIS: [(&str, Pai); 38] = [
    // As cerdas: sem a ponta Bristles nenhuma se lê.
    ("bristlePressure", Pai::CerdasNoShape),
    ("bristleNoiseLength", Pai::CerdasNoShape),
    ("bristleWave", Pai::CerdasNoShape),
    ("bristleWaveLength", Pai::CerdasNoShape),
    ("bristleOscillation", Pai::CerdasNoShape),
    ("bristleSpan", Pai::CerdasNoShape),
    ("bristleSpanLength", Pai::CerdasNoShape),
    ("bristleNoiseHue", Pai::CerdasNoShape),
    ("bristleNoiseSat", Pai::CerdasNoShape),
    ("bristleNoiseLight", Pai::CerdasNoShape),
    ("bristleLift", Pai::CerdasNoShape),
    ("bristleRemove", Pai::CerdasNoShape),
    ("bristleStreak", Pai::CerdasNoShape),
    ("bristleStreakLight", Pai::CerdasNoShape),
    ("bristleStreakLength", Pai::CerdasNoShape),
    ("bristleStreakCut", Pai::CerdasNoShape),
    ("tuftArc", Pai::CerdasNoShape),
    // …e os pais internos delas.
    ("bristleWaveLength", Pai::Knob("bristleWave")),
    ("bristleSpanLength", Pai::Knob("bristleSpan")),
    ("bristleStreakLight", Pai::Knob("bristleStreak")),
    ("bristleStreakLength", Pai::Knob("bristleStreak")),
    ("bristleStreakCut", Pai::Knob("bristleStreak")),
    ("bristleRemove", Pai::Knob("bristleHeight")),
    // O Material: os três do Real Oil e os pais internos.
    ("oilDepth", Pai::OleoReal),
    ("oilGloss", Pai::OleoReal),
    ("paperPress", Pai::OleoReal),
    ("coatRoughness", Pai::Knob("coat")),
    ("subsurfaceRadius", Pai::Knob("subsurface")),
    ("transmissionMode", Pai::Knob("transmission")),
    // As soltas do Stroke e do Wet effects.
    ("extRivulets", Pai::Knob("extFingering")),
    ("strokeFilmDome", Pai::Knob("strokeFilm")),
    ("strokeFilmEnds", Pai::Knob("strokeFilm")),
    ("textureSizeRef", Pai::Knob("textureSizeExp")),
    ("textureAcrossExp", Pai::Knob("textureAcrossOwn")),
    ("dustDrop", Pai::Knob("dust")),
    ("wetOpenPush", Pai::Knob("wetOpen")),
    ("wetOpenEdge", Pai::Knob("wetOpen")),
    ("pressureGrain", Pai::Knob("pressureOpacity")),
];

impl Pai {
    #[must_use]
    pub fn ligado(self, brush: &BrushSettings) -> bool {
        match self {
            Pai::Knob(chave) => KNOB_DEFS
                .iter()
                .position(|d| d.key == chave)
                .is_some_and(|i| brush.wet_knobs.knobs[i] > 0.0),
            Pai::CerdasNoShape => {
                brush.shape_kind == ph2d_tool_painter::TextureKind::Bristles.to_u8()
            }
            Pai::OleoReal => brush.wet_oleo_real,
        }
    }
}

/// A linha `chave` tem todos os pais ligados (sem pai = sempre).
#[must_use]
pub fn pai_ligado(chave: &str, brush: &BrushSettings) -> bool {
    PAIS.iter()
        .filter(|(filha, _)| *filha == chave)
        .all(|(_, pai)| pai.ligado(brush))
}

#[cfg(test)]
#[path = "pais_tests.rs"]
mod tests;
