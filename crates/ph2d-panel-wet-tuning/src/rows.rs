//! **The row table — built FROM the engine's registry.** One list, four
//! consumers (`paint`, `populate`, `event`, the seam sweep): every VISIBLE
//! `KNOB_DEFS` entry becomes a slider+chip+reset row with ids derived from
//! its key (the dynamic family the tool's route resolves), so there is no
//! second copy of a range, step, default or key to drift.

use ph2d_a11y::NodeId;
use ph2d_wet_paint::tuning::{KNOB_DEFS, KnobGroup};
use std::sync::OnceLock;

/// One knob row.
pub struct TuneRow {
    /// The engine knob's registry key (`"pigmentPerDab"`, ...).
    pub key: &'static str,
    /// Index into `KNOB_DEFS` (== `Knob as usize`).
    pub knob: usize,
    pub group: KnobGroup,
    /// i18n key for the label (`panel.wet_tuning.knob.<key>`).
    pub label: String,
    pub slider: NodeId,
    pub chip: NodeId,
    pub reset: NodeId,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub default: f64,
    /// Readout decimals, derived from the step's granularity.
    pub decimals: usize,
}

impl TuneRow {
    /// Slider track (`0..=1`) → the value it means.
    pub fn value_of(&self, track: f32) -> f64 {
        self.min + f64::from(track) * (self.max - self.min)
    }

    /// The value → its slider track (the inverse — seed must match sample).
    pub fn track_of(&self, value: f64) -> f32 {
        if self.max > self.min {
            (((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)) as f32
        } else {
            0.0
        }
    }
}

/// Step-granularity thresholds for the readout decimals (fmt-stable const
/// lines so the allowlist marker stays glued to its literal).
const STEP_2DP: f64 = 0.01; // LITERAL-PX-OK: step-granularity threshold (math, not design)
const STEP_3DP: f64 = 0.001; // LITERAL-PX-OK: step-granularity threshold (math, not design)

fn decimals_for_step(step: f64) -> usize {
    if step >= 1.0 {
        0
    } else if step >= STEP_2DP {
        2
    } else if step >= STEP_3DP {
        3
    } else {
        4
    }
}

/// Every visible knob, in registry (display) order.
pub fn rows() -> &'static [TuneRow] {
    static ROWS: OnceLock<Vec<TuneRow>> = OnceLock::new();
    ROWS.get_or_init(|| {
        KNOB_DEFS
            .iter()
            .enumerate()
            // O PAPER não tem linhas aqui (W1e, F2): o papel é o da secção Paper do painel — só o RELEVO.
            .filter(|(_, d)| match d.group {
                KnobGroup::Hidden => false,
                KnobGroup::Paper => ph2d_tool_painter::ids::WET_TUNING_RELEVO.contains(&d.key),
                _ => true,
            })
            .map(|(i, d)| TuneRow {
                key: d.key,
                knob: i,
                group: d.group,
                label: format!("panel.wet_tuning.knob.{}", d.key),
                slider: ph2d_tool_painter::ids::wet_tuning_slider_id(d.key),
                chip: ph2d_tool_painter::ids::wet_tuning_chip_id(d.key),
                reset: ph2d_tool_painter::ids::wet_tuning_reset_id(d.key),
                min: d.min,
                max: d.max,
                step: d.step,
                default: d.default,
                decimals: decimals_for_step(d.step),
            })
            .collect()
    })
}

/// The row an id belongs to (slider, chip or reset face), or `None`.
pub fn row_for(id: NodeId) -> Option<&'static TuneRow> {
    rows()
        .iter()
        .find(|r| r.slider == id || r.chip == id || r.reset == id)
}

/// One collapsible group section.
pub struct Section {
    pub group: KnobGroup,
    pub header: NodeId,
    /// The header's reset-group button.
    pub reset: NodeId,
    /// i18n key of the section title.
    pub label: &'static str,
}

/// The knob groups shown, in the model's display order (o PAPER saiu na W1e, F2; o Relief é o grupo
/// `Paper` do motor reduzido ao relevo da tinta), e depois os quatro AVANÇADOS (25/10, dono: «os
/// escondidos do Wet Paint podem mostrar dentro do Tuning»), que nascem dobrados. (Experimental is
/// not a knob group — it is the two K–M checkboxes, painted after these.)
pub const SECTIONS: [Section; 9] = [
    Section {
        group: KnobGroup::Paint,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[0],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[0],
        label: "panel.wet_tuning.group.paint",
    },
    Section {
        group: KnobGroup::Water,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[1],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[1],
        label: "panel.wet_tuning.group.water",
    },
    Section {
        group: KnobGroup::Physics,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[2],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[2],
        label: "panel.wet_tuning.group.physics",
    },
    Section {
        group: KnobGroup::Tools,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[3],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[3],
        label: "panel.wet_tuning.group.tools",
    },
    Section {
        group: KnobGroup::Paper,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[5],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[4],
        label: "panel.wet_tuning.group.relief",
    },
    Section {
        group: KnobGroup::Bristles,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[6],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[5],
        label: "panel.wet_tuning.group.bristles",
    },
    Section {
        group: KnobGroup::Stroke,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[7],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[6],
        label: "panel.wet_tuning.group.stroke",
    },
    Section {
        group: KnobGroup::WetEffects,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[8],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[7],
        label: "panel.wet_tuning.group.wet_effects",
    },
    Section {
        group: KnobGroup::Look,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[9],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[8],
        label: "panel.wet_tuning.group.look",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::ids;
    use std::collections::BTreeSet;

    /// The dynamic id family cannot collide — across faces, across knobs,
    /// and against the section's static ids (the hand table in
    /// `node_id_collisions` cannot see runtime-derived ids, so the family
    /// proves itself here, over the REAL keys).
    #[test]
    fn wet_tuning_ids_dont_collide() {
        let mut seen: BTreeSet<u64> = BTreeSet::new();
        let mut put = |id: NodeId, what: &str| {
            assert!(seen.insert(id.0), "NodeId collision at {what} ({id:?})");
        };
        for r in rows() {
            put(r.slider, r.key);
            put(r.chip, r.key);
            put(r.reset, r.key);
        }
        for s in SECTIONS {
            put(s.header, s.label);
            put(s.reset, s.label);
        }
        for id in [
            ids::WET_TUNING_PANEL,
            ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[4],
            ph2d_tool_painter::ids::WET_TUNING_KM_MIXING,
            ph2d_tool_painter::ids::WET_TUNING_KM_GLAZE,
            ph2d_tool_painter::ids::WET_TUNING_DRAG_HANDLE,
            ph2d_tool_painter::ids::WET_TUNING_RESIZE_HANDLE,
            ph2d_tool_painter::ids::WET_TUNING_RESIZE_HANDLE_BL,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_TUNING,
        ] {
            put(id, "static chrome");
        }
    }

    /// 44 visible knobs (25 Paint — the 12 of the pen in the brush (W6, doc 47) + incl. Staining, the brush Glaze of prompt 42 (o Dust do prompt 43 foi para o cartão Wet Paint a 19/10), the brush
    /// Pressure size/opacity of prompt 44 and its opacity curve of prompt 45, promoted out of Hidden
    /// by doc 23; as 4 cerdas saíram para o Shape a 13/10, plano 27 W2 — + 7 Water + 6 Physics (Leveling, Level Clamp, Capillary e Brake foram para o cartão Wet Paint a 13/10) + 5 Tools — incl. the doc-23
    /// Rewet lift — + 1 Relief) — the registry's visible surface, whole. O PAPER (6) saiu
    /// na W1e (F2); do grupo dele só o relevo da tinta (Emboss) voltou, a 11/10. O Spacing saiu
    /// para o Hidden a 11/10 (o do pincel é o do Stroke).
    /// Mais 40 avançados a 25/10 (dono: os escondidos do Wet Paint vão para o Tuning): 19 Bristles · 7 Stroke
    /// · 7 Wet effects · 5 Look, e o WaterPush/DryerEdge no Tools (5 → 7). Menos 5 no mesmo dia (dono, R3: o que está
    /// no cartão Wet Paint não se repete aqui): Backruns, Granulation, Pigment diffusion, Wet bloom (Wet effects
    /// 7 → 3) e Tip pickup (Stroke 7 → 6). Menos 4 a 25/10 (a linha da ferramenta da água no cartão): Blend force
    /// (Paint 25 → 24), Dryer, Blow e Smear (Tools 7 → 4).
    #[test]
    fn every_visible_knob_has_a_row() {
        assert_eq!(rows().len(), 75);
        for g in [
            (KnobGroup::Paint, 24),
            (KnobGroup::Water, 7),
            (KnobGroup::Physics, 6),
            (KnobGroup::Tools, 4),
            (KnobGroup::Paper, 1),
            (KnobGroup::Bristles, 19),
            (KnobGroup::Stroke, 6),
            (KnobGroup::WetEffects, 3),
            (KnobGroup::Look, 5),
        ] {
            let n = rows().iter().filter(|r| r.group == g.0).count();
            assert_eq!(n, g.1, "group {:?} row count", g.0);
        }
    }

    /// ⭐ **O que fica FORA do Tuning, e porquê** (25/10) — o censo CONTA o grupo `Hidden` do motor contra
    /// esta lista: um knob novo nasce aqui (e alguém decide onde mora) ou num grupo à vista.
    #[test]
    fn os_escondidos_sao_so_os_de_outra_superficie_ou_de_calibracao() {
        const NOUTRA_SUPERFICIE: [&str; 20] = [
            // a Shape › Bristles (plano 27 W2)
            "felt",
            "bristleCount",
            "bristleStrength",
            "bristleSize",
            // o Spacing do Stroke manda (dono 11/10)
            "spacing",
            // o cartão Wet Paint do Painter (dono 13/10 e 19/10)
            "leveling",
            "capillary",
            "brake",
            "levelClamp",
            "dust",
            // os do pincel que mais decidem o preset, só no cartão (dono 25/10, R2 e R3)
            "extBackrun",
            "extGranulation",
            "extDiffusion",
            "wetOpen",
            "tipReservoir",
            // a linha da ferramenta da água na mão, no cartão Wet Paint (dono 25/10, o painel do Rebelle)
            "blendForce",
            "dryer",
            "blow",
            "smear",
            "smearWater",
        ];
        const CALIBRACAO: [&str; 8] = [
            // normaliza o pigmento pelo comprimento (o preset diz a que Spacing foi afinado)
            "pigmentSpacing",
            // passo, tecto e arredondamento do sub-dab da cápsula: amostragem e custo
            "capsuleSpacing",
            "capsuleMaxPx",
            "capsuleRoundNearest",
            // leis de ESCALA com o diâmetro, afinadas contra o Rebelle
            "embossSizeExp",
            "embossSizeRef",
            "thinStrokeFade",
            "thinStrokeRef",
        ];
        let hidden: BTreeSet<&str> = KNOB_DEFS
            .iter()
            .filter(|d| d.group == KnobGroup::Hidden)
            .map(|d| d.key)
            .collect();
        let lista: BTreeSet<&str> = NOUTRA_SUPERFICIE
            .iter()
            .chain(CALIBRACAO.iter())
            .copied()
            .collect();
        assert_eq!(hidden, lista);
    }

    /// Cada secção repõe o grupo que pinta — a tabela da ferramenta e a do painel são a mesma.
    #[test]
    fn o_repor_de_cada_seccao_repoe_o_grupo_dela() {
        let resets = ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS;
        assert_eq!(SECTIONS.len(), resets.len());
        for s in SECTIONS {
            let i = resets.iter().position(|r| *r == s.reset).expect("reset");
            assert_eq!(ph2d_tool_painter::ids::WET_TUNING_RESET_GROUPS[i], s.group);
        }
    }
}
