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
            let w = &mut self.paint.wetpaint;
            w.knobs
                .reset_group_a(crate::ids::WET_TUNING_RESET_GROUPS[gi], &w.preset_base);
            return true;
        }
        if let Some(&(idx, kind)) = wet_tuning_id_map().get(&id)
            && kind == TuneWidget::Reset
        {
            let def = &KNOB_DEFS[idx];
            let w = &mut self.paint.wetpaint;
            w.knobs.set(def.knob, w.preset_base.get(def.knob));
            return true;
        }
        // O interruptor (B35.7): a caixa de marcar (o id do chip) de um knob liga/desliga inverte-o.
        if let Some(&(idx, TuneWidget::Chip)) = wet_tuning_id_map().get(&id) {
            let def = &KNOB_DEFS[idx];
            if def.min == 0.0 && def.max == 1.0 && def.step == 1.0 {
                let v = 1.0 - self.paint.wetpaint.knobs.get(def.knob).round();
                self.paint.wetpaint.knobs.set(def.knob, v);
                return true;
            }
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

/// Uma linha do grupo Bristle Combing do Shape: a faixa e o valor vêm do motor, sem segunda cópia no painel.
#[derive(Clone, Copy, Debug)]
pub struct LinhaDoPenteado {
    pub chave: &'static str,
    pub valor: f64,
    pub min: f64,
    pub max: f64,
    pub passo: f64,
}

impl WetKnobs {
    /// As linhas de [`crate::ids::CERDAS_DO_PENTEADO`], por essa ordem.
    #[must_use]
    pub fn penteado(&self) -> [LinhaDoPenteado; 4] {
        crate::ids::CERDAS_DO_PENTEADO.map(|chave| {
            let d = KNOB_DEFS
                .iter()
                .find(|d| d.key == chave)
                .expect("chave do motor");
            LinhaDoPenteado {
                chave,
                valor: self.get(d.knob),
                min: d.min,
                max: d.max,
                passo: d.step,
            }
        })
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
            if (def.group == KnobGroup::Hidden
                && !crate::ids::CERDAS_DO_PENTEADO.contains(&def.key))
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

impl WetKnobs {
    /// ⭐ **A fábrica de um knob** (B35.5): a ÚNICA que o ↺ da linha, o ↺ do grupo, o Reset do cartão Pen, o NaN e a
    /// linha do Tuning leem — [`Self::DEFAULT`], nunca o `KNOB_DEFS[..].default` do motor (que é o boot da referência).
    #[must_use]
    pub const fn fabrica(knob: Knob) -> f64 {
        Self::DEFAULT.knobs[knob as usize]
    }
}

#[cfg(test)]
mod fabrica_tests;
#[cfg(test)]
mod preset_base_tests; // B35.8
#[cfg(test)]
mod cartao_ou_tuning_tests; // B35.12
