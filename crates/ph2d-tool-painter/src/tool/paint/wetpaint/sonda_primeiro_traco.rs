//! **Sonda (ignorada): o travamento do começo do 1.º traço** (dono 05/11, Watercolor Basic e Wet Wash). Cronometra cada
//! evento do 1.º e do 2.º traço, com o quadro composto: ms do maior e dos 5 primeiros. `PH2D_SONDA_TELA=LxA`.

use super::offthread::otica::forca_no_teste;
use super::sonda_bordas_longe::por_nome;
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use std::time::Instant;

fn traco(t: &mut PainterTool, y: f32) -> Vec<f64> {
    let mut ms = Vec::new();
    let mut ev = |t: &mut PainterTool, x: f32, phase| {
        let t0 = Instant::now();
        t.on_canvas_pointer(CanvasPointer {
            pos: [x, y],
            pressure: 0.8,
            tilt: [0.0, 0.0],
            phase,
        });
        let p = t0.elapsed().as_secs_f64() * 1e3;
        t.on_tick(1000.0 / 60.0);
        let q = t0.elapsed().as_secs_f64() * 1e3;
        let _ = t.take_preview_arc();
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
        if ms.len() <= 3 && std::env::var("PH2D_SONDA_PARTES").is_ok() {
            println!(
                "  ev{} ponteiro {p:.1} tique {:.1} saída {:.1}",
                ms.len() - 1,
                q - p,
                ms[ms.len() - 1] - q
            );
        }
    };
    ev(t, 100.0, PointerPhase::Down);
    for k in 1..=30 {
        ev(t, 100.0 + 12.0 * k as f32, PointerPhase::Move);
    }
    ev(t, 460.0, PointerPhase::Up);
    ms
}

#[test]
#[ignore = "sonda: corre à mão, no perfil smoke"]
fn sonda_primeiro_traco() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let (w, h): (usize, usize) = std::env::var("PH2D_SONDA_TELA")
        .ok()
        .and_then(|v| {
            let (a, b) = v.split_once('x')?;
            Some((a.parse().ok()?, b.parse().ok()?))
        })
        .unwrap_or((1920, 1080));
    forca_no_teste(Some(true));
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let nomes = [
        "Watercolor Basic",
        "Wet Wash",
        "Wet Round",
        "Wet Liner",
        "Marker Bullet",
    ];
    for n in nomes
        .iter()
        .filter(|n| so.as_deref().is_none_or(|s| s == **n))
    {
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
        t.set_paint_tool_mode("wetpaint");
        t.preset_aplicar(por_nome(&lib, n)).expect("preset");
        t.set_brush_size_px(60.0);
        for _ in 0..3 {
            t.on_tick(1000.0 / 60.0);
        }
        if std::env::var("PH2D_SONDA_NASCE").is_ok() {
            let t0 = Instant::now();
            t.ensure_wet_session();
            println!(
                "  nascer a sessão {:.1} ms",
                t0.elapsed().as_secs_f64() * 1e3
            );
            for _ in 0..3 {
                t.on_tick(1000.0 / 60.0);
                let _ = t.take_preview_arc();
            }
        }
        let (um, dois) = (traco(&mut t, 200.0), traco(&mut t, 500.0));
        // Para o perfil: só pen-downs, com o motor na mão.
        let toques: usize = std::env::var("PH2D_SONDA_TOQUES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        for k in 0..toques {
            let p = |phase| CanvasPointer {
                pos: [100.0 + (k % 40) as f32 * 20.0, 700.0],
                pressure: 0.8,
                tilt: [0.0, 0.0],
                phase,
            };
            t.on_canvas_pointer(p(PointerPhase::Down));
            t.on_canvas_pointer(p(PointerPhase::Up));
        }
        let fmt = |v: &[f64]| {
            let c: Vec<String> = v.iter().take(5).map(|x| format!("{x:.1}")).collect();
            let mx = v.iter().copied().fold(0.0, f64::max);
            let mut o = v.to_vec();
            o.sort_by(f64::total_cmp);
            format!(
                "[{}] máx {mx:.1} mediana {:.1}",
                c.join(" "),
                o[o.len() / 2]
            )
        };
        println!("TRACO\t{n}\t1.º {}\t2.º {}", fmt(&um), fmt(&dois));
    }
    forca_no_teste(None);
}
