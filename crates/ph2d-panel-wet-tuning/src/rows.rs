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
                KnobGroup::Paper => ph2d_tool_painter::ids::WET_TUNING_RELEVO.contains(&d.key),
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
/// controlos mortos). (Experimental is not a knob group — it is the two K–M checkboxes, painted after these.)
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
mod tests {
    use super::*;
    use ph2d_editor_core::ids;
    use std::collections::BTreeSet;

    /// B35.7: todo knob liga/desliga visível (`0..1`, degrau `1`) que não é uma ESCOLHA é uma caixa de marcar.
    #[test]
    fn interruptor_nao_e_slider() {
        for r in rows() {
            let liga_desliga = r.min == 0.0 && r.max == 1.0 && r.step == 1.0 && r.options.is_none();
            assert_eq!(r.interruptor(), liga_desliga, "{}", r.key);
        }
        for k in INTERRUPTORES {
            assert!(rows().iter().any(|r| r.key == k), "{k} não é linha visível");
        }
    }

    /// B35.5: o ↺ da linha (o `default` da linha) é a fábrica da ferramenta, nunca o boot do motor (Edge 50 × 12).
    #[test]
    fn a_linha_do_tuning_mostra_a_fabrica() {
        for r in rows() {
            let f = ph2d_tool_painter::WetKnobs::fabrica(KNOB_DEFS[r.knob].knob);
            assert_eq!(r.default.to_bits(), f.to_bits(), "{}", r.key);
        }
    }

    /// Cada unidade nomeia uma linha VISÍVEL — uma chave com erro de grafia deixava o número sem unidade.
    #[test]
    fn cada_unidade_nomeia_uma_linha_visivel() {
        for (k, u) in UNIDADES {
            let r = rows()
                .iter()
                .find(|r| r.key == k)
                .expect("a chave é de uma linha visível");
            assert_eq!(r.unit, Some(u));
        }
    }

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
            for (_, o) in r.options.iter().flatten() {
                put(*o, r.key);
            }
        }
        for s in SECTIONS {
            put(s.header, s.label);
            put(s.reset, s.label);
        }
        for id in [
            ids::WET_TUNING_PANEL,
            ph2d_tool_painter::ids::WET_TUNING_GROUP_HEADERS[4],
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
    /// (Paint 25 → 24), Dryer, Blow e Smear (Tools 7 → 4). Mais 11 a 27/10 (Óleo Real W2 E3): o rebordo da faca no
    /// Stroke (6 → 7) e os 10 do Material da tinta (o Principled por célula e o Lock). Menos 1 a
    /// 27/10 (W3): o Material são os 8 do Principled simplificado e o Lock (10 → 9). Mais 2 a 27/10 (W3 K2): a textura
    /// que acompanha o tamanho do pincel (expoente e diâmetro de referência) no Stroke (7 → 9). Mais 1 a 27/10 (W3 K3a):
    /// a faca que nivela (`knifeLevel`) no Stroke (9 → 10). Mais 1 a 27/10 (W3 K5): o expoente da textura através
    /// do traço (`textureSizeExpAcross`) no Stroke (10 → 11). Mais 1 a 27/10 (dono): a Transparency (`1 − Alpha`)
    /// no Material (9 → 10). Mais 2 (W3b): a Transmission e o Relevo do papel (10 → 12). Menos 5 a 27/10 (W3b item 3):
    /// o grupo Look inteiro (controlos mortos: só o `render_region`, sem chamador no produto, os lia). Mais 1 a 28/10
    /// (dono): o modo da Transmission, um seletor (12 → 13). Mais 2 a 28/10 (dono): o Depth e o Gloss do Óleo Real,
    /// vindos do cartão Real Oil (13 → 15). Mais 2 a 28/10 (J6 aguarela): o domo do filme no Stroke (11 → 12) e a
    /// borda da mancha molhada no Wet effects (3 → 4). Mais 1 (J6): as tampas do filme no Stroke (12 → 13). Mais 1 (J6 mistura): o
    /// nível do secador no Tools (+1). Mais 1 (J6 mistura, lacuna 6): o alcance do misturador no Tools (+1). Mais 1 (J6 guache, lacuna 7): a tinta que cobre a molhada no
    /// Wet effects (4 → 5). Mais 1 (J6 cerdas): os fios que crescem com o pincel no Bristles (19 → 20).
    /// Mais 1 a 28/10 (dono): o Paper Press no Material (15 → 16). Mais 1 a 31/10: a borda graduada no Water (7 → 8). Contagem refeita do código a 31/10: 98 → 95 (o cartão Wet Paint levou Staining, Glaze, Cover e Wet soften de Paint/Stroke/Wet effects para Hidden a 30/10 sem este teste; Paint 24 → 22, Stroke 13 → 12, Wet effects 5 → 4).
    /// Mais 1 a 31/10: o arco dos tufos do leque (`tuftArc`) no Bristles (20 → 21).
    /// B35.2: o `gateSaturation` pinta-se na gaveta do solver (Paint 3 → 2, Physics 5 → 6).
    /// Menos 19 a 10/11 (B35.1, [`ph2d_tool_painter::ids::NO_INSPECTOR`]): 88 → 69 (Paint 19 → 3, Water 7 → 5, Physics 6 → 5).
    /// Menos 1 (B35.9): o Eraser fundiu-se no Erase Strength do cartão (Tools 6 → 5; 69 → 68).
    /// Menos 1 (B35.10): o Base evaporation fundiu-se no Dry Speed (Water 5 → 4; 68 → 67).
    #[test]
    fn every_visible_knob_has_a_row() {
        assert_eq!(rows().len(), 67);
        for g in [
            (KnobGroup::Paint, 2),
            (KnobGroup::Water, 4),
            (KnobGroup::Physics, 6),
            (KnobGroup::Tools, 5),
            (KnobGroup::Paper, 1),
            (KnobGroup::Bristles, 17),
            (KnobGroup::Stroke, 12),
            (KnobGroup::WetEffects, 4),
            (KnobGroup::Material, 16),
        ] {
            let n = rows().iter().filter(|r| r.group == g.0).count();
            assert_eq!(n, g.1, "group {:?} row count", g.0);
        }
    }

    /// ⭐ B35.1 (R3 do dono, 25/10): um knob com linha no Inspector não tem segunda linha no Tuning — os 15 da caneta
    /// são os `Knob` de [`ph2d_tool_painter::LINHAS_DA_MAO`] e os 4 do cartão Wet os que ele escreve.
    #[test]
    fn nenhum_knob_tem_duas_linhas_no_mesmo_painel() {
        let caneta: BTreeSet<&str> = ph2d_tool_painter::LINHAS_DA_MAO
            .iter()
            .filter_map(|l| match l {
                ph2d_tool_painter::LinhaDaMao::Knob(k) => Some(KNOB_DEFS[*k as usize].key),
                _ => None,
            })
            .collect();
        let cartao_wet = ["pickup", "evaporation", "edgeDarkening", "gravity"];
        let no_inspector: BTreeSet<&str> = caneta.iter().copied().chain(cartao_wet).collect();
        let lista: BTreeSet<&str> = ph2d_tool_painter::ids::NO_INSPECTOR.into_iter().collect();
        assert_eq!(no_inspector, lista, "a lista é a caneta + o cartão Wet");
        for r in rows() {
            assert!(
                !lista.contains(r.key),
                "{}: já tem linha no Inspector",
                r.key
            );
        }
    }

    /// ⭐ **O que fica FORA do Tuning, e porquê** (25/10) — o censo CONTA o grupo `Hidden` do motor contra
    /// esta lista: um knob novo nasce aqui (e alguém decide onde mora) ou num grupo à vista.
    #[test]
    fn os_escondidos_sao_so_os_de_outra_superficie_ou_de_calibracao() {
        const NOUTRA_SUPERFICIE: [&str; 42] = [
            // a Shape › Bristles (plano 27 W2)
            "felt",
            "bristleCount",
            "bristleStrength",
            "bristleSize",
            // a Shape › Bristle Combing (dono 07/11)
            "bristleLaneWidth",
            "bristleHeight",
            "bristleTipRagged",
            "bristleLaneSizeExp",
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
            // a linha do Wet (31/10): o Drag
            "waterDrag",
            // o cartão Wet Paint do pincel (ordem do dono 30/10): Staining, Glaze, Cover, Wet soften e a Drag (J10)
            "extStaining",
            "glaze",
            "wetCover",
            // e o Dry cover ao lado do Cover (31/10)
            "dryCover",
            "extWetSoften",
            "rewetPaint",
            "solubility",
            "rewetDrag",
            // os controlos de lei da E10 (F3 §9): os volumes do topo e o cartão Wet Paint
            "medium",
            "dilution",
            "volume",
            "body",
            "smoothEdges",
            // e os fundidos neles: a mesma grandeza que um controlo acima já escreve (o Hardness escreve o paperGate)
            "pigmentPerDab",
            "waterPerDab",
            "intensity",
            "paperGate",
        ];
        const CALIBRACAO: [&str; 9] = [
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
            // a tinta opaca (31/10): o ponto no segmento das duas tintas, declarado pelo preset de fábrica
            "opaque",
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

    /// ⭐ U5 (auditoria W3b): o grupo Material mostra os 8 do Principled, os três PESOS da tinta juntos (Transparency ·
    /// Transmission · Paper Relief), o modo da Transmission logo abaixo dela (dono 28/10) e o «Lock material» no FIM;
    /// o Depth e o Gloss do Óleo Real (dono 28/10, vindos do cartão) à cabeça.
    #[test]
    fn a_ordem_do_material_junta_os_pesos_e_o_lock_no_fim() {
        let m: Vec<&str> = rows()
            .iter()
            .filter(|r| r.group == KnobGroup::Material)
            .map(|r| r.key)
            .collect();
        assert_eq!(
            m,
            [
                "oilDepth",
                "oilGloss",
                "metallic",
                "roughness",
                "iorLevel",
                "coat",
                "coatRoughness",
                "sheen",
                "subsurface",
                "subsurfaceRadius",
                "transparency",
                "transmission",
                "transmissionMode",
                "paperRelief",
                "paperPress",
                "lockMaterial",
            ]
        );
        let escolhas: Vec<&str> = rows()
            .iter()
            .filter(|r| r.options.is_some())
            .map(|r| r.key)
            .collect();
        assert_eq!(
            escolhas.len(),
            CHOICES.len(),
            "os seletores são os de CHOICES"
        );
        assert!(CHOICES.iter().all(|k| escolhas.contains(k)));
        for k in CHOICES {
            let d = KNOB_DEFS.iter().find(|d| d.key == k).expect("knob");
            assert_eq!(
                (d.min, d.max, d.step),
                (0.0, 1.0, 1.0),
                "{k}: um seletor de 2 é 0..1 de passo 1"
            );
        }
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
