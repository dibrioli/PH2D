//! **R3 (31/10) — a latência de PINTAR, no produto**: quanto custa cada evento de ponteiro de um traço,
//! por motor (Digital · Wet · Wet + Óleo Real · as facas do Óleo de fábrica) e por raio, numa tela 4K
//! (3840×2160, a do oráculo `D_rebelle_4k.md`). Entra só pelas portas públicas que o app usa
//! (`set_paint_media`, `set_wet_oleo_real_ligado`, `wet_preset_aplicar`, `on_canvas_pointer`), compilada
//! sem `cfg(test)`. A água fica no relógio FIXO e sem `on_tick` — o que se mede é o dab, determinista,
//! e o hash da tela no fim prova que um corte não moveu um bit.
//!
//! Régua: `docs/DevOps/MEDIR_VELOCIDADE.md` — as variantes vivem no MESMO processo, cada uma na sua tela,
//! intercaladas em rondas de ordem rodada; o resumo é o MÍNIMO (mediana ao lado).
//! O traço `facas` é o do oráculo: as 10 facas (`t_facas.jsonl`: `x ∈ {70, 650}`, `y = 90 + 150·k`,
//! 150 movimentos de 3 px, `size_px 90`) ×2, numa tela nova por ronda.
//! Doc: `docs/Painter/49_realshader/R3_PINTAR_RELATORIO.md`.
//! ```text
//! CARGO_PROFILE_SMOKE_STRIP=none CARGO_PROFILE_SMOKE_DEBUG=line-tables-only bash scripts/ph2d-run.sh \
//!   env PH2D_RELEASE="mede velocidade" cargo build --profile smoke -p ph2d-tool-painter --example mede_o_dab
//! target/smoke/examples/mede_o_dab [rondas] [so: raios|facas]
//! ```

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{PaintMedia, PainterTool, wet_diag};
use std::time::Instant;

const W: u32 = 3840;
const H: u32 = 2160;
const RAIOS: [f32; 4] = [10.0, 30.0, 60.0, 120.0];
/// Um traço da varredura por raio: os MESMOS 150 movimentos de 3 px das facas.
const MOVES: usize = 150;
const PASSO: f32 = 3.0;
/// Eventos por quadro de 60 Hz: o oráculo recebeu um a cada 2 ms.
const EV_POR_QUADRO: usize = 8;
const FACAS: &str = "assets/brushes/WetPainter/Oil/Knife";
/// Os nomes do `t_facas.jsonl` na ordem do script; os que a fábrica não tem caem na Smooth 3.
const NOMES: [&str; 10] = [
    "Knife Smooth",
    "Knife Smooth 2",
    "Knife Smooth 3",
    "Knife Textured",
    "Knife Oily",
    "Knife Dry",
    "Knife Dry 2",
    "Knife Grainy",
    "Knife Soft",
    "Knife Stain",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Motor {
    Digital,
    Wet,
    Oleo,
}

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tela(m: Motor, quadro: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    // O hospedeiro do app drena por quadro (`ph2d-app-painter` semeia-o); `false` = a tela compõe no evento.
    t.set_compor_por_quadro(quadro);
    if m != Motor::Digital {
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_wet_relogio_fixo(true);
        assert_eq!(t.paint_media(), PaintMedia::WetPaint, "o meio não armou");
    }
    if m == Motor::Oleo {
        t.set_wet_oleo_real_ligado(true);
        let o = t.wet_oleo_real().expect("o Óleo Real não ligou");
        if std::env::var_os("MEDE_ALCANCE").is_some() {
            println!("alcance do óleo: {} células", ph2d_wet_paint::oleo_real::alcance(&o));
        }
    }
    t.set_brush_color_srgb8([20, 150, 230]);
    t
}

/// Um traço horizontal de `MOVES` movimentos: `(ms dos Move, dabs, depósito ms, composite ms)`.
fn traco(t: &mut PainterTool, x0: f32, y: f32) -> (f64, u64, f64, f64) {
    let _ = wet_diag::take_stamp();
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    let t0 = Instant::now();
    let mut x = x0;
    for k in 0..MOVES {
        x += PASSO;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        if k % EV_POR_QUADRO == EV_POR_QUADRO - 1 {
            let _ = t.take_preview_arc();
        }
    }
    let _ = t.take_preview_arc();
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
    let (dabs, dep, comp, _) = wet_diag::take_stamp();
    (ms, dabs, dep, comp)
}

fn fnv(px: &[u8]) -> u64 {
    px.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn hash_da_tela(t: &mut PainterTool) -> u64 {
    let _ = t.take_preview_arc();
    t.imagem_aplicada().map_or(0, |(a, _, _)| fnv(&a))
}

fn faca(nome: &str) -> ph2d_brush_presets::Preset {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let f = raiz.join(FACAS).join(format!("{nome}.ph2d-brush.png"));
    let f = if f.exists() {
        f
    } else {
        raiz.join(FACAS).join("Knife Smooth 3.ph2d-brush.png")
    };
    let bytes = std::fs::read(&f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
    ph2d_brush_presets::ler(&bytes).expect("preset da faca")
}

/// As 20 pinceladas do oráculo numa tela nova: `(ms de parede dos 3 060 eventos, dabs, hash)`.
fn facas(presets: &[ph2d_brush_presets::Preset], quadro: bool) -> (f64, u64, u64) {
    let mut t = tela(Motor::Oleo, quadro);
    let _ = wet_diag::take_stamp();
    let t0 = Instant::now();
    for _ in 0..2 {
        for (k, p) in presets.iter().enumerate() {
            let falhas = t.wet_preset_aplicar(p);
            assert!(falhas.is_empty(), "texturas por ler: {falhas:?}");
            t.set_brush_size_px(45.0);
            t.set_brush_color_srgb8([20, 150, 230]);
            let (x0, y) = (if k < 5 { 70.0 } else { 650.0 }, 90.0 + 150.0 * (k % 5) as f32);
            t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
            let mut x = x0;
            for i in 0..MOVES {
                x += PASSO;
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
                if i % EV_POR_QUADRO == EV_POR_QUADRO - 1 {
                    let _ = t.take_preview_arc();
                }
            }
            t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
            let _ = t.take_preview_arc();
        }
    }
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    let (dabs, ..) = wet_diag::take_stamp();
    (ms, dabs, hash_da_tela(&mut t))
}

fn min_med(v: &[f64]) -> (f64, f64) {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    (s[0], s[s.len() / 2])
}

/// CPU do processo (user + sys, s) — `/proc/self/stat`, campos 14 e 15, em tiques de 100 Hz.
fn cpu_s() -> f64 {
    let st = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let resto = st.rsplit_once(')').map_or("", |(_, r)| r);
    let f: Vec<&str> = resto.split_whitespace().collect();
    let t = |i: usize| f.get(i).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
    (t(11) + t(12)) / 100.0
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim().to_owned()
}

fn main() {
    let rondas: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(5);
    let so = std::env::args().nth(2).unwrap_or_default();
    println!(
        "mede_o_dab {W}x{H} · rondas {rondas} · rayon {} · loadavg antes {}",
        rayon::current_num_threads(),
        loadavg()
    );

    // Variantes: (motor, modo) — cada uma na sua tela; modo `quadro` = a tela compõe na drenagem.
    let modos = [false, true];
    if so != "facas" {
        let motores = [Motor::Digital, Motor::Wet, Motor::Oleo];
        // `MOTOR=Wet` · `MODO=quadro` · `RAIO=10`: só essas variantes (o `perf record` de uma).
        let motor_env = std::env::var("MOTOR").ok();
        let modo_env = std::env::var("MODO").ok();
        let telas_de: Vec<(Motor, bool)> = motores
            .iter()
            .flat_map(|&m| modos.map(|q| (m, q)))
            .filter(|(m, _)| motor_env.as_ref().is_none_or(|e| *e == format!("{m:?}")))
            .filter(|(_, q)| modo_env.as_ref().is_none_or(|e| (e == "quadro") == *q))
            .collect();
        let mut telas: Vec<PainterTool> = telas_de.iter().map(|&(m, q)| tela(m, q)).collect();
        let n = telas.len() * RAIOS.len();
        // (tela, raio) → amostras de (µs/move, µs/dab, depósito µs/dab, composite µs/move, dabs)
        let mut am: Vec<Vec<(f64, f64, f64, f64, f64)>> = vec![Vec::new(); n];
        // Aquecimento fora da régua: um traço por variante, numa faixa só dele.
        for (ti, t) in telas.iter_mut().enumerate() {
            for (ri, &r) in RAIOS.iter().enumerate() {
                if std::env::var("RAIO").is_ok_and(|e| e != format!("{r}")) {
                    continue;
                }
                t.set_brush_size_px(r);
                let _ = traco(t, 40.0 + 900.0 * ri as f32, 2100.0 - 10.0 * (ti / 2) as f32);
            }
        }
        for ronda in 0..rondas {
            for k in 0..n {
                let v = (k + ronda) % n;
                let (ti, ri) = (v / RAIOS.len(), v % RAIOS.len());
                if std::env::var("RAIO").is_ok_and(|r| r != format!("{}", RAIOS[ri])) {
                    continue;
                }
                let t = &mut telas[ti];
                t.set_brush_size_px(RAIOS[ri]);
                // Cada (raio, ronda) pinta tela FRESCA: faixas de 300 px, colunas por raio.
                let (x0, y) = (40.0 + 900.0 * ri as f32, 140.0 + (300.0 * ronda as f32) % 1800.0);
                let (ms, dabs, dep, comp) = traco(t, x0, y);
                am[v].push((
                    1e3 * ms / MOVES as f64,
                    1e3 * ms / dabs.max(1) as f64,
                    1e3 * dep / dabs.max(1) as f64,
                    1e3 * comp / MOVES as f64,
                    dabs as f64,
                ));
            }
        }
        println!("motor   modo    raio | µs/move min (med) | dabs/traço | µs/dab | depósito µs/dab | composite µs/move");
        for (v, a) in am.iter().enumerate().filter(|(_, a)| !a.is_empty()) {
            let (ti, ri) = (v / RAIOS.len(), v % RAIOS.len());
            let col = |f: fn(&(f64, f64, f64, f64, f64)) -> f64| {
                min_med(&a.iter().map(f).collect::<Vec<_>>())
            };
            let (mv, mvm) = col(|x| x.0);
            let (pd, _) = col(|x| x.1);
            let (dp, _) = col(|x| x.2);
            let (cm, _) = col(|x| x.3);
            let (_, nd) = col(|x| x.4);
            let (m, q) = telas_de[ti];
            println!(
                "{:<7} {:<7} {:>4} | {mv:9.1} ({mvm:9.1}) | {nd:>10} | {pd:7.1} | {dp:15.1} | {cm:10.1}",
                format!("{m:?}"),
                if q { "quadro" } else { "evento" },
                RAIOS[ri],
            );
        }
        for (ti, t) in telas.iter_mut().enumerate() {
            let (m, q) = telas_de[ti];
            println!("hash {m:?} {} {:016x}", if q { "quadro" } else { "evento" }, hash_da_tela(t));
        }
    }

    if so != "raios" {
        let presets: Vec<_> = NOMES.iter().map(|n| faca(n)).collect();
        let mut am: [Vec<(f64, f64)>; 2] = [Vec::new(), Vec::new()];
        let mut hashes: [Vec<u64>; 2] = [Vec::new(), Vec::new()];
        let mut dabs = 0;
        for ronda in 0..rondas.min(3) {
            for k in 0..2 {
                let qi = (k + ronda) % 2;
                // `MODO=quadro|evento`: só essa variante (o `perf record` de uma).
                if std::env::var("MODO").is_ok_and(|m| (m == "quadro") != modos[qi]) {
                    continue;
                }
                let c0 = cpu_s();
                let (m, d, h) = facas(&presets, modos[qi]);
                am[qi].push((m, cpu_s() - c0));
                hashes[qi].push(h);
                dabs = d;
            }
        }
        for qi in (0..2).filter(|&q| !am[q].is_empty()) {
            let (mn, md) = min_med(&am[qi].iter().map(|x| x.0).collect::<Vec<_>>());
            let (cpu, _) = min_med(&am[qi].iter().map(|x| x.1).collect::<Vec<_>>());
            let h = &hashes[qi];
            println!(
                "facas {} 20 pinceladas ({} eventos, {dabs} dabs): {mn:.0} ms min ({md:.0} med) · CPU {cpu:.1} s · Rebelle span 8 s, CPU 9,1–11,4 s · hash {:016x} (iguais: {})",
                if modos[qi] { "quadro" } else { "evento" },
                2 * NOMES.len() * (MOVES + 2),
                h[0],
                h.iter().all(|&x| x == h[0])
            );
        }
    }
    println!("loadavg depois {}", loadavg());
}
