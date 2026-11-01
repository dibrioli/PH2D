//! SONDA J8 — o óleo diluído é lento (smoke do dono 01/11) por correr mais TEMPO ou por CUSTAR mais por passo?
//! Estágios de `sim_step_stage`: 0 região · 1 secagem/assenta · 2 fluxo · 3 advecção · 4 bordas · 5 projecção · 6 fecho.
//! Régua `docs/DevOps/MEDIR_VELOCIDADE.md`: réplicas deterministas intercaladas, resumo = o MÍNIMO; o tique
//! replica à mão o `wet_step_sync` (relógio fixo) para cronometrar cada pedaço.

use super::agua_sobre_oleo_tests::{H, PAPEL, W, ponteiro, preset};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Knob;
use std::time::Instant;

const Y0: f32 = 40.0;
const HZ: usize = 60;
const SEG: usize = 120;
const REP: usize = 3;
const BLOCO: usize = 30;
const AMOSTRAS: [usize; 9] = [1, 2, 5, 10, 20, 40, 60, 90, 120];
const JANELAS: [usize; 9] = [0, 1, 2, 5, 10, 20, 40, 60, 120];
const NJ: usize = JANELAS.len() - 1;
const NF: usize = 7;

/// (nome, preset, Dilution — `None` = a do preset —, Paint type 0 Water · 1 Oil).
type Var = (&'static str, &'static str, Option<f64>, f64);

const DRIP: &str = "Watercolor/Fluid/Wet Drip.ph2d-brush.png";
const FILBERT: &str = "Oil/Flat/Filbert.ph2d-brush.png";
const VARS: [Var; 5] = [
    ("Wet Drip Water d0.9", DRIP, Some(0.9), 0.0),
    ("Wet Drip Oil d0.9", DRIP, Some(0.9), 1.0),
    ("Filbert Oil d0.9", FILBERT, Some(0.9), 1.0),
    ("Filbert Oil d fábrica", FILBERT, None, 1.0),
    (
        "Wet Round Water d0.9",
        "Watercolor/Fluid/Wet Round.ph2d-brush.png",
        Some(0.9),
        0.0,
    ),
];

#[derive(Default, Clone, Copy)]
struct Acum {
    fase: [f64; NF],
    passos: u32,
    rodaram: u32,
    celulas: u64,
    compoe: f64,
    varre: f64,
    ticks: u32,
}

impl Acum {
    fn sim(&self) -> f64 {
        self.fase.iter().sum()
    }
}

struct Corrida {
    t: PainterTool,
    divida: f64,
    jan: [Acum; NJ],
    /// (s, células activas, bbox em células, has_fluid) nos instantes de `AMOSTRAS`.
    amostra: Vec<(usize, usize, i64, bool)>,
    /// O último tique em que um passo correu de facto (`Some(true)`).
    ultimo_vivo: Option<usize>,
}

/// O mesmo traço do `oleo_diluido_tests::traco` (horizontal em `Y0`, 24 px, 40 movimentos).
fn traco(v: &Var) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    t.preset_aplicar(&preset(v.1)).expect("o meio");
    if let Some(d) = v.2 {
        t.paint.wetpaint.knobs.set(Knob::Dilution, d);
    }
    t.paint.wetpaint.knobs.set(Knob::Medium, v.3);
    t.set_brush_size_px(24.0);
    t.set_brush_color_srgb8([40, 90, 220]);
    t.on_canvas_pointer(ponteiro(30.0, Y0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(ponteiro(30.0 + 4.5 * k as f32, Y0, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(210.0, Y0, PointerPhase::Up));
    t
}

/// (células activas, área da bbox, has_fluid) da grade activa.
fn ativas(t: &PainterTool) -> (usize, i64, bool) {
    let e = &t.paint.wetpaint.session.as_ref().expect("a sessão").engine;
    let g = &e.layers[e.active_layer].grid;
    if !g.has_fluid {
        return (0, 0, false);
    }
    let n = g.active.iter().filter(|&&a| a != 0).count();
    let area = i64::from((g.bx1 - g.bx0 + 1).max(0)) * i64::from((g.by1 - g.by0 + 1).max(0));
    (n, area, true)
}

fn janela(k: usize) -> usize {
    let s = k / HZ;
    (0..NJ).find(|&j| s < JANELAS[j + 1]).unwrap_or(NJ - 1)
}

/// UM tique de 60 Hz pelo caminho do relógio fixo (`wetpaint_tick` → `wet_step_sync`), cronometrado.
fn tique(c: &mut Corrida, k: usize) {
    let a = &mut c.jan[janela(k)];
    a.ticks += 1;
    let t0 = Instant::now();
    c.t.wet_compoe_o_pendente();
    c.t.wet_varre_oleo();
    a.varre += t0.elapsed().as_secs_f64();
    c.divida += (1.0 / HZ as f64) / offthread::STEP_S;
    let n = c.divida.floor();
    c.divida -= n;
    c.t.wet_bring_home();
    let mut moved = false;
    for _ in 0..n as usize {
        let e = &mut c
            .t
            .paint
            .wetpaint
            .session
            .as_mut()
            .expect("a sessão")
            .engine;
        if !e.sim_should_run() {
            break;
        }
        let ran = loop {
            let f = e.sim.stage_in_flight().map_or(0, usize::from).min(NF - 1);
            let t1 = Instant::now();
            let r = e.step_stage();
            a.fase[f] += t1.elapsed().as_secs_f64();
            if let Some(x) = r {
                break x;
            }
        };
        moved = true;
        a.passos += 1;
        if ran {
            a.rodaram += 1;
            c.ultimo_vivo = Some(k);
        }
        a.celulas += ativas(&c.t).0 as u64;
    }
    if moved {
        let t2 = Instant::now();
        c.t.wetpaint_composite();
        a.compoe += t2.elapsed().as_secs_f64();
    }
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// (mínimo, mediana) de `REP` valores.
fn min_med(mut v: Vec<f64>) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

#[test]
#[ignore = "sonda J8"]
fn diag_desempenho_do_oleo_diluido() {
    println!("loadavg início: {}", loadavg());
    let mut cs: Vec<Vec<Corrida>> = Vec::new();
    for v in &VARS {
        cs.push(
            (0..REP)
                .map(|_| Corrida {
                    t: traco(v),
                    divida: 0.0,
                    jan: [Acum::default(); NJ],
                    amostra: Vec::new(),
                    ultimo_vivo: None,
                })
                .collect(),
        );
    }
    let total = SEG * HZ;
    let nv = VARS.len();
    for (b, k0) in (0..total).step_by(BLOCO).enumerate() {
        for o in 0..nv * REP {
            let idx = (o + b) % (nv * REP);
            let c = &mut cs[idx % nv][idx / nv];
            for k in k0..(k0 + BLOCO).min(total) {
                tique(c, k);
                if (k + 1) % HZ == 0 && AMOSTRAS.contains(&((k + 1) / HZ)) {
                    let (n, area, viva) = ativas(&c.t);
                    c.amostra.push(((k + 1) / HZ, n, area, viva));
                }
            }
        }
    }
    println!("loadavg fim: {}", loadavg());

    println!("\n## P1 — células activas (máscara do solver) após o traço; bbox entre parênteses");
    println!(
        "variante | {} | último passo vivo (s) | passos vivos em {SEG} s",
        AMOSTRAS.map(|s| format!("{s} s")).join(" | ")
    );
    for (v, rs) in VARS.iter().zip(&cs) {
        let c = &rs[0];
        let cols: Vec<String> = c
            .amostra
            .iter()
            .map(|&(_, n, a, viva)| {
                if viva {
                    format!("{n} ({a})")
                } else {
                    "seco".into()
                }
            })
            .collect();
        let vivos: u32 = c.jan.iter().map(|a| a.rodaram).sum();
        let ult = c.ultimo_vivo.map_or("—".into(), |k| {
            format!("{:.2}", (k + 1) as f64 / HZ as f64)
        });
        println!("{} | {} | {ult} | {vivos}", v.0, cols.join(" | "));
        for r in &rs[1..] {
            if r.amostra != c.amostra {
                println!("  ⚠️ réplica diverge de réplica 0 (não determinista)");
            }
        }
    }

    println!(
        "\n## P2 — custo por passo por janela: ms/passo (mín · med) · µs/passo por 1000 células activas (mín) · composite ms/tique (mín)"
    );
    for (v, rs) in VARS.iter().zip(&cs) {
        println!("### {}", v.0);
        for j in 0..NJ {
            let a0 = rs[0].jan[j];
            if a0.passos == 0 {
                continue;
            }
            let (mn, md) = min_med(rs.iter().map(|r| r.jan[j].sim()).collect());
            let (cm, _) = min_med(rs.iter().map(|r| r.jan[j].compoe).collect());
            let (vm, _) = min_med(rs.iter().map(|r| r.jan[j].varre).collect());
            let cel = a0.celulas as f64 / f64::from(a0.passos);
            println!(
                "{:>3}–{:>3} s | passos {:>4} (vivos {:>4}) | células/passo {:>7.0} | {:.3} · {:.3} ms/passo | {:.2} µs/passo/kcél | composite {:.3} ms/tique | varre {:.3} ms/tique",
                JANELAS[j],
                JANELAS[j + 1],
                a0.passos,
                a0.rodaram,
                cel,
                mn * 1e3 / f64::from(a0.passos),
                md * 1e3 / f64::from(a0.passos),
                if cel > 0.0 {
                    mn * 1e6 / f64::from(a0.passos) / (cel / 1e3)
                } else {
                    0.0
                },
                cm * 1e3 / f64::from(a0.ticks),
                vm * 1e3 / f64::from(a0.ticks)
            );
        }
    }

    println!("\n## Fases — ms totais em {SEG} s (mín das réplicas) e ms/passo vivo");
    println!(
        "variante | 0 região | 1 secagem | 2 fluxo | 3 advecção | 4 bordas | 5 projecção | 6 fecho | sim total | composite total | varre total"
    );
    for (v, rs) in VARS.iter().zip(&cs) {
        let soma = |r: &Corrida, f: &dyn Fn(&Acum) -> f64| r.jan.iter().map(f).sum::<f64>();
        let vivos: u32 = rs[0].jan.iter().map(|a| a.rodaram).sum::<u32>().max(1);
        let mut cols = Vec::new();
        for f in 0..NF {
            let (m, _) = min_med(rs.iter().map(|r| soma(r, &|a| a.fase[f])).collect());
            cols.push(format!(
                "{:.0} ({:.3})",
                m * 1e3,
                m * 1e3 / f64::from(vivos)
            ));
        }
        let (s, _) = min_med(rs.iter().map(|r| soma(r, &|a| a.sim())).collect());
        let (c, _) = min_med(rs.iter().map(|r| soma(r, &|a| a.compoe)).collect());
        let (w, _) = min_med(rs.iter().map(|r| soma(r, &|a| a.varre)).collect());
        println!(
            "{} | {} | {:.0} | {:.0} | {:.0}",
            v.0,
            cols.join(" | "),
            s * 1e3,
            c * 1e3,
            w * 1e3
        );
    }
}
