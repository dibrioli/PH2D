//! Sonda (08/11, lote B CSP): os pincéis de `familias/csp_secos.rs` pela RECEITA (o preset que o gerador gravaria,
//! conferido), na cena de zigue-zagues dos blocos secos do oráculo (`sonda_csp_comparacao::sonda_wet_blend_vs_csp`).
//! `PH2D_CSP_CENA=<pasta com base_celula.rgba e zz_*.txt>`; RGBA cru em `<pasta>/out/lb_<nome>.rgba`. Corre à mão.

use crate::familias::csp_secos::MEIOS;
use crate::presets_de_fabrica::{confere_receita_do_meio, preset_do_meio};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;
use std::path::PathBuf;

const W: usize = 420;
const H: usize = 280;

fn le(d: &std::path::Path, nome: &str) -> Vec<[f32; 2]> {
    std::fs::read_to_string(d.join(format!("{nome}.txt")))
        .expect("cena")
        .split_whitespace()
        .map(|p| {
            let (x, y) = p.split_once(',').expect("x,y");
            [x.parse().expect("x"), y.parse().expect("y")]
        })
        .collect()
}

fn traca(t: &mut PainterTool, s: &[[f32; 2]]) {
    let cp = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(s[0], PointerPhase::Down));
    for &p in &s[1..] {
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp(*s.last().expect("ponto"), PointerPhase::Up));
    t.on_tick(8.0);
}

#[test]
#[ignore = "sonda"]
fn sonda_csp_lote_b() {
    let d = PathBuf::from(std::env::var("PH2D_CSP_CENA").expect("PH2D_CSP_CENA"));
    let o = d.join("out");
    std::fs::create_dir_all(&o).expect("out");
    let base = std::fs::read(d.join("base_celula.rgba")).expect("base");
    assert_eq!(base.len(), W * H * 4);
    // Diâmetro na cena: o de fábrica do CSP limitado para caber (Splash 300 → 120).
    let so = std::env::var("PH2D_LB_SO").ok();
    for r in MEIOS {
        if so
            .as_deref()
            .is_some_and(|s| !s.split(',').any(|n| n == r.nome))
        {
            continue;
        }
        let p = preset_do_meio(r);
        confere_receita_do_meio(r, &p);
        let mut t = PainterTool::default();
        t.set_source(base.clone(), W as u32, H as u32);
        assert_eq!(t.preset_aplicar(&p), Ok(Vec::new()));
        let diam = r.size_px.unwrap_or(30.0).min(120.0) as f32;
        t.sonda_raio(diam / 2.0);
        for (nome, cor) in [
            ("zz_y", [240u8, 200, 0]),
            ("zz_b", [0, 70, 190]),
            ("zz_r", [200, 20, 20]),
            ("zz_g", [78, 160, 60]),
            ("zz_x", [200, 20, 20]),
        ] {
            t.set_brush_color_srgb8(cor);
            traca(&mut t, &le(&d, nome));
        }
        // Um traço recto no branco de baixo (a largura e a textura sozinhas).
        t.set_brush_color_srgb8([0, 70, 190]);
        traca(
            &mut t,
            &(0..=80)
                .map(|i| [160.0 + 3.0 * i as f32, 250.0])
                .collect::<Vec<_>>(),
        );
        let slug = r.nome.replace(' ', "_");
        std::fs::write(o.join(format!("lb_{slug}.rgba")), t.run_full().0).expect("grava");
        eprintln!("[lb] {} ok", r.nome);
    }
}
