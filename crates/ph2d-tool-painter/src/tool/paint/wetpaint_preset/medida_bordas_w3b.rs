//! §BUG-bordas-W3b (smoke do dono 27/10, foto `smoke_27_10/7.png`): traços HORIZONTAIS do Oil Bristle, do Dabber e
//! da Knife Oily com o Óleo Real ligado — os pontinhos ESCUROS soltos na borda de baixo e a orla borrada do Smooth
//! Edges. A tela FINAL (composite + a luz do relevo), como em [`super::tela_da_borda`].

use super::*;

/// A tela de três traços quase horizontais (y 90, 220, 350; uma deriva de 12 px) com o pincel `rel`, Smooth Edges
/// `suave`, papel branco (`texturizado` = Cold, Relief 0,6).
pub(super) fn tela_horizontal(rel: &str, texturizado: bool, suave: f32) -> Vec<u8> {
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&preset_rel(&b, rel)).is_empty());
    if texturizado {
        t.set_brush_paper_kind(1);
        t.set_substrate_depth(0.6);
    }
    t.set_wet_oleo_real_ligado(true);
    t.set_wet_smooth_edges(suave);
    t.set_brush_size_px(40.0);
    t.set_brush_color_srgb8([30, 70, 210]);
    for y in [90.0f32, 220.0, 350.0] {
        arrasta(&mut t, [60.0, y], [640.0, y + 12.0]);
    }
    let mut px = t.run_full().0;
    t.apply_impasto_light(
        &mut px,
        crate::Region {
            x: 0,
            y: 0,
            w: OW,
            h: OH,
        },
    );
    px
}

pub(super) const PINCEIS_W3B: [(&str, &str); 3] = [
    (
        "bristle",
        "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png",
    ),
    ("dabber", "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png"),
    (
        "knifeoily",
        "WetPainter/Oil/Knife/Knife Oily.ph2d-brush.png",
    ),
];

/// `PH2D_OLEO_PNG=<pasta>`: `hor_<pincel>_<papel>_s<suave>.ppm` — régua `oraculo/bordas_regua.py` e
/// `oraculo/pontinhos_regua.py`.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_bordas_horizontais() {
    let d = std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>");
    let so = std::env::var("PH2D_OLEO_SO").unwrap_or_default();
    for (nome, rel) in PINCEIS_W3B {
        if !so.is_empty() && !so.split(',').any(|s| s == nome) {
            continue;
        }
        for tex in [false, true] {
            for suave in [0.0f32, 0.5, 1.0] {
                let t0 = std::time::Instant::now();
                let px = tela_horizontal(rel, tex, suave);
                let mut ppm = format!("P6\n{OW} {OH}\n255\n").into_bytes();
                for p in px.as_chunks::<4>().0 {
                    ppm.extend_from_slice(&p[..3]);
                }
                let f = format!(
                    "{d}/hor_{nome}_{}_s{suave}.ppm",
                    if tex { "tex" } else { "branco" }
                );
                std::fs::write(&f, ppm).expect("grava");
                println!("[borda] {f} ({:.1} s)", t0.elapsed().as_secs_f64());
            }
        }
    }
}
