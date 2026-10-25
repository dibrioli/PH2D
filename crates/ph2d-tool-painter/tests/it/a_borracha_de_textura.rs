//! ⭐ **A Texture Eraser morde o grão também à Strength 1** (handoff FERRAMENTAS_2 §4 e): sem o tecto do traço
//! (`stroke_cover_wanted`) os dabs sobrepostos saturam e o grão apaga tudo (255 uniforme), enquanto a 0,99 o texel
//! pára no peso do grão. A Strength 1 tem de ser o limite da 0,99, não um salto.

use crate::familias::mistura::MEIOS;
use ph2d_editor_core::tool::RasterEditTool;

/// A Texture Eraser de fábrica, na tela 256 × 128 de vermelho opaco, com a Strength `s`: (média, desvio) do alfa
/// APAGADO (`255 − α`) no miolo da passada.
fn apagado(s: f32) -> (f64, f64) {
    let r = MEIOS.iter().find(|r| r.nome == "Texture Eraser").unwrap();
    let (parametros, texturas) = crate::presets_de_fabrica::parametros_do_meio(r);
    let p = ph2d_brush_presets::Preset {
        parametros,
        origem: ph2d_brush_presets::Origem::Fabrica,
        miniatura: ph2d_brush_presets::Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![255; 4],
        },
        texturas,
    };
    let mut t = ph2d_tool_painter::PainterTool::default();
    t.set_source([191u8, 0, 18, 255].repeat(256 * 128), 256, 128);
    assert_eq!(t.preset_aplicar(&p), Ok(Vec::new()));
    t.set_brush_size_px(20.0);
    t.set_brush_strength(s);
    crate::sonda_gestos::gesto(&mut t, &[[30.0, 64.0], [226.0, 64.0]], 3.0, |_| 1.0);
    let (px, w, _) = t.run_full();
    let v: Vec<f64> = (60..68)
        .flat_map(|y| (60..196).map(move |x| (y, x)))
        .map(|(y, x)| 255.0 - f64::from(px[(y * w as usize + x) * 4 + 3]))
        .collect();
    let n = v.len() as f64;
    let media = v.iter().sum::<f64>() / n;
    let desvio = (v.iter().map(|a| (a - media).powi(2)).sum::<f64>() / n).sqrt();
    (media, desvio)
}

#[test]
fn texture_eraser_a_strength_1_morde_o_grao() {
    let medidas: Vec<(f32, (f64, f64))> = [0.5f32, 0.9, 0.99, 1.0]
        .into_iter()
        .map(|s| (s, apagado(s)))
        .collect();
    for (s, (m, d)) in &medidas {
        println!("[texture eraser] Strength {s}: apagado {m:.1} ± {d:.1}");
    }
    let (q, um) = (medidas[2].1, medidas[3].1);
    assert!(q.1 > 10.0, "fixtura: a 0,99 o grão morde ({:.1})", q.1);
    assert!(
        um.1 > 0.8 * q.1 && (um.0 - q.0).abs() < 0.1 * q.0,
        "a Strength 1 não é o limite da 0,99: {um:?} contra {q:?}"
    );
}
