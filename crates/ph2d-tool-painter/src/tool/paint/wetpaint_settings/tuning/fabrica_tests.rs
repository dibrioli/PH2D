//! B35.5: o ↺ da linha, o ↺ do grupo, o do cartão Pen e o NaN repõem a MESMA fábrica ([`WetKnobs::fabrica`]).

use super::*;

/// Cada knob com ↺ no Tuning: longe da fábrica, o ↺ da linha e o ↺ do grupo dele dão o mesmo valor, e é o da fábrica.
#[test]
fn defaults_do_wet_e_do_reset_coincidem() {
    let mut t = PainterTool::default();
    t.set_paint_tool_mode("wetpaint");
    let mut diferem = Vec::new();
    for (&id, &(i, kind)) in wet_tuning_id_map() {
        if kind != TuneWidget::Reset {
            continue;
        }
        let def = &KNOB_DEFS[i];
        let fabrica = WetKnobs::fabrica(def.knob);
        let longe = if fabrica == def.max { def.min } else { def.max };
        t.paint.wetpaint.knobs.set(def.knob, longe);
        assert!(t.route_wet_tuning_click(id), "{}", def.key);
        let da_linha = t.paint.wetpaint.knobs.get(def.knob);
        t.paint.wetpaint.knobs.set(def.knob, f64::NAN);
        let do_nan = t.paint.wetpaint.knobs.get(def.knob);
        t.paint.wetpaint.knobs.set(def.knob, longe);
        t.paint.wetpaint.knobs.reset_group(def.group);
        let do_grupo = t.paint.wetpaint.knobs.get(def.knob);
        if da_linha != fabrica || do_grupo != fabrica || do_nan != fabrica {
            diferem.push((def.key, da_linha, do_grupo, do_nan, fabrica));
        }
    }
    assert!(diferem.is_empty(), "(knob, linha, grupo, NaN, fábrica): {diferem:?}");
}
