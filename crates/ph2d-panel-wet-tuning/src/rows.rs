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
    /// O grupo da SECÇÃO que a pinta: o do motor, salvo os de [`ph2d_tool_painter::ids::WET_TUNING_SOLVER_AVANCADO`]
    /// (a gaveta «Solver (advanced)», a secção do Physics).
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
    /// ⭐ A row painted as a CHOICE (a segmented group, not a slider): one `(i18n key, id)` per option `i`, whose
    /// click forwards `SetValue(slider, i)` — the tool's route stays the slider's. `None` = slider + chip.
    pub options: Option<[(String, NodeId); 2]>,
    /// A unidade do valor, colada ao número dentro do chip ([`UNIDADES`]); o rótulo nunca a leva (spec
    /// `03_a_linha_de_propriedade.md` §7).
    pub unit: Option<ph2d_editor_core::widget::Unit>,
}

/// Os knobs com grandeza física, pela chave do registo (B35.7: os que o doc do `Knob` diz em px de tela ou de caminho);
/// os outros são fracções, ganhos, expoentes ou células sem unidade.
pub const UNIDADES: [(&str, ph2d_editor_core::widget::Unit); 10] = {
    use ph2d_editor_core::widget::Unit::Px;
    [
        ("textureSizeRef", Px),
        ("knifeRidge", Px),
        ("subsurfaceRadius", Px),
        ("wetOpenEdge", Px),
        ("bristleNoiseLength", Px),
        ("bristleWaveLength", Px),
        ("bristleOscillation", Px),
        ("bristleSpan", Px),
        ("bristleSpanLength", Px),
        ("bristleStreakLength", Px),
    ]
};

/// The knobs painted as a 2-way CHOICE (dono 28/10: the Transmission mode — Blender · Clear glass — is a selector,
/// never a slider nor a checkbox). Each is `0..1` with step `1` in the registry.
pub const CHOICES: [&str; 2] = ["transmissionMode", "textureAcrossOwn"];

/// Os INTERRUPTORES (liga/desliga, `0..1` de degrau `1`): uma caixa de marcar, nunca slider nem pílula (dono 10/11).
/// O clique manda `Click(chip)` e a ferramenta inverte o valor.
pub const INTERRUPTORES: [&str; 2] = ["strokeFilm", "lockMaterial"];

impl TuneRow {
    /// A linha é uma caixa de marcar ([`INTERRUPTORES`]).
    #[must_use]
    pub fn interruptor(&self) -> bool {
        INTERRUPTORES.contains(&self.key)
    }

    /// ⭐ **A linha é da ferramenta na MÃO?** — a de um knob que só uma ferramenta da água lê
    /// ([`ph2d_tool_painter::WetKnobs::dona_do_knob`]) só aparece com ela: o slot dela guarda o valor, e editado com
    /// outra na mão ele voltava atrás sozinho ao trocar para ela (auditoria J6, 28/10).
    #[must_use]
    pub fn na_mao(&self, brush: &ph2d_tool_painter::BrushSettings) -> bool {
        ph2d_wet_paint::tuning::KNOB_DEFS
            .get(self.knob)
            .and_then(|d| ph2d_tool_painter::WetKnobs::dona_do_knob(d.knob))
            .is_none_or(|dona| dona == brush.wet_tool && !brush.eraser)
    }

    /// A linha pinta-se: da ferramenta na mão e com o controlo-pai ligado (B35.3, [`crate::pais`]).
    #[must_use]
    pub fn visivel(&self, brush: &ph2d_tool_painter::BrushSettings) -> bool {
        self.na_mao(brush) && crate::pais::pai_ligado(self.key, brush)
    }

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
        let mut rows = KNOB_DEFS
            .iter()
            .enumerate()
            // O PAPER não tem linhas aqui (W1e, F2): o papel é o da secção Paper do painel — só o RELEVO.
            .filter(|(_, d)| match d.group {
                KnobGroup::Hidden => false,
                KnobGroup::Paper => {
                    ph2d_tool_painter::ids::WET_TUNING_RELEVO.contains(&d.key)
                        && !ph2d_tool_painter::ids::NO_INSPECTOR.contains(&d.key)
                }
                _ => !ph2d_tool_painter::ids::NO_INSPECTOR.contains(&d.key),
            })
            .map(|(i, d)| TuneRow {
                key: d.key,
                knob: i,
                group: if ph2d_tool_painter::ids::WET_TUNING_SOLVER_AVANCADO.contains(&d.key) {
                    KnobGroup::Physics
                } else {
                    d.group
                },
                label: format!("panel.wet_tuning.knob.{}", d.key),
                slider: ph2d_tool_painter::ids::wet_tuning_slider_id(d.key),
                chip: ph2d_tool_painter::ids::wet_tuning_chip_id(d.key),
                reset: ph2d_tool_painter::ids::wet_tuning_reset_id(d.key),
                min: d.min,
                max: d.max,
                step: d.step,
                default: ph2d_tool_painter::WetKnobs::fabrica(d.knob),
                decimals: decimals_for_step(d.step),
                unit: UNIDADES.iter().find(|(k, _)| *k == d.key).map(|&(_, u)| u),
                options: CHOICES.contains(&d.key).then(|| {
                    [0, 1].map(|i| {
                        (
                            format!("panel.wet_tuning.option.{}.{i}", d.key),
                            ph2d_tool_painter::ids::wet_tuning_option_id(d.key, i),
                        )
                    })
                }),
            })
            .collect::<Vec<_>>();
        // U5 (auditoria W3b): no Material os PESOS ficam juntos e o «Lock material» no FIM do grupo — ordem de
        // EXIBIÇÃO (o enum `Knob` é append-only: a Transmission e o Paper Relief nasceram depois do Lock). O modo da
        // Transmission (28/10) logo abaixo dela.
        if let (Some(i), Some(_)) = (
            rows.iter().position(|r| r.key == "transmissionMode"),
            rows.iter().position(|r| r.key == "transmission"),
        ) {
            let modo = rows.remove(i);
            let j = rows
                .iter()
                .position(|r| r.key == "transmission")
                .map_or(0, |j| j + 1);
            rows.insert(j, modo);
        }
        // O Paper Press (dono 28/10) logo abaixo do Paper Relief: os dois dizem o que a tinta faz ao dente do papel.
        if let Some(i) = rows.iter().position(|r| r.key == "paperPress") {
            let press = rows.remove(i);
            let j = rows
                .iter()
                .position(|r| r.key == "paperRelief")
                .map_or(rows.len(), |j| j + 1);
            rows.insert(j, press);
        }
        if let Some(i) = rows.iter().position(|r| r.key == "lockMaterial") {
            let lock = rows.remove(i);
            let fim = rows
                .iter()
                .rposition(|r| r.group == KnobGroup::Material)
                .map_or(rows.len(), |j| j + 1);
            rows.insert(fim, lock);
        }
        // O Depth e o Gloss do Óleo Real (dono 28/10, vindos do cartão Real Oil) ABREM o grupo.
        for key in ["oilGloss", "oilDepth"] {
            if let Some(i) = rows.iter().position(|r| r.key == key) {
                let r = rows.remove(i);
                let j = rows
                    .iter()
                    .position(|r| r.group == KnobGroup::Material)
                    .unwrap_or(rows.len());
                rows.insert(j, r);
            }
        }
        // O seletor logo acima do expoente que ele acorda (o enum é append-only; a ordem aqui é de EXIBIÇÃO).
        if let Some(i) = rows.iter().position(|r| r.key == "textureAcrossOwn") {
            let seletor = rows.remove(i);
            let j = rows
                .iter()
                .position(|r| r.key == "textureAcrossExp")
                .unwrap_or(rows.len());
            rows.insert(j, seletor);
        }
        rows
    })
}

/// The row an id belongs to (slider, chip or reset face), or `None`.
pub fn row_for(id: NodeId) -> Option<&'static TuneRow> {
    rows()
        .iter()
        .find(|r| r.slider == id || r.chip == id || r.reset == id)
}

/// The CHOICE row and option index an id is (`None` for any other id).
pub fn option_for(id: NodeId) -> Option<(&'static TuneRow, usize)> {
    rows().iter().find_map(|r| {
        r.options
            .as_ref()?
            .iter()
            .position(|(_, o)| *o == id)
            .map(|i| (r, i))
    })
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
/// `Paper` do motor reduzido ao relevo da tinta), e depois os AVANÇADOS (25/10, dono: «os
/// escondidos do Wet Paint podem mostrar dentro do Tuning»), que nascem dobrados — o Look saiu a 27/10 (W3b:
/// controlos mortos). O Relief (o `emboss`, único knob do `Paper`) subiu ao cartão Wet no B35.12: a secção saiu; o
/// cabeçalho e o repor dela ficam nos ids (macros). (Experimental is not a knob group — it is the two K–M checkboxes, painted after these.)
pub const SECTIONS: [Section; 8] = [
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
        group: KnobGroup::Tools,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[3],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[3],
        label: "panel.wet_tuning.group.tools",
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
        group: KnobGroup::Material,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[9],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[8],
        label: "panel.wet_tuning.group.material",
    },
    Section {
        group: KnobGroup::Physics,
        header: ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[2],
        reset: ph2d_tool_painter::ids::WET_TUNING_GROUP_RESETS[2],
        label: "panel.wet_tuning.group.solver",
    },
];

#[cfg(test)]
#[path = "rows_tests.rs"]
mod tests;
