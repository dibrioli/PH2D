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
/// `Paper` do motor reduzido ao relevo da tinta). (Experimental is not a knob group — it is the two K–M
/// checkboxes, painted after these.)
pub const SECTIONS: [Section; 5] = [
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

    /// 32 visible knobs (13 Paint — incl. Staining, the brush Glaze of prompt 42, the brush Dust of prompt 43 and the brush
    /// Pressure size/opacity of prompt 44, promoted out of Hidden
    /// by doc 23; as 4 cerdas saíram para o Shape a 13/10, plano 27 W2 — + 7 Water + 6 Physics (Leveling, Level Clamp, Capillary e Brake foram para o cartão Wet Paint a 13/10) + 5 Tools — incl. the doc-23
    /// Rewet lift — + 1 Relief) — the registry's visible surface, whole. O PAPER (6) saiu
    /// na W1e (F2); do grupo dele só o relevo da tinta (Emboss) voltou, a 11/10. O Spacing saiu
    /// para o Hidden a 11/10 (o do pincel é o do Stroke).
    #[test]
    fn every_visible_knob_has_a_row() {
        assert_eq!(rows().len(), 32);
        for g in [
            (KnobGroup::Paint, 13),
            (KnobGroup::Water, 7),
            (KnobGroup::Physics, 6),
            (KnobGroup::Tools, 5),
            (KnobGroup::Paper, 1),
        ] {
            let n = rows().iter().filter(|r| r.group == g.0).count();
            assert_eq!(n, g.1, "group {:?} row count", g.0);
        }
    }
}
