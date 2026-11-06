//! **A tinta adormecida pesa nos outros modos?** Régua intercalada ([MEDIR_VELOCIDADE](../../../../../../../docs/DevOps/MEDIR_VELOCIDADE.md)):
//! `sem` e `sem2` (a distância entre as duas é o ruído), `dorme` e `dorme+tela` (segura a imagem: a 1.ª escrita copia).
//! `cargo test --profile smoke -p ph2d-tool-painter sonda_tinta_adormecida -- --ignored --nocapture`

use super::agua_sobre_oleo_tests::{ponteiro, preset};
use super::*;
use crate::tool::paint::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use std::time::Instant;

const RODADAS: usize = 9;
const NOMES: [&str; 4] = ["sem", "sem2", "dorme", "dorme+tela"];
const MEIOS: [(&str, PaintMedia); 3] = [
    ("Digital", PaintMedia::Digital),
    ("Watercolor", PaintMedia::Watercolor),
    ("Impasto", PaintMedia::Impasto),
];

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg").unwrap_or_default()
}

/// A tela com óleo em 14 passadas (a sessão viva e cheia), já fora do Wet.
fn tela_pintada(w: usize, h: usize, variante: usize) -> (PainterTool, f64, usize) {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(60.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    for p in 0..14 {
        let y = (p as f32 + 0.5) * h as f32 / 14.0;
        t.on_canvas_pointer(ponteiro(20.0, y, PointerPhase::Down));
        for k in 1..=60 {
            let x = 20.0 + k as f32 * (w as f32 - 40.0) / 60.0;
            t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(ponteiro(w as f32 - 20.0, y, PointerPhase::Up));
        t.on_tick(1000.0 / 60.0);
    }
    let (mut ms, mut bytes) = (0.0, 0);
    if variante >= 2 {
        bytes = t.wet_bytes_retidos();
        let t0 = Instant::now();
        assert!(t.wet_adormece());
        ms = t0.elapsed().as_secs_f64() * 1e3;
    } else {
        t.wetpaint_end_session(); // `sem`: a tinta acabou, como antes de ela adormecer
    }
    t.set_paint_media(PaintMedia::Digital);
    assert_eq!(
        !t.paint.wetpaint.adormecidas.is_empty(),
        variante >= 2,
        "CONTROLO: a variante é a que diz ser"
    );
    (t, ms, bytes)
}

/// Um traço: `(pen-down, o traço inteiro com os tiques, em ms)`.
fn traco(t: &mut PainterTool, w: usize, y: f32) -> (f64, f64) {
    let t0 = Instant::now();
    t.on_canvas_pointer(ponteiro(40.0, y, PointerPhase::Down));
    let down = t0.elapsed().as_secs_f64() * 1e3;
    for k in 1..=30 {
        let x = 40.0 + k as f32 * (w as f32 / 2.0) / 30.0;
        t.on_canvas_pointer(ponteiro(x, y, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(w as f32 / 2.0 + 40.0, y, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
    (down, t0.elapsed().as_secs_f64() * 1e3)
}

fn min_med(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

fn mede(w: usize, h: usize) {
    println!("\n== {w}×{h} · loadavg início: {}", loadavg().trim());
    let mut ts: Vec<PainterTool> = Vec::new();
    for (v, nome) in NOMES.iter().enumerate() {
        let t0 = Instant::now();
        let (t, ms, bytes) = tela_pintada(w, h, v);
        println!(
            "{nome:>11}: pintar {:.1} s · adormecer {ms:.2} ms · retém {} MB",
            t0.elapsed().as_secs_f64(),
            bytes / 1_000_000
        );
        ts.push(t);
    }
    // meio -> variante -> amostras
    let mut down = vec![vec![Vec::new(); NOMES.len()]; MEIOS.len()];
    let mut todo = vec![vec![Vec::new(); NOMES.len()]; MEIOS.len()];
    let mut tique = vec![Vec::new(); NOMES.len()];
    for r in 0..RODADAS {
        for (m, (_, meio)) in MEIOS.iter().enumerate() {
            for i in 0..NOMES.len() {
                let v = (i + r) % NOMES.len();
                let t = &mut ts[v];
                t.set_paint_media(*meio);
                t.set_brush_size_px(40.0);
                // A variante que segura a imagem: outra referência à tela de agora, a cada traço.
                let presa = (v == 3).then(|| Arc::clone(&t.canvas_rgba));
                let y = 30.0 + (r * MEIOS.len() + m) as f32 * (h as f32 - 60.0) / 40.0;
                let (d, a) = traco(t, w, y);
                drop(presa);
                down[m][v].push(d);
                todo[m][v].push(a);
            }
        }
        for i in 0..NOMES.len() {
            let v = (i + r) % NOMES.len();
            let t0 = Instant::now();
            for _ in 0..120 {
                ts[v].on_tick(1000.0 / 60.0);
            }
            tique[v].push(t0.elapsed().as_secs_f64() * 1e6 / 120.0);
        }
    }
    for (m, (nome, _)) in MEIOS.iter().enumerate() {
        println!("-- {nome}: mínimo (mediana) em ms");
        for v in 0..NOMES.len() {
            let (dm, dd) = min_med(&mut down[m][v]);
            let (am, ad) = min_med(&mut todo[m][v]);
            println!(
                "{:>11}: pen-down {dm:8.3} ({dd:8.3}) · traço {am:9.2} ({ad:9.2})",
                NOMES[v]
            );
        }
    }
    println!("-- tique ocioso: mínimo (mediana) em µs");
    for v in 0..NOMES.len() {
        let (a, b) = min_med(&mut tique[v]);
        println!("{:>11}: {a:8.2} ({b:8.2})", NOMES[v]);
    }
    println!("loadavg fim: {}", loadavg().trim());
}

/// O peso e o tempo da compressão, ao vivo: Dabber cheio, e o Wet Round por cima.
#[test]
#[ignore = "sonda da tinta adormecida"]
fn sonda_compressao_adormecida() {
    for (w, h) in [(1920usize, 1080usize), (4096, 2560)] {
        let (mut t, _, _) = tela_pintada(w, h, 2);
        let a = t
            .paint
            .wetpaint
            .adormecidas
            .first()
            .expect("adormecida")
            .clone();
        let (cru, grade, fotos, registos) = {
            let s = a.sess.lock().expect("trinco");
            let g: usize = s.engine.layers.iter().map(|l| l.grid.bytes_retidos()).sum();
            let m = s.engine.bytes_retidos();
            let f: usize = s.bytes_retidos()
                - m
                - s.planos.pigment.len()
                - s.planos.sombra.len()
                - s.planos.grao.len()
                - s.planos.suave.len();
            (s.bytes_retidos(), g, f, m - g)
        };
        let rss = || -> usize {
            std::fs::read_to_string("/proc/self/status")
                .unwrap_or_default()
                .lines()
                .find_map(|l| {
                    l.strip_prefix("VmRSS:")
                        .map(|r| r.trim().trim_end_matches(" kB").trim().parse().unwrap_or(0))
                })
                .unwrap_or(0)
                / 1024
        };
        println!("RSS antes de comprimir: {} MB", rss());
        let t0 = Instant::now();
        a.encolhe_ja();
        let comprimir = t0.elapsed().as_secs_f64() * 1e3;
        let z = a.bytes.load(std::sync::atomic::Ordering::Relaxed);
        println!("RSS comprimida: {} MB", rss());
        drop(a);
        let t0 = Instant::now();
        t.set_paint_media(PaintMedia::WetPaint);
        let acordar = t0.elapsed().as_secs_f64() * 1e3;
        println!(
            "{w}x{h}: cru {} MB (grade {} · fotos {} · histórico e papel {}) → {} MB · comprimir {comprimir:.0} ms · acordar {acordar:.0} ms",
            cru >> 20,
            grade >> 20,
            fotos >> 20,
            registos >> 20,
            z >> 20
        );
    }
    println!("loadavg fim: {}", loadavg().trim());
}

#[test]
#[ignore = "sonda da tinta adormecida"]
fn sonda_tinta_adormecida_nos_outros_modos() {
    mede(1920, 1080);
    mede(4096, 2560);
}
