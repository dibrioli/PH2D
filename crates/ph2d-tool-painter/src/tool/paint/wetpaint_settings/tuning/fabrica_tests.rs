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

/// B35.6: o `Tuning::set` do motor e o [`WetKnobs::set`] da ferramenta escrevem o MESMO número (faixa e degrau),
/// fora o NaN (cada um cai na sua fábrica). Uma sonda que escreve no motor vê o que o app veria.
#[test]
fn tuning_set_e_wetknobs_set_tem_a_mesma_lei() {
    let mut diferem = Vec::new();
    for def in KNOB_DEFS.iter() {
        let meio = 0.5 * (def.min + def.max);
        for v in [def.min - 1.0, def.max + 1.0, meio, 0.7 * def.max] {
            let mut motor = ph2d_wet_paint::tuning::Tuning::default();
            motor.set(def.knob, v);
            let mut app = WetKnobs::DEFAULT;
            app.set(def.knob, v);
            if motor.get(def.knob).to_bits() != app.get(def.knob).to_bits() {
                diferem.push((def.key, v, motor.get(def.knob), app.get(def.knob)));
            }
        }
    }
    assert!(diferem.is_empty(), "(knob, escrito, motor, app): {:?}", &diferem[..diferem.len().min(6)]);
}

/// B35.7: a caixa de marcar de um interruptor do Tuning manda o `Click` do chip dele; a ferramenta inverte o knob.
#[test]
fn o_clique_na_caixa_do_interruptor_inverte_o_knob() {
    let mut t = PainterTool::default();
    t.set_paint_tool_mode("wetpaint");
    let id = crate::ids::wet_tuning_chip_id("strokeFilm");
    let antes = t.paint.wetpaint.knobs.get(Knob::StrokeFilm);
    assert!(t.route_wet_tuning_click(id));
    assert_eq!(t.paint.wetpaint.knobs.get(Knob::StrokeFilm), 1.0 - antes);
    assert!(t.route_wet_tuning_click(id));
    assert_eq!(t.paint.wetpaint.knobs.get(Knob::StrokeFilm), antes);
    assert!(!t.route_wet_tuning_click(crate::ids::wet_tuning_chip_id("drag")), "chip de número não é caixa");
}
