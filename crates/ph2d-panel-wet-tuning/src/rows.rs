//! **The row table — built FROM the engine's registry.** One list, four
//! consumers (`paint`, `populate`, `event`, the seam sweep): every VISIBLE
//! `KNOB_DEFS` entry becomes a slider+chip+reset row with ids derived from
//! its key (the dynamic family the tool's route resolves), so there is no
//! second copy of a range, step, default or key to drift.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
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

fn decimals_for_step(step: f64) -> usize {
    if step >= 1.0 {
        0
    } else if step >= 0.01 { // LITERAL-PX-OK: step-granularity threshold (math, not design)
        2
    } else if step >= 0.001 { // LITERAL-PX-OK: step-granularity threshold (math, not design)
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
            .filter(|(_, d)| d.group != KnobGroup::Hidden)
            .map(|(i, d)| TuneRow {
                key: d.key,
                knob: i,
                group: d.group,
                label: format!("panel.wet_tuning.knob.{}", d.key),
                slider: ids::wet_tuning_slider_id(d.key),
                chip: ids::wet_tuning_chip_id(d.key),
                reset: ids::wet_tuning_reset_id(d.key),
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

/// The five knob groups, in the model's display order. (Experimental is not
/// a knob group — it is the two K–M checkboxes, painted after these.)
pub const SECTIONS: [Section; 5] = [
    Section {
        group: KnobGroup::Paint,
        header: ids::WET_TUNING_GROUP_HEADERS[0],
        reset: ids::WET_TUNING_GROUP_RESETS[0],
        label: "panel.wet_tuning.group.paint",
    },
    Section {
        group: KnobGroup::Water,
        header: ids::WET_TUNING_GROUP_HEADERS[1],
        reset: ids::WET_TUNING_GROUP_RESETS[1],
        label: "panel.wet_tuning.group.water",
    },
    Section {
        group: KnobGroup::Physics,
        header: ids::WET_TUNING_GROUP_HEADERS[2],
        reset: ids::WET_TUNING_GROUP_RESETS[2],
        label: "panel.wet_tuning.group.physics",
    },
    Section {
        group: KnobGroup::Tools,
        header: ids::WET_TUNING_GROUP_HEADERS[3],
        reset: ids::WET_TUNING_GROUP_RESETS[3],
        label: "panel.wet_tuning.group.tools",
    },
    Section {
        group: KnobGroup::Paper,
        header: ids::WET_TUNING_GROUP_HEADERS[4],
        reset: ids::WET_TUNING_GROUP_RESETS[4],
        label: "panel.wet_tuning.group.paper",
    },
];

/// The three PAPER knobs that re-bake the ENGINE's own tile — inert (and so
/// hidden, lei 3) while the artist's Paper slot drives the tooth.
pub fn is_engine_paper_physical(key: &str) -> bool {
    matches!(key, "paperContrast" | "paperFibres" | "paperGrooves")
}

#[cfg(test)]
mod tests {
    use super::*;
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
            ids::WET_TUNING_SCROLL,
            ids::WET_TUNING_GROUP_HEADERS[5],
            ids::WET_TUNING_PAPER_EYE,
            ids::WET_TUNING_KM_MIXING,
            ids::WET_TUNING_KM_GLAZE,
            ids::PAINTER_WETPAINT_TUNING,
        ] {
            put(id, "static chrome");
        }
    }

    /// 39 visible knobs (13 Paint + 7 Water + 10 Physics + 4 Tools + 5 Paper
    /// + the PaperVisibility master) — the registry's visible surface, whole.
    #[test]
    fn every_visible_knob_has_a_row() {
        assert_eq!(rows().len(), 40);
        for g in [
            (KnobGroup::Paint, 13),
            (KnobGroup::Water, 7),
            (KnobGroup::Physics, 10),
            (KnobGroup::Tools, 4),
            (KnobGroup::Paper, 6),
        ] {
            let n = rows().iter().filter(|r| r.group == g.0).count();
            assert_eq!(n, g.1, "group {:?} row count", g.0);
        }
    }
}
