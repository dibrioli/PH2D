//! O custo do Wet Wash Mix (queda de fps, 10/11): sem a cópia do véu × o de antes × a mistura `Off`, no MESMO processo,
//! intercalados, o MÍNIMO de 3 (`docs/DevOps/MEDIR_VELOCIDADE.md`); e quanto a tela muda (Δ por canal).
//! `cargo test --profile smoke -p ph2d-tool-painter --test it sonda_custo_do_veu -- --ignored --nocapture`
//! Grava `target/sonda-veu/<pincel>_<tamanho>_<variante>.ppm`.

use crate::familias::csp_secos;
use crate::presets_de_fabrica::preset_do_meio;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{ModoMisturaOpaca, PainterTool};
use std::time::Instant;

const W: usize = 1400;
const H: usize = 360;
const Y: f32 = 180.0;
const EVENTOS: usize = 200;

#[derive(Clone, Copy, Debug)]
enum Variante {
    Antigo,
    SoACopia,
    Off,
}

fn cena(nome: &str, tamanho: f32, v: Variante) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    let r = csp_secos::MEIOS
        .iter()
        .chain(crate::familias::csp_tinta_grossa::MEIOS.iter())
        .find(|r| r.nome == nome)
        .unwrap_or_else(|| panic!("receita {nome}"));
    assert_eq!(
        t.preset_aplicar(&preset_do_meio(r)),
        Ok(Vec::new()),
        "{nome}"
    );
    t.set_brush_size_px(tamanho);
    t.set_brush_color_srgb8([0, 40, 220]);
    match v {
        Variante::Antigo => t.set_mistura_salta_a_copia_do_veu(false),
        Variante::SoACopia => t.set_mistura_salta_a_copia_do_veu(true),
        Variante::Off => t.set_brush_mistura_modo(ModoMisturaOpaca::Off),
    }
    t
}

/// O traço de 1 200 px em 200 eventos; devolve os segundos de Down→Up.
fn traco(t: &mut PainterTool) -> f64 {
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let t0 = Instant::now();
    t.on_canvas_pointer(cp([100.0, Y], PointerPhase::Down));
    for k in 1..=EVENTOS {
        let u = k as f32 / EVENTOS as f32;
        t.on_canvas_pointer(cp([100.0 + 1200.0 * u, Y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([1300.0, Y], PointerPhase::Up));
    t0.elapsed().as_secs_f64()
}

fn grava(nome: &str, p: &[u8]) {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/sonda-veu");
    let _ = std::fs::create_dir_all(&d);
    let mut b = format!("P6\n{W} {H}\n255\n").into_bytes();
    for px in p.chunks_exact(4) {
        b.extend_from_slice(&px[..3]);
    }
    let _ = std::fs::write(d.join(format!("{nome}.ppm")), b);
}

fn delta(a: &[u8], b: &[u8]) -> (f64, u8, usize) {
    let (mut s, mut m, mut n) = (0u64, 0u8, 0usize);
    for (x, y) in a.iter().zip(b) {
        let d = x.abs_diff(*y);
        s += u64::from(d);
        m = m.max(d);
        n += usize::from(d > 2);
    }
    (s as f64 / a.len() as f64, m, n / 4)
}

#[test]
#[ignore = "sonda de custo"]
fn sonda_custo_do_veu() {
    let casos: &[(&str, f32)] = &[
        ("Wet Wash Mix", 30.0),
        ("Wet Wash Mix", 80.0),
        ("Round Mixing", 80.0),
    ];
    let variantes = [Variante::Antigo, Variante::SoACopia, Variante::Off];
    const N: usize = 3;
    for &(nome, tam) in casos {
        let mut tempos = [f64::INFINITY; N];
        let mut telas: [Vec<u8>; N] = Default::default();
        for volta in 0..3 {
            // Blocos rodados: a ordem das variantes muda a cada volta.
            for j in 0..N {
                let vi = (j + volta) % N;
                let mut t = cena(nome, tam, variantes[vi]);
                let s = traco(&mut t);
                tempos[vi] = tempos[vi].min(s);
                if volta == 0 {
                    telas[vi] = t.run_full().0;
                }
            }
        }
        for (vi, v) in variantes.iter().enumerate() {
            let (m, x, n) = delta(&telas[vi], &telas[0]);
            eprintln!(
                "[veu] {nome} {tam} px {v:?}: {:.3} s ({:.2}× o antigo) · Δ ao antigo médio {m:.3} máx {x} Δ>2 {n} px",
                tempos[vi],
                tempos[0] / tempos[vi]
            );
            grava(
                &format!("{}_{tam}_{v:?}", nome.replace(' ', "_")),
                &telas[vi],
            );
        }
    }
}
