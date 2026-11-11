//! **A vista em %** do cartão Wet (B35.7, decisão 4): Pickup, Edge Darkening e Gravity aparecem 0–100 % e voltam ao
//! motor pelas MESMAS [`ph2d_wet_paint::tuning::KnobDef::em_porcento`] / `do_porcento`. Filho por LOC.

use super::WetKnobs;
use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob, VISTA_EM_PORCENTO};

/// O id do cartão de cada knob da vista em %, pela ordem de [`VISTA_EM_PORCENTO`].
const IDS: [ph2d_a11y::NodeId; 3] = [
    crate::ids::PAINTER_WETPAINT_PICKUP,
    crate::ids::PAINTER_WETPAINT_EDGE,
    crate::ids::PAINTER_WETPAINT_GRAVITY,
];

fn knob_do_id(id: ph2d_a11y::NodeId) -> Option<Knob> {
    IDS.iter()
        .position(|i| *i == id)
        .map(|n| VISTA_EM_PORCENTO[n])
}

impl WetKnobs {
    /// `(valor em %, passo em %)` do knob do cartão `id`, se ele é da vista em %.
    #[must_use]
    pub fn em_porcento(&self, id: ph2d_a11y::NodeId) -> Option<(f64, f64)> {
        let knob = knob_do_id(id)?;
        let d = &KNOB_DEFS[knob as usize];
        Some((d.em_porcento(self.get(knob)), d.passo_em_porcento()))
    }

    /// O setter da vista em %: `true` se `id` é dela (o NaN cai na fábrica, pelo [`WetKnobs::set`]).
    pub(super) fn set_em_porcento(&mut self, id: ph2d_a11y::NodeId, p: f64) -> bool {
        let Some(knob) = knob_do_id(id) else {
            return false;
        };
        let v = if p.is_nan() {
            p
        } else {
            KNOB_DEFS[knob as usize].do_porcento(p)
        };
        self.set(knob, v);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::tool::PanelEvent;

    fn tool_in_mode(mode: &str) -> crate::tool::PainterTool {
        let mut t = crate::tool::PainterTool::default();
        t.set_paint_tool_mode(mode);
        t
    }

    /// ⭐ O GATE (B35.7): a vista em % e a volta ao motor são a mesma faixa — 0 % = mín, 100 % = máx, a fábrica ida e
    /// volta, e o evento do cartão em % chega ao knob do motor pela função do motor.
    #[test]
    fn faixa_visivel_em_porcento_ida_e_volta_ao_motor() {
        for (id, knob) in IDS.into_iter().zip(VISTA_EM_PORCENTO) {
            let d = &KNOB_DEFS[knob as usize];
            let mut k = WetKnobs::default();
            for (p, motor) in [(0.0, d.min), (100.0, d.max), (50.0, (d.min + d.max) / 2.0)] {
                let mut t = tool_in_mode("wetpaint");
                assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(id, p)));
                let chegou = t.wet_knobs().get(knob);
                assert!(
                    (chegou - motor).abs() < 1e-12,
                    "{}: {p} % → {chegou} ≠ {motor}",
                    d.key
                );
                let (volta, _) = t.wet_knobs().em_porcento(id).expect("da vista");
                assert!((volta - p).abs() < 1e-9, "{}: {p} % → {volta} %", d.key);
            }
            let f = WetKnobs::fabrica(knob);
            let (pf, passo) = k.em_porcento(id).expect("da vista");
            assert!(
                (0.0..=100.0).contains(&pf) && passo > 0.0 && passo <= 1.0,
                "{}",
                d.key
            );
            assert!(k.set_em_porcento(id, pf));
            assert!(
                (k.get(knob) - f).abs() < 1e-12,
                "{}: a fábrica não volta",
                d.key
            );
            assert!(k.set_em_porcento(id, 1e9));
            assert_eq!(
                k.get(knob),
                d.max,
                "{}: fora dos 100 % não fica preso",
                d.key
            );
        }
    }
}
