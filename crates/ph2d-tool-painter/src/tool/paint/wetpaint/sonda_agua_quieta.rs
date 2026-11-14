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
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    for (w, h) in [(1920usize, 1080usize), (3840, 2160)] {
        let k = w as f32 / 1920.0;
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
        t.set_paint_tool_mode("wetpaint");
        t.preset_aplicar(por_nome(&lib, "Wet Wash"))
            .expect("preset");
        t.sonda_raio(120.0 * k);
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
    t.sonda_raio(120.0 * k);
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
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let eps: f32 = std::env::var("PH2D_SONO_EPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    // fora = máscara de estágios (bits 2..5) + 64 × raio extra da ilha.
    let foras: Vec<u8> = std::env::var("PH2D_SONO_FORA")
        .ok()
        .map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect())
        .unwrap_or_else(|| vec![128]);
    let res: Vec<(usize, usize)> = match std::env::var("PH2D_SONO_RES").as_deref() {
        Ok("1080") => vec![(1920, 1080)],
        Ok("4k") => vec![(3840, 2160)],
        _ => vec![(1920, 1080), (3840, 2160)],
    };
    for fora in foras {
        for &(w, h) in &res {
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
                    g.sono.fora = fora % 64;
                    g.sono.ilha_extra = usize::from(fora / 64);
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
                "\n  {w}x{h} · ε {eps:e} · fora {fora:#08b} · passos que diferem {dif}/400 · tiles-linha saltados {salt}"
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
}

/// B13.6: ms por passo e por ESTÁGIO (0..6) a 4K com 3 traços (Wet Wash, branco), a saída do advect por linha
/// (`SAIDA_BLOCO = 0`, antes) × por bloco (`PH2D_SAIDA_BLOCO`, 16 por omissão, o produto), duas telas
/// INTERCALADAS no mesmo processo, o mínimo por janela de 40 passos; o digest de film/susp/sett/vel ao bit.
#[test]
#[ignore = "sonda: corre à mão, no perfil smoke"]
fn sonda_b13_6_saida_por_bloco() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    use std::sync::atomic::Ordering::SeqCst;
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let bloco: usize = std::env::var("PH2D_SAIDA_BLOCO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16);
    let (w, h) = match std::env::var("PH2D_SONO_RES").as_deref() {
        Ok("1080") => (1920, 1080),
        _ => (3840, 2160),
    };
    let produto = ph2d_wet_paint::solver::SAIDA_BLOCO.load(SeqCst);
    let mut ts = [tela_pintada(&lib, w, h), tela_pintada(&lib, w, h)];
    let mut ms = [Vec::new(), Vec::new()];
    let mut est = [Vec::new(), Vec::new()];
    let mut dif = 0;
    for step in 0..400u32 {
        let mut hs = [0u64; 2];
        for lado in [step as usize % 2, 1 - step as usize % 2] {
            ph2d_wet_paint::solver::SAIDA_BLOCO.store(if lado == 0 { 0 } else { bloco }, SeqCst);
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
            let mut por = [0f64; 7];
            let a = Instant::now();
            for k in 0.. {
                let b = Instant::now();
                let fim = sim_step_stage(sim, g, tuning).is_some();
                por[k.min(6)] += b.elapsed().as_secs_f64() * 1e3;
                if fim {
                    break;
                }
            }
            ms[lado].push(a.elapsed().as_secs_f64() * 1e3);
            est[lado].push(por);
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
    ph2d_wet_paint::solver::SAIDA_BLOCO.store(produto, SeqCst);
    println!("\n  B13.6 {w}x{h} · bloco {bloco} · passos que diferem {dif}/400 (tem de ser 0)");
    for j in 0..10 {
        let r = j * 40..(j + 1) * 40;
        let m = |v: &[f64]| v.iter().copied().fold(f64::MAX, f64::min);
        // Estágio k: o mínimo, entre os passos da janela que o correram (o advect é o 3).
        let e = |l: usize| -> Vec<f64> {
            (0..7)
                .map(|k| {
                    (m(&est[l][r.clone()].iter().map(|p| p[k]).collect::<Vec<_>>()) * 100.0).round()
                        / 100.0
                })
                .collect()
        };
        println!(
            "    {:>4.1} s: passo antes {:>6.2} · depois {:>6.2} ms · estágios antes {:?} · depois {:?}",
            (j + 1) as f64,
            m(&ms[0][r.clone()]),
            m(&ms[1][r.clone()]),
            e(0),
            e(1)
        );
    }
    println!(
        "  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// `n` traços longos de `preset` (faixas a cada 140 px de 1080p, cabem 6 a 4K), sessão em casa.
fn tela_n(lib: &Biblioteca, w: usize, h: usize, preset: &str, n: usize) -> PainterTool {
    let k = w as f32 / 1920.0;
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; w * h * 4], w as u32, h as u32);
    t.set_paint_tool_mode("wetpaint");
    t.preset_aplicar(por_nome(lib, preset)).expect("preset");
    t.sonda_raio(120.0 * k);
    for lane in 0..n {
        let y = (250.0 + 140.0 * lane as f32) * k;
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

/// B13.12: o escorrer em TEMPO REAL? Para 1..`PH2D_TRACOS` (6) traços de `PH2D_SONDA_PRESET` (Wet Wash) a
/// `PH2D_SONO_RES` (4K), duas telas iguais passo a passo INTERCALADAS, uma num pool de 1 fio e outra no de
/// `PH2D_FIOS` (todos): ms/passo (mín. e mediana de 120 passos = 3 s simulados), o mínimo por ESTÁGIO 0..6
/// (Amdahl: o estágio que não encolhe com os fios é o serial que sobra), o digest de film/susp/sett/vel ao
/// bit entre os dois pools (tem de ser 0 passos diferentes) e sim-s/s = min(1, 25 ms / mediana) — o worker
/// corre `STEP_S` = 25 ms de sim por passo e nunca recupera (câmera lenta = sim-s/s < 1).
#[test]
#[ignore = "sonda: corre à mão, no perfil smoke"]
fn sonda_n_tracos_em_tempo_real() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<usize>().ok());
    let preset = std::env::var("PH2D_SONDA_PRESET").unwrap_or_else(|_| "Wet Wash".into());
    let (w, h) = match std::env::var("PH2D_SONO_RES").as_deref() {
        Ok("1080") => (1920, 1080),
        _ => (3840, 2160),
    };
    let fios = env("PH2D_FIOS").unwrap_or_else(rayon::current_num_threads);
    let pools = [1, fios].map(|f| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(f)
            .build()
            .expect("pool")
    });
    const PASSOS: usize = 120;
    println!("\n  B13.12 {w}x{h} · {preset} · pools 1 × {fios} fios");
    for n in 1..=env("PH2D_TRACOS").unwrap_or(6) {
        let mut ts = [
            tela_n(&lib, w, h, &preset, n),
            tela_n(&lib, w, h, &preset, n),
        ];
        let mut ms = [Vec::new(), Vec::new()];
        let mut est = [[f64::MAX; 7]; 2];
        let mut dif = 0;
        let mut celulas = 0u64;
        for step in 0..PASSOS {
            let mut hs = [0u64; 2];
            for lado in [step % 2, 1 - step % 2] {
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
                let mut por = [0f64; 7];
                let a = Instant::now();
                pools[lado].install(|| {
                    for k in 0.. {
                        let b = Instant::now();
                        let fim = sim_step_stage(sim, g, tuning).is_some();
                        por[k.min(6)] += b.elapsed().as_secs_f64() * 1e3;
                        if fim {
                            break;
                        }
                    }
                });
                ms[lado].push(a.elapsed().as_secs_f64() * 1e3);
                for (m, p) in est[lado].iter_mut().zip(por) {
                    if p > 0.0 {
                        *m = m.min(p);
                    }
                }
                celulas = g.live_span_cells();
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
        for (l, f) in [1, fios].into_iter().enumerate() {
            let mut v = ms[l].clone();
            v.sort_by(f64::total_cmp);
            let med = v[v.len() / 2];
            let r: Vec<f64> = est[l].iter().map(|m| (m * 100.0).round() / 100.0).collect();
            println!(
                "    {n} traço(s) · {f:>2} fios · passo mín {:>6.2} med {med:>6.2} ms · sim-s/s {:.2} · estágios mín {r:?} · células vivas {celulas} · passos ≠ {dif}/{PASSOS}",
                v[0],
                (25.0 / med).min(1.0)
            );
        }
    }
    println!(
        "  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// Banda de cópia (leitura + escrita) em GB/s no pool, o mínimo de 5 cópias de 512 MiB.
fn banda_gbs(pool: &rayon::ThreadPool) -> f64 {
    use rayon::prelude::*;
    const N: usize = 512 << 20;
    let a = vec![1u8; N];
    let mut b = vec![0u8; N];
    let mut melhor = f64::MAX;
    for _ in 0..5 {
        let t = Instant::now();
        pool.install(|| {
            b.par_chunks_mut(1 << 20)
                .zip(a.par_chunks(1 << 20))
                .for_each(|(d, s)| d.copy_from_slice(s));
        });
        melhor = melhor.min(t.elapsed().as_secs_f64());
    }
    assert_eq!(b[N - 1], 1);
    2.0 * N as f64 / melhor / 1e9
}

/// Os bits de todo campo que o passo pode mexer, por célula fina (`film`, `susp`, `sett`, `susp_rgb`) e por célula de
/// fluxo (`vel`, `flow`).
fn bits(g: &ph2d_wet_paint::grid::Grid) -> (Vec<[u32; 6]>, Vec<[u32; 4]>) {
    let fina = (0..g.cells)
        .map(|i| {
            let r = g.susp_rgb[i];
            [g.film[i], g.susp[i], g.sett[i], r[0], r[1], r[2]].map(f32::to_bits)
        })
        .collect();
    let fluxo = (0..g.vel_x.len())
        .map(|i| [g.vel_x[i], g.vel_y[i], g.flow_x[i], g.flow_y[i]].map(f32::to_bits))
        .collect();
    (fina, fluxo)
}

/// B13.12 passo 1 — QUEM MEXE e se é BANDA: para 1..`PH2D_TRACOS` (6) traços (a cena da
/// [`sonda_n_tracos_em_tempo_real`]), em cada 10.º de 120 passos no pool de todos os fios: as células da faixa
/// (`live_span_cells`), as dos troços de tiles vivos (o que os estágios ANDAM), as que têm film > 0, as de fluxo com
/// vel ≠ 0, e as que o passo MUDOU algum bit (fina e fluxo, contadas também só nos passos sem secagem). Com a banda
/// de cópia medida no mesmo processo: B* = banda × ms do passo / células dos tiles = os bytes por célula que o passo
/// teria de mover para ser limitado pela banda (o fundido toca ~110–150 B/célula; B* ≫ isso ⇒ não é banda).
#[test]
#[ignore = "sonda: corre à mão, no perfil smoke"]
fn sonda_quem_mexe_e_a_banda() {
    let _ = crate::raio_da_sonda::perfil_da_sonda();
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<usize>().ok());
    let preset = std::env::var("PH2D_SONDA_PRESET").unwrap_or_else(|_| "Wet Wash".into());
    let (w, h) = match std::env::var("PH2D_SONO_RES").as_deref() {
        Ok("1080") => (1920, 1080),
        _ => (3840, 2160),
    };
    let fios = env("PH2D_FIOS").unwrap_or_else(rayon::current_num_threads);
    let pools = [1, fios].map(|f| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(f)
            .build()
            .expect("pool")
    });
    let banda = [banda_gbs(&pools[0]), banda_gbs(&pools[1])];
    println!(
        "\n  B13.12 quem mexe · {w}x{h} · {preset} · banda de cópia 1 fio {:.1} GB/s · {fios} fios {:.1} GB/s",
        banda[0], banda[1]
    );
    for n in 1..=env("PH2D_TRACOS").unwrap_or(6) {
        let mut t = tela_n(&lib, w, h, &preset, n);
        let e: &mut ph2d_wet_paint::painter::Engine =
            &mut t.paint.wetpaint.session.as_mut().expect("sessao").engine;
        let idx = e.active_layer;
        let ph2d_wet_paint::painter::Engine {
            sim,
            layers,
            tuning,
            ..
        } = &mut *e;
        let g = &mut layers[idx].grid;
        let mut ms = f64::MAX;
        let mut linhas = Vec::new();
        for step in 0..120u64 {
            let medir = step % 10 == 9;
            let antes = medir.then(|| bits(g));
            let seca = (sim.frame + 1).is_multiple_of(sim.dry_every);
            let mut por = [0f64; 7];
            let a = Instant::now();
            pools[1].install(|| {
                for k in 0.. {
                    let b = Instant::now();
                    let fim = sim_step_stage(sim, g, tuning).is_some();
                    por[k.min(6)] += b.elapsed().as_secs_f64() * 1e3;
                    if fim {
                        break;
                    }
                }
            });
            let dt = a.elapsed().as_secs_f64() * 1e3;
            ms = ms.min(dt);
            let Some((f0, q0)) = antes else { continue };
            let (f1, q1) = bits(g);
            let mudou_f = f0.iter().zip(&f1).filter(|(a, b)| a != b).count();
            let mudou_q = q0.iter().zip(&q1).filter(|(a, b)| a != b).count();
            let vel = q1
                .iter()
                .filter(|c| c[0] << 1 != 0 || c[1] << 1 != 0)
                .count();
            let film = g.film.iter().filter(|v| **v > 0.0).count();
            let (tw, on) = (g.tile_words, g.spans_enabled && g.tiles_enabled);
            let mut troco = 0u64;
            for y in g.by0.max(0)..=g.by1 {
                let (lo, hi) = g.span_x(y);
                let ts = ph2d_wet_paint::grid::faixa_tiles::tiles_row_of(&g.row_tiles, tw, on, y);
                for (a, b) in ph2d_wet_paint::grid::faixa_tiles::span_runs(ts, lo, hi) {
                    troco += u64::try_from(b - a + 1).unwrap_or(0);
                }
            }
            linhas.push((
                step,
                seca,
                por.map(|m| (m * 10.0).round() / 10.0),
                dt,
                g.live_span_cells(),
                troco,
                film,
                vel,
                mudou_f,
                mudou_q,
            ));
        }
        println!(
            "    {n} traço(s) · passo mín {ms:.2} ms ({fios} fios) · ecra {:?}",
            sim.gather_params(tuning).ecra
        );
        for (step, seca, por, dt, faixa, troco, film, vel, mf, mq) in linhas {
            let bstar = banda[1] * 1e9 * dt * 1e-3 / troco.max(1) as f64;
            println!(
                "      passo {step:>3}{} {dt:>6.2} ms · estágios {por:?} · faixa {faixa} · troços {troco} · film>0 {film} · vel≠0 {vel} · mudou fina {mf} ({:.3} da faixa) · fluxo {mq} · {:.2} ns/célula · B* {bstar:.0} B/célula",
                if seca { " seca" } else { "     " },
                mf as f64 / faixa.max(1) as f64,
                dt * 1e6 / troco.max(1) as f64,
            );
        }
    }
    println!(
        "  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
