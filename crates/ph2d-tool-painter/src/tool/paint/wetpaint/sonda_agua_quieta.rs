//! **Sonda (ignorada) B13.3 — quanto da água ainda MEXE depois de soltar** (gate b do B13: se a fracção viva de tiles
//! 64² não cai abaixo de ~0,5 em 4 s, dormir não vale). Wet Wash na tela branca, 3 traços longos; depois do Up o passo
//! do produto (`sim_step_stage`, 40/s) sem pincel. Tile quieto = max|vel| e max|Δfilm| ≤ ε nos últimos 20 passos;
//! molhado = film > 0. Imprime também o ms do passo (o custo da tela parada). `PH2D_SONDA_TELA=LxA`.

use super::sonda_bordas_longe::por_nome;
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_wet_paint::sim::sim_step_stage;
use std::time::Instant;

const T: usize = 64;
const EPS: [f32; 4] = [0.0, 1e-5, 1e-4, 1e-3];
const STREAK: u32 = 20;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 0.8,
        tilt: [0.0, 0.0],
        phase,
    }
}

#[test]
#[ignore = "sonda: corre à mão, no perfil smoke"]
fn sonda_agua_quieta_depois_de_soltar() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    for (w, h) in [(1920usize, 1080usize), (3840, 2160)] {
        let k = w as f32 / 1920.0;
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
        t.set_paint_tool_mode("wetpaint");
        t.preset_aplicar(por_nome(&lib, "Wet Wash"))
            .expect("preset");
        t.set_brush_size_px(120.0 * k);
        for lane in 0..3 {
            let y = (300.0 + 200.0 * lane as f32) * k;
            t.on_canvas_pointer(cp([200.0 * k, y], PointerPhase::Down));
            for s in 1..=60 {
                t.on_canvas_pointer(cp([(200.0 + 25.0 * s as f32) * k, y], PointerPhase::Move));
                t.on_tick(1000.0 / 60.0);
                let _ = t.take_preview_arc();
            }
            t.on_canvas_pointer(cp([1700.0 * k, y], PointerPhase::Up));
        }
        t.wet_bring_home();
        let e: &mut ph2d_wet_paint::painter::Engine =
            &mut t.paint.wetpaint.session.as_mut().expect("sessao").engine;
        let idx = e.active_layer;
        let (tw, th) = (w.div_ceil(T), h.div_ceil(T));
        let nt = tw * th;
        let mut prev = e.layers[idx].grid.film.clone();
        let mut streak = vec![[0u32; 4]; nt];
        let mut streak_v = vec![[0u32; 4]; nt];
        let mut estagio = [0f64; 7];
        let mut desde = 0u32;
        let mut ms = Vec::new();
        println!("\n  {w}x{h} · tiles {tw}x{th}");
        for step in 1..=400u32 {
            let a = Instant::now();
            let ph2d_wet_paint::painter::Engine {
                sim,
                layers,
                tuning,
                ..
            } = &mut *e;
            let mut k = 0usize;
            loop {
                let b = Instant::now();
                let fim = sim_step_stage(sim, &mut layers[idx].grid, tuning).is_some();
                estagio[k.min(6)] += b.elapsed().as_secs_f64() * 1e3;
                k += 1;
                if fim {
                    break;
                }
            }
            ms.push(a.elapsed().as_secs_f64() * 1e3);
            desde += 1;
            let g = &e.layers[idx].grid;
            let mut dmax = vec![0f32; nt];
            let mut vmax = vec![0f32; nt];
            let mut live = vec![false; nt];
            for y in 0..g.h {
                for x in 0..g.w {
                    let i = (y + 1) * g.s + x + 1;
                    let tt = (y / T) * tw + x / T;
                    let f = g.film[i];
                    live[tt] |= f > 0.0;
                    dmax[tt] = dmax[tt].max((f - prev[i]).abs());
                }
            }
            let fg = &g.flow;
            for fy in 0..fg.h {
                for fx in 0..fg.w {
                    let i = (fy + 1) * fg.s + fx + 1;
                    let tt = ((fy * fg.rf).min(h - 1) / T) * tw + (fx * fg.rf).min(w - 1) / T;
                    vmax[tt] = vmax[tt].max(g.vel_x[i].abs().max(g.vel_y[i].abs()));
                }
            }
            for tt in 0..nt {
                for (j, &eps) in EPS.iter().enumerate() {
                    let q = dmax[tt] <= eps && vmax[tt] <= eps;
                    streak[tt][j] = if q { streak[tt][j] + 1 } else { 0 };
                    let qv = vmax[tt] <= eps;
                    streak_v[tt][j] = if qv { streak_v[tt][j] + 1 } else { 0 };
                }
            }
            prev.copy_from_slice(&g.film);
            if [40, 80, 160, 400].contains(&step) {
                let nl = live.iter().filter(|&&l| l).count();
                let fr: Vec<String> = (0..4)
                    .map(|j| {
                        let mexe = (0..nt)
                            .filter(|&tt| live[tt] && streak[tt][j] < STREAK)
                            .count();
                        format!("{:e}: {:.3}", EPS[j], mexe as f64 / nl.max(1) as f64)
                    })
                    .collect();
                let fv: Vec<String> = (0..4)
                    .map(|j| {
                        let mexe = (0..nt)
                            .filter(|&tt| live[tt] && streak_v[tt][j] < STREAK)
                            .count();
                        format!("{:e}: {:.3}", EPS[j], mexe as f64 / nl.max(1) as f64)
                    })
                    .collect();
                let q = |v: &[f32]| {
                    let mut x: Vec<f32> = (0..nt).filter(|&tt| live[tt]).map(|tt| v[tt]).collect();
                    x.sort_by(f32::total_cmp);
                    if x.is_empty() {
                        return [0.0; 3];
                    }
                    [x[x.len() / 10], x[x.len() / 2], x[x.len() * 9 / 10]]
                };
                println!(
                    "    so vel: {} · |vel| p10/50/90 {:?} · |dfilm| p10/50/90 {:?}",
                    fv.join(" · "),
                    q(&vmax),
                    q(&dmax)
                );
                println!(
                    "    ms/passo por estagio 0..6 (media desde o ultimo ponto): {:?}",
                    estagio.map(|v| (v / f64::from(desde) * 100.0).round() / 100.0)
                );
                estagio = [0.0; 7];
                desde = 0;
                let mut ult: Vec<f64> = ms[ms.len() - 40..].to_vec();
                ult.sort_by(f64::total_cmp);
                println!(
                    "  {:>4.1} s · molhados {nl}/{nt} · passo min {:.2} med {:.2} ms · viva por eps: {}",
                    f64::from(step) / 40.0,
                    ult[0],
                    ult[20],
                    fr.join(" · ")
                );
            }
        }
    }
    println!(
        "  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

fn tela_pintada(lib: &Biblioteca, w: usize, h: usize) -> PainterTool {
    let k = w as f32 / 1920.0;
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.preset_aplicar(por_nome(lib, "Wet Wash")).expect("preset");
    t.set_brush_size_px(120.0 * k);
    for lane in 0..3 {
        let y = (300.0 + 200.0 * lane as f32) * k;
        t.on_canvas_pointer(cp([200.0 * k, y], PointerPhase::Down));
        for s in 1..=60 {
            t.on_canvas_pointer(cp([(200.0 + 25.0 * s as f32) * k, y], PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp([1700.0 * k, y], PointerPhase::Up));
    }
    t.wet_bring_home();
    t
}

/// B13.3: a mesma tela com o sono ligado × desligado, passo a passo INTERCALADO no mesmo processo: o digest de
/// film/susp/sett/vel ao bit (ε = `PH2D_SONO_EPS`, 0 por omissão) e o ms do passo (mínimo por janela de 40).
#[test]
#[ignore = "sonda: corre à mão, no perfil smoke"]
fn sonda_sono_ligado_e_desligado() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let eps: f32 = std::env::var("PH2D_SONO_EPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    for (w, h) in [(1920usize, 1080usize), (3840, 2160)] {
        let mut ts = [tela_pintada(&lib, w, h), tela_pintada(&lib, w, h)];
        let mut ms = [Vec::new(), Vec::new()];
        let mut dif = 0;
        for step in 0..400u32 {
            let mut hs = [0u64; 2];
            for lado in [step as usize % 2, 1 - step as usize % 2] {
                let e: &mut ph2d_wet_paint::painter::Engine = &mut ts[lado]
                    .paint
                    .wetpaint
                    .session
                    .as_mut()
                    .expect("sessao")
                    .engine;
                let idx = e.active_layer;
                let ph2d_wet_paint::painter::Engine {
                    sim,
                    layers,
                    tuning,
                    ..
                } = &mut *e;
                let g = &mut layers[idx].grid;
                g.sono.ligado = lado == 0;
                g.sono.eps = eps;
                let a = Instant::now();
                while sim_step_stage(sim, g, tuning).is_none() {}
                ms[lado].push(a.elapsed().as_secs_f64() * 1e3);
                let mut x = 0xcbf2_9ce4_8422_2325u64;
                for c in [&g.film, &g.susp, &g.sett, &g.vel_x, &g.vel_y] {
                    for v in c.iter() {
                        x = (x ^ u64::from(v.to_bits())).wrapping_mul(0x0100_0000_01b3);
                    }
                }
                hs[lado] = x;
            }
            dif += usize::from(hs[0] != hs[1]);
        }
        let e: &ph2d_wet_paint::painter::Engine = &ts[0]
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("sessao")
            .engine;
        let salt = e.layers[e.active_layer].grid.sono.saltados;
        println!(
            "\n  {w}x{h} · ε {eps:e} · passos que diferem {dif}/400 · tiles-linha saltados {salt}"
        );
        for j in 0..10 {
            let m = |v: &[f64]| {
                v[j * 40..(j + 1) * 40]
                    .iter()
                    .copied()
                    .fold(f64::MAX, f64::min)
            };
            println!(
                "    {:>4.1} s: com sono {:>6.2} ms · sem {:>6.2} ms (mín. de 40)",
                (j + 1) as f64,
                m(&ms[0]),
                m(&ms[1])
            );
        }
    }
}
