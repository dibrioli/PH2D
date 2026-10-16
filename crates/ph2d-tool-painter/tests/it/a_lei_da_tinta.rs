//! **A lei da tinta por pincel** (prompt 42, doc [42_medicao](../../../../docs/Painter/NewFeaturesWetPaint/42_medicao_a_lei_da_tinta.md)).
//!
//! O portão de IDENTIDADE nasceu antes da lei: a cena da bancada (`sonda_gestos::pinta_cena_ate`, o
//! cruzamento incluído) de cada pincel de fábrica, ao byte, como o motor a pintava em `f46fcffb2a` — e duas
//! com as caixas da OBRA ligadas. Um pincel sem Glaze pinta a lei da obra, ao byte.

use super::midias_de_fabrica::de_fabrica;
use super::sonda_gestos::{fnv1a, pinta_cena_ate, tela};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::ids;

/// O raio das cenas de identidade: o do recto da bancada a meia escala dos presets.
const RAIO_DA_CENA: f32 = 18.0;

/// `(preset, caixas da obra ligadas, fnv1a da cena)` — pinados em `f46fcffb2a`.
#[rustfmt::skip]
const CENAS: [(&str, bool, u64); 14] = [
    ("Wet Round", false, 0xfce64c9627dc5bc3),
    ("Wet Wash", false, 0x782a0a8a8ed9c439),
    ("Wet Dry Brush", false, 0x1a361614679dde9a),
    ("Wet Drip", false, 0x8159963aa484f688),
    ("Wet Liner", false, 0x27defb305741fea9),
    ("Oil Bristle", false, 0x53cf6eb66d3857e1),
    ("Acrylic Round", false, 0xfea6bfd5e31bf152),
    ("Express Oil", false, 0x6e348caa0df4a42e),
    ("Graphite 8B", false, 0x55368fbf0685ad46),
    ("Charcoal", false, 0x3b6500d86d772fd6),
    ("Soft Pastel", false, 0xf99dd413f5f0218e),
    ("Oil Pastel", false, 0x235fe3c132334588),
    ("Wet Round", true, 0x44016d14e50f2119),
    ("Wet Liner", true, 0x7a276b4a53ec4531),
];

fn cena(preset: &str, obra: bool) -> u64 {
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&de_fabrica(preset)).is_empty());
    t.set_brush_size_px(RAIO_DA_CENA);
    if obra {
        for id in [ids::WET_TUNING_KM_MIXING, ids::WET_TUNING_KM_GLAZE] {
            t.handle_panel_event(PanelEvent::Click(id));
        }
    }
    let (px, _, _) = pinta_cena_ate(&mut t, 480);
    fnv1a(&px)
}

#[test]
fn sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes() {
    let hoje: Vec<u64> = std::thread::scope(|s| {
        let h: Vec<_> = CENAS
            .iter()
            .map(|&(p, obra, _)| s.spawn(move || cena(p, obra)))
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mudaram: Vec<String> = CENAS
        .iter()
        .zip(&hoje)
        .filter(|((_, _, antes), agora)| antes != *agora)
        .map(|((p, obra, antes), agora)| {
            format!("(\"{p}\", {obra}, {agora:#018x}), // era {antes:#018x}")
        })
        .collect();
    assert!(
        mudaram.is_empty(),
        "a cena mudou — a lei da obra deixou de ser a de antes:\n{}",
        mudaram.join("\n")
    );
}
