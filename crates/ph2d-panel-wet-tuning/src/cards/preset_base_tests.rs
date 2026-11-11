//! B35.8: o duplo clique num campo do Tuning volta ao pincel que abriu (`wet_preset_base`), não à fábrica.

use super::*;
use ph2d_wet_paint::tuning::KNOB_DEFS;

/// Cada campo numérico declara como fábrica do duplo clique o valor que o preset P deu ao knob.
#[test]
fn o_duplo_clique_do_tuning_devolve_o_preset_base() {
    let mut b = ph2d_tool_painter::PainterTool::default().brush_settings();
    let campos = || {
        rows::rows()
            .iter()
            .filter(|r| r.options.is_none() && !r.interruptor())
    };
    for row in campos() {
        let d = &KNOB_DEFS[row.knob];
        b.wet_preset_base.knobs[row.knob] = if row.default == d.max { d.min } else { d.max };
    }
    let mut store = WidgetStore::default();
    prepare(&mut store, &b);
    let mut n = 0;
    for row in campos() {
        let dc = store.card_field(row.chip).and_then(|s| s.default);
        assert_eq!(
            dc.map(f64::to_bits),
            Some(b.wet_preset_base.knobs[row.knob].to_bits()),
            "{}: {dc:?}",
            row.key
        );
        n += 1;
    }
    assert!(n > 30, "{n} campos");
}
