//! SONDA (ignored): o custo do PEN-UP numa tela 4K (dono 10/11: «no mouse up em imagem 4k há um travamento mesmo se o
//! traço for pequeno»). Um traço CURTO (6 eventos de 10 px) de cada pincel, o Down, os Moves e o Up cronometrados à parte,
//! o mínimo de 3 traços; a tela 4K × a 1024² no MESMO processo — o que escala com a TELA e não com o traço aparece aqui.
//! `PH2D_SONDA_PRESETS="Wet Wash Mix|Round Mixing"` (omissão: uma amostra de cada meio).
//! `cargo test -p ph2d-tool-painter --profile smoke --test it sonda_pen_up_4k -- --ignored --nocapture`

use crate::exame_bancada::pinceis;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;
use std::time::Instant;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// `[down, moves, up, drenagem depois do up]` em ms.
fn traco_curto(t: &mut PainterTool, y: f32) -> [f64; 4] {
    let ms = |t0: Instant| t0.elapsed().as_secs_f64() * 1e3;
    let mut x = 200.0f32;
    let t0 = Instant::now();
    t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
    let down = ms(t0);
    let t0 = Instant::now();
    for _ in 0..6 {
        x += 10.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    let _ = t.take_preview_dirty();
    let moves = ms(t0);
    let t0 = Instant::now();
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
    let up = ms(t0);
    let t0 = Instant::now();
    let _ = t.take_preview_dirty();
    [down, moves, up, ms(t0)]
}

#[test]
#[ignore = "sonda de custo"]
fn sonda_pen_up_4k() {
    let _ = ph2d_tool_painter::raio_da_sonda::perfil_da_sonda();
    let spec = std::env::var("PH2D_SONDA_PRESETS")
        .unwrap_or_else(|_| "Wet Wash Mix|Round Mixing|Hard Round|Filbert|Wet Round".into());
    let telas = [(1024u32, 1024u32), (3840, 2160)];
    for p in pinceis(&spec) {
        for &(w, h) in &telas {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
            p.aplica(&mut t, false);
            t.set_compor_por_quadro(true);
            let p1 = traco_curto(&mut t, 100.0);
            eprintln!(
                "[pen-up] {:<16} {w}×{h} 1.º traço: down {:7.2} · moves {:7.2} · UP {:7.2} ms",
                p.nome, p1[0], p1[1], p1[2]
            );
            let mut min = [f64::INFINITY; 4];
            for k in 0..3 {
                let v = traco_curto(&mut t, 200.0 + 120.0 * k as f32);
                for (m, x) in min.iter_mut().zip(v) {
                    *m = m.min(x);
                }
            }
            // A releitura da folha de animação ~30 quadros depois do Up (`sheet_cache::follow_live` →
            // `imagem_aplicada`): a imagem inteira com a luz do relevo.
            let mut leitura = f64::INFINITY;
            for _ in 0..3 {
                let t0 = Instant::now();
                let img = t.imagem_aplicada();
                leitura = leitura.min(t0.elapsed().as_secs_f64() * 1e3);
                assert!(img.is_some());
            }
            eprintln!(
                "[pen-up] {:<16} {w}×{h}: releitura da folha {leitura:7.2} ms",
                p.nome
            );
            eprintln!(
                "[pen-up] {:<16} {w}×{h}: down {:7.2} · moves {:7.2} · UP {:7.2} · drenagem {:6.2} ms",
                p.nome, min[0], min[1], min[2], min[3]
            );
        }
    }
}
