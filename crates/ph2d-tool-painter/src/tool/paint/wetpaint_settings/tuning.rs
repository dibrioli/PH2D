//! O roteamento do painel Tuning (reset por knob/grupo e os `SetValue`) — filho de `wetpaint_settings` por LOC.

use super::*;

impl PainterTool {
    /// The Tuning side panel's per-knob RESET / per-group RESET clicks
    /// (dynamic id family — resolved through [`wet_tuning_id_map`]).
    pub(super) fn route_wet_tuning_click(&mut self, id: ph2d_a11y::NodeId) -> bool {
        if let Some(gi) = crate::ids::WET_TUNING_GROUP_RESETS
            .iter()
            .position(|g| *g == id)
        {
            self.paint
                .wetpaint
                .knobs
                .reset_group(crate::ids::WET_TUNING_RESET_GROUPS[gi]);
            return true;
        }
        if let Some(&(idx, kind)) = wet_tuning_id_map().get(&id)
            && kind == TuneWidget::Reset
        {
            let def = &KNOB_DEFS[idx];
            self.paint.wetpaint.knobs.set(def.knob, def.default);
            return true;
        }
        false
    }

    /// The Tuning side panel's knob `SetValue`s (slider or chip commit).
    pub(super) fn route_wet_tuning_set(&mut self, id: ph2d_a11y::NodeId, v: f64) -> bool {
        if let Some(&(idx, kind)) = wet_tuning_id_map().get(&id)
            && kind != TuneWidget::Reset
        {
            self.paint.wetpaint.knobs.set(KNOB_DEFS[idx].knob, v);
            return true;
        }
        false
    }
}

/// Which face of a Tuning-panel knob row an id belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TuneWidget {
    Slider,
    Chip,
    Reset,
}

/// The dynamic id family, resolved ONCE: every VISIBLE knob's slider / chip /
/// reset id (the hidden §17 group has no UI, exactly like the model). Built
/// from the same `KNOB_DEFS` + id-derive functions the panel paints from —
/// no table to drift.
pub(super) fn wet_tuning_id_map()
-> &'static std::collections::BTreeMap<ph2d_a11y::NodeId, (usize, TuneWidget)> {
    static MAP: std::sync::OnceLock<
        std::collections::BTreeMap<ph2d_a11y::NodeId, (usize, TuneWidget)>,
    > = std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = std::collections::BTreeMap::new();
        for (i, def) in KNOB_DEFS.iter().enumerate() {
            // O PAPER não tem linhas no Tuning (W1e, F2): o papel é o da secção Paper do painel — só o
            // RELEVO da tinta (11/10).
            if def.group == KnobGroup::Hidden
                || (def.group == KnobGroup::Paper
                    && !crate::ids::WET_TUNING_RELEVO.contains(&def.key))
            {
                continue;
            }
            m.insert(
                crate::ids::wet_tuning_slider_id(def.key),
                (i, TuneWidget::Slider),
            );
            m.insert(
                crate::ids::wet_tuning_chip_id(def.key),
                (i, TuneWidget::Chip),
            );
            m.insert(
                crate::ids::wet_tuning_reset_id(def.key),
                (i, TuneWidget::Reset),
            );
        }
        m
    })
}
