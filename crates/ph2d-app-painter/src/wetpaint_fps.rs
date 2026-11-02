//! `PH2D_WETPAINT_SMOKE=3` — **a cena do FPS** (J13 §1): o app pinta SOZINHO, ao relógio de parede, e
//! cronometra cada quadro. Por rodada: um traço do Dabber (Real Oil por `PH2D_FPS_OLEO=0|1`, omissão 1 — por
//! PROCESSO: a grade que já viu óleo fica com a óptica dele), a espera, o Wet Drip por cima, a água a correr. O período é o tempo entre duas chamadas
//! seguidas de [`conduz`] (uma por quadro, de dentro da ponte); o ponteiro chega a `HZ` eventos por
//! segundo como um rato real. As faixas acumulam (a água das anteriores ainda corre): o relato é POR RODADA.
//! Tamanho da tela: `PH2D_FPS_TELA` (lado em px, omissão 1024).

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_tool_painter::PainterTool;
use std::time::Instant;

const HZ: f64 = 250.0;
const RODADAS: usize = 6;
/// `(nome, segundos, pincel)` — `pincel` = o preset que risca nesta fase (`None` = mãos paradas).
const FASES: [(&str, f64, Option<&str>); 4] = [
    (
        "dabber",
        2.5,
        Some("WetPainter/Oil/Dabber/Dabber.ph2d-brush.png"),
    ),
    ("seca", 2.0, None),
    (
        "wet drip",
        2.5,
        Some("WetPainter/Watercolor/Fluid/Wet Drip.ph2d-brush.png"),
    ),
    ("corre", 5.0, None),
];

use crate::wetpaint_smoke::lado_do_fps as lado;

#[derive(Default)]
struct Estado {
    /// `rodada · FASES + fase`; `None` antes do primeiro quadro.
    passo: Option<usize>,
    inicio_fase: Option<Instant>,
    ultimo_quadro: Option<Instant>,
    eventos: usize,
    /// Por quadro da fase: `(período ms, ponteiro ms, eventos)`.
    quadros: Vec<(f64, f64, usize)>,
    /// `[fase]` → por rodada `(p50, p95, max, ponteiro p50, eventos/quadro)`.
    tabela: [Vec<[f64; 5]>; FASES.len()],
    feito: bool,
}

thread_local! {
    static ESTADO: std::cell::RefCell<Estado> = std::cell::RefCell::default();
}

fn preset(rel: &str) -> Option<ph2d_brush_presets::Preset> {
    let f = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/brushes")
        .join(rel);
    ph2d_brush_presets::ler(&std::fs::read(f).ok()?).ok()
}

fn quantil(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v.get(((v.len().max(1) - 1) as f64 * q).round() as usize)
        .copied()
        .unwrap_or(0.0)
}

fn ponto(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 0.7,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um quadro da cena. Chamada pela ponte a cada quadro com o Painter activo; não faz nada fora do `=3`.
pub fn conduz(painter: &mut PainterTool) {
    if !crate::wetpaint_smoke::e_a_cena_do_fps() {
        return;
    }
    ESTADO.with(|e| quadro(&mut e.borrow_mut(), painter));
}

fn quadro(e: &mut Estado, painter: &mut PainterTool) {
    if e.feito {
        return;
    }
    let agora = Instant::now();
    let periodo = e.ultimo_quadro.map(|t| (agora - t).as_secs_f64() * 1e3);
    e.ultimo_quadro = Some(agora);
    let total = RODADAS * FASES.len();
    let passo = *e.passo.get_or_insert(0);
    let fase = passo % FASES.len();
    let (_, dur, pincel) = FASES[fase];
    if e.inicio_fase.is_none() {
        e.inicio_fase = Some(agora);
        e.eventos = 0;
        e.quadros.clear();
        if let Some(rel) = pincel {
            let Some(p) = preset(rel) else {
                eprintln!("[fps] o preset {rel} não foi encontrado");
                e.feito = true;
                return;
            };
            let _ = painter.preset_aplicar(&p);
            if fase == 0 {
                painter.set_wet_oleo_real_ligado(oleo());
            }
            painter.set_brush_size_px(30.0);
            painter.set_brush_color_srgb8(if fase == 0 {
                [40, 90, 220]
            } else {
                [230, 200, 30]
            });
        }
    }
    let t = (agora - e.inicio_fase.unwrap_or(agora)).as_secs_f64();
    let lado = lado() as f32;
    // As faixas sobem de baixo para cima: a água de uma faixa nova escorre para as já medidas.
    let y = lado * (0.9 - 0.8 * (passo / FASES.len()) as f32 / RODADAS as f32);
    let x = |k: usize| lado * 0.1 + lado * 0.8 * ((k as f64 / HZ) / dur).min(1.0) as f32;
    let t0 = Instant::now();
    let mut n = 0;
    if pincel.is_some() {
        let alvo = ((t * HZ) as usize).min((dur * HZ) as usize);
        if e.eventos == 0 && alvo > 0 {
            painter.on_canvas_pointer(ponto(x(0), y, PointerPhase::Down));
        }
        while e.eventos < alvo {
            e.eventos += 1;
            n += 1;
            painter.on_canvas_pointer(ponto(x(e.eventos), y, PointerPhase::Move));
        }
    }
    let ponteiro = t0.elapsed().as_secs_f64() * 1e3;
    if let Some(p) = periodo {
        e.quadros.push((p, ponteiro, n));
    }
    if t < dur {
        return;
    }
    if pincel.is_some() {
        painter.on_canvas_pointer(ponto(x(e.eventos), y, PointerPhase::Up));
    }
    let mut per: Vec<f64> = e.quadros.iter().map(|q| q.0).collect();
    let mut pon: Vec<f64> = e.quadros.iter().map(|q| q.1).collect();
    let ev = e.quadros.iter().map(|q| q.2).sum::<usize>() as f64 / e.quadros.len().max(1) as f64;
    let max = per.iter().copied().fold(0.0, f64::max);
    let linha = [
        quantil(&mut per, 0.5),
        quantil(&mut per, 0.95),
        max,
        quantil(&mut pon, 0.5),
        ev,
    ];
    eprintln!(
        "[fps] rodada {} {} {:<9}| quadros {:>4} | período p50 {:.1} p95 {:.1} max {:.1} ms | ponteiro p50 {:.2} ms | {:.1} eventos/quadro",
        passo / FASES.len(),
        ["sem real", "real oil"][usize::from(oleo())],
        FASES[fase].0,
        e.quadros.len(),
        linha[0],
        linha[1],
        linha[2],
        linha[3],
        linha[4]
    );
    e.tabela[fase].push(linha);
    e.inicio_fase = None;
    e.passo = Some(passo + 1);
    if passo + 1 == total {
        e.feito = true;
        relatorio(e);
    }
}

fn oleo() -> bool {
    std::env::var("PH2D_FPS_OLEO").as_deref() != Ok("0")
}

fn relatorio(e: &Estado) {
    let nome = ["sem real", "real oil"][usize::from(oleo())];
    eprintln!(
        "[fps] === tela {0}x{0}, {nome}: período p50 (ms) por rodada ===",
        lado()
    );
    for (f, (fase, ..)) in FASES.iter().enumerate() {
        let r: Vec<String> = e.tabela[f].iter().map(|x| format!("{:.1}", x[0])).collect();
        eprintln!("[fps] {nome} {fase:<9}| {}", r.join(" · "));
    }
}
