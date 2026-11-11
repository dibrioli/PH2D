//! B35.8: o duplo clique num campo do cartão Wet volta ao pincel que abriu (`wet_preset_base`).

use super::*;

/// Cada número do cartão Wet e dos volumes declara como fábrica do duplo clique o valor do preset P.
#[test]
fn o_duplo_clique_do_cartao_wet_devolve_o_preset_base() {
    let mut t = ph2d_tool_painter::PainterTool::default();
    t.set_paint_media(ph2d_tool_painter::PaintMedia::WetPaint);
    let mut b = t.brush_settings();
    assert!(presente(&b));
    b.wet_preset_base
        .knobs
        .iter_mut()
        .for_each(|v| *v = *v * 0.5 + 0.01);
    let mut store = WidgetStore::default();
    prepare(&mut store, &b);
    prepara_volumes(&mut store, &b);
    let p = paint_brush::fabrica_do_pincel(&b);
    assert_eq!(p.wet_knobs.knobs, b.wet_preset_base.knobs);
    let mut fora_da_fabrica = 0;
    let numeros = todos_os_numeros(&b, &p);
    let volumes = paint_wetpaint_presets::numeros_dos_volumes(&b, &p);
    let fabrica: Vec<Numero> = todos_os_numeros(&b, &paint_brush::FALLBACK_BRUSH);
    for (n, f) in numeros.iter().zip(&fabrica) {
        let dc = store.card_field(n.id).and_then(|s| s.default);
        assert_eq!(dc, Some(f64::from(n.valor)), "{}", n.chave);
        fora_da_fabrica += usize::from(n.valor != f.valor);
    }
    for (n, _) in &volumes {
        let dc = store.card_field(n.id).and_then(|s| s.default);
        assert_eq!(dc, Some(f64::from(n.valor)), "{}", n.chave);
    }
    assert!(
        fora_da_fabrica > 5,
        "{fora_da_fabrica} campos seguem o preset"
    );
}
