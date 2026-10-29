//! As TABELAS dos knobs do Wet Paint por ferramenta e por cartão — filho de `wetpaint_settings` por LOC.

use super::*;

/// O corpo de cada ferramenta da água fora os knobs dela: os do pincel que mudam o traço dela e não a tela parada
/// (medido, `corpo_da_agua_tests::o_corpo_de_cada_ferramenta_da_agua_e_o_que_ela_le`). A física da tela (Leveling,
/// Capillary, Brake, Level Clamp, os efeitos da água) muda o traço dela também, mas muda a tela sem ela: é partilhada.
pub(crate) const CORPO_DA_AGUA: [(WetTool, &[Knob]); 5] = [
    (
        WetTool::Smear,
        &[
            CERDAS[0], CERDAS[1], CERDAS[2], CERDAS[3], CERDAS[4], CERDAS[5],
        ],
    ),
    (
        WetTool::Blend,
        &[
            CERDAS[0],
            CERDAS[1],
            CERDAS[2],
            CERDAS[3],
            CERDAS[4],
            CERDAS[5],
            Knob::PaperGate,
            Knob::Spacing,
            Knob::GateSaturation,
            Knob::BlendReach,
        ],
    ),
    (
        WetTool::Wet,
        &[
            CERDAS[0],
            CERDAS[1],
            CERDAS[2],
            CERDAS[3],
            CERDAS[4],
            CERDAS[5],
            Knob::WaterPush,
        ],
    ),
    (
        WetTool::Dry,
        &[
            CERDAS[0],
            CERDAS[1],
            CERDAS[2],
            CERDAS[3],
            CERDAS[4],
            CERDAS[5],
            Knob::DryerEdge,
            Knob::DryerLevel,
        ],
    ),
    (
        WetTool::Blow,
        &[
            CERDAS[0], CERDAS[1], CERDAS[2], CERDAS[3], CERDAS[4], CERDAS[5],
        ],
    ),
];

/// Os knobs do [`CORPO_DA_AGUA`] que só o leitor de UMA ferramenta lê (`tools/secador.rs`, `tools/bordas.rs`,
/// `trail/transfer/perto.rs`) — os outros do corpo o Pincel também os lê.
pub(super) const LIDOS_SO_PELA_FERRAMENTA: [(Knob, WetTool); 4] = [
    (Knob::BlendReach, WetTool::Blend),
    (Knob::WaterPush, WetTool::Wet),
    (Knob::DryerEdge, WetTool::Dry),
    (Knob::DryerLevel, WetTool::Dry),
];

/// O carimbo de cerdas e a tinta que ele leva: o que as cinco lêem.
const CERDAS: [Knob; 6] = [
    Knob::Felt,
    Knob::BristleCount,
    Knob::BristleStrength,
    Knob::BristleSize,
    Knob::Intensity,
    Knob::CapsuleSpacing,
];

/// Os knobs de cada ferramenta da água: o id do cartão, o knob, a ferramenta que o lê e o rótulo.
pub(super) const KNOBS_DA_FERRAMENTA: [(ph2d_a11y::NodeId, Knob, WetTool, &str); 5] = [
    (
        crate::ids::PAINTER_WETPAINT_SMEAR_STRENGTH,
        Knob::Smear,
        WetTool::Smear,
        "panel.painter_wet_cartao.strength",
    ),
    (
        crate::ids::PAINTER_WETPAINT_SMEAR_WATER,
        Knob::SmearWater,
        WetTool::Smear,
        "panel.painter_wet_cartao.smear_water",
    ),
    (
        crate::ids::PAINTER_WETPAINT_BLEND_STRENGTH,
        Knob::BlendForce,
        WetTool::Blend,
        "panel.painter_wet_cartao.strength",
    ),
    (
        crate::ids::PAINTER_WETPAINT_DRY_ABSORBENCY,
        Knob::Dryer,
        WetTool::Dry,
        "panel.painter_wet_cartao.absorbency",
    ),
    (
        crate::ids::PAINTER_WETPAINT_BLOW_STRENGTH,
        Knob::Blow,
        WetTool::Blow,
        "panel.painter_wet_cartao.strength",
    ),
];

/// Os knobs do motor que saíram do Tuning para o cartão: Leveling · Level Clamp · Capillary · Brake (dono 13/10) e
/// o Brush dust (dono 19/10) — o id do cartão e o knob. Os cinco de baixo (dono 25/10: os que mais decidem o pincel e
/// o preset; o Pigment diffusion pedido no smoke) saíram também do Tuning: o que está no cartão não se repete lá
/// (R3). O `ExtDryBrush` ficou de fora: o `dry_gate` é 0 nos 29 presets de fábrica (medido 25/10); só a cobertura
/// que cai (a velocidade → opacidade) o abre (`o_extdrybrush_so_mexe_com_dry_gate_positivo`).
pub(super) const KNOBS_NO_CARTAO: [(ph2d_a11y::NodeId, Knob); 10] = [
    (crate::ids::PAINTER_WETPAINT_LEVELING, Knob::Leveling),
    (crate::ids::PAINTER_WETPAINT_LEVEL_CLAMP, Knob::LevelClamp),
    (crate::ids::PAINTER_WETPAINT_CAPILLARY, Knob::Capillary),
    (crate::ids::PAINTER_WETPAINT_BRAKE, Knob::Brake),
    (crate::ids::PAINTER_WETPAINT_DUST, Knob::Dust),
    (crate::ids::PAINTER_WETPAINT_BACKRUN, Knob::ExtBackrun),
    (
        crate::ids::PAINTER_WETPAINT_GRANULATION,
        Knob::ExtGranulation,
    ),
    (crate::ids::PAINTER_WETPAINT_DIFFUSION, Knob::ExtDiffusion),
    (crate::ids::PAINTER_WETPAINT_WET_BLOOM, Knob::WetOpen),
    (crate::ids::PAINTER_WETPAINT_TIP_PICKUP, Knob::TipReservoir),
];
